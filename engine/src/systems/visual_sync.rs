//! Visual-sync system — pushes Transform2D positions into scene object properties
//! so that Rhai scripts do not need to manually call `scene.set(id, "position.x", x)`
//! every frame.

use crate::behavior::BehaviorCommand;
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

    let mut commands: Vec<BehaviorCommand> = Vec::with_capacity(sync_data.len() * 3);

    for (visual_id, x, y, heading) in &sync_data {
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "position.x".to_string(),
            value: serde_json::Value::from(*x),
        });
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "position.y".to_string(),
            value: serde_json::Value::from(*y),
        });
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "transform.heading".to_string(),
            value: serde_json::Value::from(*heading),
        });
    }

    let Some(runtime) = world.scene_runtime_mut() else {
        return;
    };
    let resolver = runtime.target_resolver();
    runtime.apply_behavior_commands(&resolver, &commands);
}
