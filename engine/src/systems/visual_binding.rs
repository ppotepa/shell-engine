use crate::behavior::BehaviorCommand;
use engine_api::scene::mutation::SceneMutationRequest;
use std::collections::HashSet;

#[derive(Default)]
pub struct VisualCleanupBuffer {
    pub targets: Vec<String>,
}

pub fn queue_visual_despawn(world: &mut engine_core::world::World, target: String) {
    let target = target.trim();
    if target.is_empty() {
        return;
    }
    let target = target.to_string();
    if let Some(buffer) = world.get_mut::<VisualCleanupBuffer>() {
        buffer.targets.push(target);
    } else {
        world.register(VisualCleanupBuffer {
            targets: vec![target],
        });
    }
}

pub fn cleanup_visuals(world: &mut engine_core::world::World) {
    if world.get::<crate::scene_runtime::SceneRuntime>().is_none() {
        return;
    }

    let Some(buffer) = world.get_mut::<VisualCleanupBuffer>() else {
        return;
    };
    if buffer.targets.is_empty() {
        return;
    }
    let mut seen = HashSet::new();
    let targets: Vec<String> = std::mem::take(&mut buffer.targets)
        .into_iter()
        .filter_map(|target| {
            let normalized = target.trim();
            if normalized.is_empty() {
                return None;
            }
            if seen.insert(normalized.to_string()) {
                Some(normalized.to_string())
            } else {
                None
            }
        })
        .collect();
    if targets.is_empty() {
        return;
    }

    let Some(runtime) = world.get_mut::<crate::scene_runtime::SceneRuntime>() else {
        return;
    };
    let resolver = runtime.target_resolver();
    let collapsed_targets = collapse_runtime_cleanup_targets(runtime, &resolver, targets);
    let mut deferred_targets = Vec::new();
    for target in collapsed_targets {
        if !runtime.remove_runtime_object_subtree(&target) {
            deferred_targets.push(target);
        }
    }
    let commands: Vec<BehaviorCommand> = deferred_targets
        .into_iter()
        .map(|target| BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::DespawnObject { target },
        })
        .collect();
    if !commands.is_empty() {
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(&resolver, &commands);
    }
}

fn collapse_runtime_cleanup_targets(
    runtime: &crate::scene_runtime::SceneRuntime,
    resolver: &engine_scene_runtime::TargetResolver,
    targets: Vec<String>,
) -> Vec<String> {
    let mut unresolved = Vec::new();
    let mut resolved_ids = Vec::new();
    let mut seen_ids = HashSet::new();

    for target in targets {
        if let Some(object_id) = resolver.resolve_alias(&target) {
            let object_id = object_id.to_string();
            if seen_ids.insert(object_id.clone()) {
                resolved_ids.push(object_id);
            }
        } else {
            unresolved.push(target);
        }
    }

    resolved_ids.sort_by_key(|id| runtime_object_depth(runtime, id));
    let mut collapsed_ids: Vec<String> = Vec::new();
    for object_id in resolved_ids {
        if collapsed_ids
            .iter()
            .any(|ancestor| runtime_object_is_descendant_of(runtime, &object_id, ancestor))
        {
            continue;
        }
        collapsed_ids.push(object_id);
    }

    unresolved.extend(collapsed_ids);
    unresolved
}

fn runtime_object_depth(runtime: &crate::scene_runtime::SceneRuntime, object_id: &str) -> usize {
    let mut depth = 0usize;
    let mut current = runtime.object(object_id).and_then(|object| object.parent_id.as_deref());
    while let Some(parent_id) = current {
        depth += 1;
        current = runtime.object(parent_id).and_then(|object| object.parent_id.as_deref());
    }
    depth
}

fn runtime_object_is_descendant_of(
    runtime: &crate::scene_runtime::SceneRuntime,
    object_id: &str,
    ancestor_id: &str,
) -> bool {
    let mut current = runtime.object(object_id).and_then(|object| object.parent_id.as_deref());
    while let Some(parent_id) = current {
        if parent_id == ancestor_id {
            return true;
        }
        current = runtime.object(parent_id).and_then(|object| object.parent_id.as_deref());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_visuals, collapse_runtime_cleanup_targets, queue_visual_despawn,
        VisualCleanupBuffer,
    };
    use crate::services::EngineWorldAccess;
    use crate::world::World;
    use engine_core::scene::Scene;
    use engine_scene_runtime::SceneRuntime;

    #[test]
    fn cleanup_visuals_preserves_queue_until_runtime_exists() {
        let mut world = World::new();
        queue_visual_despawn(&mut world, "  ship  ".to_string());
        queue_visual_despawn(&mut world, "ship".to_string());
        queue_visual_despawn(&mut world, "  ".to_string());

        cleanup_visuals(&mut world);

        let buffer = world
            .get::<VisualCleanupBuffer>()
            .expect("cleanup buffer should stay registered");
        assert_eq!(buffer.targets, vec!["ship".to_string(), "ship".to_string()]);
    }

    #[test]
    fn cleanup_visuals_deduplicates_targets_once_runtime_is_available() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-cleanup
title: Visual Cleanup
layers:
  - name: hud
    sprites:
      - type: text
        id: ship
        content: SHIP
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        queue_visual_despawn(&mut world, " ship ".to_string());
        queue_visual_despawn(&mut world, "ship".to_string());
        queue_visual_despawn(&mut world, "".to_string());

        cleanup_visuals(&mut world);

        let buffer = world
            .get::<VisualCleanupBuffer>()
            .expect("cleanup buffer should stay registered");
        assert!(buffer.targets.is_empty());
        let runtime = world.scene_runtime_mut().expect("runtime");
        let resolver = runtime.target_resolver();
        assert!(resolver.resolve_alias("ship").is_none());
    }

    #[test]
    fn cleanup_visuals_despawns_runtime_object_subtree_by_path_alias() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-cleanup
