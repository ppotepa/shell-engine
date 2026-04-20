use serde::{Deserialize, Serialize};

#[path = "frame.rs"]
mod frame;
#[path = "telemetry.rs"]
mod telemetry;

pub use frame::{VehicleEnvironmentBinding, VehicleReferenceFrame};
pub use telemetry::VehicleTelemetrySnapshot;

pub const DEFAULT_VEHICLE_PROFILE_ID: &str = "arcade";
pub const SIM_LITE_VEHICLE_PROFILE_ID: &str = "sim-lite";
pub const BUILTIN_SHIP_VEHICLE_PROFILE_IDS: [&str; 2] =
    [DEFAULT_VEHICLE_PROFILE_ID, SIM_LITE_VEHICLE_PROFILE_ID];

#[inline]
fn clamp_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

#[inline]
fn normalize_angle_rad(value: f32) -> f32 {
    if value.is_finite() {
        value.rem_euclid(std::f32::consts::TAU)
    } else {
        0.0
    }
}

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
fn opposing_axis(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

pub fn normalize_vehicle_profile_id(profile_id: &str) -> String {
    let trimmed = profile_id.trim();
    if trimmed.is_empty() {
        return DEFAULT_VEHICLE_PROFILE_ID.to_string();
    }

    let lowered = trimmed.to_ascii_lowercase();
    match lowered.as_str() {
        "arc" | "arcade" | "default" => DEFAULT_VEHICLE_PROFILE_ID.to_string(),
        "sim lite" | "sim_lite" | "sim-lite" => SIM_LITE_VEHICLE_PROFILE_ID.to_string(),
        _ => trimmed.to_string(),
    }
}

/// Built-in ship-friendly control profiles recognized by `engine-vehicle`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VehicleShipProfile {
    #[default]
    Arcade,
    #[serde(alias = "sim_lite", alias = "sim lite")]
    SimLite,
}

impl VehicleShipProfile {
    pub const ALL: [Self; 2] = [Self::Arcade, Self::SimLite];

    pub fn profile_id(self) -> &'static str {
        match self {
            Self::Arcade => DEFAULT_VEHICLE_PROFILE_ID,
            Self::SimLite => SIM_LITE_VEHICLE_PROFILE_ID,
        }
    }

    pub fn from_profile_id(profile_id: &str) -> Option<Self> {
        match normalize_vehicle_profile_id(profile_id).as_str() {
            DEFAULT_VEHICLE_PROFILE_ID => Some(Self::Arcade),
            SIM_LITE_VEHICLE_PROFILE_ID => Some(Self::SimLite),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Arcade => Self::SimLite,
            Self::SimLite => Self::Arcade,
        }
    }

    pub fn tuning(self) -> VehicleShipProfileTuning {
        match self {
            Self::Arcade => VehicleShipProfileTuning {
                forward_accel_g: 1.10,
                side_accel_g: 0.72,
                lift_accel_g: 2.90,
                main_engine_g: 1.95,
                max_speed_ratio: 1.85,
                max_vrad_ratio: 1.05,
                linear_damp: 0.06,
                side_trim: 0.28,
                side_thrust_trim: 1.10,
                yaw_response: 8.4,
                yaw_damp: 3.2,
                yaw_max: 1.35,
                alt_hold_kp: 0.018,
                alt_hold_kd: 0.095,
                heading_hold_kp: 2.2,
                camera_sway_tau: 0.42,
                camera_sway_gain: 0.24,
                grounded_speed_threshold_wu_s: 2.5,
                surface_contact_altitude_threshold_km: 0.05,
                surface_clearance_km: 0.35,
                surface_clearance_min_wu: 0.05,
                takeoff_lift_threshold: 0.001,
            },
            Self::SimLite => VehicleShipProfileTuning {
                forward_accel_g: 0.82,
                side_accel_g: 0.55,
                lift_accel_g: 2.28,
                main_engine_g: 1.46,
                max_speed_ratio: 1.67,
                max_vrad_ratio: 0.95,
                linear_damp: 0.04,
                side_trim: 0.20,
                side_thrust_trim: 1.00,
                yaw_response: 7.2,
                yaw_damp: 2.6,
                yaw_max: 1.10,
                alt_hold_kp: 0.010,
                alt_hold_kd: 0.065,
                heading_hold_kp: 1.4,
                camera_sway_tau: 0.50,
                camera_sway_gain: 0.28,
                grounded_speed_threshold_wu_s: 2.5,
                surface_contact_altitude_threshold_km: 0.05,
                surface_clearance_km: 0.35,
                surface_clearance_min_wu: 0.05,
                takeoff_lift_threshold: 0.001,
            },
        }
    }
}

