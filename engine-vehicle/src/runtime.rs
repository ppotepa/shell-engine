use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{MotionFrame, MotionFrameInput};
use crate::{
    VehicleAssembly, VehicleCapabilities, VehicleControlState, VehicleEnvironmentBinding,
    VehicleFacing, VehicleInputIntent, VehicleKind, VehicleReferenceFrame, VehicleTelemetry,
    VehicleTelemetryInput, VehicleTelemetrySnapshot,
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
fn trim_owned(value: &str) -> String {
    value.trim().to_string()
}

#[inline]
fn clamp01(value: f32) -> f32 {
    finite_or_zero(value).clamp(0.0, 1.0)
}

#[inline]
fn normalize_angle_rad(value: f32) -> f32 {
    finite_or_zero(value).rem_euclid(std::f32::consts::TAU)
}

#[inline]
fn nearest_angle_rad(angle: f32, reference: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut best = normalize_angle_rad(angle);
    let reference = finite_or_zero(reference);
    let mut best_delta = (best - reference).abs();
    for candidate in [best + tau, best - tau] {
        let delta = (candidate - reference).abs();
        if delta < best_delta {
            best = candidate;
            best_delta = delta;
        }
    }
    best
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
fn environment_scalar(environment: &VehicleEnvironmentBinding, key: &str) -> Option<f32> {
    json_map_number(&environment.extras, key)
        .or_else(|| json_map_number(&environment.body_extras, key))
}

#[inline]
fn environment_spin_omega_rad_s(environment: &VehicleEnvironmentBinding) -> f32 {
    environment_scalar(environment, "rotspeed")
        .or_else(|| environment_scalar(environment, "rotation_speed_deg_s"))
        .or_else(|| environment_scalar(environment, "rotation_speed"))
        .unwrap_or_default()
        .to_radians()
}

#[inline]
fn anchor_basis(anchor_angle_deg: f32) -> (f32, f32, f32, f32) {
    let angle_rad = finite_or_zero(anchor_angle_deg).to_radians();
    let normal_x = angle_rad.cos();
    let normal_y = angle_rad.sin();
    let tangent_x = -angle_rad.sin();
    let tangent_y = angle_rad.cos();
    (normal_x, normal_y, tangent_x, tangent_y)
}

/// Thin seam for kind-specific vehicle stack assembly.
pub trait VehicleAssemblyModel {
    fn vehicle_kind(&self) -> VehicleKind;
    fn capabilities(&self) -> VehicleCapabilities;
    fn vehicle_assembly(&self) -> VehicleAssembly;
}

/// Thin seam for kind-specific control normalization.
pub trait VehicleControllerModel {
    fn default_control_state(&self) -> VehicleControlState;

    fn control_state_from_intent(&self, intent: VehicleInputIntent) -> VehicleControlState {
        let mut state = self.default_control_state();
        let intent = intent.normalized();
        state.throttle = intent.throttle;
        state.yaw = intent.yaw;
        state.strafe = intent.strafe;
        state.lift = intent.lift;
        state.pitch = intent.pitch;
        state.roll = intent.roll;
        state.brake_active = intent.brake;
        state.main_engine_active = intent.main_engine;
        state.stabilize_active = intent.stabilize;
        state.boost_scale = if intent.boost { 2.0 } else { 1.0 };
        state.normalized()
    }
}

/// Thin seam for kind-specific local reference-frame derivation.
pub trait VehicleReferenceFrameModel {
    fn facing_from_heading(&self, heading: f32) -> VehicleFacing {
        VehicleFacing::from_heading(heading)
    }

    fn reference_frame_from_heading(&self, heading: f32) -> VehicleReferenceFrame {
        VehicleReferenceFrame::from_heading(heading)
    }

    fn motion_frame_from_input(&self, input: MotionFrameInput, heading: f32) -> MotionFrame {
        MotionFrame::from_input(input, self.facing_from_heading(heading))
    }
}

/// Thin seam for kind-specific telemetry derivation.
pub trait VehicleTelemetryModel {
    fn telemetry_from_input(&self, input: VehicleTelemetryInput) -> VehicleTelemetry {
        VehicleTelemetry::from_runtime(input)
    }

    fn telemetry_snapshot_from_input(
        &self,
        input: VehicleTelemetryInput,
    ) -> VehicleTelemetrySnapshot {
        VehicleTelemetrySnapshot::from_runtime(input)
    }
}

/// Ship attachment mode relative to a nearby surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipSurfaceMode {
    #[default]
    Detached,
    SurfaceLocked,
    Grounded,
}

impl ShipSurfaceMode {
    pub fn is_grounded(self) -> bool {
        matches!(self, Self::Grounded)
    }

    pub fn is_surface_locked(self) -> bool {
        matches!(self, Self::SurfaceLocked | Self::Grounded)
    }

    pub fn is_detached(self) -> bool {
        matches!(self, Self::Detached)
    }
}

/// Active navigation frame for the ship runtime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipReferenceFrameKind {
    #[default]
    Inertial,
    LocalHorizon,
}

