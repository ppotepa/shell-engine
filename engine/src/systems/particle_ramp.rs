//! Particle color/radius ramp system.
//!
//! For every particle entity that has both a `Lifetime` and a `ParticleColorRamp`,
//! this system samples the ramp each frame and pushes `SetProperty` commands to
//! update `style.fg` and `vector.points` on the bound visual.
//!
//! Runs between `collect_async` and `visual_sync_system` each frame.

use crate::behavior::BehaviorCommand;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_game::GameplayWorld;
use serde_json::Value as JsonValue;

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

    let mut commands: Vec<BehaviorCommand> = Vec::with_capacity(ramp_data.len() * 2);

    for (_id, visual_id, ramp, ttl_ms, original_ttl_ms) in &ramp_data {
        // life_ratio: 1.0 = freshly spawned, 0.0 = about to die
        let life_ratio = if *original_ttl_ms > 0 {
            (*ttl_ms as f32 / *original_ttl_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let n = ramp.colors.len();
        let idx = ((1.0 - life_ratio) * n as f32).floor() as usize;
        let color = &ramp.colors[idx.min(n - 1)];

        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "style.fg".to_string(),
            value: JsonValue::from(color.as_str()),
        });

        let radius = (ramp.radius_min as f32
            + (ramp.radius_max - ramp.radius_min) as f32 * life_ratio)
            .round() as i32;
        let r = radius.max(0);
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "vector.points".to_string(),
            value: JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::from(0), JsonValue::from(0)]),
                JsonValue::Array(vec![JsonValue::from(r), JsonValue::from(0)]),
            ]),
        });
    }

    if commands.is_empty() {
        return;
    }

    let Some(runtime) = world.scene_runtime_mut() else {
        return;
    };
    let resolver = runtime.target_resolver();
    runtime.apply_behavior_commands(&resolver, &commands);
}
