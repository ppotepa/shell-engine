//! Script-facing typed world-body snapshot surface.
//!
//! `world.body(id)` returns a typed snapshot handle for common celestial/body
//! fields used by gameplay scripts. This avoids map indexing for the hot-path
//! fields while keeping `inspect()` as an escape hatch for compatibility and
//! debugging.

use rhai::{Engine as RhaiEngine, Map as RhaiMap};

/// Typed read-only body snapshot exposed through `world.body(...)`.
pub trait GameplayWorldBodySnapshotCoreApi: Clone + 'static {
    fn exists(&mut self) -> bool;
    fn id(&mut self) -> String;
    fn center_x(&mut self) -> rhai::FLOAT;
    fn center_y(&mut self) -> rhai::FLOAT;
    fn orbit_radius(&mut self) -> rhai::FLOAT;
    fn orbit_period_sec(&mut self) -> rhai::FLOAT;
    fn orbit_phase_deg(&mut self) -> rhai::FLOAT;
    fn radius_px(&mut self) -> rhai::FLOAT;
    fn surface_radius(&mut self) -> rhai::FLOAT;
    fn gravity_mu(&mut self) -> rhai::FLOAT;
    fn gravity_mu_km3_s2(&mut self) -> rhai::FLOAT;
    fn km_per_px(&mut self) -> rhai::FLOAT;
    fn km_per_world_unit(&mut self) -> rhai::FLOAT;
    fn radius_km(&mut self) -> rhai::FLOAT;
    fn resolved_radius_km(&mut self) -> rhai::FLOAT;
    fn resolved_gravity_mu(&mut self) -> rhai::FLOAT;
    fn atmosphere_top_km(&mut self) -> rhai::FLOAT;
    fn atmosphere_dense_start_km(&mut self) -> rhai::FLOAT;
    fn resolved_atmosphere_top_km(&mut self) -> rhai::FLOAT;
    fn resolved_atmosphere_dense_start_km(&mut self) -> rhai::FLOAT;
    fn atmosphere_drag_max(&mut self) -> rhai::FLOAT;
    fn cloud_bottom_km(&mut self) -> rhai::FLOAT;
    fn cloud_top_km(&mut self) -> rhai::FLOAT;
    fn planet_type(&mut self) -> String;
    fn parent(&mut self) -> String;
    fn inspect(&mut self) -> RhaiMap;
}

/// Root-world lookup surface for one typed body snapshot.
pub trait GameplayWorldBodyLookupCoreApi<TBody>: Clone + 'static
where
    TBody: GameplayWorldBodySnapshotCoreApi,
{
    fn body(&mut self, id: &str) -> TBody;
}

/// Register the typed `world.body(...)` / `world.body_snapshot(...)` surface.
pub fn register_world_body_api<TWorld, TBody>(engine: &mut RhaiEngine)
where
    TWorld: GameplayWorldBodyLookupCoreApi<TBody>,
    TBody: GameplayWorldBodySnapshotCoreApi,
{
    engine.register_type_with_name::<TBody>("WorldBodySnapshot");

    engine.register_fn("body", |world: &mut TWorld, id: &str| world.body(id));
    engine.register_fn("body_snapshot", |world: &mut TWorld, id: &str| {
        world.body(id)
    });

    macro_rules! register_body_field {
        ($name:literal, $method:ident) => {
            engine.register_get($name, |body: &mut TBody| body.$method());
            engine.register_fn($name, |body: &mut TBody| body.$method());
        };
    }

    register_body_field!("exists", exists);
    register_body_field!("id", id);
    register_body_field!("center_x", center_x);
    register_body_field!("center_y", center_y);
    register_body_field!("orbit_radius", orbit_radius);
    register_body_field!("orbit_period_sec", orbit_period_sec);
    register_body_field!("orbit_phase_deg", orbit_phase_deg);
    register_body_field!("radius_px", radius_px);
    register_body_field!("surface_radius", surface_radius);
    register_body_field!("gravity_mu", gravity_mu);
    register_body_field!("gravity_mu_km3_s2", gravity_mu_km3_s2);
    register_body_field!("km_per_px", km_per_px);
    register_body_field!("km_per_world_unit", km_per_world_unit);
    register_body_field!("radius_km", radius_km);
    register_body_field!("resolved_radius_km", resolved_radius_km);
    register_body_field!("resolved_gravity_mu", resolved_gravity_mu);
    register_body_field!("atmosphere_top_km", atmosphere_top_km);
    register_body_field!("atmosphere_dense_start_km", atmosphere_dense_start_km);
    register_body_field!("resolved_atmosphere_top_km", resolved_atmosphere_top_km);
    register_body_field!(
        "resolved_atmosphere_dense_start_km",
        resolved_atmosphere_dense_start_km
    );
    register_body_field!("atmosphere_drag_max", atmosphere_drag_max);
    register_body_field!("cloud_bottom_km", cloud_bottom_km);
    register_body_field!("cloud_top_km", cloud_top_km);
    register_body_field!("planet_type", planet_type);
    register_body_field!("parent", parent);

    engine.register_fn("inspect", |body: &mut TBody| body.inspect());
}