/// Runtime-local ship motion state that used to live as loose Rhai locals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipMotionState {
    pub radius_wu: f32,
    pub forward_speed_wu_s: f32,
    pub lateral_speed_wu_s: f32,
    pub radial_speed_wu_s: f32,
    pub yaw_rate_rad_s: f32,
    pub camera_sway: f32,
}

impl ShipMotionState {
    pub fn from_telemetry(
        telemetry: &VehicleTelemetrySnapshot,
        reference_frame: &ShipReferenceFrameState,
        environment: Option<&VehicleEnvironmentBinding>,
    ) -> Self {
        let radius_wu = if telemetry.radius_wu > 0.0 {
            telemetry.radius_wu
        } else if reference_frame.radius_wu > 0.0 {
            reference_frame.radius_wu
        } else if let Some(environment) = environment {
            if telemetry.altitude_km > 0.0 {
                environment.radius_wu_from_altitude_km(telemetry.altitude_km)
            } else {
                environment.surface_radius_wu
            }
        } else {
            0.0
        };

        Self {
            radius_wu,
            forward_speed_wu_s: telemetry.forward_speed_wu_s,
            lateral_speed_wu_s: telemetry.lateral_speed_wu_s,
            radial_speed_wu_s: telemetry.radial_speed_wu_s,
            yaw_rate_rad_s: telemetry.yaw_rate_rad_s,
            camera_sway: telemetry.camera_sway,
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.radius_wu = non_negative(self.radius_wu);
        self.forward_speed_wu_s = finite_or_zero(self.forward_speed_wu_s);
        self.lateral_speed_wu_s = finite_or_zero(self.lateral_speed_wu_s);
        self.radial_speed_wu_s = finite_or_zero(self.radial_speed_wu_s);
        self.yaw_rate_rad_s = finite_or_zero(self.yaw_rate_rad_s);
        self.camera_sway = finite_or_zero(self.camera_sway);
    }

    pub fn is_configured(&self) -> bool {
        self.radius_wu > 0.0
            || self.forward_speed_wu_s != 0.0
            || self.lateral_speed_wu_s != 0.0
            || self.radial_speed_wu_s != 0.0
            || self.yaw_rate_rad_s != 0.0
            || self.camera_sway != 0.0
    }
}

/// Derived runtime metrics for one ship frame step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipRuntimeStepReport {
    pub dt_s: f32,
    pub surface_radius_wu: f32,
    pub surface_clearance_wu: f32,
    pub surface_gravity_wu_s2: f32,
    pub surface_circular_speed_wu_s: f32,
    pub altitude_wu: f32,
    pub atmosphere_drag: f32,
    pub carrier_speed_wu_s: f32,
    pub signed_tangential_speed_wu_s: f32,
}

impl ShipRuntimeStepReport {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.dt_s = non_negative(self.dt_s);
        self.surface_radius_wu = non_negative(self.surface_radius_wu);
        self.surface_clearance_wu = non_negative(self.surface_clearance_wu);
        self.surface_gravity_wu_s2 = non_negative(self.surface_gravity_wu_s2);
        self.surface_circular_speed_wu_s = non_negative(self.surface_circular_speed_wu_s);
        self.altitude_wu = non_negative(self.altitude_wu);
        self.atmosphere_drag = non_negative(self.atmosphere_drag);
        self.carrier_speed_wu_s = finite_or_zero(self.carrier_speed_wu_s);
        self.signed_tangential_speed_wu_s = finite_or_zero(self.signed_tangential_speed_wu_s);
    }
}

impl ShipReferenceFrameKind {
    pub fn uses_local_horizon(self) -> bool {
        matches!(self, Self::LocalHorizon)
    }
}

