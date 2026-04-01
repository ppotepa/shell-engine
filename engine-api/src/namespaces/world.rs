//! world.* namespace for Rhai scripting.
//!
//! Provides world-level operations: spawning, querying, emitting effects, managing bounds, etc.
//! Accessible as `world.spawn_visual()`, `world.emit()`, etc.
//!
//! All methods are also registered as flat names for backward compatibility:
//! - `world.spawn_visual(...)` works AND `spawn_visual(...)` works

use rhai::Engine as RhaiEngine;

/// World namespace marker for Rhai registration.
/// This struct is not used at runtime but serves as a type anchor for namespace methods.
#[derive(Clone)]
pub struct WorldNamespace;

/// Register the `world.*` namespace and backward-compatible flat names.
///
/// This function should be called during Rhai engine initialization to set up
/// all world-level APIs with both namespaced and flat names.
///
/// Example of generated APIs:
/// - `world.spawn_visual(kind, template, options)`
/// - `world.emit(emitter_name, owner_id, options)`
/// - `world.set_bounds(min_x, min_y, max_x, max_y)`
///
/// Also registers flat names for backward compatibility:
/// - `spawn_visual(kind, template, options)` (old flat API)
pub fn register_world_namespace(_engine: &mut RhaiEngine) {
    // This is intentionally minimal as a proof-of-concept.
    // Actual registration happens via the parent gameplay registration,
    // which remains the single source of truth for now.
    //
    // Future work: Move registration logic here from engine-behavior.
    //
    // The namespace will be populated by:
    // 1. Creating wrapper structs in engine-api/src/namespaces/world.rs
    // 2. Implementing namespace methods
    // 3. Updating engine-behavior/src/scripting/gameplay.rs to call
    //    register_world_namespace() alongside existing registrations
}