pub fn next_builtin_ship_profile_id(profile_id: &str) -> &'static str {
    VehicleShipProfile::from_profile_id(profile_id)
        .map(VehicleShipProfile::next)
        .unwrap_or_default()
        .profile_id()
}

/// Built-in ship tuning values kept in `engine-vehicle` so mods do not need
/// to own per-profile scalar constants.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleShipProfileTuning {
    pub forward_accel_g: f32,
    pub side_accel_g: f32,
    pub lift_accel_g: f32,
    pub main_engine_g: f32,
    pub max_speed_ratio: f32,
    pub max_vrad_ratio: f32,
    pub linear_damp: f32,
    pub side_trim: f32,
    pub side_thrust_trim: f32,
    pub yaw_response: f32,
    pub yaw_damp: f32,
    pub yaw_max: f32,
    pub alt_hold_kp: f32,
    pub alt_hold_kd: f32,
    pub heading_hold_kp: f32,
    pub camera_sway_tau: f32,
    pub camera_sway_gain: f32,
    pub grounded_speed_threshold_wu_s: f32,
    pub surface_contact_altitude_threshold_km: f32,
    pub surface_clearance_km: f32,
    pub surface_clearance_min_wu: f32,
    pub takeoff_lift_threshold: f32,
}

impl Default for VehicleShipProfileTuning {
    fn default() -> Self {
        VehicleShipProfile::default().tuning()
    }
}

impl VehicleShipProfileTuning {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.forward_accel_g = finite_or_zero(self.forward_accel_g).max(0.0);
        self.side_accel_g = finite_or_zero(self.side_accel_g).max(0.0);
        self.lift_accel_g = finite_or_zero(self.lift_accel_g).max(0.0);
        self.main_engine_g = finite_or_zero(self.main_engine_g).max(0.0);
        self.max_speed_ratio = finite_or_zero(self.max_speed_ratio).max(0.0);
        self.max_vrad_ratio = finite_or_zero(self.max_vrad_ratio).max(0.0);
        self.linear_damp = finite_or_zero(self.linear_damp).max(0.0);
        self.side_trim = finite_or_zero(self.side_trim).max(0.0);
        self.side_thrust_trim = finite_or_zero(self.side_thrust_trim).max(0.0);
        self.yaw_response = finite_or_zero(self.yaw_response).max(0.0);
        self.yaw_damp = finite_or_zero(self.yaw_damp).max(0.0);
        self.yaw_max = finite_or_zero(self.yaw_max).max(0.0);
        self.alt_hold_kp = finite_or_zero(self.alt_hold_kp);
        self.alt_hold_kd = finite_or_zero(self.alt_hold_kd);
        self.heading_hold_kp = finite_or_zero(self.heading_hold_kp);
        self.camera_sway_tau = finite_or_zero(self.camera_sway_tau).max(0.0);
        self.camera_sway_gain = finite_or_zero(self.camera_sway_gain).max(0.0);
        self.grounded_speed_threshold_wu_s =
            finite_or_zero(self.grounded_speed_threshold_wu_s).max(0.0);
        self.surface_contact_altitude_threshold_km =
            finite_or_zero(self.surface_contact_altitude_threshold_km).max(0.0);
        self.surface_clearance_km = finite_or_zero(self.surface_clearance_km).max(0.0);
        self.surface_clearance_min_wu = finite_or_zero(self.surface_clearance_min_wu).max(0.0);
        self.takeoff_lift_threshold = finite_or_zero(self.takeoff_lift_threshold).max(0.0);
    }

    pub fn forward_accel_wu_s2(self, surface_gravity_wu_s2: f32) -> f32 {
        non_negative(surface_gravity_wu_s2) * self.forward_accel_g
    }

    pub fn side_accel_wu_s2(self, surface_gravity_wu_s2: f32) -> f32 {
        non_negative(surface_gravity_wu_s2) * self.side_accel_g
    }

    pub fn lift_accel_wu_s2(self, surface_gravity_wu_s2: f32) -> f32 {
        non_negative(surface_gravity_wu_s2) * self.lift_accel_g
    }

    pub fn main_engine_accel_wu_s2(self, surface_gravity_wu_s2: f32) -> f32 {
        non_negative(surface_gravity_wu_s2) * self.main_engine_g
    }

    pub fn max_speed_wu_s(self, surface_circular_speed_wu_s: f32) -> f32 {
        (non_negative(surface_circular_speed_wu_s) * self.max_speed_ratio).max(0.5)
    }

    pub fn max_reverse_speed_wu_s(self, surface_circular_speed_wu_s: f32) -> f32 {
        self.max_speed_wu_s(surface_circular_speed_wu_s) * 0.5
    }

    pub fn max_radial_speed_wu_s(self, surface_circular_speed_wu_s: f32) -> f32 {
        (non_negative(surface_circular_speed_wu_s) * self.max_vrad_ratio).max(0.25)
    }

    pub fn surface_clearance_wu(self, km_per_wu: f32) -> f32 {
        (self.surface_clearance_km / finite_or_zero(km_per_wu).max(0.0001))
            .max(self.surface_clearance_min_wu)
    }

    pub fn yaw_blend(self, dt_s: f32) -> f32 {
        (self.yaw_response * finite_or_zero(dt_s).max(0.0)).min(1.0)
    }

    pub fn camera_sway_alpha(self, dt_s: f32) -> f32 {
        let dt_s = finite_or_zero(dt_s).max(0.0);
        if dt_s <= 0.0 {
            0.0
        } else {
            dt_s / (self.camera_sway_tau.max(0.0001) + dt_s)
        }
    }

    pub fn camera_sway_target(self, yaw_rate_rad_s: f32) -> f32 {
        (-finite_or_zero(yaw_rate_rad_s) * self.camera_sway_gain).clamp(-0.20, 0.20)
    }
}