/// Ship-specific reference-frame state including local-horizon and co-rotation data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipReferenceFrameState {
    pub kind: ShipReferenceFrameKind,
    pub reference: VehicleReferenceFrame,
    pub body_id: String,
    pub body_kind: String,
    pub anchor_angle_deg: f32,
    pub radius_wu: f32,
    pub altitude_km: f32,
    pub normal_x: f32,
    pub normal_y: f32,
    pub tangent_x: f32,
    pub tangent_y: f32,
    pub co_rotation_enabled: bool,
    pub carrier_speed_wu_s: f32,
    pub carrier_velocity_x: f32,
    pub carrier_velocity_y: f32,
    pub spin_omega_rad_s: f32,
}

impl Default for ShipReferenceFrameState {
    fn default() -> Self {
        Self::detached(0.0)
    }
}

impl ShipReferenceFrameState {
    pub fn detached(heading_rad: f32) -> Self {
        Self {
            kind: ShipReferenceFrameKind::Inertial,
            reference: VehicleReferenceFrame::from_heading(heading_rad),
            body_id: String::new(),
            body_kind: String::new(),
            anchor_angle_deg: 0.0,
            radius_wu: 0.0,
            altitude_km: 0.0,
            normal_x: 1.0,
            normal_y: 0.0,
            tangent_x: 0.0,
            tangent_y: 1.0,
            co_rotation_enabled: false,
            carrier_speed_wu_s: 0.0,
            carrier_velocity_x: 0.0,
            carrier_velocity_y: 0.0,
            spin_omega_rad_s: 0.0,
        }
        .normalized()
    }

    pub fn local_horizon(heading_rad: f32, anchor_angle_deg: f32) -> Self {
        Self {
            kind: ShipReferenceFrameKind::LocalHorizon,
            reference: VehicleReferenceFrame::from_heading(heading_rad),
            anchor_angle_deg,
            co_rotation_enabled: true,
            ..Self::detached(heading_rad)
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.reference.normalize();
        self.body_id = trim_owned(&self.body_id);
        self.body_kind = trim_owned(&self.body_kind);
        self.anchor_angle_deg = finite_or_zero(self.anchor_angle_deg).rem_euclid(360.0);
        self.radius_wu = non_negative(self.radius_wu);
        self.altitude_km = non_negative(self.altitude_km);
        self.spin_omega_rad_s = finite_or_zero(self.spin_omega_rad_s);
        self.carrier_speed_wu_s = finite_or_zero(self.carrier_speed_wu_s);
        if self.radius_wu > 0.0 && self.spin_omega_rad_s != 0.0 && self.carrier_speed_wu_s == 0.0 {
            self.carrier_speed_wu_s = self.spin_omega_rad_s * self.radius_wu;
        }

        let (normal_x, normal_y, tangent_x, tangent_y) = anchor_basis(self.anchor_angle_deg);
        self.normal_x = normal_x;
        self.normal_y = normal_y;
        self.tangent_x = tangent_x;
        self.tangent_y = tangent_y;

        if !self.kind.uses_local_horizon() {
            self.co_rotation_enabled = false;
        }

        if self.co_rotation_enabled {
            self.carrier_velocity_x = self.tangent_x * self.carrier_speed_wu_s;
            self.carrier_velocity_y = self.tangent_y * self.carrier_speed_wu_s;
        } else {
            self.carrier_velocity_x = 0.0;
            self.carrier_velocity_y = 0.0;
        }
    }

    pub fn with_reference(mut self, reference: VehicleReferenceFrame) -> Self {
        self.reference = reference;
        self.normalized()
    }

    pub fn with_environment(mut self, environment: &VehicleEnvironmentBinding) -> Self {
        self.body_id = environment.body_id.clone();
        self.body_kind = environment.body_kind.clone();
        if self.radius_wu <= 0.0 {
            self.radius_wu = if self.altitude_km > 0.0 {
                environment.radius_wu_from_altitude_km(self.altitude_km)
            } else {
                environment.surface_radius_wu
            };
        }
        if self.spin_omega_rad_s == 0.0 {
            self.spin_omega_rad_s = environment_spin_omega_rad_s(environment);
        }
        if self.radius_wu > 0.0 && self.carrier_speed_wu_s == 0.0 {
            self.carrier_speed_wu_s = self.spin_omega_rad_s * self.radius_wu;
        }
        self.normalized()
    }

    pub fn with_surface_anchor(
        mut self,
        anchor_angle_deg: f32,
        radius_wu: f32,
        altitude_km: f32,
    ) -> Self {
        self.anchor_angle_deg = anchor_angle_deg;
        self.radius_wu = radius_wu;
        self.altitude_km = altitude_km;
        self.normalized()
    }

    pub fn with_carrier_speed(mut self, carrier_speed_wu_s: f32, spin_omega_rad_s: f32) -> Self {
        self.carrier_speed_wu_s = carrier_speed_wu_s;
        self.spin_omega_rad_s = spin_omega_rad_s;
        self.normalized()
    }

    pub fn with_co_rotation(mut self, enabled: bool) -> Self {
        self.co_rotation_enabled = enabled;
        self.normalized()
    }

    pub fn has_body(&self) -> bool {
        !self.body_id.is_empty() || !self.body_kind.is_empty()
    }

    pub fn has_surface_anchor(&self) -> bool {
        self.has_body() || self.radius_wu > 0.0 || self.altitude_km > 0.0
    }

    pub fn uses_local_horizon(&self) -> bool {
        self.kind.uses_local_horizon()
    }

    pub fn uses_co_rotation(&self) -> bool {
        self.uses_local_horizon() && self.co_rotation_enabled
    }

    pub fn is_configured(&self) -> bool {
        self.uses_local_horizon()
            || self.uses_co_rotation()
            || self.has_body()
            || self.radius_wu > 0.0
            || self.altitude_km > 0.0
    }

    pub fn effective_world_velocity(&self, forward_speed: f32, lateral_speed: f32) -> (f32, f32) {
        if self.uses_co_rotation() {
            self.reference.world_vector_relative(
                forward_speed,
                lateral_speed,
                self.carrier_velocity_x,
                self.carrier_velocity_y,
            )
        } else {
            self.reference.world_vector(forward_speed, lateral_speed)
        }
    }

    pub fn local_velocity_from_world(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        if self.uses_co_rotation() {
            self.reference.project_vector_relative(
                world_x,
                world_y,
                self.carrier_velocity_x,
                self.carrier_velocity_y,
            )
        } else {
            self.reference.project_vector(world_x, world_y)
        }
    }
}

/// Ship runtime state owned by `engine-vehicle`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipRuntimeState {
    pub surface_mode: ShipSurfaceMode,
    pub reference_frame: ShipReferenceFrameState,
    pub motion: ShipMotionState,
    pub control: VehicleControlState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<VehicleEnvironmentBinding>,
}

