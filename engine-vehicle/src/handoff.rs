use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{
    normalize_vehicle_profile_id, VehicleAssistState, VehicleControlState,
    VehicleEnvironmentBinding, VehicleTelemetrySnapshot,
};

pub const VEHICLE_HANDOFF_VERSION: u32 = 1;
pub const VEHICLE_LAUNCH_PACKET_KIND: &str = "vehicle_launch";
pub const VEHICLE_RETURN_PACKET_KIND: &str = "vehicle_return";
pub const LEGACY_VEHICLE_HANDOFF_PACKET_KIND: &str = "vehicle_handoff";

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

#[inline]
fn trim_owned(value: &str) -> String {
    value.trim().to_string()
}

#[inline]
fn trim_option(value: &str) -> String {
    value.trim().to_string()
}

#[inline]
fn is_launch_packet_kind(packet_kind: &str) -> bool {
    matches!(
        trim_owned(packet_kind).as_str(),
        VEHICLE_LAUNCH_PACKET_KIND | LEGACY_VEHICLE_HANDOFF_PACKET_KIND
    )
}

#[inline]
fn is_return_packet_kind(packet_kind: &str) -> bool {
    trim_owned(packet_kind) == VEHICLE_RETURN_PACKET_KIND
}

/// Packet envelope shared by launch/return handoff payloads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehiclePacketEnvelope {
    pub packet_kind: String,
    pub packet_version: u32,
    pub producer_mod_id: String,
    pub source_scene_id: String,
    pub target_mod_ref: String,
    pub target_scene_id: String,
    pub return_scene_id: String,
    pub consumer_hint: String,
}

impl VehiclePacketEnvelope {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.packet_kind = trim_owned(&self.packet_kind);
        if self.packet_kind.is_empty() {
            self.packet_kind = LEGACY_VEHICLE_HANDOFF_PACKET_KIND.to_string();
        }
        if self.packet_version == 0 {
            self.packet_version = VEHICLE_HANDOFF_VERSION;
        }
        self.producer_mod_id = trim_owned(&self.producer_mod_id);
        self.source_scene_id = trim_owned(&self.source_scene_id);
        self.target_mod_ref = trim_owned(&self.target_mod_ref);
        self.target_scene_id = trim_owned(&self.target_scene_id);
        self.return_scene_id = trim_owned(&self.return_scene_id);
        self.consumer_hint = trim_option(&self.consumer_hint);
    }

    pub fn normalize_for_launch(&mut self) {
        self.normalize();
        self.packet_kind = VEHICLE_LAUNCH_PACKET_KIND.to_string();
    }

    pub fn normalize_for_return(&mut self) {
        self.normalize();
        self.packet_kind = VEHICLE_RETURN_PACKET_KIND.to_string();
    }

    pub fn is_vehicle_launch(&self) -> bool {
        is_launch_packet_kind(&self.packet_kind)
    }

    pub fn is_vehicle_return(&self) -> bool {
        is_return_packet_kind(&self.packet_kind)
    }
}

/// Typed vehicle-facing handoff state shared across launch/return packets.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehiclePacketVehicle {
    #[serde(rename = "profile", alias = "profile_id")]
    pub profile_id: String,
    pub assist_alt_hold: bool,
    pub assist_heading_hold: bool,
    pub spawn_altitude_km: f32,
}

impl VehiclePacketVehicle {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.profile_id = normalize_vehicle_profile_id(&self.profile_id);
        self.spawn_altitude_km = non_negative(self.spawn_altitude_km);
    }

    pub fn from_control_state(control: &VehicleControlState, spawn_altitude_km: f32) -> Self {
        Self {
            profile_id: normalize_vehicle_profile_id(&control.profile_id),
            assist_alt_hold: control.assists.alt_hold,
            assist_heading_hold: control.assists.heading_hold,
            spawn_altitude_km: non_negative(spawn_altitude_km),
        }
    }

    pub fn to_control_state(&self) -> VehicleControlState {
        let mut control = VehicleControlState::with_profile_id(&self.profile_id);
        control.assists =
            VehicleAssistState::from_flags(self.assist_alt_hold, self.assist_heading_hold);
        control
    }
}

