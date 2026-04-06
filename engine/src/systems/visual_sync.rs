//! Visual-sync system — pushes Transform2D positions into scene object properties
//! so that Rhai scripts do not need to manually call `scene.set(id, "position.x", x)`
//! every frame.

use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_game::GameplayWorld;

/// Iterates all entities that have both a `Transform2D` and a `VisualBinding`,
/// then applies `position.x` / `position.y` scene-property updates for the
/// primary `visual_id`.
pub fn visual_sync_system(world: &mut World) {
    let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() else {
        return;
    };

    // Single-lock batch read of all visual sync data.
    let sync_data = gameplay_world.batch_read_visual_sync();
    if sync_data.is_empty() {
        return;
    }

    let Some(runtime) = world.scene_runtime_mut() else {
        return;
    };
    // Direct mutation: bypass BehaviorCommand pipeline entirely.
    // Eliminates ~3N String allocations + 3N JsonValue allocations per frame.
    runtime.apply_particle_visual_sync(&sync_data);
}
