//! Vehicle domain Rhai registration plus thin assembly glue.
//!
//! The script-facing `vehicle.*` surface remains delegated to `engine-api`.
//! Vehicle-specific runtime assembly parsing lives here instead of
//! `gameplay_impl.rs`, and now routes through `engine-vehicle::assembly`
//! so future `input` / `handoff` / `selection` / `assembly` growth stays
//! localized.
//!
//! This module intentionally does not own vehicle control semantics or concrete
//! ship runtime logic. Its job is limited to adapting typed `engine-vehicle`
//! assembly DTOs onto primitive `engine-game` components and delegating the
//! required script-facing runtime passthrough to `engine-api`.

use engine_game::components::{AngularBody, ArcadeController, LinearBrake, ThrusterRamp};
use engine_game::GameplayWorld;
use engine_vehicle::{
    assembly::VehicleAssemblyPlan, AngularBodyConfig, ArcadeConfig, LinearBrakeConfig,
    ThrusterRampConfig, VehicleAssemblyContext, VehicleAssemblySink,
};
use rhai::Engine as RhaiEngine;
use rhai::Map as RhaiMap;

pub(crate) use engine_api::ScriptVehicleApi;

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine_api::register_vehicle_api(engine);
}

fn arcade_controller_from_config(arcade: ArcadeConfig) -> ArcadeController {
    let mut controller = ArcadeController::new(
        arcade.turn_step_ms.max(1),
        arcade.thrust_power,
        arcade.max_speed,
        arcade.heading_bits.max(1),
    );
    if let Some(heading) = arcade.initial_heading {
        controller.set_heading_radians(heading);
    }
    controller
}

fn angular_body_from_config(config: AngularBodyConfig) -> AngularBody {
    AngularBody {
        accel: config.accel,
        max: config.max,
        deadband: config.deadband,
        auto_brake: config.auto_brake,
        angular_vel: config.angular_vel,
        ..Default::default()
    }
}

fn linear_brake_from_config(config: LinearBrakeConfig) -> LinearBrake {
    LinearBrake {
        decel: config.decel,
        deadband: config.deadband,
        auto_brake: config.auto_brake,
        active: config.active,
    }
}

fn thruster_ramp_from_config(config: ThrusterRampConfig) -> ThrusterRamp {
    ThrusterRamp {
        thrust_delay_ms: config.thrust_delay_ms,
        thrust_ramp_ms: config.thrust_ramp_ms,
        no_input_threshold_ms: config.no_input_threshold_ms,
        rot_factor_max_vel: config.rot_factor_max_vel,
        burst_speed_threshold: config.burst_speed_threshold,
        burst_wave_interval_ms: config.burst_wave_interval_ms,
        burst_wave_count: config.burst_wave_count,
        rot_deadband: config.rot_deadband,
        move_deadband: config.move_deadband,
        ..Default::default()
    }
}

pub(crate) fn angular_body_from_rhai_map(config: &RhaiMap) -> AngularBody {
    angular_body_from_config(AngularBodyConfig::from_map(config))
}

pub(crate) fn linear_brake_from_rhai_map(config: &RhaiMap) -> LinearBrake {
    linear_brake_from_config(LinearBrakeConfig::from_map(config))
}

pub(crate) fn thruster_ramp_from_rhai_map(config: &RhaiMap) -> ThrusterRamp {
    thruster_ramp_from_config(ThrusterRampConfig::from_map(config))
}

struct GameplayVehicleAssemblySink<'a> {
    world: &'a GameplayWorld,
    id: u64,
}

impl VehicleAssemblySink for GameplayVehicleAssemblySink<'_> {
    type Error = ();

    fn attach_arcade(&mut self, arcade: ArcadeConfig) -> Result<(), Self::Error> {
        self.world
            .attach_controller(self.id, arcade_controller_from_config(arcade))
            .then_some(())
            .ok_or(())
    }

    fn attach_angular_body(&mut self, angular_body: AngularBodyConfig) -> Result<(), Self::Error> {
        self.world
            .attach_angular_body(self.id, angular_body_from_config(angular_body))
            .then_some(())
            .ok_or(())
    }

    fn attach_linear_brake(&mut self, linear_brake: LinearBrakeConfig) -> Result<(), Self::Error> {
        self.world
            .attach_linear_brake(self.id, linear_brake_from_config(linear_brake))
            .then_some(())
            .ok_or(())
    }

    fn attach_thruster_ramp(
        &mut self,
        thruster_ramp: ThrusterRampConfig,
    ) -> Result<(), Self::Error> {
        self.world
            .attach_thruster_ramp(self.id, thruster_ramp_from_config(thruster_ramp))
            .then_some(())
            .ok_or(())
    }
}