pub fn builtin_ship_profile_tuning(profile_id: &str) -> VehicleShipProfileTuning {
    VehicleShipProfile::from_profile_id(profile_id)
        .unwrap_or_default()
        .tuning()
}

/// Assist toggles carried independently of any one runtime/input backend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleAssistState {
    pub alt_hold: bool,
    pub heading_hold: bool,
}

impl VehicleAssistState {
    pub fn from_flags(alt_hold: bool, heading_hold: bool) -> Self {
        Self {
            alt_hold,
            heading_hold,
        }
    }

    pub fn any_enabled(&self) -> bool {
        self.alt_hold || self.heading_hold
    }
}

/// Button-backed ship input helper that centralizes the common "opposing
/// directions cancel out" glue used by mods before they build typed intents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleButtonInput {
    pub forward: bool,
    pub reverse: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
    pub lift_up: bool,
    pub lift_down: bool,
    /// Left maps to positive yaw to match the current ship controller scripts.
    pub yaw_left: bool,
    pub yaw_right: bool,
    pub pitch_up: bool,
    pub pitch_down: bool,
    pub roll_left: bool,
    pub roll_right: bool,
    pub brake: bool,
    pub boost: bool,
    pub stabilize: bool,
    pub main_engine: bool,
}

impl VehicleButtonInput {
    pub fn throttle_axis(&self) -> f32 {
        opposing_axis(self.reverse, self.forward)
    }

    pub fn strafe_axis(&self) -> f32 {
        opposing_axis(self.strafe_left, self.strafe_right)
    }

    pub fn lift_axis(&self) -> f32 {
        opposing_axis(self.lift_down, self.lift_up)
    }

    pub fn yaw_axis(&self) -> f32 {
        opposing_axis(self.yaw_right, self.yaw_left)
    }

    pub fn pitch_axis(&self) -> f32 {
        opposing_axis(self.pitch_down, self.pitch_up)
    }

    pub fn roll_axis(&self) -> f32 {
        opposing_axis(self.roll_left, self.roll_right)
    }

    pub fn intent(&self) -> VehicleInputIntent {
        VehicleInputIntent {
            throttle: self.throttle_axis(),
            yaw: self.yaw_axis(),
            strafe: self.strafe_axis(),
            lift: self.lift_axis(),
            pitch: self.pitch_axis(),
            roll: self.roll_axis(),
            brake: self.brake,
            boost: self.boost,
            stabilize: self.stabilize,
            main_engine: self.main_engine,
        }
    }

    pub fn motion(&self) -> VehicleMotionIntent {
        self.intent().motion()
    }

    pub fn control_state(
        &self,
        profile_id: &str,
        assists: VehicleAssistState,
    ) -> VehicleControlState {
        VehicleControlState::from_button_input(profile_id, *self, assists)
    }

