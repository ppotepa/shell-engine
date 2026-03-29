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

impl SceneRuntime {
    /// Returns the runtime scene model after load-time normalization and sorting.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn set_scene_rendered_mode(&mut self, mode: SceneRenderedMode) {
        self.scene.rendered_mode = mode;
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
        if let Some(cached) = &self.cached_object_states {
            if self.cached_object_states_gen == self.object_mutation_gen {
                return std::sync::Arc::clone(cached);
            }
        }
        let arc = std::sync::Arc::new(self.object_states.clone());
        self.cached_object_states = Some(std::sync::Arc::clone(&arc));
        self.cached_object_states_gen = self.object_mutation_gen;
        arc
    }

    pub fn object_kind_snapshot(&self) -> std::sync::Arc<HashMap<String, String>> {
        std::sync::Arc::clone(&self.cached_object_kinds)
    }

    pub fn obj_camera_states_snapshot(
        &mut self,
    ) -> std::sync::Arc<HashMap<String, ObjCameraState>> {
        if let Some(cached) = &self.cached_obj_camera_states {
            return std::sync::Arc::clone(cached);
        }
        let arc = std::sync::Arc::new(self.obj_camera_states.clone());
        self.cached_obj_camera_states = Some(std::sync::Arc::clone(&arc));
        arc
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
        if !self.effective_states_dirty {
            if let Some(cached) = &self.cached_effective_states {
                if self.cached_effective_states_gen == self.object_mutation_gen {
                    return std::sync::Arc::clone(cached);
                }
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

        for (object_id, object) in &self.objects {
            aliases.insert(object_id.clone(), object_id.clone());
            aliases.insert(object.name.clone(), object_id.clone());
            for alias in &object.aliases {
                aliases.insert(alias.clone(), object_id.clone());
            }
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
    }
}