impl Default for ShipRuntimeState {
    fn default() -> Self {
        Self {
            surface_mode: ShipSurfaceMode::Detached,
            reference_frame: ShipReferenceFrameState::default(),
            motion: ShipMotionState::default(),
            control: VehicleControlState::default(),
            environment: None,
        }
    }
}

impl ShipRuntimeState {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.control = self.control.clone().normalized();
        self.reference_frame = self.reference_frame.clone().normalized();
        self.motion = self.motion.normalized();
        self.environment = self
            .environment
            .take()
            .map(|environment| environment.normalized());

        if self.surface_mode.is_surface_locked() {
            self.reference_frame.kind = ShipReferenceFrameKind::LocalHorizon;
            self.reference_frame.co_rotation_enabled = true;
        }

        if let Some(environment) = self.environment.as_ref() {
            if self.motion.radius_wu <= 0.0 {
                self.motion.radius_wu = if self.reference_frame.radius_wu > 0.0 {
                    self.reference_frame.radius_wu
                } else {
                    environment.surface_radius_wu
                };
            }
            let derived_altitude_km = environment.altitude_km_from_radius_wu(
                self.motion.radius_wu.max(self.reference_frame.radius_wu),
            );
            let altitude_km = if self.reference_frame.altitude_km > 0.0 {
                self.reference_frame.altitude_km
            } else {
                derived_altitude_km
            };
            self.reference_frame = self.reference_frame.clone().with_environment(environment);
            self.reference_frame = self.reference_frame.clone().with_surface_anchor(
                self.reference_frame.anchor_angle_deg,
                self.motion.radius_wu.max(self.reference_frame.radius_wu),
                altitude_km,
            );
        } else {
            if self.motion.radius_wu <= 0.0 {
                self.motion.radius_wu = self.reference_frame.radius_wu;
            } else if self.reference_frame.radius_wu <= 0.0 {
                self.reference_frame.radius_wu = self.motion.radius_wu;
            }
            self.reference_frame.normalize();
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.surface_mode.is_grounded()
    }

