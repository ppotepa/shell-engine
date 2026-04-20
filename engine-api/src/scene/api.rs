//! Scene domain API: explicit live scene-object handles plus snapshot inspection.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::Value as JsonValue;

use crate::rhai::conversion::{
    map_get_path_dynamic, map_set_path_dynamic, merge_rhai_maps, normalize_set_path,
    rhai_dynamic_to_json,
};
use crate::runtime::{ObjectRegistryCoreApi, RuntimeSceneCoreApi};
use crate::scene::queries::{
    build_object_entry, object_matches_name, object_matches_tag, object_state_from_entry,
    region_to_rhai_map,
};
use crate::{
    BehaviorCommand, Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot,
    SceneMutationRequest,
};

fn render3d_profile_slot_from_str(value: &str) -> Option<Render3dProfileSlot> {
    match value.trim() {
        "view" => Some(Render3dProfileSlot::View),
        "lighting" => Some(Render3dProfileSlot::Lighting),
        "space_environment" | "space-environment" => Some(Render3dProfileSlot::SpaceEnvironment),
        _ => None,
    }
}

fn dynamic_to_string(value: RhaiDynamic) -> String {
    value.try_cast::<String>().unwrap_or_default()
}

fn dynamic_to_bool(value: RhaiDynamic) -> bool {
    value.try_cast::<bool>().unwrap_or(false)
}

fn dynamic_to_float(value: RhaiDynamic) -> rhai::FLOAT {
    let value = value.flatten();
    value
        .clone()
        .try_cast::<rhai::FLOAT>()
        .or_else(|| {
            value
                .clone()
                .try_cast::<rhai::INT>()
                .map(|v| v as rhai::FLOAT)
        })
        .unwrap_or(0.0)
}

/// Script-facing scene surface.
///
/// Primary live-handle entry points are `scene.object(target)` and
/// `scene.objects.find(target)`. Behavior runtime adapters mirror that as
/// `runtime.scene.objects.*`. `scene.inspect(target)` stays snapshot-only and
/// does not share pending live-handle mutations.
#[derive(Clone)]
pub struct ScriptSceneApi {
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    live_overlays: Arc<Mutex<HashMap<String, RhaiMap>>>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

/// Primary query surface for live scene-object handles.
///
/// Exposed as `scene.objects` and `runtime.scene.objects`.
#[derive(Clone)]
pub struct ScriptSceneObjectsApi {
    scene: ScriptSceneApi,
}

/// Live scene-object handle returned by `scene.object(...)` and
/// `scene.objects.find(...)`.
#[derive(Clone)]
pub struct ScriptObjectApi {
    target: String,
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    live_overlays: Arc<Mutex<HashMap<String, RhaiMap>>>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

/// Typed text-domain subhandle exposed as `scene.object(id).text`.
#[derive(Clone)]
pub struct ScriptObjectTextApi {
    object: ScriptObjectApi,
}

/// Typed style-domain subhandle exposed as `scene.object(id).style`.
#[derive(Clone)]
pub struct ScriptObjectStyleApi {
    object: ScriptObjectApi,
}

/// Typed frame/runtime-view subhandle exposed as `scene.object(id).frame`.
#[derive(Clone)]
pub struct ScriptObjectFrameApi {
    object: ScriptObjectApi,
}

/// Typed render-domain root exposed as `scene.object(id).render`.
#[derive(Clone)]
pub struct ScriptObjectRenderApi {
    object: ScriptObjectApi,
}

/// Split render-domain subhandle exposed under `scene.object(id).render.*`.
#[derive(Clone)]
pub struct ScriptObjectRenderGroupApi {
    object: ScriptObjectApi,
    domain: ScriptObjectRenderGroupDomain,
}

#[derive(Clone, Copy)]
enum ScriptObjectRenderGroupDomain {
    Atmosphere,
    Surface,
    Generator,
    Body,
    View,
}

impl ScriptSceneApi {
    /// Create a new scene API with the given backing state and command queue.
    pub fn new(
        object_states: Arc<HashMap<String, ObjectRuntimeState>>,
        object_kinds: Arc<HashMap<String, String>>,
        object_props: Arc<HashMap<String, JsonValue>>,
        object_regions: Arc<HashMap<String, Region>>,
        object_text: Arc<HashMap<String, String>>,
        target_resolver: Arc<TargetResolver>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            object_states,
            object_kinds,
            object_props,
            object_regions,
            object_text,
            target_resolver,
            live_overlays: Arc::new(Mutex::new(HashMap::new())),
            queue,
        }
    }