#[cfg(test)]
mod tests {
    use rhai::{Engine as RhaiEngine, Map as RhaiMap, Scope as RhaiScope};

    use super::{
        register_world_body_api, GameplayWorldBodyLookupCoreApi, GameplayWorldBodySnapshotCoreApi,
    };

    #[derive(Clone, Default)]
    struct StubBodySnapshot {
        id: String,
        exists: bool,
        center_x: rhai::FLOAT,
        center_y: rhai::FLOAT,
        orbit_radius: rhai::FLOAT,
        orbit_period_sec: rhai::FLOAT,
        orbit_phase_deg: rhai::FLOAT,
        radius_px: rhai::FLOAT,
        surface_radius: rhai::FLOAT,
        gravity_mu: rhai::FLOAT,
        gravity_mu_km3_s2: rhai::FLOAT,
        km_per_px: rhai::FLOAT,
        km_per_world_unit: rhai::FLOAT,
        radius_km: rhai::FLOAT,
        atmosphere_top_km: rhai::FLOAT,
        atmosphere_dense_start_km: rhai::FLOAT,
        atmosphere_drag_max: rhai::FLOAT,
        cloud_bottom_km: rhai::FLOAT,
        cloud_top_km: rhai::FLOAT,
        planet_type: String,
        parent: String,
    }

    impl StubBodySnapshot {
        fn generated_planet() -> Self {
            Self {
                id: "generated-planet".to_string(),
                exists: true,
                center_x: 12.0,
                center_y: -4.0,
                orbit_radius: 0.0,
                orbit_period_sec: 0.0,
                orbit_phase_deg: 0.0,
                radius_px: 210.0,
                surface_radius: 205.0,
                gravity_mu: 4321.5,
                gravity_mu_km3_s2: 4321.5,
                km_per_px: 1.0,
                km_per_world_unit: 1.0,
                radius_km: 210.0,
                atmosphere_top_km: 88.0,
                atmosphere_dense_start_km: 18.0,
                atmosphere_drag_max: 1.5,
                cloud_bottom_km: 6.0,
                cloud_top_km: 12.0,
                planet_type: "earth_like".to_string(),
                parent: String::new(),
            }
        }
    }

