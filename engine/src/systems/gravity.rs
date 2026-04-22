use engine_behavior::catalog::ModCatalogs;
use engine_game::{GameplayWorld, GravityMode2D, ReferenceFrameMode};

pub fn gravity_system(world: &mut engine_core::world::World, dt_ms: u64) {
    if dt_ms == 0 {
        return;
    }
    let dt = dt_ms as f32 / 1000.0;
    let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() else {
        return;
    };
    let Some(catalogs) = world.get::<ModCatalogs>() else {
        return;
    };
    for id in gameplay_world.ids_with_gravity() {
        let Some(gravity) = gameplay_world.gravity(id) else {
            continue;
        };
        if gravity.gravity_scale <= 0.0 {
            continue;
        }
        let Some(mut body) = gameplay_world.physics(id) else {
            continue;
        };
        let Some(xf) = gameplay_world.transform(id) else {
            continue;
        };

        let (ax, ay, az) = match gravity.mode {
            GravityMode2D::Flat => (gravity.flat_ax, gravity.flat_ay, 0.0),
            GravityMode2D::Point => {
                let Some(sample) = super::celestial_runtime::gravity_sample(
                    catalogs,
                    gravity.body_id.as_deref(),
                    engine_celestial::WorldPoint3 {
                        x: xf.x as f64,
                        y: xf.y as f64,
                        z: xf.z as f64,
                    },
                    world,
                ) else {
                    continue;
                };
                (
                    sample.accel.x as f32,
                    sample.accel.y as f32,
                    sample.accel.z as f32,
                )
            }
        };

        body.vx += ax * gravity.gravity_scale * dt;
        body.vy += ay * gravity.gravity_scale * dt;
        body.vz += az * gravity.gravity_scale * dt;
        let _ = gameplay_world.set_physics(id, body);
    }

    for id in gameplay_world.ids_with_physics3d() {
        let Some(binding) = gameplay_world.reference_frame3d(id) else {
            continue;
        };
        if !matches!(
            binding.mode,
            ReferenceFrameMode::CelestialBody
                | ReferenceFrameMode::LocalHorizon
                | ReferenceFrameMode::Orbital
        ) {
            continue;
        }
        let Some(xf) = gameplay_world.transform3d(id) else {
            continue;
        };
        let Some(mut body) = gameplay_world.physics3d(id) else {
            continue;
        };
        let Some(sample) = super::celestial_runtime::gravity_sample(
            catalogs,
            binding.body_id.as_deref(),
            engine_celestial::WorldPoint3 {
                x: xf.position[0] as f64,
                y: xf.position[1] as f64,
                z: xf.position[2] as f64,
            },
            world,
        ) else {
            continue;
        };
        body.linear_velocity[0] += sample.accel.x as f32 * dt;
        body.linear_velocity[1] += sample.accel.y as f32 * dt;
        body.linear_velocity[2] += sample.accel.z as f32 * dt;
        let _ = gameplay_world.set_physics3d(id, body);
    }
}

#[cfg(test)]
mod tests {
    use super::gravity_system;
    use crate::systems::atmosphere::atmosphere_system;
    use engine_behavior::catalog::{BodyDef, ModCatalogs};
    use engine_core::scene::Scene;
    use engine_core::world::World;
    use engine_game::{
        AtmosphereAffected2D, GameplayWorld, GravityAffected2D, GravityMode2D, PhysicsBody2D,
        Transform2D,
    };
    use engine_scene_runtime::SceneRuntime;
    use serde_json::json;

