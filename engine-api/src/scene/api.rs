//! Scene domain API: ScriptSceneApi for scene object management, ScriptObjectApi for individual object state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::Value as JsonValue;

use crate::rhai::conversion::{
    json_to_rhai_dynamic, map_get_path_dynamic, map_set_path_dynamic, merge_rhai_maps,
    normalize_set_path, rhai_dynamic_to_json,
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

/// Helpers for object state conversion (should ideally be shared or generic).
fn object_state_to_rhai_map(state: &ObjectRuntimeState) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("visible".into(), state.visible.into());
    map.insert("offset_x".into(), (state.offset_x as rhai::INT).into());
    map.insert("offset_y".into(), (state.offset_y as rhai::INT).into());
    map
}

fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("x".into(), (region.x as rhai::INT).into());
    map.insert("y".into(), (region.y as rhai::INT).into());
    map.insert("width".into(), (region.width as rhai::INT).into());
    map.insert("height".into(), (region.height as rhai::INT).into());
    map
}

fn kind_capabilities(kind: Option<&str>) -> RhaiMap {
    let mut cap = RhaiMap::new();
    // Add generic capabilities available to all kinds
    cap.insert("visible".into(), true.into());
    cap.insert("offset.x".into(), true.into());
    cap.insert("offset.y".into(), true.into());
    cap.insert("position.x".into(), true.into());
    cap.insert("position.y".into(), true.into());

    // Kind-specific capabilities
    if let Some(k) = kind {
        match k {
            "text" => {
                cap.insert("text.content".into(), true.into());
                cap.insert("text.font".into(), true.into());
                cap.insert("style.fg".into(), true.into());
                cap.insert("style.bg".into(), true.into());
            }
            "obj" => {
                cap.insert("obj.scale".into(), true.into());
                cap.insert("obj.yaw".into(), true.into());
                cap.insert("obj.pitch".into(), true.into());
                cap.insert("obj.roll".into(), true.into());
                cap.insert("obj.orbit_speed".into(), true.into());
                cap.insert("obj.surface_mode".into(), true.into());
                cap.insert("obj.world.x".into(), true.into());
                cap.insert("obj.world.y".into(), true.into());
                cap.insert("obj.world.z".into(), true.into());
            }
            _ => {}
        }
    }
    cap
}

/// Script-facing API for scene management.
#[derive(Clone)]
pub struct ScriptSceneApi {
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

/// Script-facing API for individual scene objects.
#[derive(Clone)]
pub struct ScriptObjectApi {
    target: String,
    snapshot: RhaiMap,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
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

    /// Get a single scene object API by target (alias or ID).
    pub fn get(&mut self, target: &str) -> ScriptObjectApi {
        let object_id = self.resolve_target(target);
        let snapshot = self.build_object_entry(&object_id);
        ScriptObjectApi {
            target: object_id,
            snapshot,
            queue: Arc::clone(&self.queue),
        }
    }

    /// Returns a snapshot map for the resolved object id.
    pub fn inspect(&mut self, target: &str) -> RhaiMap {
        self.build_object_entry(&self.resolve_target(target))
    }

    /// Returns the runtime region map for the resolved object id.
    pub fn region(&mut self, target: &str) -> RhaiMap {
        self.object_regions
            .get(&self.resolve_target(target))
            .map(region_to_rhai_map)
            .unwrap_or_default()
    }

