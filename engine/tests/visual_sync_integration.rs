use engine::systems::visual_binding::{cleanup_visuals, queue_visual_despawn, VisualCleanupBuffer};
use engine::systems::visual_sync::visual_sync_system;
use engine::world::World;
use engine_api::scene::{Camera3dMutationRequest, SceneMutationRequest};
use engine_behavior::BehaviorCommand;
use engine_core::render_types::DirtyMask3D;
use engine_core::scene::{Scene, Sprite};
use engine_game::components::{Transform2D, Transform3D, VisualBinding};
use engine_game::GameplayWorld;
use engine_scene_runtime::SceneRuntime;
use serde_json::json;

fn find_obj_sprite<'a>(scene: &'a Scene, alias: &str) -> &'a Sprite {
    scene
        .layers
        .iter()
        .flat_map(|layer| layer.sprites.iter())
        .find(|sprite| matches!(sprite, Sprite::Obj { id: Some(id), .. } if id == alias))
        .expect("object sprite")
}

#[test]
fn visual_sync_prefers_typed_transform3d_for_typed_visuals_and_keeps_legacy_fallbacks() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: visual-sync-integration
title: Visual Sync Integration
render-space: 3d
layers:
  - name: world
    sprites:
      - type: obj
        id: ship
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
  - name: hud
    sprites:
      - type: text
        id: marker
        content: MARKER
"#,
    )
    .expect("scene parse");

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene));

    let id = gameplay.spawn("ship", json!({})).expect("spawn ship");
    assert!(gameplay.set_visual(
        id,
        VisualBinding {
            visual_id: Some("ship".into()),
            additional_visuals: vec!["marker".into()],
        }
    ));
    assert!(gameplay.set_transform(
        id,
        Transform2D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            heading: 0.25,
        }
    ));
    assert!(gameplay.set_transform3d(
        id,
        Transform3D {
            position: [12.0, -4.0, 7.0],
            orientation: [0.0, 0.0, 0.70710677, 0.70710677],
        }
    ));

    visual_sync_system(&mut world);

    let runtime = world.get_mut::<SceneRuntime>().expect("runtime");
    let dirty = runtime.take_render3d_dirty_mask();
    assert!(dirty.contains(DirtyMask3D::TRANSFORM));

    let resolver = runtime.target_resolver();
    let ship_id = resolver.resolve_alias("ship").expect("ship alias");
    let ship_state = runtime.object_state(ship_id).expect("ship state");
    assert_eq!(ship_state.offset_x, 12);
    assert_eq!(ship_state.offset_y, -4);
    assert_eq!(ship_state.offset_z, 0);
    assert_eq!(ship_state.heading, 0.0);

    let marker_id = resolver.resolve_alias("marker").expect("marker alias");
    let marker_state = runtime.object_state(marker_id).expect("marker state");
    assert_eq!(marker_state.offset_x, 12);
    assert_eq!(marker_state.offset_y, -4);
    assert_eq!(marker_state.offset_z, 7);
    assert!(
        (marker_state.heading - std::f32::consts::FRAC_PI_2).abs() < 0.0001,
        "expected legacy mirror heading from the typed 3D pose"
    );
}

