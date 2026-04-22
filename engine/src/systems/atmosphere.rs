use engine_behavior::catalog::ModCatalogs;
use engine_celestial::WorldPoint3;
use engine_game::GameplayWorld;
use engine_game::ReferenceFrameMode;
use serde_json::json;

pub fn atmosphere_system(world: &mut engine_core::world::World, dt_ms: u64) {
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

    for id in gameplay_world.ids_with_atmosphere() {
        let Some(mut atmo) = gameplay_world.atmosphere(id) else {
            continue;
        };
        let Some(body_id) =
            super::celestial_runtime::resolve_body_id(catalogs, atmo.body_id.as_deref())
        else {
            continue;
        };
        let Some(xf) = gameplay_world.transform(id) else {
            continue;
        };
        let Some(mut phys) = gameplay_world.physics(id) else {
            continue;
        };
        let Some(sample) = super::celestial_runtime::atmosphere_sample(
            catalogs,
            Some(body_id),
            WorldPoint3 {
                x: xf.x as f64,
                y: xf.y as f64,
                z: xf.z as f64,
            },
            world,
        ) else {
            continue;
        };
        let atmo_alpha = sample.density as f32;
        let dense_alpha = sample.dense_density as f32;
        let drag = sample.drag as f32 * atmo.drag_scale.max(0.0);

        if drag > 0.0 {
            let damp = 1.0 / (1.0 + drag * dt);
            phys.vx *= damp;
            phys.vy *= damp;
            phys.vz *= damp;
            let _ = gameplay_world.set_physics(id, phys);
        }

        let speed = (phys.vx * phys.vx + phys.vy * phys.vy + phys.vz * phys.vz).sqrt();
        let heat_gain = atmo_alpha * speed * 0.0032 + dense_alpha * speed * 0.0021;
        let cooling = atmo.cooling.max(0.0) * if atmo_alpha > 0.0 { 0.35 } else { 1.0 };
        atmo.heat =
            (atmo.heat + heat_gain * atmo.heat_scale.max(0.0) * dt - cooling * dt).clamp(0.0, 1.0);
        atmo.density = atmo_alpha;
        atmo.dense_density = dense_alpha;
        atmo.altitude_km = sample.altitude_km as f32;

        let _ = gameplay_world.set_atmosphere_state(id, atmo.clone());
        let _ = gameplay_world.set(id, "/env/heat", json!(atmo.heat as f64));
        let _ = gameplay_world.set(id, "/env/density", json!(atmo.density as f64));
        let _ = gameplay_world.set(id, "/env/dense_density", json!(atmo.dense_density as f64));
        let _ = gameplay_world.set(id, "/env/altitude_km", json!(atmo.altitude_km as f64));
    }

    for id in gameplay_world.ids_with_atmosphere() {
        let Some(mut atmo) = gameplay_world.atmosphere(id) else {
            continue;
        };
        let Some(xf) = gameplay_world.transform3d(id) else {
            continue;
        };
        let Some(mut phys) = gameplay_world.physics3d(id) else {
            continue;
        };
        let binding = gameplay_world.reference_frame3d(id);
        let Some(body_id) = super::celestial_runtime::resolve_body_id(
            catalogs,
            atmo.body_id.as_deref().or_else(|| {
                binding.as_ref().and_then(|binding| {
                    if matches!(
                        binding.mode,
                        ReferenceFrameMode::CelestialBody
                            | ReferenceFrameMode::LocalHorizon
                            | ReferenceFrameMode::Orbital
                    ) {
                        binding.body_id.as_deref()
                    } else {
                        None
                    }
                })
            }),
        ) else {
            continue;
        };

        let Some(sample) = super::celestial_runtime::atmosphere_sample(
            catalogs,
            Some(body_id),
            WorldPoint3 {
                x: xf.position[0] as f64,
                y: xf.position[1] as f64,
                z: xf.position[2] as f64,
            },
            world,
        ) else {
            continue;
        };

        let drag = sample.drag as f32 * atmo.drag_scale.max(0.0);
        if drag > 0.0 {
            let damp = 1.0 / (1.0 + drag * dt);
            phys.linear_velocity[0] *= damp;
            phys.linear_velocity[1] *= damp;
            phys.linear_velocity[2] *= damp;
            let _ = gameplay_world.set_physics3d(id, phys);
        }

        let speed = (phys.linear_velocity[0] * phys.linear_velocity[0]
            + phys.linear_velocity[1] * phys.linear_velocity[1]
            + phys.linear_velocity[2] * phys.linear_velocity[2])
            .sqrt();
        let density = sample.density as f32;
        let dense_density = sample.dense_density as f32;
        let heat_gain = density * speed * 0.0032 + dense_density * speed * 0.0021;
        let cooling = atmo.cooling.max(0.0) * if density > 0.0 { 0.35 } else { 1.0 };
        atmo.heat =
            (atmo.heat + heat_gain * atmo.heat_scale.max(0.0) * dt - cooling * dt).clamp(0.0, 1.0);
        atmo.density = density;
        atmo.dense_density = dense_density;
        atmo.altitude_km = sample.altitude_km as f32;

        let _ = gameplay_world.set_atmosphere_state(id, atmo.clone());
        let _ = gameplay_world.set(id, "/env/heat", json!(atmo.heat as f64));
        let _ = gameplay_world.set(id, "/env/density", json!(atmo.density as f64));
        let _ = gameplay_world.set(id, "/env/dense_density", json!(atmo.dense_density as f64));
        let _ = gameplay_world.set(id, "/env/altitude_km", json!(atmo.altitude_km as f64));
    }
}

