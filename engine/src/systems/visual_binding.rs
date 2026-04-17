use crate::behavior::BehaviorCommand;
use engine_api::scene::mutation::SceneMutationRequest;

#[derive(Default)]
pub struct VisualCleanupBuffer {
    pub targets: Vec<String>,
}

pub fn queue_visual_despawn(world: &mut engine_core::world::World, target: String) {
    if target.is_empty() {
        return;
    }
    if let Some(buffer) = world.get_mut::<VisualCleanupBuffer>() {
        buffer.targets.push(target);
    } else {
        world.register(VisualCleanupBuffer {
            targets: vec![target],
        });
    }
}

pub fn cleanup_visuals(world: &mut engine_core::world::World) {
    let Some(buffer) = world.get_mut::<VisualCleanupBuffer>() else {
        return;
    };
    if buffer.targets.is_empty() {
        return;
    }
    let targets = std::mem::take(&mut buffer.targets);

    let Some(runtime) = world.get_mut::<crate::scene_runtime::SceneRuntime>() else {
        return;
    };
    let resolver = runtime.target_resolver();
    let commands: Vec<BehaviorCommand> = targets
        .into_iter()
        .map(|target| BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::DespawnObject { target },
        })
        .collect();
    runtime.apply_behavior_commands(&resolver, &commands);
}