    fn enqueue_scene_mutation(&mut self, request: SceneMutationRequest) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::ApplySceneMutation { request });
        true
    }

    fn enqueue_render3d_group_params(
        &mut self,
        target: &str,
        params: RhaiMap,
        build: impl FnOnce(String, JsonValue) -> Render3dMutationRequest,
    ) -> bool {
        if target.trim().is_empty() || params.is_empty() {
            return false;
        }
        let Some(params) = rhai_dynamic_to_json(&RhaiDynamic::from_map(params)) else {
            return false;
        };
        if !params.is_object() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(build(
            target.to_string(),
            params,
        )))
    }

    fn resolve_target(&self, target: &str) -> String {
        self.target_resolver
            .resolve_alias(target)
            .unwrap_or(target)
            .to_string()
    }

    fn object_handle_for_resolved(&self, object_id: String) -> ScriptObjectApi {
        ScriptObjectApi {
            target: object_id,
            object_states: Arc::clone(&self.object_states),
            object_kinds: Arc::clone(&self.object_kinds),
            object_props: Arc::clone(&self.object_props),
            object_regions: Arc::clone(&self.object_regions),
            object_text: Arc::clone(&self.object_text),
            target_resolver: Arc::clone(&self.target_resolver),
            live_overlays: Arc::clone(&self.live_overlays),
            queue: Arc::clone(&self.queue),
        }
    }

    fn object_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self.object_states.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Resolve one live scene-object handle by alias or runtime id.
    pub fn object(&mut self, target: &str) -> ScriptObjectApi {
        self.object_handle_for_resolved(self.resolve_target(target))
    }

    /// Returns the live scene-object query surface.
    pub fn objects(&mut self) -> ScriptSceneObjectsApi {
        ScriptSceneObjectsApi {
            scene: self.clone(),
        }
    }

    /// Returns a snapshot map for the resolved object id.
    pub fn inspect(&mut self, target: &str) -> RhaiMap {
        self.build_object_entry(&self.resolve_target(target))
    }

    /// Deprecated Rust-side compatibility shim kept only for narrow migration
    /// seams. It is no longer registered into the public Rhai surface.
    ///
    /// Prefer `object(target)` or `objects.find(target)` so the active mutable
    /// path stays distinct from snapshot reads like `inspect(target)`.
    #[deprecated(
        note = "use object(target) or objects.find(target) for live handles; inspect(target) is snapshot-only"
    )]
    pub fn get(&mut self, target: &str) -> ScriptObjectApi {
        self.object(target)
    }

    /// Returns the runtime region map for the resolved object id.
    pub fn region(&mut self, target: &str) -> RhaiMap {
        self.object_regions
            .get(&self.resolve_target(target))
            .map(region_to_rhai_map)
            .unwrap_or_default()
    }

    fn build_object_entry(&self, object_id: &str) -> RhaiMap {
        build_object_entry(
            &self.object_states,
            &self.object_kinds,
            &self.object_props,
            &self.object_regions,
            &self.object_text,
            &self.target_resolver,
            object_id,
        )
    }

    /// Low-level root-scene path mutation helper retained for narrow
    /// compatibility wrappers inside Rust.
    ///
    /// Public Rhai scripts should prefer `scene.object(target).set(path, value)`
    /// or `scene.mutate(...)`.
    pub fn set(&mut self, target: &str, path: &str, value: RhaiDynamic) -> bool {
        let normalized_path = normalize_set_path(path);
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        let resolved = self.resolve_target(target);
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        let Ok(request) = crate::commands::scene_mutation_request_from_set_path_result(
            &resolved,
            &normalized_path,
            &value,
            self.object_states.get(&resolved),
        ) else {
            return false;
        };
        queue.push(BehaviorCommand::ApplySceneMutation { request });
        true
    }

    /// Spawn a scene object from a template.
    pub fn spawn(&mut self, template: &str, target: &str) -> bool {
        if template.trim().is_empty() || target.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SpawnObject {
            template: template.to_string(),
            target: target.to_string(),
        })
    }

    /// Despawn a scene object.
    pub fn despawn(&mut self, target: &str) -> bool {
        if target.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::DespawnObject {
            target: target.to_string(),
        })
    }

    /// Change the scene background color.
    pub fn set_bg(&mut self, color: &str) {
        if color.trim().is_empty() {
            return;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::SetSceneBg {
            color: color.to_string(),
        });
    }

    /// Apply a typed scene mutation request encoded as a Rhai map.
    pub fn mutate(&mut self, request: RhaiMap) -> bool {
        let Some(json) = rhai_dynamic_to_json(&RhaiDynamic::from_map(request)) else {
            return false;
        };
        let Ok(request) = serde_json::from_value::<SceneMutationRequest>(json) else {
            return false;
        };
        self.enqueue_scene_mutation(request)
    }

    /// Convenience helper for typed 3D camera look-at mutation.
    pub fn set_camera3d_look_at(&mut self, eye: [f32; 3], look_at: [f32; 3]) -> bool {
        self.enqueue_scene_mutation(SceneMutationRequest::SetCamera3d(
            Camera3dMutationRequest::LookAt { eye, look_at },
        ))
    }

    /// Convenience helper for typed 3D camera up-vector mutation.
    pub fn set_camera3d_up(&mut self, up: [f32; 3]) -> bool {
        self.enqueue_scene_mutation(SceneMutationRequest::SetCamera3d(
            Camera3dMutationRequest::Up { up },
        ))
    }

    /// Convenience helper for typed object-camera look-at mutation.
    pub fn set_camera3d_object_look_at(
        &mut self,
        target: &str,
        eye: [f32; 3],
        look_at: [f32; 3],
    ) -> bool {
        if target.trim().is_empty() {
            return false;
        }
        let resolved = self.resolve_target(target);
        self.enqueue_scene_mutation(SceneMutationRequest::SetCamera3d(
            Camera3dMutationRequest::ObjectLookAt {
                target: resolved,
                eye,
                look_at,
                up: None,
            },
        ))
    }

    /// Convenience helper for typed object-camera basis mutation.
    pub fn set_camera3d_object_basis(
        &mut self,
        target: &str,
        eye: [f32; 3],
        right: [f32; 3],
        up: [f32; 3],
        forward: [f32; 3],
    ) -> bool {
        if target.trim().is_empty() {
            return false;
        }
        let resolved = self.resolve_target(target);
        self.enqueue_scene_mutation(SceneMutationRequest::SetCamera3d(
            Camera3dMutationRequest::ObjectBasis {
                target: resolved,
                eye,
                right,
                up,
                forward,
            },
        ))
    }

    /// Convenience helper for typed 3D view-profile switching.
    pub fn set_view_profile(&mut self, profile: &str) -> bool {
        if profile.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetViewProfile {
                profile: profile.to_string(),
            },
        ))
    }

    /// Convenience helper for typed 3D lighting-profile switching.
    pub fn set_lighting_profile(&mut self, profile: &str) -> bool {
        if profile.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetLightingProfile {
                profile: profile.to_string(),
            },
        ))
    }

    /// Convenience helper for typed 3D space-environment-profile switching.
    pub fn set_space_environment_profile(&mut self, profile: &str) -> bool {
        if profile.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetSpaceEnvironmentProfile {
                profile: profile.to_string(),
            },
        ))
    }

    /// Convenience helper for typed 3D lighting-profile parameter override.
    pub fn set_lighting_param(&mut self, name: &str, value: RhaiDynamic) -> bool {
        if name.trim().is_empty() {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetLightingParam {
                name: name.to_string(),
                value,
            },
        ))
    }

    /// Convenience helper for typed 3D space-environment parameter override.
    pub fn set_space_environment_param(&mut self, name: &str, value: RhaiDynamic) -> bool {
        if name.trim().is_empty() {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetSpaceEnvironmentParam {
                name: name.to_string(),
                value,
            },
        ))
    }

    /// Convenience helper for neutral scene-level 3D profile switching by slot.
    pub fn set_render3d_profile(
        &mut self,
        profile_slot: Render3dProfileSlot,
        profile: &str,
    ) -> bool {
        if profile.trim().is_empty() {
            return false;
        }
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetProfile {
                profile_slot,
                profile: profile.to_string(),
            },
        ))
    }

    /// Convenience helper for neutral scene-level 3D profile parameter override by slot.
    pub fn set_render3d_profile_param(
        &mut self,
        profile_slot: Render3dProfileSlot,
        name: &str,
        value: RhaiDynamic,
    ) -> bool {
        if name.trim().is_empty() {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        self.enqueue_scene_mutation(SceneMutationRequest::SetRender3d(
            Render3dMutationRequest::SetProfileParam {
                profile_slot,
                name: name.to_string(),
                value,
            },
        ))
    }

    /// Convenience helper for additive grouped material parameter overrides.
    pub fn set_material_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetMaterialParams { target, params }
        })
    }

    /// Convenience helper for additive grouped atmosphere parameter overrides.
    pub fn set_atmosphere_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetAtmosphereParams { target, params }
        })
    }

    /// Convenience helper for additive grouped surface parameter overrides.
    pub fn set_surface_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetSurfaceParams { target, params }
        })
    }

    /// Convenience helper for additive grouped generator parameter overrides.
    pub fn set_generator_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetGeneratorParams { target, params }
        })
    }

    /// Convenience helper for additive grouped body parameter overrides.
    pub fn set_body_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetBodyParams { target, params }
        })
    }

    /// Convenience helper for additive grouped view parameter overrides.
    pub fn set_view_params(&mut self, target: &str, params: RhaiMap) -> bool {
        self.enqueue_render3d_group_params(target, params, |target, params| {
            Render3dMutationRequest::SetViewParams { target, params }
        })
    }
}