#[test]
fn object_camera_requests_use_render_node_path_without_touching_scene_camera_state() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: visual-sync-camera-integration
title: Visual Sync Camera Integration
render-space: 3d
layers:
  - name: world
    sprites:
      - type: obj
        id: cockpit-camera
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
      - type: obj
        id: chase-camera
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
"#,
    )
    .expect("scene parse");

    let mut runtime = SceneRuntime::new(scene);
    let before = runtime.scene_camera_3d();
    let resolver = runtime.target_resolver();

    runtime.apply_behavior_commands(
        &resolver,
        &[
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectLookAt {
                    target: "cockpit-camera".to_string(),
                    eye: [1.0, 2.0, 3.0],
                    look_at: [1.0, 2.0, 4.0],
                    up: Some([0.0, 1.0, 0.0]),
                }),
            },
            BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectBasis {
                    target: "chase-camera".to_string(),
                    eye: [5.0, 6.0, 7.0],
                    right: [1.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    forward: [0.0, 0.0, 1.0],
                }),
            },
        ],
    );

    assert_eq!(runtime.scene_camera_3d(), before);
    let dirty = runtime.take_render3d_dirty_mask();
    assert!(dirty.contains(DirtyMask3D::MATERIAL));

    let cockpit = find_obj_sprite(runtime.scene(), "cockpit-camera");
    let chase = find_obj_sprite(runtime.scene(), "chase-camera");

    match cockpit {
        Sprite::Obj {
            cam_world_x,
            cam_world_y,
            cam_world_z,
            view_right_x,
            view_right_y,
            view_right_z,
            view_up_x,
            view_up_y,
            view_up_z,
            view_fwd_x,
            view_fwd_y,
            view_fwd_z,
            ..
        } => {
            assert_eq!(
                (*cam_world_x, *cam_world_y, *cam_world_z),
                (Some(1.0), Some(2.0), Some(3.0))
            );
            assert_eq!(
                (*view_right_x, *view_right_y, *view_right_z),
                (Some(-1.0), Some(0.0), Some(0.0))
            );
            assert_eq!(
                (*view_up_x, *view_up_y, *view_up_z),
                (Some(0.0), Some(1.0), Some(0.0))
            );
            assert_eq!(
                (*view_fwd_x, *view_fwd_y, *view_fwd_z),
                (Some(0.0), Some(0.0), Some(1.0))
            );
        }
        _ => unreachable!(),
    }

    match chase {
        Sprite::Obj {
            cam_world_x,
            cam_world_y,
            cam_world_z,
            view_right_x,
            view_right_y,
            view_right_z,
            view_up_x,
            view_up_y,
            view_up_z,
            view_fwd_x,
            view_fwd_y,
            view_fwd_z,
            ..
        } => {
            assert_eq!(
                (*cam_world_x, *cam_world_y, *cam_world_z),
                (Some(5.0), Some(6.0), Some(7.0))
            );
            assert_eq!(
                (*view_right_x, *view_right_y, *view_right_z),
                (Some(1.0), Some(0.0), Some(0.0))
            );
            assert_eq!(
                (*view_up_x, *view_up_y, *view_up_z),
                (Some(0.0), Some(1.0), Some(0.0))
            );
            assert_eq!(
                (*view_fwd_x, *view_fwd_y, *view_fwd_z),
                (Some(0.0), Some(0.0), Some(1.0))
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn visual_cleanup_consumes_trimmed_duplicate_targets_after_despawn() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: visual-cleanup-integration
title: Visual Cleanup Integration
layers:
  - name: hud
    sprites:
      - type: text
        id: ship
        content: SHIP
      - type: text
        id: ghost
        content: GHOST
"#,
    )
    .expect("scene parse");

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene));

    let id = gameplay.spawn("ship", json!({})).expect("spawn ship");
    assert!(gameplay.set_visual(
        id,
        VisualBinding {
            visual_id: Some("ship".into()),
            additional_visuals: vec!["ghost".into()],
        }
    ));

    let targets: Vec<String> = gameplay
        .despawn_tree_ids(id)
        .into_iter()
        .filter_map(|tree_id| gameplay.visual(tree_id))
        .flat_map(|binding| {
            binding
                .all_visual_ids()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(gameplay.despawn(id));

    for target in targets {
        queue_visual_despawn(&mut world, format!("  {target}  "));
        queue_visual_despawn(&mut world, target);
    }

    cleanup_visuals(&mut world);

    let buffer = world
        .get::<VisualCleanupBuffer>()
        .expect("cleanup buffer should stay registered");
    assert!(buffer.targets.is_empty());
}

#[test]
fn runtime_object_subtrees_materialize_without_interfering_with_gameplay_visual_sync() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-sync-bridge
title: Runtime Object Sync Bridge
render-space: 3d
runtime-objects:
  - name: runtime-root
    kind: runtime-object
    transform:
      space: 3d
      translation: [100.0, 200.0, 300.0]
      rotation-deg: [0.0, 45.0, 0.0]
      scale: [1.0, 1.0, 1.0]
    children:
      - name: cockpit
        kind: runtime-object
        transform:
          space: 3d
          translation: [1.0, 2.0, 3.0]
layers:
  - name: world
    sprites:
      - type: obj
        id: ship
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
  - name: hud
    sprites:
      - type: text
        id: marker
        content: MARKER
"#,
    )
    .expect("scene parse");

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene));

    {
        let runtime = world.get_mut::<SceneRuntime>().expect("runtime");
        let resolver = runtime.target_resolver();
        let runtime_root_id = resolver
            .resolve_alias("runtime-root")
            .expect("runtime-object root alias");
        let cockpit_id = resolver
            .resolve_alias("runtime-objects/runtime-root/cockpit")
            .expect("runtime-object child path alias");
        let runtime_root_state = runtime
            .object_state(runtime_root_id)
            .expect("runtime-object root state");
        let cockpit_state = runtime
            .object_state(cockpit_id)
            .expect("runtime-object child state");
        assert_eq!(runtime_root_state.offset_x, 100);
        assert_eq!(runtime_root_state.offset_y, 200);
        assert_eq!(runtime_root_state.offset_z, 300);
        assert_eq!(cockpit_state.offset_x, 1);
        assert_eq!(cockpit_state.offset_y, 2);
        assert_eq!(cockpit_state.offset_z, 3);
    }

    let id = gameplay.spawn("ship", json!({})).expect("spawn ship");
    assert!(gameplay.set_visual(
        id,
        VisualBinding {
            visual_id: Some("ship".into()),
            additional_visuals: vec!["marker".into()],
        }
    ));
    assert!(gameplay.set_transform3d(
        id,
        Transform3D {
            position: [8.0, 9.0, 10.0],
            orientation: [0.0, 0.0, 0.70710677, 0.70710677],
        }
    ));

    visual_sync_system(&mut world);

    let runtime = world.get_mut::<SceneRuntime>().expect("runtime");
    let dirty = runtime.take_render3d_dirty_mask();
    assert!(dirty.contains(DirtyMask3D::TRANSFORM));

    let resolver = runtime.target_resolver();
    let runtime_root_id = resolver
        .resolve_alias("runtime-root")
        .expect("runtime-object root alias");
    let cockpit_id = resolver
        .resolve_alias("runtime-objects/runtime-root/cockpit")
        .expect("runtime-object child path alias");

    let ship_id = resolver.resolve_alias("ship").expect("ship alias");
    let ship_state = runtime.object_state(ship_id).expect("ship state");
    assert_eq!(ship_state.offset_x, 8);
    assert_eq!(ship_state.offset_y, 9);
    assert_eq!(ship_state.offset_z, 0);

    let marker_id = resolver.resolve_alias("marker").expect("marker alias");
    let marker_state = runtime.object_state(marker_id).expect("marker state");
    assert_eq!(marker_state.offset_x, 8);
    assert_eq!(marker_state.offset_y, 9);
    assert_eq!(marker_state.offset_z, 10);

    let runtime_root_state = runtime
        .object_state(runtime_root_id)
        .expect("runtime-object root state");
    let cockpit_state = runtime
        .object_state(cockpit_id)
        .expect("runtime-object child state");
    assert_eq!(runtime_root_state.offset_x, 100);
    assert_eq!(runtime_root_state.offset_y, 200);
    assert_eq!(runtime_root_state.offset_z, 300);
    assert_eq!(cockpit_state.offset_x, 1);
    assert_eq!(cockpit_state.offset_y, 2);
    assert_eq!(cockpit_state.offset_z, 3);
}

