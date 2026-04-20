use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{
    handoff::{VehicleBodySnapshot, VehicleEnvironmentSnapshot},
    types::{MotionFrame, MotionFrameInput, VehicleFacing},
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

#[inline]
fn normalize_heading_rad(value: f32) -> f32 {
    finite_or_zero(value).rem_euclid(std::f32::consts::TAU)
}

#[inline]
fn trim_owned(value: &str) -> String {
    value.trim().to_string()
}

#[inline]
fn json_number(value: &JsonValue) -> Option<f32> {
    match value {
        JsonValue::Number(number) => number.as_f64().map(|value| value as f32),
        JsonValue::String(text) => text.trim().parse::<f32>().ok(),
        _ => None,
    }
    .filter(|value| value.is_finite())
}

#[inline]
fn json_map_number(map: &JsonMap<String, JsonValue>, key: &str) -> Option<f32> {
    map.get(key).and_then(json_number)
}

#[inline]
fn clamp01(value: f32) -> f32 {
    finite_or_zero(value).clamp(0.0, 1.0)
}

/// Vehicle-local 2D reference frame independent of one concrete ship runtime.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleReferenceFrame {
    pub heading_rad: f32,
    pub forward_x: f32,
    pub forward_y: f32,
    pub right_x: f32,
    pub right_y: f32,
}

impl Default for VehicleReferenceFrame {
    fn default() -> Self {
        Self::from_heading(0.0)
    }
}

impl VehicleReferenceFrame {
    pub fn from_heading(heading_rad: f32) -> Self {
        let heading_rad = normalize_heading_rad(heading_rad);
        Self {
            heading_rad,
            forward_x: heading_rad.sin(),
            forward_y: -heading_rad.cos(),
            right_x: heading_rad.cos(),
            right_y: heading_rad.sin(),
        }
    }

    pub fn from_facing(facing: VehicleFacing) -> Self {
        Self::from_heading(facing.heading)
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        *self = Self::from_heading(self.heading_rad);
    }

    pub fn facing(&self) -> VehicleFacing {
        VehicleFacing::from_heading(self.heading_rad)
    }

    pub fn heading_deg(&self) -> f32 {
        self.heading_rad.to_degrees().rem_euclid(360.0)
    }

    pub fn project_vector(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (
            finite_or_zero(world_x) * self.forward_x + finite_or_zero(world_y) * self.forward_y,
            finite_or_zero(world_x) * self.right_x + finite_or_zero(world_y) * self.right_y,
        )
    }

    pub fn project_vector_relative(
        &self,
        world_x: f32,
        world_y: f32,
        carrier_x: f32,
        carrier_y: f32,
    ) -> (f32, f32) {
        self.project_vector(
            finite_or_zero(world_x) - finite_or_zero(carrier_x),
            finite_or_zero(world_y) - finite_or_zero(carrier_y),
        )
    }

    pub fn world_vector(&self, forward: f32, lateral: f32) -> (f32, f32) {
        (
            finite_or_zero(forward) * self.forward_x + finite_or_zero(lateral) * self.right_x,
            finite_or_zero(forward) * self.forward_y + finite_or_zero(lateral) * self.right_y,
        )
    }

    pub fn world_vector_relative(
        &self,
        forward: f32,
        lateral: f32,
        carrier_x: f32,
        carrier_y: f32,
    ) -> (f32, f32) {
        let (world_x, world_y) = self.world_vector(forward, lateral);
        (
            world_x + finite_or_zero(carrier_x),
            world_y + finite_or_zero(carrier_y),
        )
    }

    pub fn resolve_motion(&self, input: MotionFrameInput) -> MotionFrame {
        MotionFrame::from_input(input, self.facing())
    }
}

impl From<VehicleFacing> for VehicleReferenceFrame {
    fn from(value: VehicleFacing) -> Self {
        Self::from_facing(value)
    }
}

impl From<VehicleReferenceFrame> for VehicleFacing {
    fn from(value: VehicleReferenceFrame) -> Self {
        value.facing()
    }
}

/// Stable environment/body binding carried alongside vehicle telemetry or handoff state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleEnvironmentBinding {
    pub body_id: String,
    pub body_kind: String,
    pub body_center_x: f32,
    pub body_center_y: f32,
    pub render_radius_wu: f32,
    pub surface_radius_wu: f32,
    pub real_radius_km: f32,
    pub scale_divisor: f32,
    pub gravity_mu_km3_s2: f32,
    pub surface_gravity_mps2: f32,
    pub atmosphere_top_km: f32,
    pub atmosphere_dense_start_km: f32,
    pub atmosphere_drag_max: f32,
    pub cloud_bottom_km: f32,
    pub cloud_top_km: f32,
    #[serde(default, skip_serializing_if = "JsonMap::is_empty")]
    pub body_extras: JsonMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "JsonMap::is_empty")]
    pub extras: JsonMap<String, JsonValue>,
}

