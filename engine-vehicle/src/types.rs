use serde::{Deserialize, Serialize};

/// Phase of the auto-brake sequence exposed through vehicle telemetry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrakePhase {
    #[default]
    Idle,
    Rotation,
    Linear,
    Stopped,
    Thrusting,
}

impl BrakePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrakePhase::Idle => "idle",
            BrakePhase::Rotation => "rotation",
            BrakePhase::Linear => "linear",
            BrakePhase::Stopped => "stopped",
            BrakePhase::Thrusting => "thrusting",
        }
    }
}

/// Pure input snapshot used to derive a [`VehicleProfile`] from lower-level runtime state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleProfileInput {
    pub heading_bits: Option<u8>,
    pub turn_step_ms: Option<u32>,
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
}

/// Normalized vehicle profile assembled from generic runtime motion components.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VehicleProfile {
    /// Stable logical profile id such as `arcade` or `sim-lite`.
    pub profile_id: String,
    /// Optional debug/editor label for UI and diagnostics.
    pub label: Option<String>,
    /// Discrete heading quantization count when using arcade turn steps.
    pub heading_bits: Option<u8>,
    /// Milliseconds between discrete heading steps.
    pub turn_step_ms: Option<u32>,
    /// Forward thrust acceleration magnitude.
    pub thrust_power: f32,
    /// Configured maximum linear speed. `0.0` means unclamped.
    pub max_speed: f32,
    /// Angular acceleration in rad/s².
    pub angular_accel: f32,
    /// Maximum angular velocity in rad/s.
    pub angular_max: f32,
    /// Angular deadband for settling to rest.
    pub angular_deadband: f32,
    /// Whether angular auto-brake is enabled.
    pub angular_auto_brake: bool,
    /// Linear deceleration magnitude in px/s².
    pub linear_brake_decel: f32,
    /// Linear speed deadband for settling to rest.
    pub linear_brake_deadband: f32,
    /// Whether linear auto-brake is enabled.
    pub linear_auto_brake: bool,
    /// True when the vehicle also uses thruster timing/factor outputs.
    pub thruster_ramp_enabled: bool,
}

impl VehicleProfile {
    /// Build a profile snapshot from normalized runtime motion inputs.
    pub fn from_runtime(input: VehicleProfileInput) -> Self {
        let mut profile = Self::default();
        profile.sync_from_runtime(input);
        profile
    }

    /// Overwrite runtime-derived fields while preserving higher-level metadata
    /// such as `profile_id` and `label`.
    pub fn sync_from_runtime(&mut self, input: VehicleProfileInput) {
        self.heading_bits = input.heading_bits.map(|bits| bits.max(1));
        self.turn_step_ms = input.turn_step_ms.map(|step| step.max(1));
        self.thrust_power = input.thrust_power;
        self.max_speed = input.max_speed;
        self.angular_accel = input.angular_accel;
        self.angular_max = input.angular_max;
        self.angular_deadband = input.angular_deadband;
        self.angular_auto_brake = input.angular_auto_brake;
        self.linear_brake_decel = input.linear_brake_decel;
        self.linear_brake_deadband = input.linear_brake_deadband;
        self.linear_auto_brake = input.linear_auto_brake;
        self.thruster_ramp_enabled = input.thruster_ramp_enabled;
    }
}

/// Vehicle-facing basis derived from the runtime heading convention.
///
/// Heading `0` points "up" (`0, -1`) to match the existing controller thrust
/// convention used elsewhere in the engine.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VehicleFacing {
    pub heading: f32,
    pub forward_x: f32,
    pub forward_y: f32,
    pub right_x: f32,
    pub right_y: f32,
}

impl VehicleFacing {
    /// Build a vehicle-local basis from a heading in radians.
    pub fn from_heading(heading: f32) -> Self {
        Self {
            heading,
            forward_x: heading.sin(),
            forward_y: -heading.cos(),
            right_x: heading.cos(),
            right_y: heading.sin(),
        }
    }
}

impl Default for VehicleFacing {
    fn default() -> Self {
        Self::from_heading(0.0)
    }
}

/// Pure world-space motion input used to derive a [`MotionFrame`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionFrameInput {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub accel_x: f32,
    pub accel_y: f32,
}

/// Velocity/acceleration resolved into vehicle-local axes for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MotionFrame {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub speed: f32,
    pub forward_speed: f32,
    pub lateral_speed: f32,
    pub forward_accel: f32,
    pub lateral_accel: f32,
}

impl MotionFrame {
    /// Decompose world-space motion into forward/lateral vehicle space.
    pub fn from_input(input: MotionFrameInput, facing: VehicleFacing) -> Self {
        let speed =
            (input.velocity_x * input.velocity_x + input.velocity_y * input.velocity_y).sqrt();
        Self {
            velocity_x: input.velocity_x,
            velocity_y: input.velocity_y,
            accel_x: input.accel_x,
            accel_y: input.accel_y,
            speed,
            forward_speed: input.velocity_x * facing.forward_x
                + input.velocity_y * facing.forward_y,
            lateral_speed: input.velocity_x * facing.right_x + input.velocity_y * facing.right_y,
            forward_accel: input.accel_x * facing.forward_x + input.accel_y * facing.forward_y,
            lateral_accel: input.accel_x * facing.right_x + input.accel_y * facing.right_y,
        }
    }
}

/// Pure runtime inputs used to derive a [`VehicleTelemetry`] snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleTelemetryInput {
    pub heading: f32,
    pub motion: Option<MotionFrameInput>,
    pub turn_input: f32,
    pub thrust_input: f32,
    pub is_thrusting: bool,
    pub angular_vel: f32,
    pub angular_deadband: f32,
    pub linear_deadband: f32,
    pub angular_settling: bool,
    pub linear_settling: bool,
    pub thrust_factor: Option<f32>,
    pub rot_factor: Option<f32>,
    pub brake_factor: Option<f32>,
    pub brake_phase: Option<BrakePhase>,
    pub final_burst_fired: bool,
    pub final_burst_wave: u8,
}

