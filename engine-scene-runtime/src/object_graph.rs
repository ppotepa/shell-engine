//! Object graph query and navigation interface.
//!
//! Provides read-only access to the materialized scene object hierarchy, including:
//! - Object lookup by ID or alias
//! - Target resolution (object aliases, layer indices, sprite paths)
//! - Object state snapshots and effective state calculation
//! - Region and camera state management
//!
//! This module is decoupled from behavior state and can be used by rendering,
//! debugging, and lifecycle systems without circular dependencies.

use super::*;
use engine_core::render_types::DirtyMask3D;
use engine_core::spatial::SpatialContext;

fn cached_arc<T, F>(cache: &mut Option<std::sync::Arc<T>>, build: F) -> std::sync::Arc<T>
where
    F: FnOnce() -> T,
{
    if let Some(cached) = cache {
        return std::sync::Arc::clone(cached);
    }
    let value = std::sync::Arc::new(build());
    *cache = Some(std::sync::Arc::clone(&value));
    value
}

fn cached_arc_with_gen<T, F>(
    cache: &mut Option<std::sync::Arc<T>>,
    cache_gen: &mut u64,
    current_gen: u64,
    build: F,
) -> std::sync::Arc<T>
where
    F: FnOnce() -> T,
{
    if let Some(cached) = cache {
        if *cache_gen == current_gen {
            return std::sync::Arc::clone(cached);
        }
    }
    let value = std::sync::Arc::new(build());
    *cache = Some(std::sync::Arc::clone(&value));
    *cache_gen = current_gen;
    value
}

impl SceneRuntime {
    /// Returns the runtime scene model after load-time normalization and sorting.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Returns the current world-space camera origin `(x, y)` in pixels.
    ///
    /// Non-UI layers are shifted by `(-x, -y)` during compositing so that entity
    /// at world position `(x, y)` maps to the screen origin.
    pub fn camera(&self) -> (i32, i32) {
        (self.camera_x, self.camera_y)
    }