    #[test]
    fn point_gravity_pulls_entity_toward_body_center() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                gravity_mu: 1000.0,
                ..BodyDef::default()
            },
        );
        world.register(gameplay.clone());
        world.register(catalogs);

        let id = gameplay.spawn("probe", json!({})).expect("spawn probe");
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(id, PhysicsBody2D::default()));
        assert!(gameplay.attach_gravity(
            id,
            GravityAffected2D {
                mode: GravityMode2D::Point,
                body_id: Some("planet".into()),
                ..GravityAffected2D::default()
            }
        ));

        gravity_system(&mut world, 1000);

        let body = gameplay.physics(id).expect("physics after gravity");
        assert!(
            body.vx < 0.0,
            "expected inward pull on x axis, got {}",
            body.vx
        );
    }

    #[test]
    fn point_gravity_pulls_entity_toward_body_center_in_z() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                gravity_mu: 1000.0,
                ..BodyDef::default()
            },
        );
        world.register(gameplay.clone());
        world.register(catalogs);

        let id = gameplay.spawn("probe", json!({})).expect("spawn probe");
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 0.0,
                y: 0.0,
                z: 10.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(id, PhysicsBody2D::default()));
        assert!(gameplay.attach_gravity(
            id,
            GravityAffected2D {
                mode: GravityMode2D::Point,
                body_id: Some("planet".into()),
                ..GravityAffected2D::default()
            }
        ));

        gravity_system(&mut world, 1000);

        let body = gameplay.physics(id).expect("physics after gravity");
        assert!(
            body.vz < 0.0,
            "expected inward pull on z axis, got {}",
            body.vz
        );
    }

    #[test]
    fn point_gravity_uses_physical_mu_with_scene_spatial_scale() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                gravity_mu: 0.0,
                gravity_mu_km3_s2: Some(1000.0),
                ..BodyDef::default()
            },
        );
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gravity-scene
title: gravity-scene
stages:
  on_idle:
    trigger: any-key
    steps: []
spatial:
  meters-per-world-unit: 2000.0
layers: []
"#,
        )
        .expect("scene should parse");
        world.register(gameplay.clone());
        world.register(catalogs);
        world.register(SceneRuntime::new(scene));

        let id = gameplay.spawn("probe", json!({})).expect("spawn probe");
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(id, PhysicsBody2D::default()));
        assert!(gameplay.attach_gravity(
            id,
            GravityAffected2D {
                mode: GravityMode2D::Point,
                body_id: Some("planet".into()),
                ..GravityAffected2D::default()
            }
        ));

        gravity_system(&mut world, 1000);

        let body = gameplay.physics(id).expect("physics after gravity");
        // mu_wu = 1000 / 2^3 = 125. At r=10 => ax = -125/100 = -1.25
        assert!(
            body.vx < -1.2,
            "expected converted physical mu to affect vx"
        );
    }

    #[test]
    fn scene_spatial_drives_gravity_and_atmosphere_without_km_per_px() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                surface_radius: 90.0,
                radius_px: 90.0,
                gravity_mu: 0.0,
                gravity_mu_km3_s2: Some(1000.0),
                atmosphere_top_km: Some(10.0),
                atmosphere_dense_start_km: Some(2.0),
                atmosphere_drag_max: Some(0.0),
                ..BodyDef::default()
            },
        );
        let scene: Scene = serde_yaml::from_str(
            r#"
id: physical-scene
title: physical-scene
stages:
  on_idle:
    trigger: any-key
    steps: []
spatial:
  meters-per-world-unit: 2000.0
layers: []
"#,
        )
        .expect("scene should parse");
        world.register(gameplay.clone());
        world.register(catalogs);
        world.register(SceneRuntime::new(scene));

        let gravity_probe = gameplay
            .spawn("gravity-probe", json!({}))
            .expect("spawn gravity");
        assert!(gameplay.set_transform(
            gravity_probe,
            Transform2D {
                x: 10.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(gravity_probe, PhysicsBody2D::default()));
        assert!(gameplay.attach_gravity(
            gravity_probe,
            GravityAffected2D {
                mode: GravityMode2D::Point,
                body_id: Some("planet".into()),
                ..GravityAffected2D::default()
            }
        ));

        let atmo_probe = gameplay
            .spawn("atmo-probe", json!({}))
            .expect("spawn atmosphere");
        assert!(gameplay.set_transform(
            atmo_probe,
            Transform2D {
                x: 91.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(atmo_probe, PhysicsBody2D::default()));
        assert!(gameplay.attach_atmosphere(
            atmo_probe,
            AtmosphereAffected2D {
                body_id: Some("planet".into()),
                ..AtmosphereAffected2D::default()
            }
        ));

        gravity_system(&mut world, 1000);
        atmosphere_system(&mut world, 16);

        let gravity_state = gameplay.physics(gravity_probe).expect("gravity physics");
        assert!(
            gravity_state.vx < -1.2,
            "expected physical gravity converted via scene spatial scale"
        );

        let atmo_state = gameplay.atmosphere(atmo_probe).expect("atmosphere state");
        assert!(
            atmo_state.density > 0.75,
            "expected atmosphere density from km top + scene spatial scale, got {}",
            atmo_state.density
        );
    }
}
