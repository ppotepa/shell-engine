//! Namespaced Rhai API registration.
//!
//! Organizes script-facing APIs into logical domains for better ergonomics.
//! Each domain can be accessed via dot notation (e.g., `world.spawn_visual()`)
//! while maintaining backward compatibility with flat names.

pub mod world;

pub use world::WorldNamespace;

use rhai::Engine as RhaiEngine;

/// Register all namespaced APIs with the Rhai engine.
///
/// This function sets up both the namespaced APIs (e.g., `world.spawn_visual`)
/// and backward-compatible flat names (e.g., `spawn_visual`).
pub fn register_namespaces(engine: &mut RhaiEngine) {
    world::register_world_namespace(engine);
}
