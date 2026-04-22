use engine_animation::Animator;
use engine_behavior::catalog::ModCatalogs;
use engine_celestial::{
    resolve_official_clock_seconds, AtmosphereSample, BodyPose3, CelestialQueryContext,
    GravitySample3, LocalFrame3, OfficialClockResolution, SitePose3, SurfaceAnchor3, SurfacePoint3,
    SystemQuery3, WorldPoint3,
};
use engine_core::game_state::GameState;
use engine_core::scene::model::SceneWorldModel;
use engine_core::scene::CelestialClockSource;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedCelestialClock {
    pub source: CelestialClockSource,
    pub elapsed_sec: f64,
    pub used_path: Option<&'static str>,
}

impl Default for ResolvedCelestialClock {
    fn default() -> Self {
        Self {
            source: CelestialClockSource::Scene,
            elapsed_sec: 0.0,
            used_path: None,
        }
    }
}

fn assert_celestial_world_model(world: &engine_core::world::World, operation: &str) {
    debug_assert!(
        world
            .get::<engine_scene_runtime::SceneRuntime>()
            .map(|runtime| runtime.scene().world_model == SceneWorldModel::Celestial3D)
            .unwrap_or(true),
        "{operation} is celestial-only and requires `world-model: celestial-3d`"
    );
}

pub fn active_clock_source(world: &engine_core::world::World) -> CelestialClockSource {
    if let Some(clock_source) = world
        .get::<crate::systems::scene_bootstrap::ResolvedSceneBootstrap>()
        .and_then(|bootstrap| bootstrap.authored.resolved_clock_source)
    {
        return clock_source;
    }
    world
        .get::<engine_scene_runtime::SceneRuntime>()
        .map(|runtime| runtime.scene().celestial.clock_source)
        .unwrap_or(CelestialClockSource::Scene)
}

pub fn scene_elapsed_seconds(world: &engine_core::world::World) -> f64 {
    world
        .get::<Animator>()
        .map(|animator| animator.scene_elapsed_ms as f64 / 1000.0)
        .unwrap_or(0.0)
}

pub fn resolve_clock(world: &engine_core::world::World) -> ResolvedCelestialClock {
    let source = active_clock_source(world);
    let scene_sec = scene_elapsed_seconds(world);
    let game_state = world.get::<GameState>();
    match source {
        CelestialClockSource::Scene => ResolvedCelestialClock {
            source,
            elapsed_sec: scene_sec,
            used_path: None,
        },
        CelestialClockSource::Campaign => {
            if let Some(OfficialClockResolution {
                elapsed_sec,
                used_path,
            }) = resolve_official_clock_seconds(game_state, source)
            {
                return ResolvedCelestialClock {
                    source,
                    elapsed_sec,
                    used_path: Some(used_path),
                };
            }
            ResolvedCelestialClock {
                source,
                elapsed_sec: 0.0,
                used_path: None,
            }
        }
        CelestialClockSource::Fixed => {
            if let Some(OfficialClockResolution {
                elapsed_sec,
                used_path,
            }) = resolve_official_clock_seconds(game_state, source)
            {
                return ResolvedCelestialClock {
                    source,
                    elapsed_sec,
                    used_path: Some(used_path),
                };
            }
            ResolvedCelestialClock {
                source,
                elapsed_sec: 0.0,
                used_path: None,
            }
        }
    }
}

pub fn resolve_query_context(world: &engine_core::world::World) -> CelestialQueryContext {
    assert_celestial_world_model(world, "celestial query context");
    let clock = resolve_clock(world);
    CelestialQueryContext::from_elapsed_sec(clock.elapsed_sec)
        .with_scene_meters_per_world_unit(scene_meters_per_world_unit(world))
}

pub fn scene_meters_per_world_unit(world: &engine_core::world::World) -> Option<f64> {
    world
        .get::<engine_scene_runtime::SceneRuntime>()
        .map(|runtime| runtime.spatial_context().scale.meters_per_world_unit)
}

pub fn resolve_body_id<'a>(catalogs: &'a ModCatalogs, body_id: Option<&'a str>) -> Option<&'a str> {
    if let Some(id) = body_id.filter(|id| catalogs.celestial.bodies.contains_key(*id)) {
        return Some(id);
    }
    if catalogs.celestial.bodies.len() == 1 {
        return catalogs
            .celestial
            .bodies
            .keys()
            .next()
            .map(|id| id.as_str());
    }
    None
}

pub fn gravity_sample(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    point: WorldPoint3,
    world: &engine_core::world::World,
) -> Option<GravitySample3> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs
        .celestial
        .gravity_sample_in_context(body_id, point, resolve_query_context(world))
}

pub fn atmosphere_sample(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    point: WorldPoint3,
    world: &engine_core::world::World,
) -> Option<AtmosphereSample> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs
        .celestial
        .atmosphere_sample_in_context(body_id, point, resolve_query_context(world))
}