    pub fn is_surface_locked(&self) -> bool {
        self.surface_mode.is_surface_locked()
    }

    pub fn is_detached(&self) -> bool {
        self.surface_mode.is_detached()
    }
}

/// One ship-runtime evaluation input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipRuntimeInput {
    pub dt_s: f32,
    pub control: VehicleControlState,
    pub telemetry: VehicleTelemetrySnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<VehicleEnvironmentBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_contact: Option<bool>,
    pub request_surface_lock: bool,
    pub request_detach: bool,
    pub request_local_horizon: bool,
    pub request_inertial_frame: bool,
    pub prefer_grounded_on_contact: bool,
}

impl Default for ShipRuntimeInput {
    fn default() -> Self {
        Self {
            dt_s: 0.0,
            control: VehicleControlState::default(),
            telemetry: VehicleTelemetrySnapshot::default(),
            environment: None,
            surface_contact: None,
            request_surface_lock: false,
            request_detach: false,
            request_local_horizon: false,
            request_inertial_frame: false,
            prefer_grounded_on_contact: true,
        }
    }
}

impl ShipRuntimeInput {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.dt_s = non_negative(self.dt_s);
        self.control = self.control.clone().normalized();
        self.telemetry = self.telemetry.clone().normalized();
        self.environment = self
            .environment
            .take()
            .map(|environment| environment.normalized());
        if self.request_inertial_frame {
            self.request_local_horizon = false;
        }
    }
}

/// Ship-runtime evaluation result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShipRuntimeOutput {
    pub state: ShipRuntimeState,
    pub telemetry: VehicleTelemetrySnapshot,
    pub report: ShipRuntimeStepReport,
    pub surface_mode_changed: bool,
    pub reference_frame_changed: bool,
    pub motion_changed: bool,
}

impl Default for ShipRuntimeOutput {
    fn default() -> Self {
        Self {
            state: ShipRuntimeState::default(),
            telemetry: VehicleTelemetrySnapshot::default(),
            report: ShipRuntimeStepReport::default(),
            surface_mode_changed: false,
            reference_frame_changed: false,
            motion_changed: false,
        }
    }
}

/// Ship-specific runtime seam for attachment and local-horizon state.
pub trait ShipRuntimeModel: VehicleControllerModel {
    fn ship_grounded_speed_threshold_wu_s(&self) -> f32;
    fn ship_surface_contact_altitude_threshold_km(&self) -> f32;

    fn default_ship_runtime_state(&self) -> ShipRuntimeState {
        ShipRuntimeState {
            control: self.default_control_state(),
            ..ShipRuntimeState::default()
        }
        .normalized()
    }

