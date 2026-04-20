use serde::{Deserialize, Serialize};

use crate::{
    builtin_ship_profile_tuning, AngularBodyConfig, ArcadeConfig, LinearBrakeConfig,
    ThrusterRampConfig, VehicleAssembly, VehicleAssistState, VehicleCapabilities,
    VehicleControlState, VehicleDescriptor, VehicleEnvironmentBinding, VehicleKind,
    VehicleProfileInput, VehicleShipProfileTuning,
};
use crate::{
    runtime::{
        ShipMotionState, ShipReferenceFrameKind, ShipReferenceFrameState, ShipRuntimeInput,
        ShipRuntimeModel, ShipRuntimeOutput, ShipRuntimeState, ShipRuntimeStepReport,
        ShipSurfaceMode,
    },
    VehicleAssemblyModel, VehicleControllerModel, VehicleReferenceFrameModel,
    VehicleTelemetryModel,
};

/// First concrete typed vehicle model backed by the current ship-like motion stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipModel {
    pub profile_id: String,
    pub label: Option<String>,
    pub tuning: VehicleShipProfileTuning,
    pub heading_bits: u8,
    pub turn_step_ms: u32,
    pub thrust_power: f32,
    pub max_speed: f32,
    pub angular_accel: f32,
    pub angular_max: f32,
    pub angular_deadband: f32,
    pub angular_auto_brake: bool,
    pub linear_brake_decel: f32,
    pub linear_brake_deadband: f32,
    pub linear_auto_brake: bool,
    pub thruster_ramp_enabled: bool,
    pub assists: VehicleAssistState,
}

impl Default for ShipModel {
    fn default() -> Self {
        Self {
            profile_id: "arcade".to_string(),
            label: None,
            tuning: builtin_ship_profile_tuning("arcade"),
            heading_bits: 32,
            turn_step_ms: 60,
            thrust_power: 8.0,
            max_speed: 20.0,
            angular_accel: 5.5,
            angular_max: 7.0,
            angular_deadband: 0.10,
            angular_auto_brake: true,
            linear_brake_decel: 45.0,
            linear_brake_deadband: 2.5,
            linear_auto_brake: true,
            thruster_ramp_enabled: true,
            assists: VehicleAssistState::default(),
        }
    }
}

impl ShipModel {
    pub fn new(profile_id: &str) -> Self {
        let profile_id = crate::normalize_vehicle_profile_id(profile_id);
        Self {
            tuning: builtin_ship_profile_tuning(&profile_id),
            profile_id,
            ..Self::default()
        }
    }

    pub fn kind(&self) -> VehicleKind {
        VehicleKind::Ship
    }

    pub fn capabilities(&self) -> VehicleCapabilities {
        VehicleCapabilities::ship()
    }

    pub fn profile_input(&self) -> VehicleProfileInput {
        self.vehicle_assembly()
            .to_profile_input()
            .unwrap_or_default()
    }

    pub fn descriptor(&self) -> VehicleDescriptor {
        VehicleDescriptor::ship(self.clone())
    }

    pub fn ship_tuning(&self) -> VehicleShipProfileTuning {
        self.tuning.normalized()
    }

    pub fn ship_surface_clearance_wu(&self, km_per_wu: f32) -> f32 {
        self.ship_tuning().surface_clearance_wu(km_per_wu)
    }

    fn ship_requests_takeoff(&self, control: &VehicleControlState) -> bool {
        control.lift > self.ship_tuning().takeoff_lift_threshold
    }

    fn ship_surface_anchor_available(
        &self,
        previous: &ShipRuntimeState,
        input: &ShipRuntimeInput,
        reference_frame: &ShipReferenceFrameState,
        environment: Option<&VehicleEnvironmentBinding>,
    ) -> bool {
        input.request_surface_lock
            || previous.reference_frame.has_surface_anchor()
            || input.telemetry.ship_reference.has_surface_anchor()
            || reference_frame.has_surface_anchor()
            || environment
                .map(|environment| environment.has_body())
                .unwrap_or(false)
    }