impl ScriptSceneObjectsApi {
    /// Resolve one live scene-object handle by alias or runtime id.
    pub fn find(&mut self, target: &str) -> ScriptObjectApi {
        self.scene.object(target)
    }

    /// Return all known live scene-object handles in stable runtime-id order.
    pub fn all(&mut self) -> RhaiArray {
        self.scene
            .object_ids()
            .into_iter()
            .map(|object_id| RhaiDynamic::from(self.scene.object_handle_for_resolved(object_id)))
            .collect()
    }

    /// Return live scene-object handles whose metadata advertises the requested tag.
    ///
    /// Scene tags are derived from runtime snapshot metadata when present and always
    /// include the object kind as a virtual tag.
    pub fn by_tag(&mut self, tag: &str) -> RhaiArray {
        self.scene
            .object_ids()
            .into_iter()
            .filter(|object_id| {
                object_matches_tag(
                    &self.scene.object_kinds,
                    &self.scene.object_props,
                    object_id,
                    tag,
                )
            })
            .map(|object_id| RhaiDynamic::from(self.scene.object_handle_for_resolved(object_id)))
            .collect()
    }

    /// Return live scene-object handles whose runtime/authored names match `name`.
    pub fn by_name(&mut self, name: &str) -> RhaiArray {
        self.scene
            .object_ids()
            .into_iter()
            .filter(|object_id| object_matches_name(&self.scene.target_resolver, object_id, name))
            .map(|object_id| RhaiDynamic::from(self.scene.object_handle_for_resolved(object_id)))
            .collect()
    }

    /// Iteration-friendly alias for [`Self::all`].
    pub fn iter(&mut self) -> RhaiArray {
        self.all()
    }
}

impl ScriptObjectApi {
    fn build_live_entry(&self) -> RhaiMap {
        let mut entry = build_object_entry(
            &self.object_states,
            &self.object_kinds,
            &self.object_props,
            &self.object_regions,
            &self.object_text,
            &self.target_resolver,
            &self.target,
        );
        let Ok(overlays) = self.live_overlays.lock() else {
            return entry;
        };
        if let Some(overlay) = overlays.get(&self.target) {
            merge_rhai_maps(&mut entry, overlay);
        }
        entry
    }

    /// Read from the live object handle.
    pub fn get(&mut self, path: &str) -> RhaiDynamic {
        let entry = self.build_live_entry();
        map_get_path_dynamic(&entry, path)
            .or_else(|| map_get_path_dynamic(&entry, &format!("props.{path}")))
            .unwrap_or_else(|| ().into())
    }

    /// Mutate the live object handle and queue the typed scene mutation.
    pub fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let normalized_path = normalize_set_path(path);
        let mut entry = self.build_live_entry();
        if !map_set_path_dynamic(&mut entry, &normalized_path, value.clone()) {
            return false;
        }
        let overlay_value = value.clone();
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        let object_state = object_state_from_entry(&entry);
        let Ok(request) = crate::commands::scene_mutation_request_from_set_path_result(
            &self.target,
            &normalized_path,
            &value,
            object_state.as_ref(),
        ) else {
            return false;
        };
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::ApplySceneMutation { request });
        drop(queue);
        let Ok(mut overlays) = self.live_overlays.lock() else {
            return false;
        };
        let overlay = overlays.entry(self.target.clone()).or_default();
        map_set_path_dynamic(overlay, &normalized_path, overlay_value)
    }

    /// Return the resolved runtime id behind this live handle.
    pub fn id(&mut self) -> String {
        self.target.clone()
    }

    /// Return the typed text-domain subhandle.
    pub fn text(&mut self) -> ScriptObjectTextApi {
        ScriptObjectTextApi {
            object: self.clone(),
        }
    }

    /// Return the typed style-domain subhandle.
    pub fn style(&mut self) -> ScriptObjectStyleApi {
        ScriptObjectStyleApi {
            object: self.clone(),
        }
    }

    /// Return the typed transient frame/view-state subhandle.
    pub fn frame(&mut self) -> ScriptObjectFrameApi {
        ScriptObjectFrameApi {
            object: self.clone(),
        }
    }

    /// Return the typed render-domain root.
    pub fn render(&mut self) -> ScriptObjectRenderApi {
        ScriptObjectRenderApi {
            object: self.clone(),
        }
    }

    fn enqueue_render_request(&mut self, request: Render3dMutationRequest) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetRender3d(request),
        });
        true
    }

    fn enqueue_render_group_params(
        &mut self,
        params: RhaiMap,
        build: impl FnOnce(String, JsonValue) -> Render3dMutationRequest,
    ) -> bool {
        if params.is_empty() {
            return false;
        }
        let Some(params) = rhai_dynamic_to_json(&RhaiDynamic::from_map(params)) else {
            return false;
        };
        if !params.is_object() {
            return false;
        }
        self.enqueue_render_request(build(self.target.clone(), params))
    }

    fn overlay_set(&self, path: &str, value: RhaiDynamic) -> bool {
        let Ok(mut overlays) = self.live_overlays.lock() else {
            return false;
        };
        let overlay = overlays.entry(self.target.clone()).or_default();
        map_set_path_dynamic(overlay, path, value)
    }

    fn set_surface_mode(&mut self, mode: &str) -> bool {
        if mode.trim().is_empty() {
            return false;
        }
        if !self.enqueue_render_request(Render3dMutationRequest::SetSurfaceMode {
            target: self.target.clone(),
            mode: mode.trim().to_string(),
        }) {
            return false;
        }
        self.overlay_set("obj.surface_mode", mode.trim().to_string().into())
    }
}

impl ScriptObjectTextApi {
    pub fn content(&mut self) -> String {
        dynamic_to_string(self.object.get("text.content"))
    }

    pub fn set_content(&mut self, value: &str) -> bool {
        self.object.set("text.content", value.to_string().into())
    }

    pub fn fg(&mut self) -> String {
        dynamic_to_string(self.object.get("style.fg"))
    }

    pub fn set_fg(&mut self, value: &str) -> bool {
        self.object.set("style.fg", value.to_string().into())
    }

    pub fn bg(&mut self) -> String {
        dynamic_to_string(self.object.get("style.bg"))
    }

    pub fn set_bg(&mut self, value: &str) -> bool {
        self.object.set("style.bg", value.to_string().into())
    }

    pub fn font(&mut self) -> String {
        dynamic_to_string(self.object.get("text.font"))
    }

    pub fn set_font(&mut self, value: &str) -> bool {
        self.object.set("text.font", value.to_string().into())
    }
}

impl ScriptObjectStyleApi {
    pub fn fg(&mut self) -> String {
        dynamic_to_string(self.object.get("style.fg"))
    }