    fn ship_runtime_state_from_telemetry(
        &self,
        telemetry: VehicleTelemetrySnapshot,
    ) -> ShipRuntimeState {
        let telemetry = telemetry.normalized();
        ShipRuntimeState {
            surface_mode: telemetry.surface_mode,
            reference_frame: telemetry.ship_reference.clone(),
            motion: ShipMotionState::from_telemetry(
                &telemetry,
                &telemetry.ship_reference,
                telemetry.environment.as_ref(),
            ),
            control: self.default_control_state(),
            environment: telemetry.environment.clone(),
        }
        .normalized()
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

        if input.request_inertial_frame {
            reference_frame.kind = ShipReferenceFrameKind::Inertial;
            reference_frame.co_rotation_enabled = false;
        } else if input.request_local_horizon
            || previous.reference_frame.uses_local_horizon()
            || reference_frame.uses_local_horizon()
            || environment.is_some()
        {
            reference_frame.kind = ShipReferenceFrameKind::LocalHorizon;
        }

        let surface_contact_altitude_km = self.ship_surface_contact_altitude_threshold_km();
        let surface_contact = input.surface_contact.unwrap_or_else(|| {
            input.telemetry.grounded
                || input.telemetry.altitude_km <= surface_contact_altitude_km
                || environment
                    .as_ref()
                    .map(|environment| {
                        environment.is_near_surface(
                            input.telemetry.radius_wu.max(reference_frame.radius_wu),
                            surface_contact_altitude_km,
                        )
                    })
                    .unwrap_or(false)
        });
        let grounded_speed_threshold = self.ship_grounded_speed_threshold_wu_s().max(0.0);
        let motion_is_groundable = input.telemetry.forward_speed_wu_s.abs()
            <= grounded_speed_threshold
            && input.telemetry.lateral_speed_wu_s.abs() <= grounded_speed_threshold
            && input.telemetry.radial_speed_wu_s.abs() <= grounded_speed_threshold;
        let keep_grounded = previous.surface_mode.is_grounded()
            && input.telemetry.radial_speed_wu_s.abs() <= grounded_speed_threshold;

        let surface_mode = if input.request_detach || !surface_contact {
            ShipSurfaceMode::Detached
        } else if input.request_surface_lock || !input.prefer_grounded_on_contact {
            ShipSurfaceMode::SurfaceLocked
        } else if keep_grounded || motion_is_groundable {
            ShipSurfaceMode::Grounded
        } else {
            ShipSurfaceMode::SurfaceLocked
        };

        if surface_mode.is_surface_locked() {
            reference_frame.kind = ShipReferenceFrameKind::LocalHorizon;
            reference_frame.co_rotation_enabled = true;
        } else if input.request_inertial_frame {
            reference_frame.kind = ShipReferenceFrameKind::Inertial;
            reference_frame.co_rotation_enabled = false;
        } else if reference_frame.uses_local_horizon() {
            reference_frame.co_rotation_enabled =
                input.telemetry.ship_reference.co_rotation_enabled
                    || previous.reference_frame.co_rotation_enabled
                    || input.request_local_horizon;
        }

        let motion =
            ShipMotionState::from_telemetry(&input.telemetry, &reference_frame, environment.as_ref());
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
    use serde_json::{Map as JsonMap, Value as JsonValue};

    use crate::{
        ShipModel, VehicleControllerModel, VehicleEnvironmentBinding, VehiclePacketTelemetry,
    };

    use super::{ShipReferenceFrameState, ShipRuntimeInput, ShipRuntimeModel, ShipSurfaceMode};

    #[test]
    fn ship_reference_frame_state_applies_carrier_motion_in_local_horizon() {
        let frame = ShipReferenceFrameState::local_horizon(0.0, 90.0).with_carrier_speed(4.0, 0.1);
        let world = frame.effective_world_velocity(10.0, -2.0);
        let local = frame.local_velocity_from_world(world.0, world.1);

        assert!(frame.uses_local_horizon());
        assert!(frame.uses_co_rotation());
        assert!(frame.normal_x.abs() < 0.001);
        assert!((frame.normal_y - 1.0).abs() < 0.001);
        assert!((local.0 - 10.0).abs() < 0.001);
        assert!((local.1 + 2.0).abs() < 0.001);
    }

    #[test]
    fn ship_reference_frame_state_derives_carrier_motion_from_environment_rotation() {
        let frame = ShipReferenceFrameState::local_horizon(0.0, 90.0)
            .with_surface_anchor(90.0, 120.0, 0.0)
            .with_environment(
                &VehicleEnvironmentBinding {
                    surface_radius_wu: 120.0,
                    scale_divisor: 50.0,
                    extras: JsonMap::from_iter([("rotspeed".to_string(), JsonValue::from(3.0))]),
                    ..VehicleEnvironmentBinding::default()
                }
                .normalized(),
            );

        assert!(frame.has_surface_anchor());
        assert!((frame.spin_omega_rad_s - 3.0_f32.to_radians()).abs() < 0.0001);
        assert!((frame.carrier_speed_wu_s - (120.0 * 3.0_f32.to_radians())).abs() < 0.0001);
    }

    #[test]
    fn ship_runtime_step_tracks_attachment_transitions() {
        let model = ShipModel::default();
        let grounded_telemetry =
            crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                heading_deg: 0.0,
                altitude_km: 0.0,
                radius_wu: 120.0,
                spawn_angle_deg: 45.0,
                grounded: true,
                ..VehiclePacketTelemetry::default()
            });
        let grounded = model.ship_runtime_step(
            model.default_ship_runtime_state(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: grounded_telemetry.clone(),
                surface_contact: Some(true),
                ..ShipRuntimeInput::default()
            },
        );
        assert_eq!(grounded.state.surface_mode, ShipSurfaceMode::Grounded);
        assert!(grounded.telemetry.grounded);