/// Typed snapshot of the current primary navigation/gravity body.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleBodySnapshot {
    #[serde(rename = "id", alias = "body_id")]
    pub body_id: String,
    #[serde(rename = "planet_type", alias = "body_kind")]
    pub body_kind: String,
    pub center_x: f32,
    pub center_y: f32,
    #[serde(rename = "radius_px", alias = "render_radius_wu")]
    pub render_radius_wu: f32,
    #[serde(rename = "surface_radius", alias = "surface_radius_wu")]
    pub surface_radius_wu: f32,
    pub radius_km: f32,
    pub gravity_mu_km3_s2: f32,
    pub atmosphere_top_km: f32,
    pub atmosphere_dense_start_km: f32,
    pub atmosphere_drag_max: f32,
    pub cloud_bottom_km: f32,
    pub cloud_top_km: f32,
    #[serde(flatten)]
    pub extras: JsonMap<String, JsonValue>,
}

impl VehicleBodySnapshot {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.body_id = trim_owned(&self.body_id);
        self.body_kind = trim_owned(&self.body_kind);
        self.center_x = finite_or_zero(self.center_x);
        self.center_y = finite_or_zero(self.center_y);
        self.render_radius_wu = non_negative(self.render_radius_wu);
        self.surface_radius_wu = non_negative(self.surface_radius_wu);
        self.radius_km = non_negative(self.radius_km);
        self.gravity_mu_km3_s2 = non_negative(self.gravity_mu_km3_s2);
        self.atmosphere_top_km = non_negative(self.atmosphere_top_km);
        self.atmosphere_dense_start_km =
            non_negative(self.atmosphere_dense_start_km).min(self.atmosphere_top_km);
        self.atmosphere_drag_max = non_negative(self.atmosphere_drag_max);
        self.cloud_bottom_km = non_negative(self.cloud_bottom_km);
        self.cloud_top_km = non_negative(self.cloud_top_km).max(self.cloud_bottom_km);
    }
}

/// Typed environment snapshot for vehicle launch/return handoff.
///
/// Unknown producer-specific fields are preserved in `extras` so mods can
/// migrate incrementally toward the typed core surface.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleEnvironmentSnapshot {
    pub body: VehicleBodySnapshot,
    pub real_radius_km: f32,
    pub scale_divisor: f32,
    pub surface_gravity_mps2: f32,
    #[serde(rename = "atmo_top_km", alias = "atmosphere_top_km")]
    pub atmosphere_top_km: f32,
    #[serde(rename = "atmo_dense_start_km", alias = "atmosphere_dense_start_km")]
    pub atmosphere_dense_start_km: f32,
    #[serde(rename = "atmo_drag_max", alias = "atmosphere_drag_max")]
    pub atmosphere_drag_max: f32,
    #[serde(flatten)]
    pub extras: JsonMap<String, JsonValue>,
}

impl VehicleEnvironmentSnapshot {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.body.normalize();
        self.real_radius_km = non_negative(self.real_radius_km);
        self.scale_divisor = finite_or_zero(self.scale_divisor).max(0.0001);
        self.surface_gravity_mps2 = non_negative(self.surface_gravity_mps2);
        self.atmosphere_top_km = non_negative(self.atmosphere_top_km);
        self.atmosphere_dense_start_km =
            non_negative(self.atmosphere_dense_start_km).min(self.atmosphere_top_km);
        self.atmosphere_drag_max = non_negative(self.atmosphere_drag_max);
    }
}

/// Optional 3D basis that callers can carry through handoff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleBasis3 {
    pub normal: [f32; 3],
    pub forward: [f32; 3],
    pub right: [f32; 3],
}

impl VehicleBasis3 {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        for component in &mut self.normal {
            *component = finite_or_zero(*component);
        }
        for component in &mut self.forward {
            *component = finite_or_zero(*component);
        }
        for component in &mut self.right {
            *component = finite_or_zero(*component);
        }
    }
}