    pub fn set_fg(&mut self, value: &str) -> bool {
        self.object.set("style.fg", value.to_string().into())
    }

    pub fn bg(&mut self) -> String {
        dynamic_to_string(self.object.get("style.bg"))
    }

    pub fn set_bg(&mut self, value: &str) -> bool {
        self.object.set("style.bg", value.to_string().into())
    }
}

impl ScriptObjectFrameApi {
    pub fn visible(&mut self) -> bool {
        dynamic_to_bool(self.object.get("visible"))
    }

    pub fn set_visible(&mut self, value: bool) -> bool {
        self.object.set("visible", value.into())
    }

    pub fn dx(&mut self) -> rhai::FLOAT {
        dynamic_to_float(self.object.get("dx"))
    }

    pub fn set_dx(&mut self, value: rhai::FLOAT) -> bool {
        self.object.set("dx", value.into())
    }

    pub fn dy(&mut self) -> rhai::FLOAT {
        dynamic_to_float(self.object.get("dy"))
    }

    pub fn set_dy(&mut self, value: rhai::FLOAT) -> bool {
        self.object.set("dy", value.into())
    }

    pub fn set_offset(&mut self, dx: rhai::FLOAT, dy: rhai::FLOAT) -> bool {
        self.set_dx(dx) & self.set_dy(dy)
    }
}

impl ScriptObjectRenderApi {
    pub fn atmosphere(&mut self) -> ScriptObjectRenderGroupApi {
        ScriptObjectRenderGroupApi {
            object: self.object.clone(),
            domain: ScriptObjectRenderGroupDomain::Atmosphere,
        }
    }

    pub fn surface(&mut self) -> ScriptObjectRenderGroupApi {
        ScriptObjectRenderGroupApi {
            object: self.object.clone(),
            domain: ScriptObjectRenderGroupDomain::Surface,
        }
    }

    pub fn generator(&mut self) -> ScriptObjectRenderGroupApi {
        ScriptObjectRenderGroupApi {
            object: self.object.clone(),
            domain: ScriptObjectRenderGroupDomain::Generator,
        }
    }

    pub fn body(&mut self) -> ScriptObjectRenderGroupApi {
        ScriptObjectRenderGroupApi {
            object: self.object.clone(),
            domain: ScriptObjectRenderGroupDomain::Body,
        }
    }

    pub fn view(&mut self) -> ScriptObjectRenderGroupApi {
        ScriptObjectRenderGroupApi {
            object: self.object.clone(),
            domain: ScriptObjectRenderGroupDomain::View,
        }
    }

    pub fn surface_mode(&mut self) -> String {
        dynamic_to_string(self.object.get("obj.surface_mode"))
    }

    pub fn set_surface_mode(&mut self, mode: &str) -> bool {
        self.object.set_surface_mode(mode)
    }
}

impl ScriptObjectRenderGroupApi {
    pub fn set(&mut self, name: &str, value: RhaiDynamic) -> bool {
        if name.trim().is_empty() {
            return false;
        }
        let mut params = RhaiMap::new();
        params.insert(name.trim().into(), value);
        self.set_many(params)
    }

    pub fn set_many(&mut self, params: RhaiMap) -> bool {
        match self.domain {
            ScriptObjectRenderGroupDomain::Atmosphere => self
                .object
                .enqueue_render_group_params(params, |target, params| {
                    Render3dMutationRequest::SetAtmosphereParams { target, params }
                }),
            ScriptObjectRenderGroupDomain::Surface => self
                .object
                .enqueue_render_group_params(params, |target, params| {
                    Render3dMutationRequest::SetSurfaceParams { target, params }
                }),
            ScriptObjectRenderGroupDomain::Generator => self
                .object
                .enqueue_render_group_params(params, |target, params| {
                    Render3dMutationRequest::SetGeneratorParams { target, params }
                }),
            ScriptObjectRenderGroupDomain::Body => self
                .object
                .enqueue_render_group_params(params, |target, params| {
                    Render3dMutationRequest::SetBodyParams { target, params }
                }),
            ScriptObjectRenderGroupDomain::View => self
                .object
                .enqueue_render_group_params(params, |target, params| {
                    Render3dMutationRequest::SetViewParams { target, params }
                }),
        }
    }
}

impl RuntimeSceneCoreApi<ScriptSceneObjectsApi> for ScriptSceneApi {
    fn objects(&mut self) -> ScriptSceneObjectsApi {
        ScriptSceneApi::objects(self)
    }
}

impl ObjectRegistryCoreApi<ScriptObjectApi> for ScriptSceneObjectsApi {
    fn find(&mut self, target: &str) -> ScriptObjectApi {
        ScriptSceneObjectsApi::find(self, target)
    }

    fn all(&mut self) -> RhaiArray {
        ScriptSceneObjectsApi::all(self)
    }

    fn by_tag(&mut self, tag: &str) -> RhaiArray {
        ScriptSceneObjectsApi::by_tag(self, tag)
    }

    fn by_name(&mut self, name: &str) -> RhaiArray {
        ScriptSceneObjectsApi::by_name(self, name)
    }
}