    impl GameplayWorldBodySnapshotCoreApi for StubBodySnapshot {
        fn exists(&mut self) -> bool {
            self.exists
        }
        fn id(&mut self) -> String {
            self.id.clone()
        }
        fn center_x(&mut self) -> rhai::FLOAT {
            self.center_x
        }
        fn center_y(&mut self) -> rhai::FLOAT {
            self.center_y
        }
        fn orbit_radius(&mut self) -> rhai::FLOAT {
            self.orbit_radius
        }
        fn orbit_period_sec(&mut self) -> rhai::FLOAT {
            self.orbit_period_sec
        }
        fn orbit_phase_deg(&mut self) -> rhai::FLOAT {
            self.orbit_phase_deg
        }
        fn radius_px(&mut self) -> rhai::FLOAT {
            self.radius_px
        }
        fn surface_radius(&mut self) -> rhai::FLOAT {
            self.surface_radius
        }
        fn gravity_mu(&mut self) -> rhai::FLOAT {
            self.gravity_mu
        }
        fn gravity_mu_km3_s2(&mut self) -> rhai::FLOAT {
            self.gravity_mu_km3_s2
        }
        fn km_per_px(&mut self) -> rhai::FLOAT {
            self.km_per_px
        }
        fn km_per_world_unit(&mut self) -> rhai::FLOAT {
            self.km_per_world_unit
        }
        fn radius_km(&mut self) -> rhai::FLOAT {
            self.radius_km
        }
        fn resolved_radius_km(&mut self) -> rhai::FLOAT {
            self.radius_km
        }
        fn resolved_gravity_mu(&mut self) -> rhai::FLOAT {
            self.gravity_mu
        }
        fn atmosphere_top_km(&mut self) -> rhai::FLOAT {
            self.atmosphere_top_km
        }
        fn atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
            self.atmosphere_dense_start_km
        }
        fn resolved_atmosphere_top_km(&mut self) -> rhai::FLOAT {
            self.atmosphere_top_km
        }
        fn resolved_atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
            self.atmosphere_dense_start_km
        }
        fn atmosphere_drag_max(&mut self) -> rhai::FLOAT {
            self.atmosphere_drag_max
        }
        fn cloud_bottom_km(&mut self) -> rhai::FLOAT {
            self.cloud_bottom_km
        }
        fn cloud_top_km(&mut self) -> rhai::FLOAT {
            self.cloud_top_km
        }
        fn planet_type(&mut self) -> String {
            self.planet_type.clone()
        }
        fn parent(&mut self) -> String {
            self.parent.clone()
        }
        fn inspect(&mut self) -> RhaiMap {
            let mut map = RhaiMap::new();
            map.insert("id".into(), self.id.clone().into());
            map.insert("surface_radius".into(), self.surface_radius.into());
            map.insert("gravity_mu_km3_s2".into(), self.gravity_mu_km3_s2.into());
            map
        }
    }

    #[derive(Clone, Default)]
    struct StubWorldBodies;

    impl GameplayWorldBodyLookupCoreApi<StubBodySnapshot> for StubWorldBodies {
        fn body(&mut self, id: &str) -> StubBodySnapshot {
            if id == "generated-planet" {
                StubBodySnapshot::generated_planet()
            } else {
                StubBodySnapshot::default()
            }
        }
    }

    #[test]
    fn register_world_body_api_exposes_typed_snapshot_fields() {
        let mut engine = RhaiEngine::new();
        register_world_body_api::<StubWorldBodies, StubBodySnapshot>(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", StubWorldBodies);

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let body = world.body("generated-planet");
                    #{
                        exists: body.exists,
                        id: body.id,
                        surface_radius: body.surface_radius,
                        gravity_mu: body.gravity_mu,
                        atmosphere_top_km: body.atmosphere_top_km,
                        planet_type: body.planet_type,
                        inspect_id: body.inspect()["id"]
                    }
                "#,
            )
            .expect("typed world body API should evaluate");

        assert_eq!(
            result
                .get("exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("generated-planet".to_string())
        );
        assert_eq!(
            result
                .get("surface_radius")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(205.0)
        );
        assert_eq!(
            result
                .get("gravity_mu")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(4321.5)
        );
        assert_eq!(
            result
                .get("atmosphere_top_km")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(88.0)
        );
        assert_eq!(
            result
                .get("planet_type")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("earth_like".to_string())
        );
        assert_eq!(
            result
                .get("inspect_id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("generated-planet".to_string())
        );
    }

    #[test]
    fn register_world_body_api_returns_missing_snapshot_for_unknown_body() {
        let mut engine = RhaiEngine::new();
        register_world_body_api::<StubWorldBodies, StubBodySnapshot>(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", StubWorldBodies);

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let body = world.body_snapshot("missing");
                    #{
                        exists: body.exists,
                        id: body.id,
                        radius_km: body.radius_km
                    }
                "#,
            )
            .expect("missing world body snapshot should still evaluate");

        assert_eq!(
            result
                .get("exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(false)
        );
        assert_eq!(
            result
                .get("id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some(String::new())
        );
        assert_eq!(
            result
                .get("radius_km")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(0.0)
        );
    }
}
