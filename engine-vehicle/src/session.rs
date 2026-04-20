use serde::{Deserialize, Serialize};

use crate::{
    ShipRuntimeOutput, ShipRuntimeState, VehicleControlState, VehicleEnvironmentBinding,
    VehicleLaunchPacket, VehiclePacketVehicle, VehicleReturnPacket, VehicleTelemetrySnapshot,
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

/// Narrow reusable vehicle-domain aggregate for scene/runtime handoff.
///
/// This intentionally stops at durable vehicle state:
/// - control profile + assists,
/// - ship runtime state,
/// - telemetry snapshot,
/// - bound environment,
/// - spawn altitude/angle metadata.
///
/// It does not own scene-specific camera rigs, HUD state, object ids, or
/// producer/consumer routing. Those remain mod/runtime concerns.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleSessionState {
    pub control: VehicleControlState,
    pub runtime: ShipRuntimeState,
    pub telemetry: VehicleTelemetrySnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<VehicleEnvironmentBinding>,
    pub spawn_altitude_km: f32,
    pub spawn_angle_deg: f32,
}

impl Default for VehicleSessionState {
    fn default() -> Self {
        Self {
            control: VehicleControlState::default(),
            runtime: ShipRuntimeState::default(),
            telemetry: VehicleTelemetrySnapshot::default(),
            environment: None,
            spawn_altitude_km: 0.0,
            spawn_angle_deg: 0.0,
        }
    }
}

impl VehicleSessionState {
    pub fn from_launch_packet(packet: &VehicleLaunchPacket) -> Self {
        let environment = packet.environment_binding();
        let telemetry = packet.telemetry_snapshot_or_default();
        let spawn_altitude_km = packet.vehicle.spawn_altitude_km;
        let spawn_angle_deg = telemetry.spawn_angle_deg;
        Self {
            control: packet.control_state(),
            runtime: ShipRuntimeState::default(),
            telemetry,
            environment: Some(environment),
            spawn_altitude_km,
            spawn_angle_deg,
        }
        .normalized()
    }

    pub fn from_return_packet(packet: &VehicleReturnPacket) -> Self {
        let environment = packet.environment_binding();
        let telemetry = packet.telemetry_snapshot();
        let spawn_altitude_km = packet.vehicle.spawn_altitude_km.max(telemetry.altitude_km);
        let spawn_angle_deg = telemetry.spawn_angle_deg;
        Self {
            control: packet.control_state(),
            runtime: ShipRuntimeState::default(),
            telemetry,
            environment: Some(environment),
            spawn_altitude_km,
            spawn_angle_deg,
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.control = self.control.clone().normalized();
        self.runtime = self.runtime.clone().normalized();
        self.telemetry = self.telemetry.clone().normalized();
        self.environment = self
            .environment
            .take()
            .map(|environment| environment.normalized())
            .or_else(|| self.runtime.environment.clone())
            .or_else(|| self.telemetry.environment.clone());
        self.spawn_altitude_km = if self.spawn_altitude_km > 0.0 {
            non_negative(self.spawn_altitude_km)
        } else {
            non_negative(self.telemetry.altitude_km)
        };
        self.spawn_angle_deg = if self.spawn_angle_deg != 0.0 {
            finite_or_zero(self.spawn_angle_deg).rem_euclid(360.0)
        } else {
            finite_or_zero(self.telemetry.spawn_angle_deg).rem_euclid(360.0)
        };

        if let Some(environment) = self.environment.as_ref() {
            self.runtime.environment = Some(environment.clone());
            self.telemetry = self.telemetry.clone().with_environment(environment.clone());
        }
        self.runtime.control = self.control.clone();
        self.runtime.normalize();
        self.telemetry = self
            .telemetry
            .clone()
            .with_ship_runtime_state(&self.runtime);
        if self.spawn_altitude_km <= 0.0 {
            self.spawn_altitude_km = self.telemetry.altitude_km;
        }
        self.spawn_angle_deg = finite_or_zero(self.spawn_angle_deg).rem_euclid(360.0);
    }

    pub fn with_control(mut self, control: VehicleControlState) -> Self {
        self.control = control;
        self.normalized()
    }

    pub fn with_runtime(mut self, runtime: ShipRuntimeState) -> Self {
        self.runtime = runtime;
        self.normalized()
    }

    pub fn with_telemetry(mut self, telemetry: VehicleTelemetrySnapshot) -> Self {
        self.telemetry = telemetry;
        self.normalized()
    }

    pub fn with_environment(mut self, environment: VehicleEnvironmentBinding) -> Self {
        self.environment = Some(environment);
        self.normalized()
    }

    pub fn with_spawn(mut self, spawn_altitude_km: f32, spawn_angle_deg: f32) -> Self {
        self.spawn_altitude_km = spawn_altitude_km;
        self.spawn_angle_deg = spawn_angle_deg;
        self.normalized()
    }

    pub fn apply_runtime_output(&mut self, output: &ShipRuntimeOutput) {
        self.runtime = output.state.clone();
        self.telemetry = output.telemetry.clone();
        self.environment = self
            .runtime
            .environment
            .clone()
            .or_else(|| self.telemetry.environment.clone())
            .or_else(|| self.environment.clone());
        self.control = self.runtime.control.clone();
        if self.spawn_altitude_km <= 0.0 {
            self.spawn_altitude_km = self.telemetry.altitude_km;
        }
        self.spawn_angle_deg = self.telemetry.spawn_angle_deg;
        self.normalize();
    }

    pub fn packet_vehicle(&self) -> VehiclePacketVehicle {
        VehiclePacketVehicle::from_control_state(&self.control, self.spawn_altitude_km).normalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        VehicleBodySnapshot, VehicleEnvironmentSnapshot, VehiclePacketEnvelope,
        VehiclePacketTelemetry,
    };

    #[test]
    fn session_state_normalizes_and_keeps_vehicle_domain_in_sync() {
        let session = VehicleSessionState {
            control: VehicleControlState::with_profile_id(" sim_lite "),
            runtime: ShipRuntimeState::default(),
            telemetry: VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                altitude_km: 12.0,
                spawn_angle_deg: -45.0,
                ..VehiclePacketTelemetry::default()
            }),
            environment: Some(VehicleEnvironmentBinding {
                body_id: " generated-planet ".into(),
                scale_divisor: 30.0,
                surface_radius_wu: 212.0,
                ..VehicleEnvironmentBinding::default()
            }),
            spawn_altitude_km: -2.0,
            spawn_angle_deg: -30.0,
        }
        .normalized();