/// Typed telemetry snapshot carried across packet handoff.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct VehicleLegacyBasis3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    snx: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sny: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snz: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sfx: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sfy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sfz: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    srx: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sry: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    srz: Option<f32>,
}

impl VehicleLegacyBasis3 {
    fn normalized_basis(&self) -> Option<VehicleBasis3> {
        let has_any_component = self.snx.is_some()
            || self.sny.is_some()
            || self.snz.is_some()
            || self.sfx.is_some()
            || self.sfy.is_some()
            || self.sfz.is_some()
            || self.srx.is_some()
            || self.sry.is_some()
            || self.srz.is_some();
        if !has_any_component {
            return None;
        }

        Some(
            VehicleBasis3 {
                normal: [
                    finite_or_zero(self.snx.unwrap_or_default()),
                    finite_or_zero(self.sny.unwrap_or_default()),
                    finite_or_zero(self.snz.unwrap_or_default()),
                ],
                forward: [
                    finite_or_zero(self.sfx.unwrap_or_default()),
                    finite_or_zero(self.sfy.unwrap_or_default()),
                    finite_or_zero(self.sfz.unwrap_or_default()),
                ],
                right: [
                    finite_or_zero(self.srx.unwrap_or_default()),
                    finite_or_zero(self.sry.unwrap_or_default()),
                    finite_or_zero(self.srz.unwrap_or_default()),
                ],
            }
            .normalized(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct VehiclePacketTelemetry {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub heading_deg: f32,
    pub altitude_km: f32,
    pub tangent_speed_kms: f32,
    pub radial_speed_kms: f32,
    pub spawn_angle_deg: f32,
    #[serde(alias = "cam_sway")]
    pub camera_sway: f32,
    pub radius_wu: f32,
    pub vfwd_wu_s: f32,
    pub vright_wu_s: f32,
    pub vrad_wu_s: f32,
    pub yaw_rate_rad_s: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<VehicleBasis3>,
    pub grounded: bool,
}

impl VehiclePacketTelemetry {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.x = finite_or_zero(self.x);
        self.y = finite_or_zero(self.y);
        self.vx = finite_or_zero(self.vx);
        self.vy = finite_or_zero(self.vy);
        self.heading_deg = finite_or_zero(self.heading_deg).rem_euclid(360.0);
        self.altitude_km = non_negative(self.altitude_km);
        self.tangent_speed_kms = non_negative(self.tangent_speed_kms);
        self.radial_speed_kms = finite_or_zero(self.radial_speed_kms);
        self.spawn_angle_deg = finite_or_zero(self.spawn_angle_deg).rem_euclid(360.0);
        self.camera_sway = finite_or_zero(self.camera_sway);
        self.radius_wu = non_negative(self.radius_wu);
        self.vfwd_wu_s = finite_or_zero(self.vfwd_wu_s);
        self.vright_wu_s = finite_or_zero(self.vright_wu_s);
        self.vrad_wu_s = finite_or_zero(self.vrad_wu_s);
        self.yaw_rate_rad_s = finite_or_zero(self.yaw_rate_rad_s);
        self.basis = self.basis.take().map(|basis| basis.normalized());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct VehiclePacketTelemetryWire {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    heading_deg: f32,
    altitude_km: f32,
    tangent_speed_kms: f32,
    radial_speed_kms: f32,
    spawn_angle_deg: f32,
    #[serde(alias = "cam_sway")]
    camera_sway: f32,
    radius_wu: f32,
    vfwd_wu_s: f32,
    vright_wu_s: f32,
    vrad_wu_s: f32,
    yaw_rate_rad_s: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    basis: Option<VehicleBasis3>,
    #[serde(flatten)]
    legacy_basis: VehicleLegacyBasis3,
    grounded: bool,
}

impl<'de> Deserialize<'de> for VehiclePacketTelemetry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = VehiclePacketTelemetryWire::deserialize(deserializer)?;
        Ok(Self {
            x: wire.x,
            y: wire.y,
            vx: wire.vx,
            vy: wire.vy,
            heading_deg: wire.heading_deg,
            altitude_km: wire.altitude_km,
            tangent_speed_kms: wire.tangent_speed_kms,
            radial_speed_kms: wire.radial_speed_kms,
            spawn_angle_deg: wire.spawn_angle_deg,
            camera_sway: wire.camera_sway,
            radius_wu: wire.radius_wu,
            vfwd_wu_s: wire.vfwd_wu_s,
            vright_wu_s: wire.vright_wu_s,
            vrad_wu_s: wire.vrad_wu_s,
            yaw_rate_rad_s: wire.yaw_rate_rad_s,
            basis: wire
                .basis
                .map(|basis| basis.normalized())
                .or_else(|| wire.legacy_basis.normalized_basis()),
            grounded: wire.grounded,
        })
    }
}

/// Typed launch packet for entering a vehicle-focused runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleLaunchPacket {
    #[serde(flatten)]
    pub envelope: VehiclePacketEnvelope,
    #[serde(rename = "planet", alias = "environment")]
    pub environment: VehicleEnvironmentSnapshot,
    pub vehicle: VehiclePacketVehicle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<VehiclePacketTelemetry>,
    #[serde(default, skip_serializing_if = "JsonMap::is_empty")]
    pub ui: JsonMap<String, JsonValue>,
}

impl Default for VehicleLaunchPacket {
    fn default() -> Self {
        let mut envelope = VehiclePacketEnvelope::default();
        envelope.normalize_for_launch();
        Self {
            envelope,
            environment: VehicleEnvironmentSnapshot::default(),
            vehicle: VehiclePacketVehicle::default(),
            telemetry: None,
            ui: JsonMap::new(),
        }
    }
}

impl VehicleLaunchPacket {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.envelope.normalize_for_launch();
        self.environment.normalize();
        self.vehicle.normalize();
        self.telemetry = self
            .telemetry
            .take()
            .map(|telemetry| telemetry.normalized());
    }

    pub fn is_vehicle_handoff(&self) -> bool {
        self.envelope.is_vehicle_launch()
    }

    pub fn control_state(&self) -> VehicleControlState {
        self.vehicle.to_control_state().normalized()
    }

    pub fn environment_binding(&self) -> VehicleEnvironmentBinding {
        VehicleEnvironmentBinding::from_snapshot(&self.environment)
    }

    pub fn telemetry_snapshot(&self) -> Option<VehicleTelemetrySnapshot> {
        self.telemetry.as_ref().map(|telemetry| {
            VehicleTelemetrySnapshot::from_packet(telemetry)
                .with_environment(self.environment_binding())
        })
    }

    pub fn telemetry_snapshot_or_default(&self) -> VehicleTelemetrySnapshot {
        self.telemetry_snapshot().unwrap_or_else(|| {
            VehicleTelemetrySnapshot::default().with_environment(self.environment_binding())
        })
    }
}

/// Typed return packet for restoring state back to a producer/runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleReturnPacket {
    #[serde(flatten)]
    pub envelope: VehiclePacketEnvelope,
    #[serde(rename = "planet", alias = "environment")]
    pub environment: VehicleEnvironmentSnapshot,
    pub vehicle: VehiclePacketVehicle,
    pub telemetry: VehiclePacketTelemetry,
    #[serde(default, skip_serializing_if = "JsonMap::is_empty")]
    pub ui: JsonMap<String, JsonValue>,
}