    fn build_object_entry(&self, object_id: &str) -> RhaiMap {
        let Some(state) = self.object_states.get(object_id) else {
            return RhaiMap::new();
        };
        let kind = self
            .object_kinds
            .get(object_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut entry = RhaiMap::new();
        entry.insert("id".into(), object_id.to_string().into());
        entry.insert("kind".into(), kind.clone().into());
        entry.insert("state".into(), object_state_to_rhai_map(state).into());
        if let Some(region) = self.object_regions.get(object_id) {
            entry.insert("region".into(), region_to_rhai_map(region).into());
        }
        if let Some(text) = self.object_text.get(object_id) {
            let mut text_map = RhaiMap::new();
            text_map.insert("content".into(), text.clone().into());
            entry.insert("text".into(), text_map.into());
        }
        let mut props = RhaiMap::new();
        props.insert("visible".into(), state.visible.into());
        let mut offset = RhaiMap::new();
        offset.insert("x".into(), (state.offset_x as rhai::INT).into());
        offset.insert("y".into(), (state.offset_y as rhai::INT).into());
        props.insert("offset".into(), offset.into());
        if let Some(text) = self.object_text.get(object_id) {
            let mut text_props = RhaiMap::new();
            text_props.insert("content".into(), text.clone().into());
            props.insert("text".into(), text_props.into());
        }
        if let Some(extra_props) = self.object_props.get(object_id) {
            if let Some(extra_map) = json_to_rhai_dynamic(extra_props).try_cast::<RhaiMap>() {
                merge_rhai_maps(&mut props, &extra_map);
            }
        }
        entry.insert("props".into(), props.into());
        entry.insert(
            "capabilities".into(),
            kind_capabilities(Some(kind.as_str())).into(),
        );
        entry
    }

    /// Set a property on a scene object.
    pub fn set(&mut self, target: &str, path: &str, value: RhaiDynamic) {
        let normalized_path = normalize_set_path(path);
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        let resolved = self.resolve_target(target);
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        if let Some(request) = crate::commands::scene_mutation_request_from_set_path(
            &resolved,
            &normalized_path,
            &value,
            self.object_states.get(&resolved),
        ) {
            queue.push(BehaviorCommand::ApplySceneMutation { request });
        }
    }

    /// Set the text body of a scene text object.
    pub fn set_text(&mut self, id: &str, text: &str) -> bool {
        let resolved = self.resolve_target(id);
        self.enqueue_scene_mutation(SceneMutationRequest::Set2dProps {
            target: resolved,
            visible: None,
            dx: None,
            dy: None,
            text: Some(text.to_string()),
        })
    }

    /// Set common text style fields using a small ergonomic map.
    pub fn set_text_style(&mut self, id: &str, style: RhaiMap) -> bool {
        if style.is_empty() {
            return false;
        }
        let resolved = self.resolve_target(id);
        let mut queued_any = false;
        let current_state = self.object_states.get(&resolved);
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };

        for (aliases, normalized_path) in [
            (&["fg", "style.fg", "text.fg"][..], "style.fg"),
            (&["bg", "style.bg", "text.bg"][..], "style.bg"),
            (&["font", "text.font"][..], "text.font"),
        ] {
            let Some(value) = aliases.iter().find_map(|key| style.get(*key)) else {
                continue;
            };
            let Some(value) = rhai_dynamic_to_json(value) else {
                continue;
            };
            let Some(request) = crate::commands::scene_mutation_request_from_set_path(
                &resolved,
                normalized_path,
                &value,
                current_state,
            ) else {
                continue;
            };
            queue.push(BehaviorCommand::ApplySceneMutation { request });
            queued_any = true;
        }

        queued_any
    }
    ///
    /// ```rhai
    /// scene.set_multi(["star-0", "star-1", ..., "star-19"], "style.fg", col);
    /// ```
    pub fn set_multi(&mut self, targets: RhaiDynamic, path: &str, value: RhaiDynamic) {
        let Ok(arr) = targets.into_array() else {
            return;
        };
        let normalized_path = normalize_set_path(path);
        let Some(json_value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        for t in arr {
            let Ok(target_str) = t.into_string() else {
                continue;
            };
            let resolved = self
                .target_resolver
                .resolve_alias(&target_str)
                .unwrap_or(&target_str)
                .to_string();
            if let Some(request) = crate::commands::scene_mutation_request_from_set_path(
                &resolved,
                &normalized_path,
                &json_value,
                self.object_states.get(&resolved),
            ) {
                queue.push(BehaviorCommand::ApplySceneMutation { request });
            }
        }
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

    /// Set vector sprite points and colors.
    pub fn set_vector(&mut self, id: &str, points: RhaiDynamic, fg: &str, bg: &str) {
        self.set(id, "vector.points", points);
        self.set(id, "vector.fg", fg.to_string().into());
        self.set(id, "vector.bg", bg.to_string().into());
    }

    /// Set object visibility.
    pub fn set_visible(&mut self, id: &str, visible: bool) {
        let resolved = self.resolve_target(id);
        let _ = self.enqueue_scene_mutation(SceneMutationRequest::Set2dProps {
            target: resolved,
            visible: Some(visible),
            dx: None,
            dy: None,
            text: None,
        });
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

    /// Batch set multiple properties on a scene object.
    pub fn batch(&mut self, id: &str, props: RhaiMap) {
        for (key, value) in props {
            self.set(id, key.as_str(), value);
        }
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

impl ScriptObjectApi {
    /// Get a property from the object.
    pub fn get(&mut self, path: &str) -> RhaiDynamic {
        map_get_path_dynamic(&self.snapshot, path)
            .or_else(|| map_get_path_dynamic(&self.snapshot, &format!("props.{path}")))
            .unwrap_or_else(|| ().into())
    }

    /// Set a property on the object.
    pub fn set(&mut self, path: &str, value: RhaiDynamic) {
        let normalized_path = normalize_set_path(path);
        if !map_set_path_dynamic(&mut self.snapshot, &normalized_path, value.clone()) {
            return;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        let object_state = map_get_path_dynamic(&self.snapshot, "state")
            .and_then(|state| state.try_cast::<RhaiMap>())
            .and_then(|state| {
                let visible = state.get("visible")?.clone().try_cast::<bool>()?;
                let offset_x = state
                    .get("offset_x")?
                    .clone()
                    .try_cast::<rhai::INT>()
                    .and_then(|value| i32::try_from(value).ok())?;
                let offset_y = state
                    .get("offset_y")?
                    .clone()
                    .try_cast::<rhai::INT>()
                    .and_then(|value| i32::try_from(value).ok())?;
                Some(ObjectRuntimeState {
                    visible,
                    offset_x,
                    offset_y,
                    ..ObjectRuntimeState::default()
                })
            });
        if let Some(request) = crate::commands::scene_mutation_request_from_set_path(
            &self.target,
            &normalized_path,
            &value,
            object_state.as_ref(),
        ) {
            queue.push(BehaviorCommand::ApplySceneMutation { request });
        }
    }
}

/// Register scene API into the Rhai engine.
pub fn register_scene_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptSceneApi>("SceneApi");
    engine.register_type_with_name::<ScriptObjectApi>("SceneObject");

    engine.register_fn("get", |scene: &mut ScriptSceneApi, target: &str| {
        scene.get(target)
    });
    engine.register_fn("inspect", |scene: &mut ScriptSceneApi, target: &str| {
        scene.inspect(target)
    });
    engine.register_fn("region", |scene: &mut ScriptSceneApi, target: &str| {
        scene.region(target)
    });
    engine.register_fn(
        "set",
        |scene: &mut ScriptSceneApi, target: &str, path: &str, value: RhaiDynamic| {
            scene.set(target, path, value);
        },
    );
    engine.register_fn(
        "set_multi",
        |scene: &mut ScriptSceneApi, targets: RhaiDynamic, path: &str, value: RhaiDynamic| {
            scene.set_multi(targets, path, value);
        },
    );
    engine.register_fn(
        "set_text",
        |scene: &mut ScriptSceneApi, id: &str, text: &str| scene.set_text(id, text),
    );
    engine.register_fn(
        "set_text_style",
        |scene: &mut ScriptSceneApi, id: &str, style: RhaiMap| scene.set_text_style(id, style),
    );
    engine.register_fn(
        "spawn_object",
        |scene: &mut ScriptSceneApi, template: &str, target: &str| scene.spawn(template, target),
    );
    engine.register_fn(
        "despawn_object",
        |scene: &mut ScriptSceneApi, target: &str| scene.despawn(target),
    );
    engine.register_fn(
        "set_vector",
        |scene: &mut ScriptSceneApi, id: &str, points: RhaiDynamic, fg: &str, bg: &str| {
            scene.set_vector(id, points, fg, bg);
        },
    );
    engine.register_fn(
        "set_visible",
        |scene: &mut ScriptSceneApi, id: &str, visible: bool| {
            scene.set_visible(id, visible);
        },
    );
    engine.register_fn("set_bg", |scene: &mut ScriptSceneApi, color: &str| {
        scene.set_bg(color);
    });
    engine.register_fn(
        "batch",
        |scene: &mut ScriptSceneApi, id: &str, props: RhaiMap| {
            scene.batch(id, props);
        },
    );
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

    engine.register_fn("get", |object: &mut ScriptObjectApi, path: &str| {
        object.get(path)
    });
    engine.register_fn(
        "set",
        |object: &mut ScriptObjectApi, path: &str, value: RhaiDynamic| {
            object.set(path, value);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::ScriptSceneApi;
    use crate::rhai::conversion::map_set_path_dynamic;
    use crate::{
        BehaviorCommand, Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot,
        SceneMutationRequest,
    };
    use engine_core::effects::Region;
    use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
    use rhai::{Dynamic as RhaiDynamic, Map as RhaiMap};
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
    fn set_visible_enqueues_typed_2d_mutation() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud".to_string(), "scene-root/layer:0:ui/hud".to_string());
        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            resolver,
            &queue,
        );

        api.set_visible("hud", false);

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "scene-root/layer:0:ui/hud".to_string(),
                    visible: Some(false),
                    dx: None,
                    dy: None,
                    text: None,
                },
            }
        );
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

        api.set("title", "text.content", "HELLO".into());

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

        api.set("title", "position.y", 9.into());

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
    fn object_set_routes_position_y_to_typed_2d_mutation_when_snapshot_has_state() {
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
        let mut object = api.get("title");
        map_set_path_dynamic(&mut object.snapshot, "state.visible", true.into());
        map_set_path_dynamic(&mut object.snapshot, "state.offset_x", 0.into());
        map_set_path_dynamic(&mut object.snapshot, "state.offset_y", 2.into());

        object.set("position.y", 6.into());

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
    fn set_drops_unsupported_paths_without_enqueuing_commands() {
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

        api.set("title", "audio.pitch", 2.0.into());

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
    fn set_text_enqueues_typed_2d_mutation() {
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

        assert!(api.set_text("hud-score", "108"));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target: "hud-score".to_string(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some("108".to_string()),
                },
            }
        );
    }

    #[test]
    fn set_text_style_enqueues_supported_properties() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut style = RhaiMap::new();
        style.insert("fg".into(), "amber".into());
        style.insert("bg".into(), "black".into());
        style.insert("font".into(), "generic:2".into());

        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        assert!(api.set_text_style("hud-score", style));

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 3);
        assert_eq!(
            queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty {
                    target: "hud-score".to_string(),
                    path: "style.fg".to_string(),
                    value: serde_json::json!("amber"),
                },
            }
        );
        assert_eq!(
            queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty {
                    target: "hud-score".to_string(),
                    path: "style.bg".to_string(),
                    value: serde_json::json!("black"),
                },
            }
        );
        assert_eq!(
            queue[2],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty {
                    target: "hud-score".to_string(),
                    path: "text.font".to_string(),
                    value: serde_json::json!("generic:2"),
                },
            }
        );
    }

    #[test]
    fn set_multi_routes_text_style_updates_for_each_target() {
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

        api.set_multi(
            RhaiDynamic::from_array(vec![
                RhaiDynamic::from("score-left"),
                RhaiDynamic::from("score-right"),
            ]),
            "style.fg",
            "amber".into(),
        );

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert!(matches!(
            &queue[0],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty { target, path, .. }
            } if target == "score-left" && path == "style.fg"
        ));
        assert!(matches!(
            &queue[1],
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty { target, path, .. }
            } if target == "score-right" && path == "style.fg"
        ));
    }

    #[test]
    fn batch_routes_multiple_text_updates() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut props = RhaiMap::new();
        props.insert("text.content".into(), "READY".into());
        props.insert("style.fg".into(), "green".into());

        let mut api = build_api(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            TargetResolver::new("scene-root".to_string()),
            &queue,
        );

        api.batch("hud-message", props);

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert!(queue.iter().any(|command| matches!(
            command,
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::Set2dProps {
                    target,
                    text: Some(text),
                    ..
                },
            } if target == "hud-message" && text == "READY"
        )));
        assert!(queue.iter().any(|command| matches!(
            command,
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetSpriteProperty { target, path, .. }
            } if target == "hud-message" && path == "style.fg"
        )));
    }
}
