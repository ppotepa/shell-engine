//! Particle color/radius ramp system.
//!
//! For every particle entity that has both a `Lifetime` and a `ParticleColorRamp`,
//! this system samples the ramp each frame and pushes direct property mutations
//! (fg colour + vector points) to the scene runtime.
//!
//! Runs between `collect_async` and `visual_sync_system` each frame.

use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_game::GameplayWorld;

/// Apply color and radius ramps to all live particle entities.
pub fn particle_ramp_system(world: &mut World) {
    let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() else {
        return;
    };

    // Single-lock batch read of all ramp data.
    let ramp_data = gameplay_world.batch_read_particle_ramps();
    if ramp_data.is_empty() {
        return;
    }

    // Pre-compute typed ramp values: (visual_id, colour_str, radius).
    // No BehaviorCommand, no JsonValue — direct typed mutation.
    let mut updates: Vec<(String, String, i32)> = Vec::with_capacity(ramp_data.len());

    for (_id, visual_id, ramp, ttl_ms, original_ttl_ms) in &ramp_data {
        let life_ratio = if *original_ttl_ms > 0 {
            (*ttl_ms as f32 / *original_ttl_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let n = ramp.colors.len();
        let idx = ((1.0 - life_ratio) * n as f32).floor() as usize;
        let color = ramp.colors[idx.min(n - 1)].clone();

        let radius = (ramp.radius_min as f32
            + (ramp.radius_max - ramp.radius_min) as f32 * life_ratio)
            .round() as i32;

        updates.push((visual_id.clone(), color, radius));
    }

    if updates.is_empty() {
        return;
    }

    let Some(runtime) = world.scene_runtime_mut() else {
        return;
    };
    // Direct mutation: bypass BehaviorCommand pipeline entirely.
    // Eliminates ~2N String allocations + 2N JsonValue allocations per frame.
    runtime.apply_particle_ramps(&updates);
}
