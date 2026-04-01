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

    let ids = gameplay_world.ids_with_visual_binding();
    let mut commands: Vec<BehaviorCommand> = Vec::new();

    for id in ids {
        let Some(transform) = gameplay_world.transform(id) else {
            continue;
        };
        let Some(binding) = gameplay_world.visual(id) else {
            continue;
        };
        let Some(ref visual_id) = binding.visual_id else {
            continue;
        };

        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "position.x".to_string(),
            value: serde_json::Value::from(transform.x),
        });
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "position.y".to_string(),
            value: serde_json::Value::from(transform.y),
        });
        commands.push(BehaviorCommand::SetProperty {
            target: visual_id.clone(),
            path: "transform.heading".to_string(),
            value: serde_json::Value::from(transform.heading),
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