pub fn local_frame(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    point: WorldPoint3,
    world: &engine_core::world::World,
) -> Option<LocalFrame3> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs
        .celestial
        .local_frame_in_context(body_id, point, resolve_query_context(world))
}

pub fn body_pose(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    world: &engine_core::world::World,
) -> Option<BodyPose3> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs
        .celestial
        .body_pose_in_context(body_id, resolve_query_context(world))
}

#[allow(dead_code)]
pub fn surface_anchor(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    world: &engine_core::world::World,
) -> Option<SurfaceAnchor3> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs
        .celestial
        .surface_anchor_in_context(body_id, resolve_query_context(world))
}

#[allow(dead_code)]
pub fn surface_point(
    catalogs: &ModCatalogs,
    body_id: Option<&str>,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_world: f64,
    world: &engine_core::world::World,
) -> Option<SurfacePoint3> {
    let body_id = resolve_body_id(catalogs, body_id)?;
    catalogs.celestial.surface_point_in_context(
        body_id,
        latitude_deg,
        longitude_deg,
        altitude_world,
        resolve_query_context(world),
    )
}

#[allow(dead_code)]
pub fn site_pose(
    catalogs: &ModCatalogs,
    site_id: &str,
    world: &engine_core::world::World,
) -> Option<SitePose3> {
    catalogs
        .celestial
        .site_pose_in_context(site_id, resolve_query_context(world))
}

#[allow(dead_code)]
pub fn system_query(
    catalogs: &ModCatalogs,
    system_id: &str,
    world: &engine_core::world::World,
) -> Option<SystemQuery3> {
    catalogs
        .celestial
        .system_query_in_context(system_id, resolve_query_context(world))
}

#[cfg(test)]
mod tests {
    use super::{resolve_clock, resolve_query_context, system_query};
    use crate::systems::scene_bootstrap::{ResolvedSceneBootstrap, SceneSimulationBootstrap};
    use engine_animation::Animator;
    use engine_behavior::catalog::{BodyDef, ModCatalogs, SiteDef, SystemDef};
    use engine_celestial::{CAMPAIGN_CLOCK_MS_PATH, CAMPAIGN_CLOCK_SEC_PATH, FIXED_CLOCK_SEC_PATH};
    use engine_core::game_state::GameState;
    use engine_core::scene::CelestialClockSource;
    use engine_core::scene::Scene;
    use engine_core::world::World;
    use engine_scene_runtime::SceneRuntime;

    fn world_with_scene(scene_yaml: &str) -> World {
        let scene: Scene = serde_yaml::from_str(scene_yaml).expect("scene parse");
        let mut world = World::default();
        world.register(SceneRuntime::new(scene));
        world
    }

    fn resolved_bootstrap(
        scene: &Scene,
        clock_source: CelestialClockSource,
    ) -> ResolvedSceneBootstrap {
        let mut authored = SceneSimulationBootstrap::from_scene(scene);
        authored.resolved_clock_source = Some(clock_source);
        ResolvedSceneBootstrap::from_authored(authored)
    }

    #[test]
    fn scene_clock_uses_animator_elapsed_ms() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: scene
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 2300,
            ..Animator::new()
        });

        let clock = resolve_clock(&world);
        assert_eq!(
            clock.source,
            engine_core::scene::CelestialClockSource::Scene
        );
        assert!((clock.elapsed_sec - 2.3).abs() < 0.0001);
    }

    #[test]
    fn resolved_bootstrap_clock_source_wins_over_scene_runtime_source() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: scene
layers: []
"#,
        )
        .expect("scene parse");
        let mut world = World::default();
        world.register(SceneRuntime::new(scene.clone()));
        world.register_scoped(Animator {
            scene_elapsed_ms: 2300,
            ..Animator::new()
        });
        world.register_scoped(resolved_bootstrap(&scene, CelestialClockSource::Campaign));

        let clock = resolve_clock(&world);
        assert_eq!(clock.source, CelestialClockSource::Campaign);
    }

    #[test]
    fn campaign_clock_prefers_official_runtime_path() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: campaign