    pub fn is_idle(&self) -> bool {
        !self.forward
            && !self.reverse
            && !self.strafe_left
            && !self.strafe_right
            && !self.lift_up
            && !self.lift_down
            && !self.yaw_left
            && !self.yaw_right
            && !self.pitch_up
            && !self.pitch_down
            && !self.roll_left
            && !self.roll_right
            && !self.brake
            && !self.boost
            && !self.stabilize
            && !self.main_engine
    }
}

/// Vehicle-local translation request resolved independently of any concrete ship implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleTranslationIntent {
    pub forward: f32,
    pub lateral: f32,
    pub vertical: f32,
    pub main_engine: bool,
}

impl VehicleTranslationIntent {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.forward = clamp_unit(self.forward);
        self.lateral = clamp_unit(self.lateral);
        self.vertical = clamp_unit(self.vertical);
    }

    pub fn has_input(&self) -> bool {
        self.forward != 0.0 || self.lateral != 0.0 || self.vertical != 0.0 || self.main_engine
    }
}

/// Vehicle-local rotational request resolved independently of any concrete ship implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleRotationIntent {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub stabilize: bool,
}

impl VehicleRotationIntent {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.yaw = clamp_unit(self.yaw);
        self.pitch = clamp_unit(self.pitch);
        self.roll = clamp_unit(self.roll);
    }

    pub fn has_input(&self) -> bool {
        self.yaw != 0.0 || self.pitch != 0.0 || self.roll != 0.0
    }
}

/// Grouped vehicle motion request that can be mapped onto ships today and
/// future vehicle rigs later without hard-coding one specific controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleMotionIntent {
    pub translation: VehicleTranslationIntent,
    pub rotation: VehicleRotationIntent,
    pub brake: bool,
    pub boost: bool,
}

impl VehicleMotionIntent {
    pub fn from_buttons(buttons: VehicleButtonInput) -> Self {
        buttons.motion()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.translation.normalize();
        self.rotation.normalize();
    }

    pub fn is_idle(&self) -> bool {
        !self.translation.has_input() && !self.rotation.has_input() && !self.brake && !self.boost
    }
}

/// High-level operator intent for one vehicle update tick.
///
/// Axes are normalized to `[-1, 1]` by [`VehicleInputIntent::normalize`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleInputIntent {
    /// Forward/reverse translation request.
    pub throttle: f32,
    /// Left/right yaw request.
    pub yaw: f32,
    /// Left/right translation request.
    pub strafe: f32,
    /// Up/down translation request.
    pub lift: f32,
    /// Up/down pitch request.
    pub pitch: f32,
    /// Left/right roll request.
    pub roll: f32,
    /// Request immediate braking / retro-thrust.
    pub brake: bool,
    /// Request higher-thrust / boost mode.
    pub boost: bool,
    /// Request stabilization / assist damping.
    pub stabilize: bool,
    /// Request a dedicated main-engine channel in addition to axis input.
    pub main_engine: bool,
}

impl VehicleInputIntent {
    pub fn from_buttons(buttons: VehicleButtonInput) -> Self {
        buttons.intent().normalized()
    }

    pub fn from_motion(motion: VehicleMotionIntent) -> Self {
        let motion = motion.normalized();
        Self {
            throttle: motion.translation.forward,
            yaw: motion.rotation.yaw,
            strafe: motion.translation.lateral,
            lift: motion.translation.vertical,
            pitch: motion.rotation.pitch,
            roll: motion.rotation.roll,
            brake: motion.brake,
            boost: motion.boost,
            stabilize: motion.rotation.stabilize,
            main_engine: motion.translation.main_engine,
        }
    }

    pub fn motion(&self) -> VehicleMotionIntent {
        VehicleMotionIntent {
            translation: self.translation(),
            rotation: self.rotation(),
            brake: self.brake,
            boost: self.boost,
        }
    }

    pub fn translation(&self) -> VehicleTranslationIntent {
        VehicleTranslationIntent {
            forward: self.throttle,
            lateral: self.strafe,
            vertical: self.lift,
            main_engine: self.main_engine,
        }
        .normalized()
    }