impl Default for VehicleReturnPacket {
    fn default() -> Self {
        let mut envelope = VehiclePacketEnvelope::default();
        envelope.normalize_for_return();
        Self {
            envelope,
            environment: VehicleEnvironmentSnapshot::default(),
            vehicle: VehiclePacketVehicle::default(),
            telemetry: VehiclePacketTelemetry::default(),
            ui: JsonMap::new(),
        }
    }
}

impl VehicleReturnPacket {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.envelope.normalize_for_return();
        self.environment.normalize();
        self.vehicle.normalize();
        self.telemetry.normalize();
    }

    pub fn is_vehicle_return(&self) -> bool {
        self.envelope.is_vehicle_return()
    }

    pub fn control_state(&self) -> VehicleControlState {
        self.vehicle.to_control_state().normalized()
    }

    pub fn environment_binding(&self) -> VehicleEnvironmentBinding {
        VehicleEnvironmentBinding::from_snapshot(&self.environment)
    }

    pub fn telemetry_snapshot(&self) -> VehicleTelemetrySnapshot {
        VehicleTelemetrySnapshot::from_packet(&self.telemetry)
            .with_environment(self.environment_binding())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_normalizes_and_trims_fields() {
        let mut envelope = VehiclePacketEnvelope {
            packet_kind: "".to_string(),
            packet_version: 0,
            producer_mod_id: " planet-generator ".to_string(),
            source_scene_id: " source ".to_string(),
            target_mod_ref: " vehicle-playground ".to_string(),
            target_scene_id: " vehicle-scene ".to_string(),
            return_scene_id: " return-scene ".to_string(),
            consumer_hint: " generator-launch ".to_string(),
        };
        envelope.normalize_for_launch();

        assert_eq!(envelope.packet_kind, VEHICLE_LAUNCH_PACKET_KIND);
        assert_eq!(envelope.packet_version, VEHICLE_HANDOFF_VERSION);
        assert_eq!(envelope.producer_mod_id, "planet-generator");
        assert_eq!(envelope.consumer_hint, "generator-launch");
    }

    #[test]
    fn legacy_launch_packet_kind_is_still_recognized_and_upgraded() {
        let envelope = VehiclePacketEnvelope {
            packet_kind: " vehicle_handoff ".to_string(),
            packet_version: 1,
            ..VehiclePacketEnvelope::default()
        }
        .normalized();

        assert!(envelope.is_vehicle_launch());
        assert!(!envelope.is_vehicle_return());

        let packet = VehicleLaunchPacket {
            envelope,
            ..VehicleLaunchPacket::default()
        }
        .normalized();

        assert_eq!(packet.envelope.packet_kind, VEHICLE_LAUNCH_PACKET_KIND);
        assert!(packet.is_vehicle_handoff());
    }

    #[test]
    fn vehicle_payload_normalizes_profile_and_converts_to_control_state() {
        let packet_vehicle = VehiclePacketVehicle {
            profile_id: " sim_lite ".to_string(),
            assist_alt_hold: true,
            assist_heading_hold: false,
            spawn_altitude_km: -2.0,
        }
        .normalized();

        assert_eq!(packet_vehicle.profile_id, "sim-lite");
        assert_eq!(packet_vehicle.spawn_altitude_km, 0.0);

        let control = packet_vehicle.to_control_state();
        assert_eq!(control.profile_id, "sim-lite");
        assert!(control.assists.alt_hold);
        assert!(!control.assists.heading_hold);
    }

    #[test]
    fn launch_packet_normalizes_environment_and_roundtrips_json() {
        let packet = VehicleLaunchPacket {
            envelope: VehiclePacketEnvelope {
                packet_kind: LEGACY_VEHICLE_HANDOFF_PACKET_KIND.to_string(),
                packet_version: 0,
                producer_mod_id: "planet-generator".to_string(),
                source_scene_id: "planet-generator-main".to_string(),
                target_mod_ref: "vehicle-playground".to_string(),
                target_scene_id: "vehicle-playground-vehicle".to_string(),
                return_scene_id: "planet-generator-main".to_string(),
                consumer_hint: "vehicle-runtime".to_string(),
            },
            environment: VehicleEnvironmentSnapshot {
                body: VehicleBodySnapshot {
                    body_id: "generated-planet".to_string(),
                    body_kind: "earth_like".to_string(),
                    atmosphere_top_km: 80.0,
                    atmosphere_dense_start_km: 120.0,
                    ..VehicleBodySnapshot::default()
                },
                real_radius_km: 6371.0,
                scale_divisor: 0.0,
                surface_gravity_mps2: 9.81,
                atmosphere_top_km: 80.0,
                atmosphere_dense_start_km: 120.0,
                atmosphere_drag_max: 2.0,
                ..VehicleEnvironmentSnapshot::default()
            },
            vehicle: VehiclePacketVehicle {
                profile_id: "arcade".to_string(),
                assist_alt_hold: true,
                assist_heading_hold: false,
                spawn_altitude_km: 12.0,
            },
            telemetry: Some(VehiclePacketTelemetry {
                heading_deg: -45.0,
                altitude_km: 12.0,
                tangent_speed_kms: 1.5,
                basis: Some(VehicleBasis3 {
                    normal: [0.0, 1.0, 0.0],
                    forward: [1.0, 0.0, 0.0],
                    right: [0.0, 0.0, 1.0],
                }),
                ..VehiclePacketTelemetry::default()
            }),
            ui: JsonMap::new(),
        }
        .normalized();

        assert_eq!(packet.envelope.packet_kind, VEHICLE_LAUNCH_PACKET_KIND);
        assert_eq!(packet.environment.scale_divisor, 0.0001);
        assert_eq!(packet.environment.body.atmosphere_dense_start_km, 80.0);
        assert_eq!(
            packet.telemetry.as_ref().map(|t| t.heading_deg),
            Some(315.0)
        );

        let json = serde_json::to_string(&packet).expect("serialize packet");
        let decoded: VehicleLaunchPacket = serde_json::from_str(&json).expect("deserialize packet");
        assert_eq!(decoded, packet);
    }

    #[test]
    fn return_packet_defaults_and_roundtrips_json() {
        let packet = VehicleReturnPacket {
            vehicle: VehiclePacketVehicle::from_control_state(
                &VehicleControlState {
                    profile_id: "sim_lite".to_string(),
                    assists: VehicleAssistState::from_flags(true, true),
                    ..VehicleControlState::default()
                },
                8.0,
            ),
            telemetry: VehiclePacketTelemetry {
                x: 15.0,
                y: -3.0,
                vx: 1.5,
                vy: -0.5,
                heading_deg: 720.0,
                altitude_km: 4.5,
                tangent_speed_kms: 1.2,
                radial_speed_kms: -0.15,
                spawn_angle_deg: -45.0,
                camera_sway: 0.2,
                radius_wu: 120.0,
                vfwd_wu_s: 3.5,
                vright_wu_s: -1.25,
                vrad_wu_s: -0.5,
                yaw_rate_rad_s: 0.75,
                basis: Some(VehicleBasis3 {
                    normal: [0.0, 1.0, 0.0],
                    forward: [1.0, 0.0, 0.0],
                    right: [0.0, 0.0, 1.0],
                }),
                grounded: false,
            },
            ..VehicleReturnPacket::default()
        }
        .normalized();

        assert!(packet.is_vehicle_return());
        assert_eq!(packet.vehicle.profile_id, "sim-lite");
        assert_eq!(packet.telemetry.heading_deg, 0.0);
        assert_eq!(packet.telemetry.spawn_angle_deg, 315.0);

        let json = serde_json::to_string(&packet).expect("serialize return packet");
        let decoded: VehicleReturnPacket =
            serde_json::from_str(&json).expect("deserialize return packet");
        assert_eq!(decoded, packet);
    }

    #[test]
    fn telemetry_normalizes_legacy_basis_and_camera_sway_aliases() {
        let telemetry = serde_json::from_value::<VehiclePacketTelemetry>(serde_json::json!({
            "heading_deg": -90.0,
            "cam_sway": 0.35,
            "snx": 1.0,
            "sny": 2.0,
            "snz": 3.0,
            "sfx": 4.0,
            "sfy": 5.0,
            "sfz": 6.0,
            "srx": 7.0,
            "sry": 8.0,
            "srz": 9.0
        }))
        .expect("deserialize telemetry")
        .normalized();

        assert_eq!(telemetry.heading_deg, 270.0);
        assert_eq!(telemetry.camera_sway, 0.35);
        assert_eq!(
            telemetry.basis,
            Some(VehicleBasis3 {
                normal: [1.0, 2.0, 3.0],
                forward: [4.0, 5.0, 6.0],
                right: [7.0, 8.0, 9.0],
            })
        );

        let json = serde_json::to_value(&telemetry).expect("serialize telemetry");
        assert!(json.get("cam_sway").is_none());
        assert!(json.get("snx").is_none());
        assert!(json.get("basis").is_some());
    }
}
