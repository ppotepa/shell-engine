use engine_behavior::catalog::{BodyDef, ModCatalogs};
use engine_game::{point_gravity_accel_2d, GameplayWorld, GravityMode2D};

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
                let Some((ax, ay)) = point_gravity_accel_2d(dx, dy, source.gravity_mu as f32)
                else {
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
    use engine_behavior::catalog::{BodyDef, ModCatalogs};
    use engine_core::world::World;
    use engine_game::{
        GameplayWorld, GravityAffected2D, GravityMode2D, PhysicsBody2D, Transform2D,
    };
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
}
