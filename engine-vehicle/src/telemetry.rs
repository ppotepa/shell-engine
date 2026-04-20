use serde::{Deserialize, Serialize};

use super::frame::{VehicleEnvironmentBinding, VehicleReferenceFrame};
use crate::{
    handoff::{VehicleBasis3, VehiclePacketTelemetry},
    runtime::{ShipReferenceFrameKind, ShipReferenceFrameState, ShipRuntimeState, ShipSurfaceMode},
    types::{BrakePhase, MotionFrame, MotionFrameInput, VehicleTelemetry, VehicleTelemetryInput},
};

#[inline]
fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[inline]
fn non_negative(value: f32) -> f32 {
    finite_or_zero(value).max(0.0)
}

/// Vehicle-facing telemetry superset bridging runtime snapshots and handoff DTOs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleTelemetrySnapshot {
    pub reference: VehicleReferenceFrame,
    pub heading_deg: f32,
    pub surface_mode: ShipSurfaceMode,
    pub ship_reference: ShipReferenceFrameState,
    pub motion: MotionFrame,
    pub position_x: f32,
    pub position_y: f32,
    pub turn_input: f32,
    pub thrust_input: f32,
    pub is_thrusting: bool,
    pub is_braking: bool,
    pub angular_vel: f32,
    pub thrust_factor: f32,
    pub rot_factor: f32,
    pub brake_factor: f32,
    pub brake_phase: BrakePhase,
    pub final_burst_fired: bool,
    pub final_burst_wave: u8,
    pub altitude_km: f32,
    pub tangent_speed_kms: f32,
    pub radial_speed_kms: f32,
    pub spawn_angle_deg: f32,
    pub camera_sway: f32,
    pub radius_wu: f32,
    pub forward_speed_wu_s: f32,
    pub lateral_speed_wu_s: f32,
    pub radial_speed_wu_s: f32,
    pub yaw_rate_rad_s: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<VehicleBasis3>,
    pub grounded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<VehicleEnvironmentBinding>,
}

impl VehicleTelemetrySnapshot {
    pub fn from_runtime(input: VehicleTelemetryInput) -> Self {
        Self::from_runtime_telemetry(&VehicleTelemetry::from_runtime(input))
    }

    pub fn from_runtime_telemetry(telemetry: &VehicleTelemetry) -> Self {
        let reference = VehicleReferenceFrame::from_facing(telemetry.facing);
        Self {
            reference,
            heading_deg: reference.heading_deg(),
            surface_mode: ShipSurfaceMode::Detached,
            ship_reference: ShipReferenceFrameState::detached(telemetry.facing.heading),
            motion: telemetry.motion,
            position_x: 0.0,
            position_y: 0.0,
            turn_input: telemetry.turn_input,
            thrust_input: telemetry.thrust_input,
            is_thrusting: telemetry.is_thrusting,
            is_braking: telemetry.is_braking,
            angular_vel: telemetry.angular_vel,
            thrust_factor: telemetry.thrust_factor,
            rot_factor: telemetry.rot_factor,
            brake_factor: telemetry.brake_factor,
            brake_phase: telemetry.brake_phase,
            final_burst_fired: telemetry.final_burst_fired,
            final_burst_wave: telemetry.final_burst_wave,
            altitude_km: 0.0,
            tangent_speed_kms: 0.0,
            radial_speed_kms: 0.0,
            spawn_angle_deg: 0.0,
            camera_sway: 0.0,
            radius_wu: 0.0,
            forward_speed_wu_s: telemetry.motion.forward_speed,
            lateral_speed_wu_s: telemetry.motion.lateral_speed,
            radial_speed_wu_s: 0.0,
            yaw_rate_rad_s: telemetry.angular_vel,
            basis: None,
            grounded: false,
            environment: None,
        }
        .normalized()
    }

