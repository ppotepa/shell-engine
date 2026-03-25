//! Scene lifecycle system generic wrapper.
//!
//! This module provides a generic scene lifecycle system that works through LifecycleProvider,
//! enabling decoupling from engine's World type.
//!
//! Full implementation of scene transitions and lifecycle management remains in engine
//! due to deep coupling with World internals (scene_loader, clear_scoped, register_scoped).
//! This module provides the generic interface that engine will use.

use crate::LifecycleProvider;
use engine_events::EngineEvent;

/// Process frame events related to scene lifecycle and transitions through a provider.
/// Returns `true` when a quit event was requested.
///
/// # Note
/// The actual implementation is in engine/src/systems/scene_lifecycle.rs for now.
/// This function signature defines the generic interface for future decoupling.
pub fn scene_lifecycle_system<T: LifecycleProvider + ?Sized>(
    _provider: &mut T,
    _events: Vec<EngineEvent>,
) -> bool {
    // Stub implementation - actual implementation would require provider trait expansion
    // to support scene_loader navigation, clear_scoped/register_scoped lifecycle,
    // and downcast-safe access to SceneRuntime and Animator (currently in engine).
    false
}