/// Register scene API into the Rhai engine.
pub fn register_scene_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptSceneApi>("SceneApi");
    engine.register_type_with_name::<ScriptSceneObjectsApi>("SceneObjects");
    engine.register_type_with_name::<ScriptObjectApi>("SceneObject");
    engine.register_type_with_name::<ScriptObjectTextApi>("SceneObjectText");
    engine.register_type_with_name::<ScriptObjectStyleApi>("SceneObjectStyle");
    engine.register_type_with_name::<ScriptObjectFrameApi>("SceneObjectFrame");
    engine.register_type_with_name::<ScriptObjectRenderApi>("SceneObjectRender");
    engine.register_type_with_name::<ScriptObjectRenderGroupApi>("SceneObjectRenderGroup");

    // Primary live-handle surfaces.
    engine.register_get("objects", |scene: &mut ScriptSceneApi| scene.objects());
    engine.register_fn("object", |scene: &mut ScriptSceneApi, target: &str| {
        scene.object(target)
    });
    // Snapshot reads stay detached from live-handle overlays.
    engine.register_fn("inspect", |scene: &mut ScriptSceneApi, target: &str| {
        scene.inspect(target)
    });
    engine.register_fn("region", |scene: &mut ScriptSceneApi, target: &str| {
        scene.region(target)
    });
    engine.register_fn(
        "instantiate",
        |scene: &mut ScriptSceneApi, template: &str, target: &str| scene.spawn(template, target),
    );
    engine.register_fn("despawn", |scene: &mut ScriptSceneApi, target: &str| {
        scene.despawn(target)
    });
    engine.register_fn("set_bg", |scene: &mut ScriptSceneApi, color: &str| {
        scene.set_bg(color);
    });
    engine.register_fn("mutate", |scene: &mut ScriptSceneApi, request: RhaiMap| {
        scene.mutate(request)
    });
    engine.register_fn(
        "set_camera3d_look_at",
        |scene: &mut ScriptSceneApi,
         ex: rhai::FLOAT,
         ey: rhai::FLOAT,
         ez: rhai::FLOAT,
         lx: rhai::FLOAT,
         ly: rhai::FLOAT,
         lz: rhai::FLOAT| {
            scene.set_camera3d_look_at(
                [ex as f32, ey as f32, ez as f32],
                [lx as f32, ly as f32, lz as f32],
            )
        },
    );
    engine.register_fn(
        "set_camera3d_up",
        |scene: &mut ScriptSceneApi, ux: rhai::FLOAT, uy: rhai::FLOAT, uz: rhai::FLOAT| {
            scene.set_camera3d_up([ux as f32, uy as f32, uz as f32])
        },
    );
    engine.register_fn(
        "set_camera3d_object_look_at",
        |scene: &mut ScriptSceneApi,
         target: &str,
         ex: rhai::FLOAT,
         ey: rhai::FLOAT,
         ez: rhai::FLOAT,
         lx: rhai::FLOAT,
         ly: rhai::FLOAT,
         lz: rhai::FLOAT| {
            scene.set_camera3d_object_look_at(
                target,
                [ex as f32, ey as f32, ez as f32],
                [lx as f32, ly as f32, lz as f32],
            )
        },
    );
    engine.register_fn(
        "set_camera3d_object_basis",
        |scene: &mut ScriptSceneApi,
         target: &str,
         ex: rhai::FLOAT,
         ey: rhai::FLOAT,
         ez: rhai::FLOAT,
         rx: rhai::FLOAT,
         ry: rhai::FLOAT,
         rz: rhai::FLOAT,
         ux: rhai::FLOAT,
         uy: rhai::FLOAT,
         uz: rhai::FLOAT,
         fx: rhai::FLOAT,
         fy: rhai::FLOAT,
         fz: rhai::FLOAT| {
            scene.set_camera3d_object_basis(
                target,
                [ex as f32, ey as f32, ez as f32],
                [rx as f32, ry as f32, rz as f32],
                [ux as f32, uy as f32, uz as f32],
                [fx as f32, fy as f32, fz as f32],
            )
        },
    );
    engine.register_fn(
        "set_view_profile",
        |scene: &mut ScriptSceneApi, profile: &str| scene.set_view_profile(profile),
    );
    engine.register_fn(
        "set_lighting_profile",
        |scene: &mut ScriptSceneApi, profile: &str| scene.set_lighting_profile(profile),
    );
    engine.register_fn(
        "set_space_environment_profile",
        |scene: &mut ScriptSceneApi, profile: &str| scene.set_space_environment_profile(profile),
    );
    engine.register_fn(
        "set_lighting_param",
        |scene: &mut ScriptSceneApi, name: &str, value: RhaiDynamic| {
            scene.set_lighting_param(name, value)
        },
    );
    engine.register_fn(
        "set_space_environment_param",
        |scene: &mut ScriptSceneApi, name: &str, value: RhaiDynamic| {
            scene.set_space_environment_param(name, value)
        },
    );
    engine.register_fn(
        "set_render3d_profile",
        |scene: &mut ScriptSceneApi, profile_slot: &str, profile: &str| {
            let Some(profile_slot) = render3d_profile_slot_from_str(profile_slot) else {
                return false;
            };
            scene.set_render3d_profile(profile_slot, profile)
        },
    );
    engine.register_fn(
        "set_render3d_profile_param",
        |scene: &mut ScriptSceneApi, profile_slot: &str, name: &str, value: RhaiDynamic| {
            let Some(profile_slot) = render3d_profile_slot_from_str(profile_slot) else {
                return false;
            };
            scene.set_render3d_profile_param(profile_slot, name, value)
        },
    );
    engine.register_fn(
        "set_material_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_material_params(target, params)
        },
    );
    engine.register_fn(
        "set_atmosphere_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_atmosphere_params(target, params)
        },
    );
    engine.register_fn(
        "set_surface_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_surface_params(target, params)
        },
    );
    engine.register_fn(
        "set_generator_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_generator_params(target, params)
        },
    );
    engine.register_fn(
        "set_body_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_body_params(target, params)
        },
    );
    engine.register_fn(
        "set_view_params",
        |scene: &mut ScriptSceneApi, target: &str, params: RhaiMap| {
            scene.set_view_params(target, params)
        },
    );

    engine.register_fn(
        "find",
        |objects: &mut ScriptSceneObjectsApi, target: &str| objects.find(target),
    );
    engine.register_fn("all", |objects: &mut ScriptSceneObjectsApi| objects.all());
    engine.register_fn(
        "by_tag",
        |objects: &mut ScriptSceneObjectsApi, tag: &str| objects.by_tag(tag),
    );
    engine.register_fn(
        "by_name",
        |objects: &mut ScriptSceneObjectsApi, name: &str| objects.by_name(name),
    );
    engine.register_fn("iter", |objects: &mut ScriptSceneObjectsApi| objects.iter());

    engine.register_get("id", |object: &mut ScriptObjectApi| object.id());
    engine.register_get("text", |object: &mut ScriptObjectApi| object.text());
    engine.register_get("style", |object: &mut ScriptObjectApi| object.style());
    engine.register_get("frame", |object: &mut ScriptObjectApi| object.frame());
    engine.register_get("render", |object: &mut ScriptObjectApi| object.render());
    engine.register_fn("get", |object: &mut ScriptObjectApi, path: &str| {
        object.get(path)
    });
    engine.register_fn(
        "set",
        |object: &mut ScriptObjectApi, path: &str, value: RhaiDynamic| object.set(path, value),
    );

    engine.register_get("content", |text: &mut ScriptObjectTextApi| text.content());
    engine.register_set("content", |text: &mut ScriptObjectTextApi, value: &str| {
        let _ = text.set_content(value);
    });
    engine.register_get("fg", |text: &mut ScriptObjectTextApi| text.fg());
    engine.register_set("fg", |text: &mut ScriptObjectTextApi, value: &str| {
        let _ = text.set_fg(value);
    });
    engine.register_get("bg", |text: &mut ScriptObjectTextApi| text.bg());
    engine.register_set("bg", |text: &mut ScriptObjectTextApi, value: &str| {
        let _ = text.set_bg(value);
    });
    engine.register_get("font", |text: &mut ScriptObjectTextApi| text.font());
    engine.register_set("font", |text: &mut ScriptObjectTextApi, value: &str| {
        let _ = text.set_font(value);
    });
    engine.register_fn(
        "set_content",
        |text: &mut ScriptObjectTextApi, value: &str| text.set_content(value),
    );

    engine.register_get("fg", |style: &mut ScriptObjectStyleApi| style.fg());
    engine.register_set("fg", |style: &mut ScriptObjectStyleApi, value: &str| {
        let _ = style.set_fg(value);
    });
    engine.register_get("bg", |style: &mut ScriptObjectStyleApi| style.bg());
    engine.register_set("bg", |style: &mut ScriptObjectStyleApi, value: &str| {
        let _ = style.set_bg(value);
    });

    engine.register_get("visible", |frame: &mut ScriptObjectFrameApi| {
        frame.visible()
    });
    engine.register_set(
        "visible",
        |frame: &mut ScriptObjectFrameApi, value: bool| {
            let _ = frame.set_visible(value);
        },
    );
    engine.register_get("dx", |frame: &mut ScriptObjectFrameApi| frame.dx());
    engine.register_set(
        "dx",
        |frame: &mut ScriptObjectFrameApi, value: rhai::FLOAT| {
            let _ = frame.set_dx(value);
        },
    );
    engine.register_get("dy", |frame: &mut ScriptObjectFrameApi| frame.dy());
    engine.register_set(
        "dy",
        |frame: &mut ScriptObjectFrameApi, value: rhai::FLOAT| {
            let _ = frame.set_dy(value);
        },
    );
    engine.register_fn(
        "set_offset",
        |frame: &mut ScriptObjectFrameApi, dx: rhai::FLOAT, dy: rhai::FLOAT| {
            frame.set_offset(dx, dy)
        },
    );

    engine.register_get("surface_mode", |render: &mut ScriptObjectRenderApi| {
        render.surface_mode()
    });
    engine.register_set(
        "surface_mode",
        |render: &mut ScriptObjectRenderApi, value: &str| {
            let _ = render.set_surface_mode(value);
        },
    );
    engine.register_get("atmosphere", |render: &mut ScriptObjectRenderApi| {
        render.atmosphere()
    });
    engine.register_get("surface", |render: &mut ScriptObjectRenderApi| {
        render.surface()
    });
    engine.register_get("generator", |render: &mut ScriptObjectRenderApi| {
        render.generator()
    });
    engine.register_get("body", |render: &mut ScriptObjectRenderApi| render.body());
    engine.register_get("view", |render: &mut ScriptObjectRenderApi| render.view());
    engine.register_fn(
        "set",
        |group: &mut ScriptObjectRenderGroupApi, name: &str, value: RhaiDynamic| {
            group.set(name, value)
        },
    );
    engine.register_fn(
        "set_many",
        |group: &mut ScriptObjectRenderGroupApi, params: RhaiMap| group.set_many(params),
    );
}