    pub fn from_packet(packet: &VehiclePacketTelemetry) -> Self {
        let reference = VehicleReferenceFrame::from_heading(packet.heading_deg.to_radians());
        let motion = reference.resolve_motion(MotionFrameInput {
            velocity_x: packet.vx,
            velocity_y: packet.vy,
            accel_x: 0.0,
            accel_y: 0.0,
        });
        let uses_local_horizon = packet.grounded
            || packet.radius_wu > 0.0
            || packet.altitude_km > 0.0
            || packet.spawn_angle_deg != 0.0
            || packet.basis.is_some();
        let ship_reference = if uses_local_horizon {
            ShipReferenceFrameState::local_horizon(reference.heading_rad, packet.spawn_angle_deg)
                .with_surface_anchor(packet.spawn_angle_deg, packet.radius_wu, packet.altitude_km)
                .with_co_rotation(true)
        } else {
            ShipReferenceFrameState::detached(reference.heading_rad)
        };

        Self {
            reference,
            heading_deg: packet.heading_deg.rem_euclid(360.0),
            surface_mode: if packet.grounded {
                ShipSurfaceMode::Grounded
            } else {
                ShipSurfaceMode::Detached
            },
            ship_reference,
            motion,
            position_x: packet.x,
            position_y: packet.y,
            turn_input: 0.0,
            thrust_input: 0.0,
            is_thrusting: false,
            is_braking: false,
            angular_vel: packet.yaw_rate_rad_s,
            thrust_factor: 0.0,
            rot_factor: 0.0,
            brake_factor: 0.0,
            brake_phase: if packet.grounded {
                BrakePhase::Stopped
            } else {
                BrakePhase::Idle
            },
            final_burst_fired: false,
            final_burst_wave: 0,
            altitude_km: packet.altitude_km,
            tangent_speed_kms: packet.tangent_speed_kms,
            radial_speed_kms: packet.radial_speed_kms,
            spawn_angle_deg: packet.spawn_angle_deg,
            camera_sway: packet.camera_sway,
            radius_wu: packet.radius_wu,
            forward_speed_wu_s: packet.vfwd_wu_s,
            lateral_speed_wu_s: packet.vright_wu_s,
            radial_speed_wu_s: packet.vrad_wu_s,
            yaw_rate_rad_s: packet.yaw_rate_rad_s,
            basis: packet.basis,
            grounded: packet.grounded,
            environment: None,
        }
        .normalized()
    }

    pub fn with_environment(mut self, environment: VehicleEnvironmentBinding) -> Self {
        let environment = environment.normalized();
        self.ship_reference = self.ship_reference.clone().with_environment(&environment);
        self.environment = Some(environment);
        self
    }

    pub fn with_surface_mode(mut self, surface_mode: ShipSurfaceMode) -> Self {
        self.surface_mode = surface_mode;
        self
    }

    pub fn with_ship_reference(mut self, ship_reference: ShipReferenceFrameState) -> Self {
        self.reference = ship_reference.reference;
        self.ship_reference = ship_reference.normalized();
        self
    }

