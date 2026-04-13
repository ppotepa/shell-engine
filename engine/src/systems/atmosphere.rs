use engine_behavior::catalog::{BodyDef, ModCatalogs};
use engine_game::GameplayWorld;
use serde_json::json;

fn resolve_body<'a>(catalogs: &'a ModCatalogs, body_id: Option<&str>) -> Option<&'a BodyDef> {
    if let Some(id) = body_id {
        return catalogs.celestial.bodies.get(id);
    }
    if catalogs.celestial.bodies.len() == 1 {
        return catalogs.celestial.bodies.values().next();
    }
    None
}

fn body_km_per_px(body: &BodyDef) -> f32 {
    if let Some(km_per_px) = body.km_per_px {
        return km_per_px.max(0.0001) as f32;
    }
    if let Some(radius_km) = body.radius_km {
        return (radius_km / body.radius_px.max(1.0)) as f32;
    }
    (6371.0 / body.radius_px.max(1.0)) as f32
}

fn body_atmo_top_km(body: &BodyDef, km_per_px: f32) -> f32 {
    body.atmosphere_top_km
        .or_else(|| body.atmosphere_top.map(|px| px * km_per_px as f64))
        .unwrap_or(0.0) as f32
}

fn body_atmo_dense_km(body: &BodyDef, km_per_px: f32) -> f32 {
    body.atmosphere_dense_start_km
        .or_else(|| body.atmosphere_dense_start.map(|px| px * km_per_px as f64))
        .unwrap_or(0.0) as f32
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

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
        let Some(body) = resolve_body(catalogs, atmo.body_id.as_deref()) else {
            continue;
        };
        let Some(xf) = gameplay_world.transform(id) else {
            continue;
        };
        let Some(mut phys) = gameplay_world.physics(id) else {
            continue;
        };

        let km_per_px = body_km_per_px(body);
        let atmo_top_km = body_atmo_top_km(body, km_per_px);
        let atmo_dense_km = body_atmo_dense_km(body, km_per_px);
        let drag_max = body.atmosphere_drag_max.unwrap_or(0.0) as f32;

        let dx = xf.x - body.center_x as f32;
        let dy = xf.y - body.center_y as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        let altitude_px = (dist - body.surface_radius as f32).max(0.0);
        let altitude_km = altitude_px * km_per_px;

        let atmo_alpha = if atmo_top_km > 0.0 {
            clamp01((atmo_top_km - altitude_km) / atmo_top_km.max(0.001))
        } else {
            0.0
        };
        let dense_alpha = if atmo_dense_km > 0.0 {
            clamp01((atmo_dense_km - altitude_km) / atmo_dense_km.max(0.001))
        } else {
            0.0
        };
        let drag = atmo_alpha * atmo_alpha * drag_max * atmo.drag_scale.max(0.0);

        if drag > 0.0 {
            let damp = 1.0 / (1.0 + drag * dt);
            phys.vx *= damp;
            phys.vy *= damp;
            let _ = gameplay_world.set_physics(id, phys);
        }

        let speed = (phys.vx * phys.vx + phys.vy * phys.vy).sqrt();
        let heat_gain = atmo_alpha * speed * 0.0032 + dense_alpha * speed * 0.0021;
        let cooling = atmo.cooling.max(0.0) * if atmo_alpha > 0.0 { 0.35 } else { 1.0 };
        atmo.heat =
            (atmo.heat + heat_gain * atmo.heat_scale.max(0.0) * dt - cooling * dt).clamp(0.0, 1.0);
        atmo.density = atmo_alpha;
        atmo.dense_density = dense_alpha;
        atmo.altitude_km = altitude_km;

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
    use engine_core::world::World;
    use engine_game::{AtmosphereAffected2D, GameplayWorld, PhysicsBody2D, Transform2D};
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
}