layers: []
        "#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 1000,
            ..Animator::new()
        });
        let scene = world
            .get::<SceneRuntime>()
            .map(|runtime| runtime.scene().clone())
            .expect("scene runtime");
        world.register_scoped(resolved_bootstrap(&scene, CelestialClockSource::Campaign));
        let state = GameState::new();
        assert!(state.set(CAMPAIGN_CLOCK_MS_PATH, serde_json::json!(4200)));
        world.register(state);

        let clock = resolve_clock(&world);
        assert_eq!(clock.source, CelestialClockSource::Campaign);
        assert!((clock.elapsed_sec - 4.2).abs() < 0.0001);
        assert_eq!(clock.used_path, Some(CAMPAIGN_CLOCK_MS_PATH));
    }

    #[test]
    fn campaign_clock_uses_zero_when_official_runtime_missing() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: campaign
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 1500,
            ..Animator::new()
        });

        let clock = resolve_clock(&world);
        assert_eq!(clock.used_path, None);
        assert!((clock.elapsed_sec - 0.0).abs() < 0.0001);
    }

    #[test]
    fn fixed_clock_uses_zero_when_official_runtime_missing() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: fixed
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 9999,
            ..Animator::new()
        });

        let clock = resolve_clock(&world);
        assert_eq!(clock.used_path, None);
        assert!((clock.elapsed_sec - 0.0).abs() < 0.0001);
    }

    #[test]
    fn fixed_clock_uses_official_fixed_path() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: fixed
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 9999,
            ..Animator::new()
        });
        let scene = world
            .get::<SceneRuntime>()
            .map(|runtime| runtime.scene().clone())
            .expect("scene runtime");
        world.register_scoped(resolved_bootstrap(&scene, CelestialClockSource::Fixed));
        let state = GameState::new();
        assert!(state.set(FIXED_CLOCK_SEC_PATH, serde_json::json!(12.5)));
        world.register(state);

        let clock = resolve_clock(&world);
        assert_eq!(clock.source, CelestialClockSource::Fixed);
        assert!((clock.elapsed_sec - 12.5).abs() < 0.0001);
        assert_eq!(clock.used_path, Some(FIXED_CLOCK_SEC_PATH));
    }

    #[test]
    fn scene_clock_query_context_uses_animator_elapsed_and_scene_scale() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
spatial:
  meters-per-world-unit: 3.0
celestial:
  clock-source: scene
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 3000,
            ..Animator::new()
        });

        let ctx = resolve_query_context(&world);
        assert!((ctx.elapsed_sec - 3.0).abs() < 0.0001);
        assert_eq!(ctx.scene_meters_per_world_unit, Some(3.0));
    }

    #[test]
    fn campaign_clock_query_context_uses_runtime_time() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
spatial:
  meters-per-world-unit: 2.0
celestial:
  clock-source: scene
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 1500,
            ..Animator::new()
        });
        let scene = world
            .get::<SceneRuntime>()
            .map(|runtime| runtime.scene().clone())
            .expect("scene runtime");
        world.register_scoped(resolved_bootstrap(&scene, CelestialClockSource::Campaign));
        let state = GameState::new();
        assert!(state.set(CAMPAIGN_CLOCK_SEC_PATH, serde_json::json!(8.75)));
        world.register(state);

        let ctx = resolve_query_context(&world);
        assert!((ctx.elapsed_sec - 8.75).abs() < 0.0001);
        assert_eq!(ctx.scene_meters_per_world_unit, Some(2.0));
    }

    #[test]
    fn fixed_clock_query_context_uses_runtime_time() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
spatial:
  meters-per-world-unit: 4.0
celestial:
  clock-source: scene
layers: []
"#,
        );
        world.register_scoped(Animator {
            scene_elapsed_ms: 5000,
            ..Animator::new()
        });
        let scene = world
            .get::<SceneRuntime>()
            .map(|runtime| runtime.scene().clone())
            .expect("scene runtime");
        world.register_scoped(resolved_bootstrap(&scene, CelestialClockSource::Fixed));
        let state = GameState::new();
        assert!(state.set(FIXED_CLOCK_SEC_PATH, serde_json::json!(12.5)));
        world.register(state);

        let ctx = resolve_query_context(&world);
        assert!((ctx.elapsed_sec - 12.5).abs() < 0.0001);
        assert_eq!(ctx.scene_meters_per_world_unit, Some(4.0));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "celestial query context is celestial-only")]
    fn query_context_rejects_euclidean_world_model() {
        let world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: euclidean-3d
celestial:
  clock-source: scene
layers: []
"#,
        );

        let _ = resolve_query_context(&world);
    }

    #[test]
    fn system_query_uses_resolved_runtime_clock() {
        let mut world = world_with_scene(
            r#"
id: clock-scene
title: Clock
world-model: celestial-3d
celestial:
  clock-source: campaign
layers: []
"#,
        );
        let state = GameState::new();
        assert!(state.set(CAMPAIGN_CLOCK_MS_PATH, serde_json::json!(50000)));
        world.register(state);

        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "sun".into(),
            BodyDef {
                center_x: 10.0,
                center_y: 20.0,
                ..BodyDef::default()
            },
        );
        catalogs.celestial.systems.insert(
            "sol".into(),
            SystemDef {
                star: Some("sun".into()),
                bodies: vec!["sun".into()],
                ..SystemDef::default()
            },
        );
        catalogs.celestial.sites.insert(
            "station".into(),
            SiteDef {
                system: Some("sol".into()),
                ..SiteDef::default()
            },
        );

        let query = system_query(&catalogs, "sol", &world).expect("system query");
        assert_eq!(query.star_body_id.as_deref(), Some("sun"));
        assert_eq!(query.sites, vec!["station".to_string()]);
    }
}