        assert_eq!(session.control.profile_id, "sim-lite");
        assert_eq!(session.runtime.control.profile_id, "sim-lite");
        assert_eq!(session.spawn_altitude_km, 12.0);
        assert_eq!(session.spawn_angle_deg, 330.0);
        assert_eq!(
            session
                .environment
                .as_ref()
                .map(|environment| environment.body_id.as_str()),
            Some("generated-planet")
        );
    }

    #[test]
    fn session_state_builds_from_launch_packet_without_map_roundtrip() {
        let session = VehicleSessionState::from_launch_packet(&VehicleLaunchPacket {
            envelope: VehiclePacketEnvelope::default(),
            environment: VehicleEnvironmentSnapshot {
                body: VehicleBodySnapshot {
                    body_id: "generated-planet".into(),
                    body_kind: "earth_like".into(),
                    surface_radius_wu: 212.0,
                    ..VehicleBodySnapshot::default()
                },
                real_radius_km: 6371.0,
                scale_divisor: 30.0,
                surface_gravity_mps2: 9.81,
                atmosphere_top_km: 80.0,
                atmosphere_dense_start_km: 12.0,
                atmosphere_drag_max: 1.5,
                ..VehicleEnvironmentSnapshot::default()
            },
            vehicle: VehiclePacketVehicle {
                profile_id: " sim_lite ".into(),
                assist_alt_hold: true,
                assist_heading_hold: false,
                spawn_altitude_km: 4.0,
            },
            telemetry: Some(VehiclePacketTelemetry {
                altitude_km: 4.0,
                spawn_angle_deg: -90.0,
                grounded: true,
                ..VehiclePacketTelemetry::default()
            }),
            ui: Default::default(),
        });

        assert_eq!(session.control.profile_id, "sim-lite");
        assert!(session.control.assists.alt_hold);
        assert_eq!(session.spawn_altitude_km, 4.0);
        assert_eq!(session.spawn_angle_deg, 270.0);
        assert_eq!(session.packet_vehicle().profile_id, "sim-lite");
    }
}