#[cfg(test)]
mod tests {
    use super::{register_scene_api, ScriptSceneApi};
    use crate::rhai::conversion::map_set_path_dynamic;
    use crate::{
        BehaviorCommand, Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot,
        SceneMutationRequest,
    };
    use engine_core::effects::Region;
    use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
    use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn build_api(
        object_states: HashMap<String, ObjectRuntimeState>,
        object_kinds: HashMap<String, String>,
        object_props: HashMap<String, serde_json::Value>,
        object_regions: HashMap<String, Region>,
        object_text: HashMap<String, String>,
        resolver: TargetResolver,
        queue: &Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> ScriptSceneApi {
        ScriptSceneApi::new(
            Arc::new(object_states),
            Arc::new(object_kinds),
            Arc::new(object_props),
            Arc::new(object_regions),
            Arc::new(object_text),
            Arc::new(resolver),
            Arc::clone(queue),
        )
    }

    #[test]
    fn mutate_enqueues_typed_scene_mutation_command() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        let mut request = RhaiMap::new();
        request.insert("type".into(), "set_camera2d".into());
        request.insert("x".into(), RhaiDynamic::from_float(10.0));
        request.insert("y".into(), RhaiDynamic::from_float(20.0));

        assert!(api.mutate(request));
        let queue = queue.lock().expect("queue lock");
        assert!(matches!(
            queue.first(),
            Some(BehaviorCommand::ApplySceneMutation { .. })
        ));
    }