    pub fn rotation(&self) -> VehicleRotationIntent {
        VehicleRotationIntent {
            yaw: self.yaw,
            pitch: self.pitch,
            roll: self.roll,
            stabilize: self.stabilize,
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.throttle = clamp_unit(self.throttle);
        self.yaw = clamp_unit(self.yaw);
        self.strafe = clamp_unit(self.strafe);
        self.lift = clamp_unit(self.lift);
        self.pitch = clamp_unit(self.pitch);
        self.roll = clamp_unit(self.roll);
    }

    pub fn has_translation(&self) -> bool {
        self.translation().has_input()
    }

    pub fn has_rotation(&self) -> bool {
        self.rotation().has_input()
    }

    pub fn is_idle(&self) -> bool {
        self.motion().is_idle()
    }

    pub fn control_state(
        &self,
        profile_id: &str,
        assists: VehicleAssistState,
    ) -> VehicleControlState {
        VehicleControlState::from_intent_with_profile(profile_id, *self, assists)
    }
}

/// Vehicle-facing control state that can be persisted or handed off between scenes.
///
/// This stays runtime-neutral: it carries the resolved control axes/flags plus
/// lightweight assist state and profile selection, without coupling the crate to
/// any engine-specific input device or behavior runner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleControlState {
    /// Stable control profile id such as `arcade` or `sim-lite`.
    pub profile_id: String,
    pub throttle: f32,
    pub yaw: f32,
    pub strafe: f32,
    pub lift: f32,
    pub pitch: f32,
    pub roll: f32,
    pub boost_scale: f32,
    pub brake_active: bool,
    pub stabilize_active: bool,
    pub main_engine_active: bool,
    pub assists: VehicleAssistState,
    /// Optional target altitude for altitude-hold controllers.
    pub target_altitude_km: Option<f32>,
    /// Optional target heading for heading-hold controllers.
    pub target_heading_rad: Option<f32>,
}

impl Default for VehicleControlState {
    fn default() -> Self {
        Self {
            profile_id: DEFAULT_VEHICLE_PROFILE_ID.to_string(),
            throttle: 0.0,
            yaw: 0.0,
            strafe: 0.0,
            lift: 0.0,
            pitch: 0.0,
            roll: 0.0,
            boost_scale: 1.0,
            brake_active: false,
            stabilize_active: false,
            main_engine_active: false,
            assists: VehicleAssistState::default(),
            target_altitude_km: None,
            target_heading_rad: None,
        }
    }
}

impl VehicleControlState {
    pub fn with_profile_id(profile_id: &str) -> Self {
        let mut state = Self::default();
        state.set_profile_id(profile_id);
        state
    }

    pub fn from_motion_intent_with_profile(
        profile_id: &str,
        motion: VehicleMotionIntent,
        assists: VehicleAssistState,
    ) -> Self {
        let motion = motion.normalized();
        Self {
            profile_id: normalize_vehicle_profile_id(profile_id),
            throttle: motion.translation.forward,
            yaw: motion.rotation.yaw,
            strafe: motion.translation.lateral,
            lift: motion.translation.vertical,
            pitch: motion.rotation.pitch,
            roll: motion.rotation.roll,
            boost_scale: if motion.boost { 2.0 } else { 1.0 },
            brake_active: motion.brake,
            stabilize_active: motion.rotation.stabilize,
            main_engine_active: motion.translation.main_engine,
            assists,
            target_altitude_km: None,
            target_heading_rad: None,
        }
    }

    pub fn from_intent_with_profile(
        profile_id: &str,
        intent: VehicleInputIntent,
        assists: VehicleAssistState,
    ) -> Self {
        Self::from_motion_intent_with_profile(profile_id, intent.motion(), assists)
    }

    pub fn from_button_input(
        profile_id: &str,
        buttons: VehicleButtonInput,
        assists: VehicleAssistState,
    ) -> Self {
        Self::from_intent_with_profile(profile_id, buttons.intent(), assists)
    }

    pub fn from_motion_intent(motion: VehicleMotionIntent, assists: VehicleAssistState) -> Self {
        Self::from_motion_intent_with_profile(DEFAULT_VEHICLE_PROFILE_ID, motion, assists)
    }

    pub fn from_intent(intent: VehicleInputIntent, assists: VehicleAssistState) -> Self {
        Self::from_intent_with_profile(DEFAULT_VEHICLE_PROFILE_ID, intent, assists)
    }

    pub fn motion(&self) -> VehicleMotionIntent {
        VehicleMotionIntent {
            translation: VehicleTranslationIntent {
                forward: self.throttle,
                lateral: self.strafe,
                vertical: self.lift,
                main_engine: self.main_engine_active,
            }
            .normalized(),
            rotation: VehicleRotationIntent {
                yaw: self.yaw,
                pitch: self.pitch,
                roll: self.roll,
                stabilize: self.stabilize_active,
            }
            .normalized(),
            brake: self.brake_active,
            boost: self.boost_scale > 1.0,
        }
    }