    pub fn with_ship_runtime_state(mut self, state: &ShipRuntimeState) -> Self {
        self.surface_mode = state.surface_mode;
        self.reference = state.reference_frame.reference;
        self.ship_reference = state.reference_frame.clone();
        self.environment = state.environment.clone();
        self.grounded = state.surface_mode.is_grounded();
        self
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.heading_deg = finite_or_zero(self.heading_deg).rem_euclid(360.0);
        if self.reference.heading_rad.abs() <= f32::EPSILON && self.heading_deg != 0.0 {
            self.reference = VehicleReferenceFrame::from_heading(self.heading_deg.to_radians());
        }
        self.reference.normalize();
        if self.surface_mode == ShipSurfaceMode::Detached && self.grounded {
            self.surface_mode = ShipSurfaceMode::Grounded;
        }
        self.position_x = finite_or_zero(self.position_x);
        self.position_y = finite_or_zero(self.position_y);
        self.motion = self.reference.resolve_motion(MotionFrameInput {
            velocity_x: finite_or_zero(self.motion.velocity_x),
            velocity_y: finite_or_zero(self.motion.velocity_y),
            accel_x: finite_or_zero(self.motion.accel_x),
            accel_y: finite_or_zero(self.motion.accel_y),
        });
        self.turn_input = super::clamp_unit(self.turn_input);
        self.thrust_input = finite_or_zero(self.thrust_input);
        self.angular_vel = finite_or_zero(self.angular_vel);
        self.thrust_factor = non_negative(self.thrust_factor);
        self.rot_factor = non_negative(self.rot_factor);
        self.brake_factor = non_negative(self.brake_factor);
        self.altitude_km = non_negative(self.altitude_km);
        self.tangent_speed_kms = non_negative(self.tangent_speed_kms);
        self.radial_speed_kms = finite_or_zero(self.radial_speed_kms);
        self.spawn_angle_deg = finite_or_zero(self.spawn_angle_deg).rem_euclid(360.0);
        self.camera_sway = finite_or_zero(self.camera_sway);
        self.radius_wu = non_negative(self.radius_wu);
        self.forward_speed_wu_s = finite_or_zero(self.forward_speed_wu_s);
        self.lateral_speed_wu_s = finite_or_zero(self.lateral_speed_wu_s);
        self.radial_speed_wu_s = finite_or_zero(self.radial_speed_wu_s);
        self.yaw_rate_rad_s = finite_or_zero(self.yaw_rate_rad_s);
        self.basis = self.basis.take().map(|basis| basis.normalized());
        let anchor_angle_deg = if self.ship_reference.is_configured() {
            self.ship_reference.anchor_angle_deg
        } else {
            self.spawn_angle_deg
        };
        let radius_wu = if self.ship_reference.radius_wu > 0.0 {
            self.ship_reference.radius_wu
        } else {
            self.radius_wu
        };
        let altitude_km = if self.ship_reference.altitude_km > 0.0 {
            self.ship_reference.altitude_km
        } else {
            self.altitude_km
        };
        self.ship_reference = self
            .ship_reference
            .clone()
            .with_reference(self.reference)
            .with_surface_anchor(anchor_angle_deg, radius_wu, altitude_km);
        if self.surface_mode.is_surface_locked() {
            self.ship_reference.kind = ShipReferenceFrameKind::LocalHorizon;
            self.ship_reference.co_rotation_enabled = true;
            self.ship_reference.normalize();
        }
        self.environment = self.environment.take().map(|binding| {
            let binding = binding.normalized();
            self.ship_reference = self.ship_reference.clone().with_environment(&binding);
            binding
        });
        self.reference = self.ship_reference.reference;
        self.heading_deg = self.reference.heading_deg();
        self.is_braking |= matches!(self.brake_phase, BrakePhase::Rotation | BrakePhase::Linear);
        self.grounded = self.surface_mode.is_grounded();
    }

    pub fn to_runtime_telemetry(&self) -> VehicleTelemetry {
        let snapshot = self.clone().normalized();
        VehicleTelemetry {
            facing: snapshot.reference.facing(),
            motion: snapshot.motion,
            turn_input: snapshot.turn_input,
            thrust_input: snapshot.thrust_input,
            is_thrusting: snapshot.is_thrusting,
            is_braking: snapshot.is_braking,
            angular_vel: snapshot.angular_vel,
            thrust_factor: snapshot.thrust_factor,
            rot_factor: snapshot.rot_factor,
            brake_factor: snapshot.brake_factor,
            brake_phase: snapshot.brake_phase,
            final_burst_fired: snapshot.final_burst_fired,
            final_burst_wave: snapshot.final_burst_wave,
        }
    }