title: Runtime Object Cleanup
layers:
  - name: hud
    sprites:
      - type: text
        id: label
        content: LABEL
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
          translation: [0.0, 1.0, 0.0]
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        let (carrier_id, cockpit_id) = {
            let runtime = world.scene_runtime_mut().expect("runtime");
            let resolver = runtime.target_resolver();
            (
                resolver
                    .resolve_alias("runtime-objects/carrier")
                    .expect("carrier alias")
                    .to_string(),
                resolver
                    .resolve_alias("runtime-objects/carrier/cockpit")
                    .expect("cockpit alias")
                    .to_string(),
            )
        };

        queue_visual_despawn(&mut world, "runtime-objects/carrier".to_string());
        cleanup_visuals(&mut world);

        let buffer = world
            .get::<VisualCleanupBuffer>()
            .expect("cleanup buffer should stay registered");
        assert!(buffer.targets.is_empty());
        let runtime = world.scene_runtime_mut().expect("runtime");
        assert!(runtime.object(&carrier_id).is_none());
        assert!(runtime.object(&cockpit_id).is_none());
        assert!(runtime.scene().runtime_objects.is_empty());
        let label_id = runtime
            .target_resolver()
            .resolve_alias("label")
            .expect("label should remain")
            .to_string();
        assert!(runtime.object(&label_id).is_some());
    }

    #[test]
    fn cleanup_visuals_runtime_object_child_removal_keeps_scene_payload_and_siblings_in_sync() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-cleanup-child
title: Runtime Object Cleanup Child
layers:
  - name: hud
    sprites:
      - type: text
        id: label
        content: LABEL
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
      - name: escort
        prefab: /prefabs/escort.yml
        transform:
          space: 3d
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        let (carrier_id, cockpit_id, escort_id) = {
            let runtime = world.scene_runtime_mut().expect("runtime");
            let resolver = runtime.target_resolver();
            (
                resolver
                    .resolve_alias("runtime-objects/carrier")
                    .expect("carrier alias")
                    .to_string(),
                resolver
                    .resolve_alias("runtime-objects/carrier/cockpit")
                    .expect("cockpit alias")
                    .to_string(),
                resolver
                    .resolve_alias("runtime-objects/carrier/escort")
                    .expect("escort alias")
                    .to_string(),
            )
        };

        queue_visual_despawn(&mut world, "runtime-objects/carrier/cockpit".to_string());
        cleanup_visuals(&mut world);

        let runtime = world.scene_runtime_mut().expect("runtime");
        assert!(runtime.object(&carrier_id).is_some());
        assert!(runtime.object(&cockpit_id).is_none());
        assert!(runtime.object(&escort_id).is_some());
        assert_eq!(runtime.scene().runtime_objects.len(), 1);
        assert_eq!(runtime.scene().runtime_objects[0].children.len(), 1);
        assert_eq!(runtime.scene().runtime_objects[0].children[0].name, "escort");
    }

    #[test]
    fn cleanup_visuals_collapse_prefers_runtime_object_parent_over_child_targets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-cleanup-collapse
title: Runtime Object Cleanup Collapse
layers:
  - name: hud
    sprites:
      - type: text
        id: label
        content: LABEL
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
          translation: [0.0, 1.0, 0.0]
"#,
        )
        .expect("scene parse");
        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();

        let collapsed = collapse_runtime_cleanup_targets(
            &runtime,
            &resolver,
            vec![
                "runtime-objects/carrier".to_string(),
                "runtime-objects/carrier/cockpit".to_string(),
            ],
        );

        assert_eq!(collapsed.len(), 1);
        let carrier_id = resolver
            .resolve_alias("runtime-objects/carrier")
            .expect("carrier alias");
        assert_eq!(collapsed[0], carrier_id);
    }
}