pub(crate) fn attach_vehicle_stack(world: &GameplayWorld, id: u64, config: RhaiMap) -> bool {
    let heading = world.transform(id).map(|xf| xf.heading);
    let plan = match VehicleAssemblyPlan::from_rhai_map_with_context(
        &config,
        VehicleAssemblyContext { heading },
    ) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("[attach_controller] invalid vehicle assembly for entity {id}: {err:?}");
            return false;
        }
    };

    let mut sink = GameplayVehicleAssemblySink { world, id };
    plan.apply(&mut sink).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_vehicle::VehicleAssembly;
    use rhai::{FLOAT as RhaiFloat, INT as RhaiInt};

    #[test]
    fn nested_vehicle_profile_flows_through_engine_vehicle_normalization() {
        let mut arcade = RhaiMap::new();
        arcade.insert("turn_step_ms".into(), (48 as RhaiInt).into());
        arcade.insert("thrust_power".into(), (80.0 as RhaiFloat).into());
        arcade.insert("max_speed".into(), (150.0 as RhaiFloat).into());
        arcade.insert("heading_bits".into(), (16 as RhaiInt).into());

        let mut angular = RhaiMap::new();
        angular.insert("accel".into(), (6.0 as RhaiFloat).into());
        angular.insert("max".into(), (8.0 as RhaiFloat).into());
        angular.insert("deadband".into(), (0.2 as RhaiFloat).into());
        angular.insert("auto_brake".into(), true.into());

        let mut root = RhaiMap::new();
        root.insert("arcade".into(), arcade.into());
        root.insert("angular_body".into(), angular.into());

        let profile = VehicleAssembly::from_rhai_map(&root)
            .expect("vehicle assembly")
            .to_profile_input()
            .expect("profile input");

        assert_eq!(profile.turn_step_ms, Some(48));
        assert_eq!(profile.heading_bits, Some(16));
        assert_eq!(profile.thrust_power, 80.0);
        assert_eq!(profile.angular_accel, 6.0);
        assert!(profile.angular_auto_brake);
    }

    #[test]
    fn attach_vehicle_stack_supports_angular_only_vehicle_submaps() {
        let world = GameplayWorld::new();
        let id = world
            .spawn("vehicle", serde_json::json!({}))
            .expect("vehicle id");

        let mut angular = RhaiMap::new();
        angular.insert("accel".into(), (5.0 as RhaiFloat).into());
        angular.insert("max".into(), (7.0 as RhaiFloat).into());

        let mut config = RhaiMap::new();
        config.insert("angular_body".into(), angular.into());

        assert!(attach_vehicle_stack(&world, id, config));
        assert!(world.angular_body(id).is_some());
        assert!(world.controller(id).is_none());
    }

    #[test]
    fn attach_vehicle_stack_accepts_descriptor_annotated_plan_maps() {
        let world = GameplayWorld::new();
        let id = world
            .spawn("vehicle", serde_json::json!({}))
            .expect("vehicle id");

        let mut arcade = RhaiMap::new();
        arcade.insert("turn_step_ms".into(), (48 as RhaiInt).into());
        arcade.insert("thrust_power".into(), (80.0 as RhaiFloat).into());
        arcade.insert("max_speed".into(), (150.0 as RhaiFloat).into());
        arcade.insert("heading_bits".into(), (16 as RhaiInt).into());

        let mut config = RhaiMap::new();
        config.insert("kind".into(), " sim_lite ".into());
        config.insert("profile".into(), " sim_lite ".into());
        config.insert("label".into(), " Surveyor ".into());
        config.insert("arcade".into(), arcade.into());

        assert!(attach_vehicle_stack(&world, id, config));
        assert!(world.controller(id).is_some());
    }

    #[test]
    fn register_with_rhai_exposes_vehicle_input_and_handoff_bridge() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push(
            "vehicle",
            ScriptVehicleApi::from_gameplay_world(Some(world)),
        );

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let intent = vehicle.input_intent_from(#{
                        throttle: 2.0,
                        yaw: -2.0,
                        strafe: 0.25,
                        lift: -3.0,
                        boost: true
                    });
                    let control = vehicle.control_from_intent(intent, true, true);
                    let packet_vehicle = vehicle.packet_vehicle_from_control(control, 6.0);
                    let ret = vehicle.return_packet_from(#{
                        producer_mod_id: "vehicle-playground",
                        source_scene_id: "vehicle-scene",
                        target_mod_ref: "planet-generator",
                        target_scene_id: "generator-scene",
                        return_scene_id: "generator-scene",
                        consumer_hint: "generator-return",
                        planet: #{
                            body: #{
                                id: "generated-planet",
                                planet_type: "earth_like"
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 1.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 12.0,
                            atmo_drag_max: 2.0
                        },
                        vehicle: packet_vehicle.to_map(),
                        telemetry: #{
                            heading_deg: 450.0,
                            altitude_km: 4.0,
                            tangent_speed_kms: 1.2
                        }
                    });

                    #{
                        intent: intent.to_map(),
                        control: control.to_map(),
                        packet_vehicle: packet_vehicle.to_map(),
                        return_packet: ret.to_map(),
                        is_return: ret.is_vehicle_return()
                    }
                "#,
            )
            .expect("vehicle bridge should resolve in behavior-owned engine");

        let intent = result
            .get("intent")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("intent map");
        let throttle = intent
            .get("throttle")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("intent throttle");
        let lift = intent
            .get("lift")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("intent lift");
        assert_eq!(throttle, 1.0);
        assert_eq!(lift, -1.0);

        let control = result
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control map");
        let assists = control
            .get("assists")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("assists map");
        let heading_hold = assists
            .get("heading_hold")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("heading hold");
        assert!(heading_hold);

        let packet_vehicle = result
            .get("packet_vehicle")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("packet vehicle map");
        let profile = packet_vehicle
            .get("profile")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("packet profile");
        assert_eq!(profile, "arcade");

        let return_packet = result
            .get("return_packet")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("return packet map");
        let packet_kind = return_packet
            .get("packet_kind")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("packet kind");
        assert_eq!(packet_kind, "vehicle_return");

        let is_return = result
            .get("is_return")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("is return");
        assert!(is_return);
    }

    #[test]
    fn register_with_rhai_normalizes_surface_runtime_packet_telemetry() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push(
            "vehicle",
            ScriptVehicleApi::from_gameplay_world(Some(world)),
        );

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let telemetry = vehicle.packet_telemetry_from(#{
                        heading_deg: -90.0,
                        altitude_km: 0.0,
                        tangent_speed_kms: 1.1,
                        radial_speed_kms: -0.05,
                        spawn_angle_deg: -45.0,
                        camera_sway: 0.25,
                        radius_wu: 128.0,
                        vfwd_wu_s: 2.5,
                        vright_wu_s: -0.5,
                        vrad_wu_s: -0.1,
                        yaw_rate_rad_s: 0.6,
                        grounded: true
                    });
                    let ret = vehicle.return_packet_from(#{
                        producer_mod_id: "vehicle-playground",
                        source_scene_id: "vehicle-scene",
                        target_mod_ref: "planet-generator",
                        target_scene_id: "generator-scene",
                        return_scene_id: "generator-scene",
                        consumer_hint: "generator-return",
                        planet: #{
                            body: #{
                                id: "generated-planet",
                                planet_type: "earth_like"
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 50.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 12.0,
                            atmo_drag_max: 2.0
                        },
                        telemetry: telemetry.to_map()
                    });

                    #{
                        telemetry: telemetry.to_map(),
                        return_packet: ret.to_map()
                    }
                "#,
            )
            .expect("surface telemetry should normalize through behavior-owned engine");

        let telemetry = result
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("telemetry map");
        let telemetry_heading = telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry heading");
        let telemetry_spawn_angle = telemetry
            .get("spawn_angle_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry spawn angle");
        let telemetry_grounded = telemetry
            .get("grounded")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("telemetry grounded");
        assert_eq!(telemetry_heading, 270.0);
        assert_eq!(telemetry_spawn_angle, 315.0);
        assert!(telemetry_grounded);

        let return_packet = result
            .get("return_packet")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("return packet map");
        let return_telemetry = return_packet
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("return telemetry");
        let return_radius = return_telemetry
            .get("radius_wu")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("return radius");
        let return_grounded = return_telemetry
            .get("grounded")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("return grounded");
        assert_eq!(return_radius, 128.0);
        assert!(return_grounded);
    }

    #[test]
    fn register_with_rhai_preserves_ship_runtime_passthrough() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push(
            "vehicle",
            ScriptVehicleApi::from_gameplay_world(Some(world)),
        );

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let state = vehicle.ship_runtime_state_from(#{
                        surface_mode: "surface_locked",
                        control: #{ profile_id: " sim_lite " }
                    });
                    let input = vehicle.ship_runtime_input_from(#{
                        control: #{ profile_id: " sim_lite ", throttle: 0.5 },
                        telemetry: #{
                            heading_deg: -90.0,
                            altitude_km: 0.0,
                            tangent_speed_kms: 1.1,
                            grounded: true
                        }
                    });
                    let step = vehicle.ship_runtime_step(" sim_lite ", state, input);

                    #{
                        state: state.to_map(),
                        step: step.to_map()
                    }
                "#,
            )
            .expect("behavior-owned Rhai engine should preserve runtime passthrough");

        let state = result
            .get("state")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("state map");
        let state_control = state
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("state control");
        let profile = state_control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("state profile");
        assert_eq!(profile, "sim-lite");

        let step = result
            .get("step")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step map");
        let telemetry = step
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step telemetry");
        let heading = telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("step heading");
        assert_eq!(heading, 270.0);
    }
}