    pub fn to_packet_telemetry(&self) -> VehiclePacketTelemetry {
        let snapshot = self.clone().normalized();
        let mut packet = VehiclePacketTelemetry::default();
        packet.x = snapshot.position_x;
        packet.y = snapshot.position_y;
        packet.vx = snapshot.motion.velocity_x;
        packet.vy = snapshot.motion.velocity_y;
        packet.heading_deg = snapshot.reference.heading_deg();
        packet.altitude_km =
            if snapshot.ship_reference.altitude_km > 0.0 || snapshot.surface_mode.is_grounded() {
                snapshot.ship_reference.altitude_km
            } else {
                snapshot.altitude_km
            };
        packet.tangent_speed_kms = snapshot.tangent_speed_kms;
        packet.radial_speed_kms = snapshot.radial_speed_kms;
        packet.spawn_angle_deg = if snapshot.ship_reference.uses_local_horizon() {
            snapshot.ship_reference.anchor_angle_deg
        } else {
            snapshot.spawn_angle_deg
        };
        packet.camera_sway = snapshot.camera_sway;
        packet.radius_wu = if snapshot.ship_reference.radius_wu > 0.0 {
            snapshot.ship_reference.radius_wu
        } else {
            snapshot.radius_wu
        };
        packet.vfwd_wu_s = snapshot.forward_speed_wu_s;
        packet.vright_wu_s = snapshot.lateral_speed_wu_s;
        packet.vrad_wu_s = snapshot.radial_speed_wu_s;
        packet.yaw_rate_rad_s = snapshot.yaw_rate_rad_s;
        packet.basis = snapshot.basis;
        packet.grounded = snapshot.surface_mode.is_grounded();
        packet.normalized()
    }
}

impl From<VehicleTelemetryInput> for VehicleTelemetrySnapshot {
    fn from(value: VehicleTelemetryInput) -> Self {
        Self::from_runtime(value)
    }
}

impl From<&VehicleTelemetry> for VehicleTelemetrySnapshot {
    fn from(value: &VehicleTelemetry) -> Self {
        Self::from_runtime_telemetry(value)
    }
}

impl From<VehicleTelemetry> for VehicleTelemetrySnapshot {
    fn from(value: VehicleTelemetry) -> Self {
        Self::from_runtime_telemetry(&value)
    }
}

impl From<&VehiclePacketTelemetry> for VehicleTelemetrySnapshot {
    fn from(value: &VehiclePacketTelemetry) -> Self {
        Self::from_packet(value)
    }
}

impl From<VehiclePacketTelemetry> for VehicleTelemetrySnapshot {
    fn from(value: VehiclePacketTelemetry) -> Self {
        Self::from_packet(&value)
    }
}

impl From<&VehicleTelemetrySnapshot> for VehicleTelemetry {
    fn from(value: &VehicleTelemetrySnapshot) -> Self {
        value.to_runtime_telemetry()
    }
}

impl From<VehicleTelemetrySnapshot> for VehicleTelemetry {
    fn from(value: VehicleTelemetrySnapshot) -> Self {
        value.to_runtime_telemetry()
    }
}

impl From<&VehicleTelemetrySnapshot> for VehiclePacketTelemetry {
    fn from(value: &VehicleTelemetrySnapshot) -> Self {
        value.to_packet_telemetry()
    }
}

impl From<VehicleTelemetrySnapshot> for VehiclePacketTelemetry {
    fn from(value: VehicleTelemetrySnapshot) -> Self {
        value.to_packet_telemetry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        handoff::VehicleEnvironmentSnapshot,
        runtime::{ShipMotionState, ShipReferenceFrameState, ShipRuntimeState, ShipSurfaceMode},
        VehicleControlState, VehicleLaunchPacket, VehicleReturnPacket,
    };

    #[test]
    fn runtime_snapshot_roundtrips_through_existing_vehicle_telemetry() {
        let input = VehicleTelemetryInput {
            heading: std::f32::consts::FRAC_PI_4,
            motion: Some(MotionFrameInput {
                velocity_x: 2.0,
                velocity_y: -3.0,
                accel_x: 0.5,
                accel_y: -0.25,
            }),
            turn_input: 2.0,
            thrust_input: 0.75,
            is_thrusting: true,
            angular_vel: 0.5,
            angular_deadband: 0.1,
            linear_deadband: 0.2,
            angular_settling: false,
            linear_settling: true,
            thrust_factor: Some(0.75),
            rot_factor: Some(0.25),
            brake_factor: Some(1.0),
            brake_phase: Some(BrakePhase::Linear),
            final_burst_fired: true,
            final_burst_wave: 3,
        };

        let snapshot = VehicleTelemetrySnapshot::from_runtime(input);
        let runtime = snapshot.to_runtime_telemetry();

        assert_eq!(runtime.brake_phase, BrakePhase::Linear);
        assert!(runtime.is_braking);
        assert_eq!(runtime.final_burst_wave, 3);
        assert_eq!(snapshot.forward_speed_wu_s, snapshot.motion.forward_speed);
    }