    fn ship_surface_contact(
        &self,
        input: &ShipRuntimeInput,
        reference_frame: &ShipReferenceFrameState,
        environment: Option<&VehicleEnvironmentBinding>,
        surface_anchor_available: bool,
    ) -> bool {
        let surface_contact_altitude_km = self.ship_surface_contact_altitude_threshold_km();
        input.surface_contact.unwrap_or_else(|| {
            input.telemetry.grounded
                || input.telemetry.surface_mode.is_grounded()
                || (surface_anchor_available
                    && (input.telemetry.altitude_km <= surface_contact_altitude_km
                        || reference_frame.altitude_km <= surface_contact_altitude_km
                        || environment
                            .map(|environment| {
                                environment.is_near_surface(
                                    input.telemetry.radius_wu.max(reference_frame.radius_wu),
                                    surface_contact_altitude_km,
                                )
                            })
                            .unwrap_or(false)))
        })
    }

    fn ship_local_horizon_requested(
        &self,
        previous: &ShipRuntimeState,
        input: &ShipRuntimeInput,
        reference_frame: &ShipReferenceFrameState,
        surface_anchor_available: bool,
    ) -> bool {
        input.request_local_horizon
            || input.request_surface_lock
            || previous.reference_frame.uses_local_horizon()
            || input.telemetry.ship_reference.uses_local_horizon()
            || reference_frame.uses_local_horizon()
            || surface_anchor_available
    }
}

impl VehicleAssemblyModel for ShipModel {
    fn vehicle_kind(&self) -> VehicleKind {
        self.kind()
    }

    fn capabilities(&self) -> VehicleCapabilities {
        self.capabilities()
    }

    fn vehicle_assembly(&self) -> VehicleAssembly {
        VehicleAssembly {
            arcade: Some(ArcadeConfig {
                turn_step_ms: self.turn_step_ms.max(1),
                thrust_power: self.thrust_power,
                max_speed: self.max_speed,
                heading_bits: self.heading_bits.max(1),
                initial_heading: None,
            }),
            angular_body: Some(AngularBodyConfig {
                accel: self.angular_accel,
                max: self.angular_max,
                deadband: self.angular_deadband,
                auto_brake: self.angular_auto_brake,
                angular_vel: 0.0,
            }),
            linear_brake: Some(LinearBrakeConfig {
                decel: self.linear_brake_decel,
                deadband: self.linear_brake_deadband,
                auto_brake: self.linear_auto_brake,
                active: false,
            }),
            thruster_ramp: self
                .thruster_ramp_enabled
                .then_some(ThrusterRampConfig::default()),
        }
    }
}

impl VehicleControllerModel for ShipModel {
    fn default_control_state(&self) -> VehicleControlState {
        let mut state = VehicleControlState::with_profile_id(&self.profile_id);
        state.assists = self.assists;
        state
    }
}

impl VehicleReferenceFrameModel for ShipModel {}

impl VehicleTelemetryModel for ShipModel {}

impl ShipRuntimeModel for ShipModel {
    fn ship_grounded_speed_threshold_wu_s(&self) -> f32 {
        self.ship_tuning().grounded_speed_threshold_wu_s
    }

    fn ship_surface_contact_altitude_threshold_km(&self) -> f32 {
        self.ship_tuning().surface_contact_altitude_threshold_km
    }

