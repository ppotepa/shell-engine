use engine::systems::gameplay::gameplay_system;
use engine::systems::scene_bootstrap::{
    activate_scene_bootstrap, apply_pending_scene_bootstrap_core, prepare_scene_bootstrap,
    AppliedSceneBootstrap, AppliedScenePlayerPreset, BootstrapApplyState, SceneBootstrapRouteState,
    SceneBootstrapTargetSource,
};
use engine::systems::visual_binding::{cleanup_visuals, VisualCleanupBuffer};
use engine::world::World;
use engine_behavior::catalog::{
    CatalogPresets, ControllerComponent, FlightMotor3DComponent, ModCatalogs, PlayerPreset,
    PrefabComponents,
};
use engine_core::scene::Scene;
use engine_game::components::{
    DespawnVisual, LifecyclePolicy, Lifetime, Transform3D, VisualBinding,
};
use engine_game::GameplayWorld;
use engine_scene_runtime::SceneRuntime;
use serde_json::json;

#[test]
fn runtime_object_scenes_keep_bootstrap_targeting_on_gameplay_entities_while_materializing_runtime_tree(
) {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-bootstrap-runtime
title: Runtime Object Bootstrap Runtime
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
layers: []
"#,
    )
    .expect("scene parse");

    let mut catalogs = ModCatalogs::default();
    catalogs.presets = CatalogPresets {
        players: [(
            "flight-player".to_string(),
            PlayerPreset {
                input_profile: Some("default-flight".to_string()),
                controller: Some(ControllerComponent {
                    controller_type: "VehicleAssembly".to_string(),
                    config: None,
                }),
                components: Some(PrefabComponents {
                    flight_motor_3d: Some(FlightMotor3DComponent {
                        translational_dofs: Some([true, true, true]),
                        rotational_dofs: Some([true, true, true]),
                        horizon_lock_strength: Some(0.35),
                    }),
                    ..Default::default()
                }),
                config: [("controlled".to_string(), json!(true))]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(catalogs);
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene.clone()));

    let entity_id = gameplay
        .spawn("flight-player", json!({}))
        .expect("spawn gameplay entity");
    assert!(gameplay.set_transform3d(
        entity_id,
        Transform3D {
            position: [0.0, 1.0, 2.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    ));

    prepare_scene_bootstrap(&scene, &mut world);
    activate_scene_bootstrap(&mut world);

    let applied = world
        .get::<AppliedSceneBootstrap>()
        .expect("applied bootstrap should exist");
    assert_eq!(applied.target_entity, Some(entity_id));
    assert_eq!(
        applied.target_source,
        SceneBootstrapTargetSource::Sole3dEntity
    );
    assert_eq!(
        applied.player_route_state,
        SceneBootstrapRouteState::Resolved
    );
    assert_eq!(
        applied.controlled_entity_applied,
        BootstrapApplyState::Applied
    );

    assert_eq!(
        world.get::<AppliedScenePlayerPreset>(),
        Some(&AppliedScenePlayerPreset {
            preset_id: "flight-player".to_string(),
            controlled: true,
            has_bootstrap_assembly: true,
            input_profile: Some("default-flight".to_string()),
            controller_type: Some("VehicleAssembly".to_string()),
        })
    );

    assert_eq!(gameplay.controlled_entity(), Some(entity_id));
    assert!(
        gameplay.flight_motor3d(entity_id).is_some(),
        "catalog-backed player bootstrap assembly should attach to the gameplay entity"
    );

    let runtime = world.get_mut::<SceneRuntime>().expect("scene runtime");
    let resolver = runtime.target_resolver();
    let runtime_root_id = resolver
        .resolve_alias("pilot-root")
        .expect("runtime-object root should materialize into the scene runtime")
        .to_string();
    let cockpit_id = resolver
        .resolve_alias("runtime-objects/pilot-root/cockpit")
        .expect("runtime-object child path alias should resolve")
        .to_string();
    let runtime_root = runtime
        .object(&runtime_root_id)
        .expect("runtime-object root should exist");
    let cockpit = runtime
        .object(&cockpit_id)
        .expect("runtime-object child should exist");

    assert_eq!(cockpit.parent_id.as_deref(), Some(runtime_root_id.as_str()));
    assert!(
        runtime_root
            .children
            .iter()
            .any(|child| child == &cockpit_id),
        "runtime-object hierarchy should materialize in the scene runtime"
    );
    assert_eq!(
        gameplay.controlled_entity(),
        Some(entity_id),
        "runtime-object visuals must not steal bootstrap targeting from gameplay entities"
    );
}

#[test]
fn runtime_object_scenes_do_not_satisfy_player_bootstrap_targeting_without_gameplay_entities() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-bootstrap-pending
title: Runtime Object Bootstrap Pending
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
layers: []
"#,
    )
    .expect("scene parse");

    let mut catalogs = ModCatalogs::default();
    catalogs.presets = CatalogPresets {
        players: [(
            "flight-player".to_string(),
            PlayerPreset {
                input_profile: Some("default-flight".to_string()),
                controller: Some(ControllerComponent {
                    controller_type: "VehicleAssembly".to_string(),
                    config: None,
                }),
                components: Some(PrefabComponents {
                    flight_motor_3d: Some(FlightMotor3DComponent {
                        translational_dofs: Some([true, true, true]),
                        rotational_dofs: Some([true, true, true]),
                        horizon_lock_strength: Some(0.35),
                    }),
                    ..Default::default()
                }),
                config: [("controlled".to_string(), json!(true))]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(catalogs);
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene.clone()));

    prepare_scene_bootstrap(&scene, &mut world);
    activate_scene_bootstrap(&mut world);

    let applied = world
        .get::<AppliedSceneBootstrap>()
        .expect("applied bootstrap should exist");
    assert_eq!(applied.target_entity, None);
    assert_eq!(
        applied.target_source,
        SceneBootstrapTargetSource::DeferredNo3dEntity
    );
    assert_eq!(
        applied.player_route_state,
        SceneBootstrapRouteState::Resolved
    );
    assert_eq!(applied.player_applied, BootstrapApplyState::Applied);
    assert_eq!(
        applied.controlled_entity_applied,
        BootstrapApplyState::NotRequested
    );
    assert!(applied
        .diagnostics
        .iter()
        .any(|note| note.contains("no 3D gameplay entity is registered yet")));
    assert!(applied.has_pending_work());
    assert_eq!(
        world.get::<AppliedScenePlayerPreset>(),
        Some(&AppliedScenePlayerPreset {
            preset_id: "flight-player".to_string(),
            controlled: true,
            has_bootstrap_assembly: true,
            input_profile: Some("default-flight".to_string()),
            controller_type: Some("VehicleAssembly".to_string()),
        })
    );
    assert_eq!(gameplay.controlled_entity(), None);

    let runtime = world.get_mut::<SceneRuntime>().expect("scene runtime");
    let resolver = runtime.target_resolver();
    let runtime_root_id = resolver
        .resolve_alias("pilot-root")
        .expect("runtime-object root should materialize into the scene runtime")
        .to_string();
    let cockpit_id = resolver
        .resolve_alias("runtime-objects/pilot-root/cockpit")
        .expect("runtime-object child path alias should resolve")
        .to_string();
    let runtime_root = runtime
        .object(&runtime_root_id)
        .expect("runtime-object root should exist");
    let cockpit = runtime
        .object(&cockpit_id)
        .expect("runtime-object child should exist");

    assert_eq!(cockpit.parent_id.as_deref(), Some(runtime_root_id.as_str()));
    assert!(
        runtime_root.children.iter().any(|child| child == &cockpit_id),
        "runtime-object hierarchy should still materialize while bootstrap waits for gameplay entities"
    );
}

#[test]
fn runtime_object_scenes_retry_pending_bootstrap_once_a_gameplay_entity_appears() {
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-bootstrap-retry
title: Runtime Object Bootstrap Retry
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
layers: []
"#,
    )
    .expect("scene parse");

    let mut catalogs = ModCatalogs::default();
    catalogs.presets = CatalogPresets {
        players: [(
            "flight-player".to_string(),
            PlayerPreset {
                input_profile: Some("default-flight".to_string()),
                controller: Some(ControllerComponent {
                    controller_type: "VehicleAssembly".to_string(),
                    config: None,
                }),
                components: Some(PrefabComponents {
                    flight_motor_3d: Some(FlightMotor3DComponent {
                        translational_dofs: Some([true, true, true]),
                        rotational_dofs: Some([true, true, true]),
                        horizon_lock_strength: Some(0.35),
                    }),
                    ..Default::default()
                }),
                config: [("controlled".to_string(), json!(true))]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(catalogs);
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene.clone()));

    prepare_scene_bootstrap(&scene, &mut world);
    activate_scene_bootstrap(&mut world);

    let initial = world
        .get::<AppliedSceneBootstrap>()
        .expect("initial applied bootstrap");
    assert_eq!(
        initial.target_source,
        SceneBootstrapTargetSource::DeferredNo3dEntity
    );
    assert!(initial.has_pending_work());

    let entity_id = gameplay
        .spawn("flight-player", json!({}))
        .expect("spawn gameplay entity");
    assert!(gameplay.set_transform3d(
        entity_id,
        Transform3D {
            position: [0.0, 1.0, 2.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    ));

    let retried = apply_pending_scene_bootstrap_core(&mut world);
    assert!(
        retried.is_some(),
        "bootstrap retry should update state once a gameplay entity appears"
    );

    let applied = world
        .get::<AppliedSceneBootstrap>()
        .expect("retried applied bootstrap");
    assert_eq!(applied.target_entity, Some(entity_id));
    assert_eq!(
        applied.target_source,
        SceneBootstrapTargetSource::Sole3dEntity
    );
    assert_eq!(
        applied.controlled_entity_applied,
        BootstrapApplyState::Applied
    );
    assert_eq!(gameplay.controlled_entity(), Some(entity_id));
    assert!(
        gameplay.flight_motor3d(entity_id).is_some(),
        "catalog-backed bootstrap assembly should attach after retry"
    );

    let runtime = world.get_mut::<SceneRuntime>().expect("scene runtime");
    let resolver = runtime.target_resolver();
    assert!(resolver.resolve_alias("pilot-root").is_some());
    assert!(resolver
        .resolve_alias("runtime-objects/pilot-root/cockpit")
        .is_some());
}

#[test]
fn runtime_object_scenes_queue_gameplay_visual_cleanup_without_touching_materialized_runtime_tree()
{
    let scene: Scene = serde_yaml::from_str(
        r#"
id: runtime-object-bootstrap-despawn
title: Runtime Object Bootstrap Despawn
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
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

    let mut catalogs = ModCatalogs::default();
    catalogs.presets = CatalogPresets {
        players: [(
            "flight-player".to_string(),
            PlayerPreset {
                input_profile: Some("default-flight".to_string()),
                controller: Some(ControllerComponent {
                    controller_type: "VehicleAssembly".to_string(),
                    config: None,
                }),
                components: Some(PrefabComponents {
                    flight_motor_3d: Some(FlightMotor3DComponent {
                        translational_dofs: Some([true, true, true]),
                        rotational_dofs: Some([true, true, true]),
                        horizon_lock_strength: Some(0.35),
                    }),
                    ..Default::default()
                }),
                config: [("controlled".to_string(), json!(true))]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let mut world = World::new();
    let gameplay = GameplayWorld::new();
    world.register(catalogs);
    world.register(gameplay.clone());
    world.register_scoped(SceneRuntime::new(scene.clone()));

    let entity_id = gameplay
        .spawn("flight-player", json!({}))
        .expect("spawn gameplay entity");
    assert!(gameplay.set_transform3d(
        entity_id,
        Transform3D {
            position: [0.0, 1.0, 2.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    ));
    assert!(gameplay.set_visual(
        entity_id,
        VisualBinding {
            visual_id: Some("ship".into()),
            additional_visuals: vec!["ghost".into()],
        }
    ));
    assert!(gameplay.set_lifecycle(entity_id, LifecyclePolicy::Ttl));
    assert!(gameplay.set_lifetime(
        entity_id,
        Lifetime {
            ttl_ms: 1,
            original_ttl_ms: 1,
            on_expire: DespawnVisual::None,
        }
    ));

    prepare_scene_bootstrap(&scene, &mut world);
    activate_scene_bootstrap(&mut world);

    let applied_before = world
        .get::<AppliedSceneBootstrap>()
        .expect("applied bootstrap before despawn");
    assert_eq!(applied_before.target_entity, Some(entity_id));
    assert_eq!(
        applied_before.target_source,
        SceneBootstrapTargetSource::Sole3dEntity
    );
    assert_eq!(gameplay.controlled_entity(), Some(entity_id));
    assert!(gameplay.flight_motor3d(entity_id).is_some());

    gameplay_system(&mut world, 5);

    let cleanup_buffer = world
        .get::<VisualCleanupBuffer>()
        .expect("gameplay despawn should queue visuals for cleanup");
    assert!(
        cleanup_buffer.targets.iter().any(|target| target == "ship"),
        "gameplay lifecycle despawn should enqueue the primary gameplay visual"
    );
    assert!(
        cleanup_buffer
            .targets
            .iter()
            .any(|target| target == "ghost"),
        "gameplay lifecycle despawn should enqueue additional gameplay visuals"
    );

    cleanup_visuals(&mut world);

    assert!(
        !gameplay.exists(entity_id),
        "lifecycle-driven gameplay despawn should remove the bootstrap target entity"
    );

    {
        let runtime = world.get_mut::<SceneRuntime>().expect("scene runtime");
        let resolver = runtime.target_resolver();
        assert!(
            resolver.resolve_alias("pilot-root").is_some(),
            "materialized runtime-object root should survive gameplay visual cleanup"
        );
        assert!(
            resolver
                .resolve_alias("runtime-objects/pilot-root/cockpit")
                .is_some(),
            "materialized runtime-object child should survive gameplay visual cleanup"
        );
    }
}