#[cfg(test)]
mod tests {
    use super::atmosphere_system;
    use engine_behavior::catalog::{BodyDef, ModCatalogs};
    use engine_core::scene::Scene;
    use engine_core::world::World;
    use engine_game::{AtmosphereAffected2D, GameplayWorld, PhysicsBody2D, Transform2D};
    use engine_scene_runtime::SceneRuntime;
    use serde_json::json;

    #[test]
    fn atmosphere_applies_drag_and_updates_runtime_state() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                surface_radius: 90.0,
                radius_km: Some(5000.0),
                km_per_px: Some(50.0),
                atmosphere_top_km: Some(100.0),
                atmosphere_dense_start_km: Some(20.0),
                atmosphere_drag_max: Some(2.0),
                ..BodyDef::default()
            },
        );
        world.register(gameplay.clone());
        world.register(catalogs);

        let id = gameplay.spawn("probe", json!({})).expect("spawn probe");
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 91.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(
            id,
            PhysicsBody2D {
                vx: 10.0,
                vy: 0.0,
                ..PhysicsBody2D::default()
            }
        ));
        assert!(gameplay.attach_atmosphere(
            id,
            AtmosphereAffected2D {
                body_id: Some("planet".into()),
                ..AtmosphereAffected2D::default()
            }
        ));

        atmosphere_system(&mut world, 1000);

        let body = gameplay.physics(id).expect("physics after atmosphere");
        let atmo = gameplay.atmosphere(id).expect("atmo state");
        assert!(
            body.vx < 10.0,
            "expected drag to reduce speed, got {}",
            body.vx
        );
        assert!(
            atmo.density > 0.0,
            "expected atmosphere density to be tracked"
        );
        assert!(atmo.altitude_km >= 0.0, "expected altitude to be recorded");
    }

    #[test]
    fn atmosphere_uses_scene_spatial_scale_when_body_km_scale_is_missing() {
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
                atmosphere_top_km: Some(10.0),
                atmosphere_drag_max: Some(0.0),
                ..BodyDef::default()
            },
        );
        let scene: Scene = serde_yaml::from_str(
            r#"
id: spatial-scene
title: spatial-scene
stages:
  on_idle:
    trigger: any-key
    steps: []
spatial:
  meters-per-world-unit: 2.0
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
                x: 95.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(
            id,
            PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                ..PhysicsBody2D::default()
            }
        ));
        assert!(gameplay.attach_atmosphere(
            id,
            AtmosphereAffected2D {
                body_id: Some("planet".into()),
                ..AtmosphereAffected2D::default()
            }
        ));

        atmosphere_system(&mut world, 16);

        let atmo = gameplay.atmosphere(id).expect("atmo state");
        assert!(
            atmo.density > 0.99,
            "expected near-surface density with spatial scale fallback, got {}",
            atmo.density
        );
    }

    #[test]
    fn atmosphere_uses_z_altitude_and_damps_vz() {
        let mut world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                surface_radius: 90.0,
                radius_km: Some(5000.0),
                km_per_px: Some(50.0),
                atmosphere_top_km: Some(100.0),
                atmosphere_dense_start_km: Some(20.0),
                atmosphere_drag_max: Some(2.0),
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
                z: 91.0,
                heading: 0.0
            }
        ));
        assert!(gameplay.set_physics(
            id,
            PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                vz: 10.0,
                ..PhysicsBody2D::default()
            }
        ));
        assert!(gameplay.attach_atmosphere(
            id,
            AtmosphereAffected2D {
                body_id: Some("planet".into()),
                ..AtmosphereAffected2D::default()
            }
        ));

        atmosphere_system(&mut world, 1000);

        let body = gameplay.physics(id).expect("physics after atmosphere");
        let atmo = gameplay.atmosphere(id).expect("atmo state");
        assert!(
            body.vz < 10.0,
            "expected drag to reduce z speed, got {}",
            body.vz
        );
        assert!(
            atmo.altitude_km > 0.0,
            "expected z-only altitude to contribute"
        );
    }
}