    #[test]
    fn packet_snapshot_roundtrips_packet_fields() {
        let mut packet = VehiclePacketTelemetry::default();
        packet.x = 15.0;
        packet.y = -20.0;
        packet.vx = 1.5;
        packet.vy = -0.5;
        packet.heading_deg = -90.0;
        packet.altitude_km = 12.0;
        packet.tangent_speed_kms = 1.25;
        packet.radial_speed_kms = -0.15;
        packet.spawn_angle_deg = -45.0;
        packet.camera_sway = 0.4;
        packet.radius_wu = 130.0;
        packet.vfwd_wu_s = 3.25;
        packet.vright_wu_s = -0.75;
        packet.vrad_wu_s = -0.2;
        packet.yaw_rate_rad_s = 0.8;
        packet.basis = Some(VehicleBasis3 {
            normal: [0.0, 1.0, 0.0],
            forward: [1.0, 0.0, 0.0],
            right: [0.0, 0.0, 1.0],
        });
        packet.grounded = false;

        let snapshot = VehicleTelemetrySnapshot::from_packet(&packet);
        let encoded = snapshot.to_packet_telemetry();

        assert_eq!(snapshot.reference.heading_deg(), 270.0);
        assert_eq!(snapshot.forward_speed_wu_s, 3.25);
        assert_eq!(snapshot.lateral_speed_wu_s, -0.75);
        assert_eq!(encoded, packet.normalized());
    }

    #[test]
    fn snapshot_can_attach_environment_binding() {
        let environment = VehicleEnvironmentBinding::from_snapshot(&VehicleEnvironmentSnapshot {
            real_radius_km: 6371.0,
            scale_divisor: 50.0,
            surface_gravity_mps2: 9.81,
            ..VehicleEnvironmentSnapshot::default()
        });
        let snapshot = VehicleTelemetrySnapshot::default().with_environment(environment.clone());

        assert_eq!(snapshot.environment, Some(environment));
    }

