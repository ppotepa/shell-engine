use engine_behavior::catalog::{BodyDef, ModCatalogs};
use engine_game::{point_gravity_accel_2d, GameplayWorld, GravityMode2D};
use engine_scene_runtime::SceneRuntime;

fn resolve_body<'a>(catalogs: &'a ModCatalogs, body_id: Option<&str>) -> Option<&'a BodyDef> {
    if let Some(id) = body_id {
        return catalogs.celestial.bodies.get(id);
    }
    if catalogs.celestial.bodies.len() == 1 {
        return catalogs.celestial.bodies.values().next();
    }
    None
}

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
    let spatial_meters_per_world_unit = world
        .get::<SceneRuntime>()
        .map(|runtime| runtime.spatial_context().scale.meters_per_world_unit);

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

        let (ax, ay) = match gravity.mode {
            GravityMode2D::Flat => (gravity.flat_ax, gravity.flat_ay),
            GravityMode2D::Point => {
                let Some(source) = resolve_body(catalogs, gravity.body_id.as_deref()) else {
                    continue;
                };
                let dx = source.center_x as f32 - xf.x;
                let dy = source.center_y as f32 - xf.y;
                let mu_world_units =
                    source.resolved_gravity_mu_world_units(spatial_meters_per_world_unit) as f32;
                let Some((ax, ay)) = point_gravity_accel_2d(dx, dy, mu_world_units) else {
                    continue;
                };
                (ax, ay)
            }
        };

        body.vx += ax * gravity.gravity_scale * dt;
        body.vy += ay * gravity.gravity_scale * dt;
        let _ = gameplay_world.set_physics(id, body);
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