/// Normalized runtime telemetry for vehicle-oriented gameplay/UI systems.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VehicleTelemetry {
    pub facing: VehicleFacing,
    pub motion: MotionFrame,
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
}

impl VehicleTelemetry {
    /// Build one telemetry snapshot from normalized runtime inputs.
    pub fn from_runtime(input: VehicleTelemetryInput) -> Self {
        let facing = VehicleFacing::from_heading(input.heading);
        let motion = input
            .motion
            .map(|motion| MotionFrame::from_input(motion, facing))
            .unwrap_or_default();

        let (
            thrust_factor,
            rot_factor,
            brake_factor,
            brake_phase,
            final_burst_fired,
            final_burst_wave,
        ) = if let Some(brake_phase) = input.brake_phase.clone() {
            (
                input.thrust_factor.unwrap_or(input.thrust_input),
                input.rot_factor.unwrap_or(0.0),
                input.brake_factor.unwrap_or(0.0),
                brake_phase,
                input.final_burst_fired,
                input.final_burst_wave,
            )
        } else {
            let derived_rot_factor = input.rot_factor.unwrap_or_else(|| {
                if input.angular_vel.abs() > 0.0 {
                    1.0
                } else {
                    0.0
                }
            });
            let derived_brake_factor = input.brake_factor.unwrap_or_else(|| {
                if input.angular_settling || input.linear_settling {
                    1.0
                } else {
                    0.0
                }
            });
            let derived_phase = if input.is_thrusting {
                BrakePhase::Thrusting
            } else if input.angular_settling {
                BrakePhase::Rotation
            } else if input.linear_settling {
                BrakePhase::Linear
            } else if motion.speed <= input.linear_deadband
                && input.angular_vel.abs() <= input.angular_deadband
            {
                BrakePhase::Stopped
            } else {
                BrakePhase::Idle
            };
            (
                input.thrust_input,
                derived_rot_factor,
                derived_brake_factor,
                derived_phase,
                false,
                0,
            )
        };

        Self {
            facing,
            motion,
            turn_input: input.turn_input.clamp(-1.0, 1.0),
            thrust_input: input.thrust_input,
            is_thrusting: input.is_thrusting,
            is_braking: matches!(brake_phase, BrakePhase::Rotation | BrakePhase::Linear),
            angular_vel: input.angular_vel,
            thrust_factor,
            rot_factor,
            brake_factor,
            brake_phase,
            final_burst_fired,
            final_burst_wave,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_profile_sync_preserves_metadata() {
        let mut profile = VehicleProfile {
            profile_id: "arcade".to_string(),
            label: Some("Arcade".to_string()),
            ..VehicleProfile::default()
        };

        profile.sync_from_runtime(VehicleProfileInput {
            heading_bits: Some(32),
            turn_step_ms: Some(60),
            thrust_power: 8.0,
            max_speed: 20.0,
            angular_accel: 5.0,
            angular_max: 6.0,
            angular_deadband: 0.1,
            angular_auto_brake: true,
            linear_brake_decel: 12.0,
            linear_brake_deadband: 0.5,
            linear_auto_brake: true,
            thruster_ramp_enabled: true,
        });

        assert_eq!(profile.profile_id, "arcade");
        assert_eq!(profile.label.as_deref(), Some("Arcade"));
        assert_eq!(profile.heading_bits, Some(32));
        assert!(profile.thruster_ramp_enabled);
    }

    #[test]
    fn motion_frame_decomposes_world_motion() {
        let facing = VehicleFacing::from_heading(0.0);
        let motion = MotionFrame::from_input(
            MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -10.0,
                accel_x: 0.0,
                accel_y: -2.0,
            },
            facing,
        );

        assert_eq!(motion.speed, 10.0);
        assert!((motion.forward_speed - 10.0).abs() < 0.001);
        assert!(motion.lateral_speed.abs() < 0.001);
    }

    #[test]
    fn vehicle_telemetry_derives_runtime_snapshot() {
        let telemetry = VehicleTelemetry::from_runtime(VehicleTelemetryInput {
            heading: std::f32::consts::FRAC_PI_2,
            motion: Some(MotionFrameInput {
                velocity_x: 3.0,
                velocity_y: 4.0,
                accel_x: 1.0,
                accel_y: 2.0,
            }),
            turn_input: -2.0,
            thrust_input: 1.0,
            is_thrusting: true,
            angular_vel: 1.5,
            angular_deadband: 0.1,
            linear_deadband: 0.5,
            angular_settling: false,
            linear_settling: false,
            thrust_factor: Some(0.8),
            rot_factor: Some(0.25),
            brake_factor: Some(0.6),
            brake_phase: Some(BrakePhase::Linear),
            final_burst_fired: true,
            final_burst_wave: 2,
        });

        assert!((telemetry.facing.forward_x - 1.0).abs() < 0.001);
        assert!(telemetry.facing.forward_y.abs() < 0.001);
        assert!((telemetry.motion.speed - 5.0).abs() < 0.001);
        assert_eq!(telemetry.turn_input, -1.0);
        assert_eq!(telemetry.thrust_factor, 0.8);
        assert_eq!(telemetry.rot_factor, 0.25);
        assert_eq!(telemetry.brake_factor, 0.6);
        assert_eq!(telemetry.brake_phase, BrakePhase::Linear);
        assert!(telemetry.is_braking);
        assert!(telemetry.final_burst_fired);
        assert_eq!(telemetry.final_burst_wave, 2);
    }
}