    #[test]
    fn grounded_runtime_snapshot_roundtrips_through_packet_telemetry() {
        let environment = VehicleEnvironmentBinding {
            body_id: "generated".to_string(),
            body_kind: "earth_like".to_string(),
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();
        let state = ShipRuntimeState {
            surface_mode: ShipSurfaceMode::Grounded,
            reference_frame: ShipReferenceFrameState::local_horizon(
                std::f32::consts::FRAC_PI_4,
                33.0,
            )
            .with_surface_anchor(33.0, 140.0, 2.0)
            .with_carrier_speed(1.25, 0.05),
            motion: ShipMotionState::default(),
            control: VehicleControlState::with_profile_id("sim_lite"),
            environment: Some(environment.clone()),
        }
        .normalized();

        let mut snapshot = VehicleTelemetrySnapshot::from_runtime(VehicleTelemetryInput {
            heading: std::f32::consts::FRAC_PI_4,
            motion: Some(MotionFrameInput {
                velocity_x: 1.0,
                velocity_y: -2.0,
                accel_x: 0.5,
                accel_y: -0.25,
            }),
            angular_vel: 0.25,
            ..VehicleTelemetryInput::default()
        })
        .with_ship_runtime_state(&state);
        snapshot.tangent_speed_kms = 1.5;
        snapshot.radial_speed_kms = -0.2;
        snapshot.forward_speed_wu_s = 4.0;
        snapshot.lateral_speed_wu_s = -0.5;
        snapshot.radial_speed_wu_s = 0.0;
        snapshot.camera_sway = 0.3;
        snapshot.basis = Some(VehicleBasis3 {
            normal: [0.0, 1.0, 0.0],
            forward: [1.0, 0.0, 0.0],
            right: [0.0, 0.0, 1.0],
        });

        let packet = snapshot.to_packet_telemetry();
        let decoded = VehicleTelemetrySnapshot::from_packet(&packet);

        assert!(packet.grounded);
        assert_eq!(packet.spawn_angle_deg, 33.0);
        assert_eq!(packet.radius_wu, 140.0);
        assert_eq!(packet.altitude_km, 2.0);
        assert_eq!(decoded.surface_mode, ShipSurfaceMode::Grounded);
        assert!(decoded.grounded);
        assert!(decoded.ship_reference.uses_local_horizon());
        assert!(decoded.ship_reference.uses_co_rotation());
        assert_eq!(decoded.ship_reference.anchor_angle_deg, 33.0);
        assert_eq!(decoded.ship_reference.radius_wu, 140.0);
        assert_eq!(decoded.ship_reference.altitude_km, 2.0);
        assert_eq!(decoded.reference.heading_deg(), 45.0);
        assert_eq!(decoded.camera_sway, 0.3);
        assert_eq!(decoded.basis, snapshot.basis);
    }

    #[test]
    fn launch_and_return_packets_preserve_grounded_surface_runtime_telemetry() {
        let environment_snapshot = VehicleEnvironmentSnapshot {
            body: crate::handoff::VehicleBodySnapshot {
                body_id: "generated".to_string(),
                body_kind: "earth_like".to_string(),
                surface_radius_wu: 120.0,
                ..crate::handoff::VehicleBodySnapshot::default()
            },
            real_radius_km: 6371.0,
            scale_divisor: 50.0,
            surface_gravity_mps2: 9.81,
            atmosphere_top_km: 80.0,
            atmosphere_dense_start_km: 12.0,
            atmosphere_drag_max: 1.2,
            ..VehicleEnvironmentSnapshot::default()
        }
        .normalized();
        let environment = VehicleEnvironmentBinding::from_snapshot(&environment_snapshot);
        let state = ShipRuntimeState {
            surface_mode: ShipSurfaceMode::Grounded,
            reference_frame: ShipReferenceFrameState::local_horizon(0.0, 24.0)
                .with_surface_anchor(24.0, 120.0, 0.0)
                .with_carrier_speed(1.0, 0.05),
            motion: ShipMotionState::default(),
            control: VehicleControlState::with_profile_id("arcade"),
            environment: Some(environment.clone()),
        }
        .normalized();
        let mut telemetry = VehicleTelemetrySnapshot::from_runtime(VehicleTelemetryInput {
            heading: 0.0,
            motion: Some(MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -2.0,
                accel_x: 0.0,
                accel_y: -0.5,
            }),
            ..VehicleTelemetryInput::default()
        })
        .with_ship_runtime_state(&state);
        telemetry.radius_wu = 120.0;
        telemetry.camera_sway = 0.2;

        let launch = VehicleLaunchPacket {
            environment: environment_snapshot.clone(),
            telemetry: Some(telemetry.clone().to_packet_telemetry()),
            ..VehicleLaunchPacket::default()
        }
        .normalized();
        let ret = VehicleReturnPacket {
            environment: environment_snapshot.clone(),
            telemetry: telemetry.to_packet_telemetry(),
            ..VehicleReturnPacket::default()
        }
        .normalized();

        let decoded_launch = VehicleTelemetrySnapshot::from_packet(
            launch.telemetry.as_ref().expect("launch telemetry"),
        )
        .with_environment(VehicleEnvironmentBinding::from_snapshot(
            &launch.environment,
        ));
        let decoded_return = VehicleTelemetrySnapshot::from_packet(&ret.telemetry)
            .with_environment(VehicleEnvironmentBinding::from_snapshot(&ret.environment));

        assert!(
            launch
                .telemetry
                .as_ref()
                .expect("launch telemetry")
                .grounded
        );
        assert_eq!(
            launch
                .telemetry
                .as_ref()
                .expect("launch telemetry")
                .spawn_angle_deg,
            24.0
        );
        assert!(ret.telemetry.grounded);
        assert_eq!(ret.telemetry.radius_wu, 120.0);
        assert_eq!(decoded_launch.surface_mode, ShipSurfaceMode::Grounded);
        assert_eq!(decoded_return.surface_mode, ShipSurfaceMode::Grounded);
        assert!(decoded_launch.ship_reference.uses_local_horizon());
        assert!(decoded_return.ship_reference.uses_local_horizon());
        assert_eq!(
            decoded_launch
                .environment
                .as_ref()
                .map(|environment| environment.body_id.as_str()),
            Some("generated")
        );
        assert_eq!(
            decoded_return
                .environment
                .as_ref()
                .map(|environment| environment.body_kind.as_str()),
            Some("earth_like")
        );
    }