    /// Sets the world-space camera origin directly (called from behavior_runner).
    pub(crate) fn set_camera_internal(&mut self, x: i32, y: i32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    /// Returns the current 2D camera zoom factor (default 1.0).
    pub fn camera_zoom(&self) -> f32 {
        self.camera_zoom
    }

    /// Sets the 2D camera zoom factor (called from behavior_runner).
    pub(crate) fn set_camera_zoom_internal(&mut self, zoom: f32) {
        self.camera_zoom = zoom.max(0.001);
    }

    /// Returns the scene-wide spatial contract (units + axis convention).
    pub fn spatial_context(&self) -> SpatialContext {
        self.spatial_context
    }

    /// Sets scene-wide spatial contract.
    pub fn set_spatial_context(&mut self, context: SpatialContext) {
        self.spatial_context = context;
    }

    pub fn scene_camera_3d(&self) -> SceneCamera3D {
        self.scene_camera_3d
    }

    pub fn resolved_view_profile(&self) -> &ResolvedViewProfile {
        &self.resolved_view_profile
    }

    pub(crate) fn set_scene_camera_3d_internal(&mut self, camera: SceneCamera3D) {
        self.scene_camera_3d = camera;
    }

    /// Returns the current aggregated 3D dirty mask.
    pub fn render3d_dirty_mask(&self) -> DirtyMask3D {
        self.render3d_dirty_mask
    }

    /// Returns and clears the current aggregated 3D dirty mask.
    pub fn take_render3d_dirty_mask(&mut self) -> DirtyMask3D {
        let mask = self.render3d_dirty_mask;
        self.render3d_dirty_mask = DirtyMask3D::empty();
        mask
    }

    pub fn render3d_rebuild_diagnostics(&self) -> Render3dRebuildDiagnostics {
        self.render3d_rebuild_diagnostics
    }

    pub fn take_render3d_rebuild_diagnostics(&mut self) -> Render3dRebuildDiagnostics {
        let diagnostics = self.render3d_rebuild_diagnostics;
        self.render3d_rebuild_diagnostics = Render3dRebuildDiagnostics::default();
        diagnostics
    }

    /// Returns the runtime object id assigned to the scene root node.
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Returns the number of registered behavior runtimes (for diagnostics).
    pub fn behavior_count(&self) -> usize {
        self.behaviors.len()
    }

    /// Looks up a materialized runtime object by its stable runtime id.
    pub fn object(&self, id: &str) -> Option<&GameObject> {
        self.objects.get(id)
    }

    /// Iterates over all materialized runtime objects in the scene graph.
    pub fn objects(&self) -> impl Iterator<Item = &GameObject> {
        self.objects.values()
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Returns the direct mutable runtime state stored on a specific object.
    pub fn object_state(&self, id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(id)
    }

    pub fn object_states_snapshot(
        &mut self,
    ) -> std::sync::Arc<HashMap<String, ObjectRuntimeState>> {
        cached_arc_with_gen(
            &mut self.cached_object_states,
            &mut self.cached_object_states_gen,
            self.object_mutation_gen,
            || self.object_states.clone(),
        )
    }

    pub fn object_kind_snapshot(&self) -> std::sync::Arc<HashMap<String, String>> {
        std::sync::Arc::clone(&self.cached_object_kinds)
    }

    pub fn obj_camera_states_snapshot(
        &mut self,
    ) -> std::sync::Arc<HashMap<String, ObjCameraState>> {
        cached_arc(&mut self.cached_obj_camera_states, || {
            self.obj_camera_states.clone()
        })
    }

    /// Returns the effective object state after inheriting visibility and
    /// offsets from all runtime parents.
    pub fn effective_object_state(&self, id: &str) -> Option<ObjectRuntimeState> {
        let mut state = self.object_states.get(id)?.clone();
        let mut parent_id = self
            .objects
            .get(id)
            .and_then(|object| object.parent_id.as_deref());

        while let Some(current_parent_id) = parent_id {
            let parent_state = self
                .object_states
                .get(current_parent_id)
                .cloned()
                .unwrap_or_default();
            state.visible &= parent_state.visible;
            state.offset_x = state.offset_x.saturating_add(parent_state.offset_x);
            state.offset_y = state.offset_y.saturating_add(parent_state.offset_y);
            parent_id = self
                .objects
                .get(current_parent_id)
                .and_then(|object| object.parent_id.as_deref());
        }

        Some(state)
    }

    /// Snapshots effective state for every runtime object for behavior and
    /// rendering consumers. Returns a cached Arc when nothing has mutated
    /// `object_states` since the last call — O(1) on clean frames.
    pub fn effective_object_states_snapshot(
        &mut self,
    ) -> std::sync::Arc<HashMap<String, ObjectRuntimeState>> {
        if self.effective_states_dirty {
            self.cached_effective_states = None;
        }
        if let Some(cached) = &self.cached_effective_states {
            if self.cached_effective_states_gen == self.object_mutation_gen {
                self.effective_states_dirty = false;
                return std::sync::Arc::clone(cached);
            }
        }
        let snapshot = std::sync::Arc::new(
            self.objects
                .keys()
                .filter_map(|object_id| {
                    self.effective_object_state(object_id)
                        .map(|state| (object_id.clone(), state))
                })
                .collect(),
        );
        self.cached_effective_states = Some(std::sync::Arc::clone(&snapshot));
        self.cached_effective_states_gen = self.object_mutation_gen;
        self.effective_states_dirty = false;
        snapshot
    }

    /// Returns a resolver for authored target names, layer indices, and sprite
    /// paths against the current materialized runtime scene.
    pub fn target_resolver(&self) -> TargetResolver {
        (*self.resolver_cache).clone()
    }

    pub fn target_resolver_arc(&self) -> std::sync::Arc<TargetResolver> {
        std::sync::Arc::clone(&self.resolver_cache)
    }

    pub(crate) fn build_target_resolver(&self) -> TargetResolver {
        let mut aliases = HashMap::new();

        for (object_id, _object) in &self.objects {
            aliases.insert(object_id.clone(), object_id.clone());
        }

        // Explicit aliases are the primary authoring/runtime target surface.
        // Insert them before display names so internal node names from runtime
        // clones cannot steal a layer alias (for example `ship-1` layer vs its
        // first child sprite, which is retagged to the same name).
        for (object_id, object) in &self.objects {
            for alias in &object.aliases {
                aliases
                    .entry(alias.clone())
                    .or_insert_with(|| object_id.clone());
            }
        }

        // Object names are a fallback for unnamed/generated nodes only.
        for (object_id, object) in &self.objects {
            aliases
                .entry(object.name.clone())
                .or_insert_with(|| object_id.clone());
        }

        TargetResolver::from_parts(
            self.root_id.clone(),
            aliases,
            self.layer_ids.clone(),
            self.sprite_ids.clone(),
        )
    }

    pub fn set_object_regions(&mut self, object_regions: HashMap<String, Region>) {
        let object_regions = std::sync::Arc::new(object_regions);
        self.cached_object_regions = std::sync::Arc::clone(&object_regions);
        self.object_regions = object_regions;
        self.sync_widget_layout_bounds();
    }
}