    pub fn set_motion(&mut self, motion: VehicleMotionIntent) {
        let motion = motion.normalized();
        self.throttle = motion.translation.forward;
        self.strafe = motion.translation.lateral;
        self.lift = motion.translation.vertical;
        self.main_engine_active = motion.translation.main_engine;
        self.yaw = motion.rotation.yaw;
        self.pitch = motion.rotation.pitch;
        self.roll = motion.rotation.roll;
        self.stabilize_active = motion.rotation.stabilize;
        self.brake_active = motion.brake;
        self.boost_scale = if motion.boost { 2.0 } else { 1.0 };
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.profile_id = normalize_vehicle_profile_id(&self.profile_id);
        self.throttle = clamp_unit(self.throttle);
        self.yaw = clamp_unit(self.yaw);
        self.strafe = clamp_unit(self.strafe);
        self.lift = clamp_unit(self.lift);
        self.pitch = clamp_unit(self.pitch);
        self.roll = clamp_unit(self.roll);
        self.boost_scale = if self.boost_scale.is_finite() {
            self.boost_scale.max(1.0)
        } else {
            1.0
        };
        self.target_altitude_km = if self.assists.alt_hold {
            self.target_altitude_km
                .and_then(|altitude| altitude.is_finite().then_some(altitude.max(0.0)))
        } else {
            None
        };
        self.target_heading_rad = if self.assists.heading_hold {
            self.target_heading_rad.map(normalize_angle_rad)
        } else {
            None
        };
    }

    pub fn intent(&self) -> VehicleInputIntent {
        VehicleInputIntent::from_motion(self.motion())
    }

    pub fn uses_assists(&self) -> bool {
        self.assists.any_enabled()
    }

    pub fn profile_matches(&self, profile_id: &str) -> bool {
        self.profile_id == normalize_vehicle_profile_id(profile_id)
    }

    pub fn ship_profile(&self) -> Option<VehicleShipProfile> {
        VehicleShipProfile::from_profile_id(&self.profile_id)
    }

    pub fn set_profile_id(&mut self, profile_id: &str) {
        self.profile_id = normalize_vehicle_profile_id(profile_id);
    }

    pub fn cycle_ship_profile(&mut self) -> &str {
        self.profile_id = next_builtin_ship_profile_id(&self.profile_id).to_string();
        self.profile_id.as_str()
    }

    pub fn set_assists(&mut self, assists: VehicleAssistState) {
        self.assists = assists;
        if !self.assists.alt_hold {
            self.target_altitude_km = None;
        }
        if !self.assists.heading_hold {
            self.target_heading_rad = None;
        }
    }

    pub fn set_altitude_hold(&mut self, enabled: bool, target_altitude_km: Option<f32>) {
        self.assists.alt_hold = enabled;
        self.target_altitude_km = if enabled {
            target_altitude_km
                .map(|altitude| finite_or_zero(altitude).max(0.0))
                .or(self.target_altitude_km)
        } else {
            None
        };
    }

    pub fn toggle_altitude_hold(&mut self, target_altitude_km: Option<f32>) -> bool {
        let enabled = !self.assists.alt_hold;
        self.set_altitude_hold(enabled, target_altitude_km);
        enabled
    }

    pub fn set_heading_hold(&mut self, enabled: bool, target_heading_rad: Option<f32>) {
        self.assists.heading_hold = enabled;
        self.target_heading_rad = if enabled {
            target_heading_rad
                .map(normalize_angle_rad)
                .or(self.target_heading_rad.map(normalize_angle_rad))
        } else {
            None
        };
    }