    #[test]
    fn set_camera3d_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set_camera3d_look_at([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]));
        assert!(api.set_camera3d_up([0.0, 1.0, 0.0]));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::LookAt {
                    eye: [1.0, 2.0, 3.0],
                    look_at: [0.0, 0.0, 0.0],
                }),
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::Up {
                    up: [0.0, 1.0, 0.0],
                }),
            }
        );
    }

    #[test]
    fn set_camera3d_object_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("cockpit".to_string(), "camera-cockpit".to_string());
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            resolver,
            &queue,
        );

        assert!(api.set_camera3d_object_look_at("cockpit", [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]));
        assert!(api.set_camera3d_object_basis(
            "cockpit",
            [1.0, 2.0, 3.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectLookAt {
                    target: "camera-cockpit".to_string(),
                    eye: [1.0, 2.0, 3.0],
                    look_at: [0.0, 0.0, 0.0],
                    up: None,
                }),
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectBasis {
                    target: "camera-cockpit".to_string(),
                    eye: [1.0, 2.0, 3.0],
                    right: [1.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    forward: [0.0, 0.0, 1.0],
                }),
            }
        );
    }

    #[test]
    fn set_view_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set_view_profile("orbit-realistic"));
        assert!(api.set_lighting_profile("space-hard-vacuum"));
        assert!(api.set_space_environment_profile("deep-space-sparse"));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn set_view_param_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set_lighting_param("exposure", 0.88.into()));
        assert!(api.set_space_environment_param("background_color", "#010203".into()));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetLightingParam {
                        name: "exposure".to_string(),
                        value: serde_json::json!(0.88),
                    },
                ),
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetSpaceEnvironmentParam {
                        name: "background_color".to_string(),
                        value: serde_json::json!("#010203"),
                    },
                ),
            }
        );
    }

    #[test]
    fn set_neutral_profile_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set_render3d_profile(Render3dProfileSlot::Lighting, "space-hard-vacuum"));
        assert!(api.set_render3d_profile_param(
            Render3dProfileSlot::SpaceEnvironment,
            "background_color",
            "#010203".into()
        ));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetProfile {
                    profile_slot: Render3dProfileSlot::Lighting,
                    profile: "space-hard-vacuum".to_string(),
                },),
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetProfileParam {
                        profile_slot: Render3dProfileSlot::SpaceEnvironment,
                        name: "background_color".to_string(),
                        value: serde_json::json!("#010203"),
                    },
                ),
            }
        );
    }

    #[test]
    fn set_grouped_render_param_helpers_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        let mut material = RhaiMap::new();
        material.insert("roughness".into(), 0.25.into());
        material.insert("specular".into(), 0.6.into());
        let mut atmosphere = RhaiMap::new();
        atmosphere.insert("density".into(), 0.4.into());
        let mut surface = RhaiMap::new();
        surface.insert("terrain_relief".into(), 0.8.into());
        let mut generator = RhaiMap::new();
        generator.insert("noise_seed".into(), 42.into());
        let mut body = RhaiMap::new();
        body.insert("rotation_deg".into(), 12.0.into());
        let mut view = RhaiMap::new();
        view.insert("distance".into(), 9.5.into());

        assert!(api.set_material_params("planet-main", material));
        assert!(api.set_atmosphere_params("planet-main", atmosphere));
        assert!(api.set_surface_params("planet-main", surface));
        assert!(api.set_generator_params("planet-main", generator));
        assert!(api.set_body_params("planet-main", body));
        assert!(api.set_view_params("planet-main", view));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 6);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetMaterialParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "roughness": 0.25,
                            "specular": 0.6
                        }),
                    },
                ),
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetAtmosphereParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "density": 0.4
                        }),
                    },
                ),
            }
        );
        assert_eq!(
            queue[2],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetSurfaceParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "terrain_relief": 0.8
                        }),
                    },
                ),
            }
        );
        assert_eq!(
            queue[3],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetGeneratorParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "noise_seed": 42
                        }),
                    },
                ),
            }
        );
        assert_eq!(
            queue[4],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetBodyParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "rotation_deg": 12.0
                        }),
                    },
                ),
            }
        );
        assert_eq!(
            queue[5],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetViewParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({
                            "distance": 9.5
                        }),
                    },
                ),
            }
        );
    }

    #[test]
    fn set_grouped_render_param_helpers_reject_empty_target_or_params() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(!api.set_material_params("", RhaiMap::new()));
        assert!(!api.set_material_params("planet-main", RhaiMap::new()));

        let queue = queue.lock().expect("queue lock");
        assert!(queue.is_empty());
    }

    #[test]
    fn spawn_and_despawn_enqueue_typed_mutations() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.spawn("enemy-basic", "enemy-01"));
        assert!(api.despawn("enemy-01"));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SpawnObject {
                    template: "enemy-basic".to_string(),
                    target: "enemy-01".to_string(),
                },
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::DespawnObject {
                    target: "enemy-01".to_string(),
                },
            }
        );
    }

    #[test]
    fn set_routes_render3d_paths_to_typed_mutation() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        api.set("planet-view", "obj.world.x", RhaiDynamic::from_float(2.5));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetMaterialParams {
                        target: "planet-view".to_string(),
                        params: serde_json::json!({
                            "world.x": 2.5
                        }),
                    },
                ),
            }
        );
    }

    #[test]
    fn set_routes_text_content_to_typed_2d_mutation() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set("title", "text.content", "HELLO".into()));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "title".to_string(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some("HELLO".to_string()),
                },
            }
        );
    }

    #[test]
    fn set_routes_position_y_to_typed_2d_mutation_when_state_is_available() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::<String, ObjectRuntimeState>::new();
        object_states.insert(
            "title".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 4,
                ..ObjectRuntimeState::default()
            },
        );
        let mut api = build_api(
            object_states,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set("title", "position.y", 9.into()));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "title".to_string(),
                    visible: None,
                    dx: None,
                    dy: Some(5),
                    text: None,
                },
            }
        );
    }

    #[test]
    fn object_set_routes_position_y_to_typed_2d_mutation_when_live_handle_has_state() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::<String, ObjectRuntimeState>::new();
        object_states.insert(
            "title".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 2,
                ..ObjectRuntimeState::default()
            },
        );
        let mut api = build_api(
            object_states,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );
        let mut object = api.object("title");

        assert!(object.set("position.y", 6.into()));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "title".to_string(),
                    visible: None,
                    dx: None,
                    dy: Some(4),
                    text: None,
                },
            }
        );
    }

    #[test]
    fn set_reports_unsupported_paths_without_enqueuing_commands() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(!api.set("title", "audio.pitch", 2.0.into()));

        let queue = queue.lock().expect("queue lock");
        assert!(queue.is_empty());
    }

    #[test]
    fn object_set_reports_unsupported_paths_without_enqueuing_commands() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::<String, ObjectRuntimeState>::new();
        object_states.insert("title".to_string(), ObjectRuntimeState::default());
        let mut api = build_api(
            object_states,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );
        let mut object = api.object("title");

        assert!(!object.set("audio.pitch", 2.0.into()));

        let queue = queue.lock().expect("queue lock");
        assert!(queue.is_empty());
    }

    #[test]
    fn object_set_rejects_unsupported_paths_without_leaking_live_overlay_state() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::<String, ObjectRuntimeState>::new();
        object_states.insert("title".to_string(), ObjectRuntimeState::default());
        let mut api = build_api(
            object_states,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );
        let mut object = api.object("title");

        assert!(!object.set("audio.pitch", 2.0.into()));
        assert!(object.get("audio.pitch").is_unit());

        let queue = queue.lock().expect("queue lock");
        assert!(queue.is_empty());
    }

    #[test]
    fn inspect_and_region_return_runtime_snapshot_maps() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 2,
                offset_y: 3,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert(
            "runtime-score".to_string(),
            Region {
                x: 10,
                y: 20,
                width: 30,
                height: 4,
            },
        );
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());

        let mut api = build_api(
            object_states,
            object_kinds,
            HashMap::new(),
            object_regions,
            object_text,
            resolver,
            &queue,
        );

        let inspect = api.inspect("hud-score");
        let region = api.region("hud-score");

        assert_eq!(
            inspect
                .get("id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("runtime-score".to_string())
        );
        assert_eq!(
            inspect
                .get("text")
                .and_then(|value| value.clone().try_cast::<RhaiMap>())
                .and_then(|map| map.get("content").cloned())
                .and_then(|value| value.try_cast::<String>()),
            Some("42".to_string())
        );
        assert_eq!(
            region
                .get("width")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(30)
        );
        assert_eq!(
            inspect
                .get("capabilities")
                .and_then(|value| value.clone().try_cast::<RhaiMap>())
                .and_then(|map| map.get("text.content").cloned())
                .and_then(|value| value.try_cast::<bool>()),
            Some(true)
        );
    }

    #[test]
    fn inspect_returns_detached_snapshot_even_when_locally_mutated() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 2,
                offset_y: 3,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        let mut api = build_api(
            object_states,
            object_kinds,
            HashMap::new(),
            HashMap::new(),
            object_text,
            resolver,
            &queue,
        );

        let mut inspect = api.inspect("hud-score");
        assert!(map_set_path_dynamic(
            &mut inspect,
            "text.content",
            "mutated-locally".into()
        ));

        let mut live = api.object("hud-score");
        let live_text = live
            .get("text.content")
            .try_cast::<String>()
            .expect("live text content");
        let inspect_text = inspect
            .get("text")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .and_then(|map| map.get("content").cloned())
            .and_then(|value| value.try_cast::<String>())
            .expect("inspect text content");

        assert_eq!(inspect_text, "mutated-locally");
        assert_eq!(live_text, "42");
    }

    #[test]
    fn registered_scene_object_entrypoint_is_primary_live_handle_surface() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 2,
                offset_y: 3,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        let api = build_api(
            object_states,
            object_kinds,
            HashMap::new(),
            HashMap::new(),
            object_text,
            resolver,
            &queue,
        );
        let mut engine = RhaiEngine::new();
        register_scene_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("scene", api.clone());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let handle = scene.object("hud-score");
                    handle.set("text.content", "108");

                    #{
                        via_object: scene.object("hud-score").get("text.content"),
                        via_find: scene.objects.find("hud-score").get("text.content"),
                        inspect: scene.inspect("hud-score")
                    }
                "#,
            )
            .expect("registered scene.object entrypoint should resolve");

        let via_object = result
            .get("via_object")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("via_object text");
        let via_find = result
            .get("via_find")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("via_find text");
        let inspect = result
            .get("inspect")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("inspect map");
        let inspect_text = inspect
            .get("text")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .and_then(|map| map.get("content").cloned())
            .and_then(|value| value.try_cast::<String>())
            .expect("inspect text");

        assert_eq!(via_object, "108");
        assert_eq!(via_find, "108");
        assert_eq!(inspect_text, "42");
    }

    #[test]
    fn registered_scene_object_subhandles_support_property_style_updates() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        object_kinds.insert("planet-main".to_string(), "mesh".to_string());
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        let api = build_api(
            object_states,
            object_kinds,
            HashMap::new(),
            HashMap::new(),
            object_text,
            resolver,
            &queue,
        );
        let mut engine = RhaiEngine::new();
        register_scene_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("scene", api);

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let hud = scene.object("hud-score");
                    hud.text.content = "108";
                    hud.style.fg = "amber";
                    hud.frame.visible = false;

                    let planet = scene.object("planet-main");
                    planet.render.surface_mode = "material";
                    planet.render.atmosphere.set("height", 0.11);
                    planet.render.view.set_many(#{ distance: 9.5 });

                    #{
                        text: hud.text.content,
                        fg: hud.style.fg,
                        visible: hud.frame.visible,
                        surface_mode: planet.render.surface_mode
                    }
                "#,
            )
            .expect("scene object subhandles should work");

        assert_eq!(
            result
                .get("text")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("108".to_string())
        );
        assert_eq!(
            result
                .get("fg")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("amber".to_string())
        );
        assert_eq!(
            result
                .get("visible")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(false)
        );
        assert_eq!(
            result
                .get("surface_mode")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("material".to_string())
        );

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 6);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "runtime-score".to_string(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some("108".to_string()),
                },
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty {
                    target: "runtime-score".to_string(),
                    path: "style.fg".to_string(),
                    value: serde_json::json!("amber"),
                },
            }
        );
        assert_eq!(
            queue[2],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "runtime-score".to_string(),
                    visible: Some(false),
                    dx: None,
                    dy: None,
                    text: None,
                },
            }
        );
        assert_eq!(
            queue[3],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetSurfaceMode {
                        target: "planet-main".to_string(),
                        mode: "material".to_string(),
                    }
                ),
            }
        );
        assert_eq!(
            queue[4],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetAtmosphereParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({ "height": 0.11, }),
                    }
                ),
            }
        );
        assert_eq!(
            queue[5],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetViewParams {
                        target: "planet-main".to_string(),
                        params: serde_json::json!({ "distance": 9.5 }),
                    }
                ),
            }
        );
    }

    #[test]
    fn scene_objects_live_handles_share_pending_updates_while_inspect_stays_snapshot() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 2,
                offset_y: 3,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        let mut api = build_api(
            object_states,
            object_kinds,
            HashMap::new(),
            HashMap::new(),
            object_text,
            resolver,
            &queue,
        );

        let mut first = api.objects().find("hud-score");
        first.set("text.content", "108".into());

        let mut second = api.object("hud-score");
        let pending_text = second
            .get("text.content")
            .try_cast::<String>()
            .expect("pending text content");
        let inspect = api.inspect("hud-score");
        let inspect_text = inspect
            .get("text")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .and_then(|map| map.get("content").cloned())
            .and_then(|value| value.try_cast::<String>())
            .expect("inspect text content");

        assert_eq!(pending_text, "108");
        assert_eq!(inspect_text, "42");

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "runtime-score".to_string(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some("108".to_string()),
                },
            }
        );
    }

    #[test]
    fn scene_objects_all_and_iter_return_sorted_live_handles() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut object_states = HashMap::new();
        object_states.insert(
            "beta".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
                ..ObjectRuntimeState::default()
            },
        );
        object_states.insert(
            "alpha".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 1,
                offset_y: 2,
                ..ObjectRuntimeState::default()
            },
        );

        let mut api = build_api(
            object_states,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        let all_ids: Vec<String> = api
            .objects()
            .all()
            .into_iter()
            .map(|value| {
                let mut object = value
                    .try_cast::<super::ScriptObjectApi>()
                    .expect("scene object");
                object.get("id").try_cast::<String>().expect("object id")
            })
            .collect();
        let iter_ids: Vec<String> = api
            .objects()
            .iter()
            .into_iter()
            .map(|value| {
                let mut object = value
                    .try_cast::<super::ScriptObjectApi>()
                    .expect("scene object");
                object.get("id").try_cast::<String>().expect("object id")
            })
            .collect();

        assert_eq!(all_ids, vec!["alpha".to_string(), "beta".to_string()]);
        assert_eq!(iter_ids, all_ids);
    }

    #[test]
    fn scene_objects_by_tag_and_by_name_return_filtered_live_handles() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let object_id = "scene-root/layer:ui/text:score".to_string();
        let mut object_states = HashMap::new();
        object_states.insert(object_id.clone(), ObjectRuntimeState::default());

        let mut object_kinds = HashMap::new();
        object_kinds.insert(object_id.clone(), "text".to_string());

        let mut object_props = HashMap::new();
        object_props.insert(
            object_id.clone(),
            serde_json::json!({
                "tags": ["hud", "ui"]
            }),
        );

        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), object_id.clone());

        let mut api = build_api(
            object_states,
            object_kinds,
            object_props,
            HashMap::new(),
            HashMap::new(),
            resolver,
            &queue,
        );

        let by_tag_ids: Vec<String> = api
            .objects()
            .by_tag("hud")
            .into_iter()
            .map(|value| {
                let mut object = value
                    .try_cast::<super::ScriptObjectApi>()
                    .expect("scene object");
                object.get("id").try_cast::<String>().expect("object id")
            })
            .collect();
        let by_kind_ids: Vec<String> = api
            .objects()
            .by_tag("text")
            .into_iter()
            .map(|value| {
                let mut object = value
                    .try_cast::<super::ScriptObjectApi>()
                    .expect("scene object");
                object.get("id").try_cast::<String>().expect("object id")
            })
            .collect();
        let by_name_ids: Vec<String> = api
            .objects()
            .by_name("text:score")
            .into_iter()
            .map(|value| {
                let mut object = value
                    .try_cast::<super::ScriptObjectApi>()
                    .expect("scene object");
                object.get("id").try_cast::<String>().expect("object id")
            })
            .collect();

        assert_eq!(by_tag_ids, vec![object_id.clone()]);
        assert_eq!(by_kind_ids, vec![object_id.clone()]);
        assert_eq!(by_name_ids, vec![object_id]);
    }

}