        let detached = model.ship_runtime_step(
            grounded.state.clone(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: grounded.telemetry.clone(),
                surface_contact: Some(true),
                request_detach: true,
                request_local_horizon: true,
                ..ShipRuntimeInput::default()
            },
        );
        assert_eq!(detached.state.surface_mode, ShipSurfaceMode::Detached);
        assert!(detached.state.reference_frame.uses_local_horizon());

        let mut locked_telemetry = detached.telemetry.clone();
        locked_telemetry.forward_speed_wu_s = 6.0;
        locked_telemetry.lateral_speed_wu_s = 0.5;
        locked_telemetry.radial_speed_wu_s = 0.0;
        let locked = model.ship_runtime_step(
            detached.state.clone(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: locked_telemetry,
                surface_contact: Some(true),
                request_surface_lock: true,
                ..ShipRuntimeInput::default()
            },
        );
        assert_eq!(locked.state.surface_mode, ShipSurfaceMode::SurfaceLocked);
        assert!(locked.reference_frame_changed || locked.surface_mode_changed);
    }

    #[test]
    fn ship_runtime_state_from_telemetry_restores_surface_context() {
        let model = ShipModel::new("sim_lite");
        let environment = VehicleEnvironmentBinding {
            body_id: "generated".to_string(),
            body_kind: "earth_like".to_string(),
            surface_radius_wu: 120.0,
            scale_divisor: 50.0,
            ..VehicleEnvironmentBinding::default()
        }
        .normalized();
        let telemetry = crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
            heading_deg: 45.0,
            altitude_km: 0.0,
            radius_wu: 120.0,
            spawn_angle_deg: 33.0,
            grounded: true,
            ..VehiclePacketTelemetry::default()
        })
        .with_environment(environment.clone());

        let state = model.ship_runtime_state_from_telemetry(telemetry);

        assert_eq!(state.surface_mode, ShipSurfaceMode::Grounded);
        assert_eq!(state.control.profile_id, "sim-lite");
        assert!(state.reference_frame.uses_local_horizon());
        assert!(state.reference_frame.uses_co_rotation());
        assert_eq!(state.reference_frame.anchor_angle_deg, 33.0);
        assert_eq!(state.reference_frame.body_id, "generated");
        assert_eq!(state.reference_frame.body_kind, "earth_like");
        assert_eq!(state.environment, Some(environment));
    }

    #[test]
    fn ship_runtime_step_prefers_explicit_inertial_detach_over_local_horizon_request() {
        let model = ShipModel::default();
        let previous = model.ship_runtime_state_from_telemetry(
            crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                heading_deg: 0.0,
                altitude_km: 0.0,
                radius_wu: 120.0,
                spawn_angle_deg: 20.0,
                grounded: true,
                ..VehiclePacketTelemetry::default()
            })
            .with_environment(
                VehicleEnvironmentBinding {
                    body_id: "generated".to_string(),
                    body_kind: "earth_like".to_string(),
                    surface_radius_wu: 120.0,
                    scale_divisor: 50.0,
                    ..VehicleEnvironmentBinding::default()
                }
                .normalized(),
            ),
        );

        let output = model.ship_runtime_step(
            previous.clone(),
            ShipRuntimeInput {
                control: model.default_control_state(),
                telemetry: crate::VehicleTelemetrySnapshot::from_packet(&VehiclePacketTelemetry {
                    heading_deg: 90.0,
                    altitude_km: 5.0,
                    radius_wu: 125.0,
                    spawn_angle_deg: 75.0,
                    grounded: false,
                    ..VehiclePacketTelemetry::default()
                }),
                surface_contact: Some(false),
                request_inertial_frame: true,
                request_local_horizon: true,
                ..ShipRuntimeInput::default()
            },
        );

        assert_eq!(previous.surface_mode, ShipSurfaceMode::Grounded);
        assert_eq!(output.state.surface_mode, ShipSurfaceMode::Detached);
        assert!(output.surface_mode_changed);
        assert!(output.reference_frame_changed);
        assert!(output.state.is_detached());
        assert!(!output.state.reference_frame.uses_local_horizon());
        assert!(!output.state.reference_frame.uses_co_rotation());
        assert_eq!(
            output.state.reference_frame.kind,
            super::ShipReferenceFrameKind::Inertial
        );
        assert!(!output.telemetry.grounded);
        assert_eq!(output.telemetry.surface_mode, ShipSurfaceMode::Detached);
    }
}