    pub fn toggle_heading_hold(&mut self, target_heading_rad: Option<f32>) -> bool {
        let enabled = !self.assists.heading_hold;
        self.set_heading_hold(enabled, target_heading_rad);
        enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_id_normalization_handles_known_aliases() {
        assert_eq!(normalize_vehicle_profile_id(""), "arcade");
        assert_eq!(normalize_vehicle_profile_id(" arc "), "arcade");
        assert_eq!(normalize_vehicle_profile_id("sim_lite"), "sim-lite");
        assert_eq!(normalize_vehicle_profile_id("sim lite"), "sim-lite");
        assert_eq!(
            normalize_vehicle_profile_id("custom-profile"),
            "custom-profile"
        );
    }

    #[test]
    fn ship_profile_helpers_cover_builtin_cycle() {
        assert_eq!(BUILTIN_SHIP_VEHICLE_PROFILE_IDS, ["arcade", "sim-lite"]);
        assert_eq!(
            VehicleShipProfile::from_profile_id(" arc "),
            Some(VehicleShipProfile::Arcade)
        );
        assert_eq!(
            VehicleShipProfile::from_profile_id("sim_lite"),
            Some(VehicleShipProfile::SimLite)
        );
        assert_eq!(VehicleShipProfile::from_profile_id("custom"), None);
        assert_eq!(next_builtin_ship_profile_id("arcade"), "sim-lite");
        assert_eq!(next_builtin_ship_profile_id("sim_lite"), "arcade");
        assert_eq!(next_builtin_ship_profile_id("custom"), "arcade");
        assert_eq!(VehicleShipProfile::ALL.len(), 2);
    }

    #[test]
    fn builtin_ship_profile_tuning_lives_in_engine_vehicle() {
        let arcade = builtin_ship_profile_tuning("arcade");
        let sim_lite = builtin_ship_profile_tuning("sim_lite");
        let custom = builtin_ship_profile_tuning("custom");

        assert!(arcade.forward_accel_g > sim_lite.forward_accel_g);
        assert!(arcade.main_engine_g > sim_lite.main_engine_g);
        assert!(arcade.yaw_max > sim_lite.yaw_max);
        assert_eq!(custom, VehicleShipProfileTuning::default());
        assert_eq!(VehicleShipProfile::Arcade.tuning(), arcade);
        assert_eq!(VehicleShipProfile::SimLite.tuning(), sim_lite);
        assert_eq!(arcade.grounded_speed_threshold_wu_s, 2.5);
        assert_eq!(arcade.surface_contact_altitude_threshold_km, 0.05);
        assert_eq!(arcade.surface_clearance_km, 0.35);
        assert_eq!(arcade.surface_clearance_min_wu, 0.05);
        assert_eq!(arcade.takeoff_lift_threshold, 0.001);
        assert!((arcade.forward_accel_wu_s2(10.0) - 11.0).abs() < 0.001);
        assert!((arcade.side_accel_wu_s2(10.0) - 7.2).abs() < 0.001);
        assert!((arcade.lift_accel_wu_s2(10.0) - 29.0).abs() < 0.001);
        assert!((arcade.main_engine_accel_wu_s2(10.0) - 19.5).abs() < 0.001);
        assert!((arcade.max_speed_wu_s(4.0) - 7.4).abs() < 0.001);
        assert!((arcade.max_radial_speed_wu_s(4.0) - 4.2).abs() < 0.001);
        assert!((arcade.surface_clearance_wu(50.0) - 0.05).abs() < 0.0001);
    }

    #[test]
    fn button_input_resolves_ship_axes_without_local_scalar_glue() {
        let buttons = VehicleButtonInput {
            forward: true,
            reverse: true,
            strafe_right: true,
            lift_up: true,
            yaw_left: true,
            pitch_down: true,
            roll_right: true,
            brake: true,
            boost: true,
            stabilize: true,
            main_engine: true,
            ..VehicleButtonInput::default()
        };

        let intent = VehicleInputIntent::from_buttons(buttons);
        assert_eq!(intent.throttle, 0.0);
        assert_eq!(intent.strafe, 1.0);
        assert_eq!(intent.lift, 1.0);
        assert_eq!(intent.yaw, 1.0);
        assert_eq!(intent.pitch, -1.0);
        assert_eq!(intent.roll, 1.0);
        assert!(intent.brake);
        assert!(intent.boost);
        assert!(intent.stabilize);
        assert!(intent.main_engine);
        assert!(!buttons.is_idle());

        let control = buttons.control_state(" sim_lite ", VehicleAssistState::default());
        assert_eq!(control.profile_id, "sim-lite");
        assert_eq!(control.motion().rotation.yaw, 1.0);
        assert!(control.main_engine_active);
    }

    #[test]
    fn intent_normalizes_axes_and_tracks_activity() {
        let intent = VehicleInputIntent {
            throttle: 4.0,
            yaw: -2.0,
            strafe: 0.5,
            lift: -3.0,
            pitch: f32::NAN,
            roll: f32::INFINITY,
            brake: false,
            boost: true,
            stabilize: false,
            main_engine: true,
        }
        .normalized();

        assert_eq!(intent.throttle, 1.0);
        assert_eq!(intent.yaw, -1.0);
        assert_eq!(intent.strafe, 0.5);
        assert_eq!(intent.lift, -1.0);
        assert_eq!(intent.pitch, 0.0);
        assert_eq!(intent.roll, 0.0);
        assert!(intent.has_translation());
        assert!(intent.has_rotation());
        assert!(!intent.is_idle());
        assert_eq!(intent.translation().forward, 1.0);
        assert_eq!(intent.translation().vertical, -1.0);
        assert_eq!(intent.rotation().yaw, -1.0);
    }

    #[test]
    fn control_state_carries_profile_assists_and_targets() {
        let state = VehicleControlState {
            profile_id: " sim_lite ".to_string(),
            boost_scale: 0.25,
            assists: VehicleAssistState::from_flags(true, true),
            target_altitude_km: Some(-2.0),
            target_heading_rad: Some(-0.25),
            ..VehicleControlState::from_intent(
                VehicleInputIntent {
                    throttle: 0.75,
                    boost: true,
                    brake: true,
                    stabilize: true,
                    ..VehicleInputIntent::default()
                },
                VehicleAssistState::from_flags(true, false),
            )
        }
        .normalized();

        assert_eq!(state.profile_id, "sim-lite");
        assert_eq!(state.boost_scale, 1.0);
        assert!(state.brake_active);
        assert!(state.stabilize_active);
        assert!(state.assists.alt_hold);
        assert!(state.assists.heading_hold);
        assert_eq!(state.target_altitude_km, Some(0.0));
        assert_eq!(
            state.target_heading_rad,
            Some((-0.25_f32).rem_euclid(std::f32::consts::TAU))
        );
        assert!(state.motion().rotation.stabilize);
    }

    #[test]
    fn control_state_roundtrips_through_serde_json() {
        let state = VehicleControlState {
            profile_id: "sim-lite".to_string(),
            throttle: 0.75,
            yaw: -0.25,
            main_engine_active: true,
            assists: VehicleAssistState::from_flags(true, true),
            target_altitude_km: Some(12.5),
            target_heading_rad: Some(1.5),
            ..VehicleControlState::default()
        }
        .normalized();

        let encoded = serde_json::to_string(&state).expect("encode VehicleControlState");
        let decoded: VehicleControlState =
            serde_json::from_str(&encoded).expect("decode VehicleControlState");

        assert_eq!(decoded, state);
        assert!(decoded.uses_assists());
        assert!(decoded.profile_matches("sim_lite"));
        assert_eq!(decoded.intent().yaw, -0.25);
    }

    #[test]
    fn control_state_profile_and_assist_helpers_preserve_targets() {
        let mut state = VehicleControlState::with_profile_id("custom-profile");
        assert_eq!(state.profile_id, "custom-profile");
        assert_eq!(state.ship_profile(), None);

        assert_eq!(state.cycle_ship_profile(), "arcade");
        assert_eq!(state.ship_profile(), Some(VehicleShipProfile::Arcade));

        assert!(state.toggle_altitude_hold(Some(-2.0)));
        assert!(state.assists.alt_hold);
        assert_eq!(state.target_altitude_km, Some(0.0));

        assert!(state.toggle_heading_hold(Some(-0.25)));
        assert!(state.assists.heading_hold);
        assert_eq!(
            state.target_heading_rad,
            Some((-0.25_f32).rem_euclid(std::f32::consts::TAU))
        );

        state.set_assists(VehicleAssistState::from_flags(true, false));
        assert_eq!(state.target_altitude_km, Some(0.0));
        assert_eq!(state.target_heading_rad, None);

        assert!(!state.toggle_altitude_hold(None));
        assert!(!state.assists.alt_hold);
        assert_eq!(state.target_altitude_km, None);
    }

    #[test]
    fn motion_intent_roundtrips_through_input_and_control_state() {
        let motion = VehicleMotionIntent {
            translation: VehicleTranslationIntent {
                forward: 2.0,
                lateral: -0.25,
                vertical: 0.5,
                main_engine: true,
            },
            rotation: VehicleRotationIntent {
                yaw: -2.0,
                pitch: 0.25,
                roll: 0.75,
                stabilize: true,
            },
            brake: true,
            boost: true,
        }
        .normalized();

        let intent = VehicleInputIntent::from_motion(motion);
        assert_eq!(intent.throttle, 1.0);
        assert_eq!(intent.yaw, -1.0);
        assert!(intent.main_engine);

        let mut control =
            VehicleControlState::from_motion_intent(motion, VehicleAssistState::default());
        control.set_motion(control.motion());

        assert_eq!(control.intent(), intent);
        assert!(control.brake_active);
        assert_eq!(control.boost_scale, 2.0);
    }
}