    #[test]
    fn launch_and_return_packets_preserve_detached_surface_anchor_runtime_telemetry() {
        let environment_snapshot = VehicleEnvironmentSnapshot {
            body: crate::handoff::VehicleBodySnapshot {
                body_id: "generated".to_string(),
                body_kind: "earth_like".to_string(),
                surface_radius_wu: 120.0,
                ..crate::handoff::VehicleBodySnapshot::default()
            },
            real_radius_km: 6371.0,
            scale_divisor: 50.0,
            surface_gravity_mps2: 9.81,
            atmosphere_top_km: 80.0,
            atmosphere_dense_start_km: 12.0,
            atmosphere_drag_max: 1.2,
            ..VehicleEnvironmentSnapshot::default()
        }
        .normalized();
        let environment = VehicleEnvironmentBinding::from_snapshot(&environment_snapshot);
        let state = ShipRuntimeState {
            surface_mode: ShipSurfaceMode::Detached,
            reference_frame: ShipReferenceFrameState::local_horizon(0.0, 24.0)
                .with_surface_anchor(24.0, 123.0, 1.5)
                .with_carrier_speed(1.0, 0.05)
                .with_co_rotation(false),
            motion: ShipMotionState::default(),
            control: VehicleControlState::with_profile_id("arcade"),
            environment: Some(environment.clone()),
        }
        .normalized();
        let mut telemetry = VehicleTelemetrySnapshot::from_runtime(VehicleTelemetryInput {
            heading: 0.0,
            motion: Some(MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -2.0,
                accel_x: 0.0,
                accel_y: -0.5,
            }),
            ..VehicleTelemetryInput::default()
        })
        .with_ship_runtime_state(&state);
        telemetry.camera_sway = 0.2;

        let launch = VehicleLaunchPacket {
            environment: environment_snapshot.clone(),
            telemetry: Some(telemetry.clone().to_packet_telemetry()),
            ..VehicleLaunchPacket::default()
        }
        .normalized();
        let ret = VehicleReturnPacket {
            environment: environment_snapshot.clone(),
            telemetry: telemetry.to_packet_telemetry(),
            ..VehicleReturnPacket::default()
        }
        .normalized();

        let decoded_launch = VehicleTelemetrySnapshot::from_packet(
            launch.telemetry.as_ref().expect("launch telemetry"),
        )
        .with_environment(VehicleEnvironmentBinding::from_snapshot(
            &launch.environment,
        ));
        let decoded_return = VehicleTelemetrySnapshot::from_packet(&ret.telemetry)
            .with_environment(VehicleEnvironmentBinding::from_snapshot(&ret.environment));