impl VehicleEnvironmentBinding {
    fn scalar_extra(&self, key: &str) -> Option<f32> {
        json_map_number(&self.extras, key).or_else(|| json_map_number(&self.body_extras, key))
    }

    pub fn from_snapshot(snapshot: &VehicleEnvironmentSnapshot) -> Self {
        Self {
            body_id: snapshot.body.body_id.clone(),
            body_kind: snapshot.body.body_kind.clone(),
            body_center_x: snapshot.body.center_x,
            body_center_y: snapshot.body.center_y,
            render_radius_wu: snapshot.body.render_radius_wu,
            surface_radius_wu: snapshot.body.surface_radius_wu,
            real_radius_km: snapshot.real_radius_km,
            scale_divisor: snapshot.scale_divisor,
            gravity_mu_km3_s2: snapshot.body.gravity_mu_km3_s2,
            surface_gravity_mps2: snapshot.surface_gravity_mps2,
            atmosphere_top_km: snapshot.atmosphere_top_km,
            atmosphere_dense_start_km: snapshot.atmosphere_dense_start_km,
            atmosphere_drag_max: snapshot.atmosphere_drag_max,
            cloud_bottom_km: snapshot.body.cloud_bottom_km,
            cloud_top_km: snapshot.body.cloud_top_km,
            body_extras: snapshot.body.extras.clone(),
            extras: snapshot.extras.clone(),
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.body_id = trim_owned(&self.body_id);
        self.body_kind = trim_owned(&self.body_kind);
        self.body_center_x = finite_or_zero(self.body_center_x);
        self.body_center_y = finite_or_zero(self.body_center_y);
        self.render_radius_wu = non_negative(self.render_radius_wu);
        self.surface_radius_wu = non_negative(self.surface_radius_wu);
        self.real_radius_km = non_negative(self.real_radius_km);
        self.scale_divisor = finite_or_zero(self.scale_divisor).max(0.0001);
        self.gravity_mu_km3_s2 = non_negative(self.gravity_mu_km3_s2);
        self.surface_gravity_mps2 = non_negative(self.surface_gravity_mps2);
        self.atmosphere_top_km = non_negative(self.atmosphere_top_km);
        self.atmosphere_dense_start_km =
            non_negative(self.atmosphere_dense_start_km).min(self.atmosphere_top_km);
        self.atmosphere_drag_max = non_negative(self.atmosphere_drag_max);
        self.cloud_bottom_km = non_negative(self.cloud_bottom_km);
        self.cloud_top_km = non_negative(self.cloud_top_km).max(self.cloud_bottom_km);
    }

    pub fn has_body(&self) -> bool {
        !self.body_id.is_empty() || !self.body_kind.is_empty()
    }

    pub fn has_atmosphere(&self) -> bool {
        self.atmosphere_top_km > 0.0 && self.atmosphere_drag_max > 0.0
    }

    pub fn km_per_wu(&self) -> f32 {
        self.scale_divisor.max(0.0001)
    }

    pub fn rotation_speed_deg_s(&self) -> f32 {
        self.scalar_extra("rotspeed")
            .or_else(|| self.scalar_extra("rotation_speed_deg_s"))
            .or_else(|| self.scalar_extra("rotation_speed"))
            .unwrap_or_default()
    }

    pub fn spin_omega_rad_s(&self) -> f32 {
        self.rotation_speed_deg_s().to_radians()
    }

    pub fn surface_gravity_wu_s2(&self) -> f32 {
        if self.gravity_mu_km3_s2 > 0.0 && self.surface_radius_wu > 0.0 {
            self.gravity_mu_km3_s2 / (self.surface_radius_wu * self.surface_radius_wu)
        } else {
            0.01
        }
    }

    pub fn circular_orbit_speed_wu_s(&self) -> f32 {
        if self.gravity_mu_km3_s2 > 0.0 && self.surface_radius_wu > 0.0 {
            (self.gravity_mu_km3_s2 / self.surface_radius_wu).sqrt()
        } else {
            1.0
        }
    }

    pub fn surface_clearance_radius_wu(&self, clearance_wu: f32) -> f32 {
        self.surface_radius_wu + non_negative(clearance_wu)
    }

    pub fn altitude_km_from_radius_wu(&self, radius_wu: f32) -> f32 {
        if self.scale_divisor <= 0.0 {
            return 0.0;
        }

        non_negative(radius_wu - self.surface_radius_wu) * self.scale_divisor
    }

    pub fn radius_wu_from_altitude_km(&self, altitude_km: f32) -> f32 {
        self.surface_radius_wu + non_negative(altitude_km) / self.scale_divisor.max(0.0001)
    }

    pub fn surface_contact_radius_wu(&self, contact_altitude_km: f32) -> f32 {
        self.radius_wu_from_altitude_km(contact_altitude_km)
    }

    pub fn atmosphere_top_altitude_wu(&self) -> f32 {
        if self.atmosphere_top_km > 0.0 {
            self.atmosphere_top_km / self.km_per_wu()
        } else {
            0.0
        }
    }

    pub fn atmosphere_drag_factor(&self, altitude_wu: f32) -> f32 {
        let top_alt_wu = self.atmosphere_top_altitude_wu();
        if top_alt_wu <= 0.0 {
            0.0
        } else {
            let alpha = clamp01((top_alt_wu - non_negative(altitude_wu)) / top_alt_wu.max(0.1));
            alpha * alpha * self.atmosphere_drag_max.max(0.0)
        }
    }

    pub fn carrier_speed_wu_s(&self, radius_wu: f32) -> f32 {
        self.spin_omega_rad_s() * non_negative(radius_wu)
    }

    pub fn is_near_surface(&self, radius_wu: f32, contact_altitude_km: f32) -> bool {
        non_negative(radius_wu) <= self.surface_contact_radius_wu(contact_altitude_km) + 0.0001
    }

    pub fn to_snapshot(&self) -> VehicleEnvironmentSnapshot {
        VehicleEnvironmentSnapshot {
            body: VehicleBodySnapshot {
                body_id: self.body_id.clone(),
                body_kind: self.body_kind.clone(),
                center_x: self.body_center_x,
                center_y: self.body_center_y,
                render_radius_wu: self.render_radius_wu,
                surface_radius_wu: self.surface_radius_wu,
                radius_km: self.real_radius_km,
                gravity_mu_km3_s2: self.gravity_mu_km3_s2,
                atmosphere_top_km: self.atmosphere_top_km,
                atmosphere_dense_start_km: self.atmosphere_dense_start_km,
                atmosphere_drag_max: self.atmosphere_drag_max,
                cloud_bottom_km: self.cloud_bottom_km,
                cloud_top_km: self.cloud_top_km,
                extras: self.body_extras.clone(),
            },
            real_radius_km: self.real_radius_km,
            scale_divisor: self.scale_divisor,
            surface_gravity_mps2: self.surface_gravity_mps2,
            atmosphere_top_km: self.atmosphere_top_km,
            atmosphere_dense_start_km: self.atmosphere_dense_start_km,
            atmosphere_drag_max: self.atmosphere_drag_max,
            extras: self.extras.clone(),
        }
        .normalized()
    }
}

impl From<&VehicleEnvironmentSnapshot> for VehicleEnvironmentBinding {
    fn from(value: &VehicleEnvironmentSnapshot) -> Self {
        Self::from_snapshot(value)
    }
}

impl From<VehicleEnvironmentSnapshot> for VehicleEnvironmentBinding {
    fn from(value: VehicleEnvironmentSnapshot) -> Self {
        Self::from_snapshot(&value)
    }
}

impl From<&VehicleEnvironmentBinding> for VehicleEnvironmentSnapshot {
    fn from(value: &VehicleEnvironmentBinding) -> Self {
        value.to_snapshot()
    }
}

impl From<VehicleEnvironmentBinding> for VehicleEnvironmentSnapshot {
    fn from(value: VehicleEnvironmentBinding) -> Self {
        value.to_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_frame_roundtrips_with_vehicle_facing() {
        let frame = VehicleReferenceFrame::from_heading(std::f32::consts::FRAC_PI_2);
        let facing = frame.facing();
        let roundtrip = VehicleReferenceFrame::from_facing(facing);
        let (forward, lateral) = frame.project_vector(3.0, 4.0);
        let world_relative = frame.world_vector_relative(5.0, -2.0, 1.5, 0.25);
        let local_relative =
            frame.project_vector_relative(world_relative.0, world_relative.1, 1.5, 0.25);

        assert!((frame.heading_deg() - 90.0).abs() < 0.001);
        assert_eq!(frame, roundtrip);
        assert!(forward.abs() > 0.0);
        assert!(lateral.abs() > 0.0);
        assert!((local_relative.0 - 5.0).abs() < 0.001);
        assert!((local_relative.1 + 2.0).abs() < 0.001);
    }

    #[test]
    fn reference_frame_rebuilds_world_vectors_from_local_projection() {
        let frame = VehicleReferenceFrame::from_heading(1.2345);
        let world = (2.5, -4.0);
        let projected = frame.project_vector(world.0, world.1);
        let rebuilt = frame.world_vector(projected.0, projected.1);

        assert!((rebuilt.0 - world.0).abs() < 0.001);
        assert!((rebuilt.1 - world.1).abs() < 0.001);
    }

    #[test]
    fn environment_binding_roundtrips_with_snapshot() {
        let snapshot = VehicleEnvironmentSnapshot {
            body: VehicleBodySnapshot {
                body_id: "generated".to_string(),
                body_kind: "earth_like".to_string(),
                center_x: 12.0,
                center_y: -3.0,
                render_radius_wu: 128.0,
                surface_radius_wu: 120.0,
                radius_km: 6371.0,
                gravity_mu_km3_s2: 398_600.44,
                atmosphere_top_km: 80.0,
                atmosphere_dense_start_km: 20.0,
                atmosphere_drag_max: 1.2,
                cloud_bottom_km: 3.0,
                cloud_top_km: 9.0,
                extras: JsonMap::new(),
            },
            real_radius_km: 6371.0,
            scale_divisor: 50.0,
            surface_gravity_mps2: 9.81,
            atmosphere_top_km: 80.0,
            atmosphere_dense_start_km: 20.0,
            atmosphere_drag_max: 1.2,
            extras: JsonMap::new(),
        };

        let binding = VehicleEnvironmentBinding::from_snapshot(&snapshot);
        let encoded: VehicleEnvironmentSnapshot = (&binding).into();

        assert!(binding.has_body());
        assert!(binding.has_atmosphere());
        assert_eq!(binding.km_per_wu(), 50.0);
        assert!((binding.surface_gravity_wu_s2() - 27.680586).abs() < 0.001);
        assert!((binding.circular_orbit_speed_wu_s() - 57.63446).abs() < 0.001);
        assert_eq!(binding.altitude_km_from_radius_wu(121.0), 50.0);
        assert_eq!(binding.radius_wu_from_altitude_km(50.0), 121.0);
        assert!(binding.is_near_surface(120.5, 25.0));
        assert_eq!(encoded, snapshot.normalized());
    }

    #[test]
    fn environment_binding_normalizes_invalid_ranges() {
        let binding = VehicleEnvironmentBinding {
            body_id: " generated ".to_string(),
            body_kind: " earth_like ".to_string(),
            body_center_x: f32::NAN,
            body_center_y: f32::INFINITY,
            render_radius_wu: -5.0,
            surface_radius_wu: 120.0,
            real_radius_km: -1.0,
            scale_divisor: 0.0,
            gravity_mu_km3_s2: -2.0,
            surface_gravity_mps2: -3.0,
            atmosphere_top_km: 10.0,
            atmosphere_dense_start_km: 25.0,
            atmosphere_drag_max: 0.4,
            cloud_bottom_km: 8.0,
            cloud_top_km: 2.0,
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();

        assert_eq!(binding.body_id, "generated");
        assert_eq!(binding.body_kind, "earth_like");
        assert_eq!(binding.body_center_x, 0.0);
        assert_eq!(binding.body_center_y, 0.0);
        assert_eq!(binding.render_radius_wu, 0.0);
        assert_eq!(binding.real_radius_km, 0.0);
        assert_eq!(binding.scale_divisor, 0.0001);
        assert_eq!(binding.atmosphere_dense_start_km, 10.0);
        assert_eq!(binding.cloud_top_km, 8.0);
        assert_eq!(binding.altitude_km_from_radius_wu(100.0), 0.0);
        assert_eq!(binding.radius_wu_from_altitude_km(-25.0), 120.0);
    }

    #[test]
    fn environment_binding_exposes_runtime_helpers_for_ship_motion() {
        let binding = VehicleEnvironmentBinding {
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            gravity_mu_km3_s2: 398_600.44,
            atmosphere_top_km: 80.0,
            atmosphere_drag_max: 1.2,
            extras: JsonMap::from_iter([("rotspeed".to_string(), JsonValue::from(3.0))]),
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();

        assert_eq!(binding.rotation_speed_deg_s(), 3.0);
        assert!((binding.spin_omega_rad_s() - 3.0_f32.to_radians()).abs() < 0.0001);
        assert!((binding.surface_clearance_radius_wu(0.35) - 120.35).abs() < 0.0001);
        assert!((binding.atmosphere_top_altitude_wu() - 1.6).abs() < 0.0001);
        assert!((binding.atmosphere_drag_factor(0.0) - 1.2).abs() < 0.0001);
        assert!(binding.atmosphere_drag_factor(10.0).abs() < 0.0001);
        assert!(
            (binding.carrier_speed_wu_s(120.0) - (120.0 * 3.0_f32.to_radians())).abs() < 0.0001
        );
    }
}