#[test]
fn cleanup_visuals_handles_materialized_runtime_object_targets_alongside_real_visuals() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-cleanup-bridge
title: Runtime Object Cleanup Bridge
runtime-objects:
  - name: runtime-root
    kind: runtime-object
    transform:
      space: 3d
      translation: [5.0, 6.0, 7.0]
    children:
      - name: cockpit
        kind: runtime-object
        transform:
          space: 3d
          translation: [1.0, 0.0, 0.0]
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

    queue_visual_despawn(&mut world, " runtime-root ".to_string());
    queue_visual_despawn(
        &mut world,
        "runtime-objects/runtime-root/cockpit".to_string(),
    );
    queue_visual_despawn(&mut world, "ship".to_string());
    queue_visual_despawn(&mut world, "runtime-root".to_string());

    cleanup_visuals(&mut world);

    let buffer = world
        .get::<VisualCleanupBuffer>()
        .expect("cleanup buffer should stay registered");
    assert!(buffer.targets.is_empty());

    let runtime = world.get_mut::<SceneRuntime>().expect("runtime");
    let resolver = runtime.target_resolver();
    assert!(
        resolver.resolve_alias("runtime-root").is_none(),
        "queued runtime-object root should despawn together with its subtree"
    );
    assert!(
        resolver
            .resolve_alias("runtime-objects/runtime-root/cockpit")
            .is_none(),
        "queued runtime-object child should be removed when the parent subtree despawns"
    );
    assert!(
        resolver.resolve_alias("ship").is_none(),
        "real visuals in the same cleanup batch should still despawn"
    );
}
