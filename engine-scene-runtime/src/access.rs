//! Trait for accessing SceneRuntime from World.
//!
//! Similar to engine's EngineWorldAccess but scoped to scene-runtime needs.
//! This allows crates depending on engine-scene-runtime to access scene state
//! without depending on engine or World directly.

use std::any::Any;

use crate::SceneRuntime;

/// Trait providing access to SceneRuntime for a given world type.
///
/// Implementers can provide SceneRuntime access without exposing the full World type.
/// This enables scene-runtime consumers (behaviors, UI systems, etc.) to work with
/// any type that stores a SceneRuntime.
pub trait SceneRuntimeAccess {
    /// Get immutable reference to SceneRuntime, if present.
    fn scene_runtime(&self) -> Option<&SceneRuntime>;

    /// Get mutable reference to SceneRuntime, if present.
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
}

/// Default implementation for `Any` (for testing and generic contexts).
impl SceneRuntimeAccess for dyn Any {
    fn scene_runtime(&self) -> Option<&SceneRuntime> {
        self.downcast_ref::<SceneRuntime>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime> {
        self.downcast_mut::<SceneRuntime>()
    }
}