        assert!(
            !launch
                .telemetry
                .as_ref()
                .expect("launch telemetry")
                .grounded
        );
        assert_eq!(
            launch
                .telemetry
                .as_ref()
                .expect("launch telemetry")
                .spawn_angle_deg,
            24.0
        );
        assert!(!ret.telemetry.grounded);
        assert_eq!(ret.telemetry.radius_wu, 123.0);
        assert_eq!(ret.telemetry.altitude_km, 1.5);
        assert_eq!(decoded_launch.surface_mode, ShipSurfaceMode::Detached);
        assert_eq!(decoded_return.surface_mode, ShipSurfaceMode::Detached);
        assert!(!decoded_launch.grounded);
        assert!(!decoded_return.grounded);
        assert!(decoded_launch.ship_reference.uses_local_horizon());
        assert!(decoded_return.ship_reference.uses_local_horizon());
        assert_eq!(decoded_launch.ship_reference.anchor_angle_deg, 24.0);
        assert_eq!(decoded_return.ship_reference.anchor_angle_deg, 24.0);
        assert_eq!(decoded_launch.ship_reference.radius_wu, 123.0);
        assert_eq!(decoded_return.ship_reference.altitude_km, 1.5);
        assert_eq!(
            decoded_launch
                .environment
                .as_ref()
                .map(|environment| environment.body_id.as_str()),
            Some("generated")
        );
        assert_eq!(
            decoded_return
                .environment
                .as_ref()
                .map(|environment| environment.body_kind.as_str()),
            Some("earth_like")
        );
    }

    #[test]
    fn snapshot_normalization_clamps_invalid_values_and_derives_braking() {
        let snapshot = VehicleTelemetrySnapshot {
            reference: VehicleReferenceFrame::from_heading(-std::f32::consts::FRAC_PI_2),
            motion: MotionFrame {
                velocity_x: f32::INFINITY,
                velocity_y: -2.0,
                accel_x: f32::NAN,
                accel_y: 1.0,
                ..MotionFrame::default()
            },
            turn_input: 2.0,
            thrust_factor: -1.0,
            rot_factor: f32::NAN,
            brake_factor: -0.5,
            brake_phase: BrakePhase::Rotation,
            is_braking: false,
            altitude_km: -5.0,
            tangent_speed_kms: -0.2,
            radial_speed_kms: f32::NAN,
            spawn_angle_deg: -90.0,
            radius_wu: -1.0,
            basis: Some(VehicleBasis3 {
                normal: [f32::NAN, 1.0, 0.0],
                forward: [1.0, f32::INFINITY, 0.0],
                right: [0.0, 0.0, 1.0],
            }),
            environment: Some(VehicleEnvironmentBinding {
                scale_divisor: 0.0,
                atmosphere_top_km: 12.0,
                atmosphere_dense_start_km: 18.0,
                atmosphere_drag_max: 0.8,
                ..VehicleEnvironmentBinding::default()
            }),
            ..VehicleTelemetrySnapshot::default()
        }
        .normalized();

        assert_eq!(snapshot.reference.heading_deg(), 270.0);
        assert_eq!(snapshot.motion.velocity_x, 0.0);
        assert_eq!(snapshot.motion.accel_x, 0.0);
        assert_eq!(snapshot.turn_input, 1.0);
        assert_eq!(snapshot.thrust_factor, 0.0);
        assert_eq!(snapshot.rot_factor, 0.0);
        assert_eq!(snapshot.brake_factor, 0.0);
        assert!(snapshot.is_braking);
        assert_eq!(snapshot.altitude_km, 0.0);
        assert_eq!(snapshot.tangent_speed_kms, 0.0);
        assert_eq!(snapshot.radial_speed_kms, 0.0);
        assert_eq!(snapshot.spawn_angle_deg, 270.0);
        assert_eq!(snapshot.radius_wu, 0.0);
        assert_eq!(
            snapshot
                .environment
                .as_ref()
                .map(|environment| environment.scale_divisor),
            Some(0.0001)
        );
        assert_eq!(
            snapshot
                .environment
                .as_ref()
                .map(|environment| environment.atmosphere_dense_start_km),
            Some(12.0)
        );
        assert_eq!(
            snapshot.basis,
            Some(VehicleBasis3 {
                normal: [0.0, 1.0, 0.0],
                forward: [1.0, 0.0, 0.0],
                right: [0.0, 0.0, 1.0],
            })
        );
    }
}