    fn ship_runtime_step(
        &self,
        previous: ShipRuntimeState,
        input: ShipRuntimeInput,
    ) -> ShipRuntimeOutput {
        let previous = previous.normalized();
        let input = input.normalized();
        let mut reference_frame = if input.telemetry.ship_reference.is_configured() {
            input.telemetry.ship_reference.clone()
        } else {
            previous
                .reference_frame
                .clone()
                .with_reference(input.telemetry.reference)
        };
        let environment = input
            .environment
            .clone()
            .or_else(|| input.telemetry.environment.clone())
            .or_else(|| previous.environment.clone());
        if let Some(environment) = environment.as_ref() {
            reference_frame = reference_frame.with_environment(environment);
        }

        let surface_anchor_available = self.ship_surface_anchor_available(
            &previous,
            &input,
            &reference_frame,
            environment.as_ref(),
        );
        let surface_contact = self.ship_surface_contact(
            &input,
            &reference_frame,
            environment.as_ref(),
            surface_anchor_available,
        );
        let local_horizon_requested = self.ship_local_horizon_requested(
            &previous,
            &input,
            &reference_frame,
            surface_anchor_available,
        );
        let takeoff_requested = self.ship_requests_takeoff(&input.control);
        let grounded_speed_threshold = self.ship_grounded_speed_threshold_wu_s().max(0.0);
        let motion_is_groundable = input.telemetry.forward_speed_wu_s.abs()
            <= grounded_speed_threshold
            && input.telemetry.lateral_speed_wu_s.abs() <= grounded_speed_threshold
            && input.telemetry.radial_speed_wu_s.abs() <= grounded_speed_threshold;
        let keep_grounded = previous.surface_mode.is_grounded()
            && surface_contact
            && !takeoff_requested
            && input.telemetry.radial_speed_wu_s.abs() <= grounded_speed_threshold;

        let surface_mode = if input.request_detach || input.request_inertial_frame {
            ShipSurfaceMode::Detached
        } else if surface_contact
            && input.prefer_grounded_on_contact
            && !takeoff_requested
            && (keep_grounded || motion_is_groundable)
        {
            ShipSurfaceMode::Grounded
        } else if surface_anchor_available {
            ShipSurfaceMode::SurfaceLocked
        } else {
            ShipSurfaceMode::Detached
        };

        if surface_mode.is_surface_locked() {
            reference_frame.kind = ShipReferenceFrameKind::LocalHorizon;
            reference_frame.co_rotation_enabled = true;
        } else {
            reference_frame.co_rotation_enabled = false;
            reference_frame.kind = if input.request_inertial_frame || !local_horizon_requested {
                ShipReferenceFrameKind::Inertial
            } else {
                ShipReferenceFrameKind::LocalHorizon
            };
        }

        if let Some(environment) = environment.as_ref() {
            reference_frame = reference_frame.with_environment(environment);
        }

        let motion = ShipMotionState::from_telemetry(
            &input.telemetry,
            &reference_frame,
            environment.as_ref(),
        );
        let state = ShipRuntimeState {
            surface_mode,
            reference_frame,
            motion,
            control: input.control,
            environment,
        }
        .normalized();
        let telemetry = input.telemetry.with_ship_runtime_state(&state);

        ShipRuntimeOutput {
            surface_mode_changed: previous.surface_mode != state.surface_mode,
            reference_frame_changed: previous.reference_frame != state.reference_frame,
            motion_changed: previous.motion != state.motion,
            report: ShipRuntimeStepReport::default(),
            state,
            telemetry,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        builtin_ship_profile_tuning,
        runtime::{ShipRuntimeInput, ShipRuntimeModel, ShipSurfaceMode},
        BrakePhase, MotionFrameInput, VehicleAssemblyModel, VehicleControlState,
        VehicleControllerModel, VehicleEnvironmentBinding, VehiclePacketTelemetry,
        VehicleReferenceFrameModel, VehicleTelemetryInput, VehicleTelemetryModel,
    };

    use super::ShipModel;

    #[test]
    fn ship_model_builds_current_vehicle_stack() {
        let model = ShipModel::new("arc");
        let assembly = model.vehicle_assembly();

        assert_eq!(model.profile_id, "arcade");
        assert_eq!(model.ship_tuning(), builtin_ship_profile_tuning("arcade"));
        assert!(assembly.arcade.is_some());
        assert!(assembly.angular_body.is_some());
        assert!(assembly.linear_brake.is_some());
        assert!(assembly.thruster_ramp.is_some());
    }

    #[test]
    fn ship_model_owns_builtin_runtime_tuning_by_profile() {
        let arcade = ShipModel::new("arcade");
        let sim_lite = ShipModel::new("sim_lite");

        assert!(arcade.ship_tuning().forward_accel_g > sim_lite.ship_tuning().forward_accel_g);
        assert!(arcade.ship_tuning().main_engine_g > sim_lite.ship_tuning().main_engine_g);
        assert!(arcade.ship_tuning().yaw_max > sim_lite.ship_tuning().yaw_max);
        assert_eq!(arcade.ship_grounded_speed_threshold_wu_s(), 2.5);
        assert_eq!(sim_lite.ship_surface_contact_altitude_threshold_km(), 0.05);
        assert!((arcade.ship_surface_clearance_wu(50.0) - 0.05).abs() < 0.0001);
    }

    #[test]
    fn ship_model_dispatches_control_and_telemetry() {
        let model = ShipModel::default();
        let control = model.default_control_state();
        assert!(control.profile_matches("arcade"));

        let motion = model.motion_frame_from_input(
            MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -10.0,
                accel_x: 0.0,
                accel_y: -2.0,
            },
            0.0,
        );
        assert!((motion.forward_speed - 10.0).abs() < 0.001);

        let telemetry = model.telemetry_from_input(VehicleTelemetryInput {
            heading: 0.0,
            motion: Some(MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -10.0,
                accel_x: 0.0,
                accel_y: -2.0,
            }),
            thrust_input: 1.0,
            is_thrusting: true,
            brake_phase: Some(BrakePhase::Thrusting),
            ..VehicleTelemetryInput::default()
        });
        assert!(telemetry.is_thrusting);
        assert_eq!(telemetry.brake_phase, BrakePhase::Thrusting);
    }

    #[test]
    fn ship_model_owns_ship_runtime_transition_thresholds() {
        let model = ShipModel::default();
        let grounded_telemetry =
            crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                heading_deg: 0.0,
                altitude_km: 0.0,
                radius_wu: 120.0,
                spawn_angle_deg: 15.0,
                grounded: true,
                ..VehiclePacketTelemetry::default()
            });
        let output = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: grounded_telemetry,
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(model.ship_grounded_speed_threshold_wu_s(), 2.5);
        assert_eq!(model.ship_surface_contact_altitude_threshold_km(), 0.05);
        assert_eq!(output.state.surface_mode, ShipSurfaceMode::Grounded);
        assert!(output.state.reference_frame.uses_local_horizon());
    }

    #[test]
    fn ship_model_promotes_lift_off_to_surface_locked_without_detaching() {
        let model = ShipModel::default();
        let grounded_telemetry =
            crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                heading_deg: 0.0,
                altitude_km: 0.0,
                radius_wu: 120.0,
                spawn_angle_deg: 15.0,
                grounded: true,
                ..VehiclePacketTelemetry::default()
            });
        let grounded = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: grounded_telemetry,
                environment: Some(
                    VehicleEnvironmentBinding {
                        body_id: "generated".to_string(),
                        body_kind: "earth_like".to_string(),
                        surface_radius_wu: 120.0,
                        scale_divisor: 50.0,
                        extras: serde_json::Map::from_iter([(
                            "rotspeed".to_string(),
                            serde_json::Value::from(3.0),
                        )]),
                        ..VehicleEnvironmentBinding::default()
                    }
                    .normalized(),
                ),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );
        let grounded_state = grounded.state.clone();
        let grounded_telemetry = grounded.telemetry.clone();
        let liftoff = model.ship_runtime_step(
            grounded_state.clone(),
            ShipRuntimeInput {
                control: VehicleControlState {
                    lift: 1.0,
                    ..model.default_control_state()
                },
                telemetry: grounded_telemetry,
                environment: grounded_state.environment.clone(),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(liftoff.state.surface_mode, ShipSurfaceMode::SurfaceLocked);
        assert!(liftoff.state.reference_frame.uses_local_horizon());
        assert!(liftoff.state.reference_frame.uses_co_rotation());
        assert!(liftoff.state.reference_frame.carrier_speed_wu_s > 0.0);
    }

    #[test]
    fn ship_model_reports_stable_runtime_output_when_grounded_state_is_replayed() {
        let model = ShipModel::default();
        let environment = VehicleEnvironmentBinding {
            body_id: "generated".to_string(),
            body_kind: "earth_like".to_string(),
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            extras: serde_json::Map::from_iter([(
                "rotspeed".to_string(),
                serde_json::Value::from(3.0),
            )]),
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();
        let grounded = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                    heading_deg: 0.0,
                    altitude_km: 0.0,
                    radius_wu: 120.0,
                    spawn_angle_deg: 15.0,
                    grounded: true,
                    ..VehiclePacketTelemetry::default()
                }),
                environment: Some(environment.clone()),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );
        let replay = model.ship_runtime_step(
            grounded.state.clone(),
            ShipRuntimeInput {
                control: grounded.state.control.clone(),
                telemetry: grounded.telemetry.clone(),
                environment: Some(environment),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(replay.state.surface_mode, ShipSurfaceMode::Grounded);
        assert!(!replay.surface_mode_changed);
        assert!(!replay.reference_frame_changed);
        assert!(replay.state.reference_frame.uses_local_horizon());
        assert!(replay.state.reference_frame.uses_co_rotation());
        assert!(replay.state.reference_frame.carrier_speed_wu_s > 0.0);
        assert!(replay.telemetry.grounded);
        assert_eq!(replay.telemetry.surface_mode, ShipSurfaceMode::Grounded);
    }

    #[test]
    fn ship_model_detach_keeps_anchor_but_clears_co_rotation_and_grounding() {
        let model = ShipModel::default();
        let environment = VehicleEnvironmentBinding {
            body_id: "generated".to_string(),
            body_kind: "earth_like".to_string(),
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            extras: serde_json::Map::from_iter([(
                "rotspeed".to_string(),
                serde_json::Value::from(3.0),
            )]),
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();
        let grounded = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                    heading_deg: 0.0,
                    altitude_km: 0.0,
                    radius_wu: 120.0,
                    spawn_angle_deg: 15.0,
                    grounded: true,
                    ..VehiclePacketTelemetry::default()
                }),
                environment: Some(environment.clone()),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );
        let detached = model.ship_runtime_step(
            grounded.state.clone(),
            ShipRuntimeInput {
                control: grounded.state.control.clone(),
                telemetry: grounded.telemetry.clone(),
                environment: Some(environment),
                surface_contact: Some(true),
                request_detach: true,
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(detached.state.surface_mode, ShipSurfaceMode::Detached);
        assert!(detached.surface_mode_changed);
        assert!(detached.reference_frame_changed);
        assert!(detached.state.reference_frame.uses_local_horizon());
        assert!(!detached.state.reference_frame.uses_co_rotation());
        assert!(detached.state.reference_frame.has_surface_anchor());
        assert_eq!(detached.state.reference_frame.body_id, "generated");
        assert_eq!(detached.state.reference_frame.anchor_angle_deg, 15.0);
        assert!(detached.state.reference_frame.carrier_speed_wu_s > 0.0);
        assert_eq!(detached.state.reference_frame.carrier_velocity_x, 0.0);
        assert_eq!(detached.state.reference_frame.carrier_velocity_y, 0.0);
        assert!(!detached.telemetry.grounded);
        assert_eq!(detached.telemetry.surface_mode, ShipSurfaceMode::Detached);
    }

    #[test]
    fn ship_model_surface_locked_replay_only_flips_surface_mode_output_flag() {
        let model = ShipModel::default();
        let environment = VehicleEnvironmentBinding {
            body_id: "generated".to_string(),
            body_kind: "earth_like".to_string(),
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            extras: serde_json::Map::from_iter([(
                "rotspeed".to_string(),
                serde_json::Value::from(3.0),
            )]),
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();
        let grounded = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                    heading_deg: 0.0,
                    altitude_km: 0.0,
                    radius_wu: 120.0,
                    spawn_angle_deg: 15.0,
                    grounded: true,
                    ..VehiclePacketTelemetry::default()
                }),
                environment: Some(environment.clone()),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );
        let surface_locked = model.ship_runtime_step(
            grounded.state.clone(),
            ShipRuntimeInput {
                control: grounded.state.control.clone(),
                telemetry: grounded.telemetry.clone(),
                environment: Some(environment),
                surface_contact: Some(true),
                prefer_grounded_on_contact: false,
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(
            surface_locked.state.surface_mode,
            ShipSurfaceMode::SurfaceLocked
        );
        assert!(surface_locked.surface_mode_changed);
        assert!(!surface_locked.reference_frame_changed);
        assert!(surface_locked.state.reference_frame.uses_local_horizon());
        assert!(surface_locked.state.reference_frame.uses_co_rotation());
        assert!(surface_locked.state.reference_frame.carrier_speed_wu_s > 0.0);
        assert!(!surface_locked.telemetry.grounded);
        assert_eq!(
            surface_locked.telemetry.surface_mode,
            ShipSurfaceMode::SurfaceLocked
        );
    }
}
