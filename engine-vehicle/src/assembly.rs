use rhai::{Dynamic, ImmutableString, Map as RhaiMap};
use serde::{Deserialize, Serialize};

use crate::{normalize_vehicle_profile_id, VehicleKind, VehicleProfile, VehicleProfileInput};

/// Lightweight metadata attached to an assembly plan before a full vehicle
/// descriptor/model gets resolved.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleAssemblyDescriptor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<VehicleKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl VehicleAssemblyDescriptor {
    pub fn from_rhai_map(config: &RhaiMap) -> Result<Self, VehicleAssemblyError> {
        let mut descriptor = Self {
            kind: rhai_map_get_first_present_string(config, &["kind", "vehicle_kind"])?
                .and_then(|kind| VehicleKind::from_hint(&kind)),
            profile_id: rhai_map_get_first_present_string(config, &["profile", "profile_id"])?,
            label: rhai_map_get_first_present_string(config, &["label", "name"])?,
        };
        descriptor.normalize();
        Ok(descriptor)
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.profile_id = self
            .profile_id
            .take()
            .and_then(|profile_id| trim_to_option(&profile_id))
            .map(|profile_id| normalize_vehicle_profile_id(&profile_id));
        self.label = self.label.take().and_then(|label| trim_to_option(&label));

        if self.kind.is_none()
            && self
                .profile_id
                .as_deref()
                .is_some_and(|profile_id| !profile_id.is_empty())
        {
            self.kind = Some(VehicleKind::Ship);
        }
    }

    pub fn apply_to_profile(&self, profile: &mut VehicleProfile) {
        if let Some(profile_id) = self.profile_id.as_ref() {
            profile.profile_id = profile_id.clone();
        }
        if let Some(label) = self.label.as_ref() {
            profile.label = Some(label.clone());
        }
    }
}

/// Parsed, normalized vehicle assembly plus optional higher-level descriptor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleAssemblyPlan {
    pub descriptor: VehicleAssemblyDescriptor,
    pub assembly: VehicleAssembly,
    pub context: VehicleAssemblyContext,
}

impl VehicleAssemblyPlan {
    pub fn from_rhai_map(config: &RhaiMap) -> Result<Self, VehicleAssemblyError> {
        let mut plan = Self {
            descriptor: VehicleAssemblyDescriptor::from_rhai_map(config)?,
            assembly: VehicleAssembly::from_rhai_map(config)?,
            context: VehicleAssemblyContext::default(),
        };
        plan.normalize();
        Ok(plan)
    }

    pub fn from_rhai_map_with_context(
        config: &RhaiMap,
        ctx: VehicleAssemblyContext,
    ) -> Result<Self, VehicleAssemblyError> {
        Self::from_rhai_map(config).map(|plan| plan.with_context(ctx))
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.descriptor.normalize();
        self.assembly.normalize();
        self.context.normalize();

        if self.descriptor.profile_id.is_none() && self.assembly.arcade.is_some() {
            self.descriptor.kind = Some(VehicleKind::Ship);
            self.descriptor.profile_id = Some(normalize_vehicle_profile_id("arcade"));
        } else if self.descriptor.kind.is_none()
            && self
                .descriptor
                .profile_id
                .as_deref()
                .is_some_and(|profile_id| !profile_id.is_empty())
        {
            self.descriptor.kind = Some(VehicleKind::Ship);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.assembly.is_empty()
    }

    pub fn with_context(mut self, ctx: VehicleAssemblyContext) -> Self {
        self.context = ctx.normalized();
        self
    }

    pub fn resolved_assembly(&self) -> VehicleAssembly {
        self.assembly.clone().with_context(self.context)
    }

    pub fn to_profile_input(&self) -> Option<VehicleProfileInput> {
        self.resolved_assembly().to_profile_input()
    }

    pub fn to_profile(&self) -> Option<VehicleProfile> {
        let mut profile = VehicleProfile::from_runtime(self.to_profile_input()?);
        self.descriptor.apply_to_profile(&mut profile);
        Some(profile)
    }

    pub fn apply<S: VehicleAssemblySink>(&self, sink: &mut S) -> Result<bool, S::Error> {
        self.resolved_assembly().apply(sink)
    }
}

/// Typed vehicle stack assembly assembled from script or authoring input.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleAssembly {
    pub arcade: Option<ArcadeConfig>,
    pub angular_body: Option<AngularBodyConfig>,
    pub linear_brake: Option<LinearBrakeConfig>,
    pub thruster_ramp: Option<ThrusterRampConfig>,
}

impl VehicleAssembly {
    /// Parse the same flat-or-nested Rhai shape currently accepted by
    /// `attach_vehicle_stack` in `engine-behavior`.
    pub fn from_rhai_map(config: &RhaiMap) -> Result<Self, VehicleAssemblyError> {
        let has_submaps = ["arcade", "angular_body", "linear_brake", "thruster_ramp"]
            .into_iter()
            .any(|key| config.contains_key(key));

        let arcade = match rhai_map_get_present_map(config, "arcade")? {
            Some(map) => Some(ArcadeConfig::from_required_map(&map)?),
            None if !has_submaps => Some(ArcadeConfig::from_required_map(config)?),
            None => None,
        };

        let angular_body = rhai_map_get_present_map(config, "angular_body")?
            .map(|map| AngularBodyConfig::from_map(&map));
        let linear_brake = rhai_map_get_present_map(config, "linear_brake")?
            .map(|map| LinearBrakeConfig::from_map(&map));
        let thruster_ramp = rhai_map_get_present_map(config, "thruster_ramp")?
            .map(|map| ThrusterRampConfig::from_map(&map));

        let assembly = Self {
            arcade,
            angular_body,
            linear_brake,
            thruster_ramp,
        }
        .normalized();

        if assembly.is_empty() {
            Err(VehicleAssemblyError::EmptyAssembly)
        } else {
            Ok(assembly)
        }
    }

    /// Parse and immediately layer external runtime context onto the plan.
    pub fn from_rhai_map_with_context(
        config: &RhaiMap,
        ctx: VehicleAssemblyContext,
    ) -> Result<Self, VehicleAssemblyError> {
        Self::from_rhai_map(config).map(|assembly| assembly.with_context(ctx))
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.arcade = self.arcade.take().map(|arcade| arcade.normalized());
        self.angular_body = self
            .angular_body
            .take()
            .map(|angular_body| angular_body.normalized());
        self.linear_brake = self
            .linear_brake
            .take()
            .map(|linear_brake| linear_brake.normalized());
        self.thruster_ramp = self
            .thruster_ramp
            .take()
            .map(|thruster_ramp| thruster_ramp.normalized());
    }

    pub fn is_empty(&self) -> bool {
        self.arcade.is_none()
            && self.angular_body.is_none()
            && self.linear_brake.is_none()
            && self.thruster_ramp.is_none()
    }

    /// Apply external heading context before attaching or converting.
    pub fn with_context(mut self, ctx: VehicleAssemblyContext) -> Self {
        let ctx = ctx.normalized();
        if let Some(heading) = ctx.heading {
            if let Some(arcade) = self.arcade.as_mut() {
                arcade.initial_heading = Some(heading);
            }
        }
        self.normalize();
        self
    }

    /// Convert this assembly into a vehicle profile input snapshot.
    pub fn to_profile_input(&self) -> Option<VehicleProfileInput> {
        let assembly = self.clone().normalized();
        if assembly.is_empty() {
            return None;
        }

        Some(VehicleProfileInput {
            heading_bits: assembly.arcade.as_ref().map(|cfg| cfg.heading_bits),
            turn_step_ms: assembly.arcade.as_ref().map(|cfg| cfg.turn_step_ms),
            thrust_power: assembly
                .arcade
                .as_ref()
                .map(|cfg| cfg.thrust_power)
                .unwrap_or(0.0),
            max_speed: assembly
                .arcade
                .as_ref()
                .map(|cfg| cfg.max_speed)
                .unwrap_or(0.0),
            angular_accel: assembly
                .angular_body
                .as_ref()
                .map(|cfg| cfg.accel)
                .unwrap_or(0.0),
            angular_max: assembly
                .angular_body
                .as_ref()
                .map(|cfg| cfg.max)
                .unwrap_or(0.0),
            angular_deadband: assembly
                .angular_body
                .as_ref()
                .map(|cfg| cfg.deadband)
                .unwrap_or(0.0),
            angular_auto_brake: assembly
                .angular_body
                .as_ref()
                .map(|cfg| cfg.auto_brake)
                .unwrap_or(false),
            linear_brake_decel: assembly
                .linear_brake
                .as_ref()
                .map(|cfg| cfg.decel)
                .unwrap_or(0.0),
            linear_brake_deadband: assembly
                .linear_brake
                .as_ref()
                .map(|cfg| cfg.deadband)
                .unwrap_or(0.0),
            linear_auto_brake: assembly
                .linear_brake
                .as_ref()
                .map(|cfg| cfg.auto_brake)
                .unwrap_or(false),
            thruster_ramp_enabled: assembly.thruster_ramp.is_some(),
        })
    }

    /// Neutral apply helper. Callers can implement [`VehicleAssemblySink`]
    /// against their own world/store without coupling this crate to `engine-game`.
    pub fn apply<S: VehicleAssemblySink>(&self, sink: &mut S) -> Result<bool, S::Error> {
        let assembly = self.clone().normalized();
        let mut attached_any = false;

        if let Some(arcade) = assembly.arcade {
            sink.attach_arcade(arcade)?;
            attached_any = true;
        }
        if let Some(angular_body) = assembly.angular_body {
            sink.attach_angular_body(angular_body)?;
            attached_any = true;
        }
        if let Some(linear_brake) = assembly.linear_brake {
            sink.attach_linear_brake(linear_brake)?;
            attached_any = true;
        }
        if let Some(thruster_ramp) = assembly.thruster_ramp {
            sink.attach_thruster_ramp(thruster_ramp)?;
            attached_any = true;
        }

        Ok(attached_any)
    }
}

/// External runtime context that can be layered onto a parsed assembly plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleAssemblyContext {
    pub heading: Option<f32>,
}

impl VehicleAssemblyContext {
    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.heading = self.heading.and_then(finite_option);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArcadeConfig {
    pub turn_step_ms: u32,
    pub thrust_power: f32,
    pub max_speed: f32,
    pub heading_bits: u8,
    pub initial_heading: Option<f32>,
}

impl ArcadeConfig {
    pub fn from_required_map(config: &RhaiMap) -> Result<Self, VehicleAssemblyError> {
        Ok(Self {
            turn_step_ms: rhai_map_get_u32_required(config, "turn_step_ms")?,
            thrust_power: rhai_map_get_f32_required(config, "thrust_power")?,
            max_speed: rhai_map_get_f32_required(config, "max_speed")?,
            heading_bits: rhai_map_get_u8_required(config, "heading_bits")?,
            initial_heading: rhai_map_get_f32_optional(config, "initial_heading"),
        }
        .normalized())
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.turn_step_ms = self.turn_step_ms.max(1);
        self.thrust_power = non_negative(self.thrust_power);
        self.max_speed = non_negative(self.max_speed);
        self.heading_bits = self.heading_bits.max(1);
        self.initial_heading = self.initial_heading.and_then(finite_option);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AngularBodyConfig {
    pub accel: f32,
    pub max: f32,
    pub deadband: f32,
    pub auto_brake: bool,
    pub angular_vel: f32,
}

impl Default for AngularBodyConfig {
    fn default() -> Self {
        Self {
            accel: 5.5,
            max: 7.0,
            deadband: 0.10,
            auto_brake: true,
            angular_vel: 0.0,
        }
    }
}

impl AngularBodyConfig {
    pub fn from_map(config: &RhaiMap) -> Self {
        Self {
            accel: rhai_map_get_f32(config, "accel", 5.5),
            max: rhai_map_get_f32(config, "max", 7.0),
            deadband: rhai_map_get_f32(config, "deadband", 0.10),
            auto_brake: rhai_map_get_bool(config, "auto_brake", true),
            angular_vel: rhai_map_get_f32(config, "angular_vel", 0.0),
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.accel = non_negative(self.accel);
        self.max = non_negative(self.max);
        self.deadband = non_negative(self.deadband);
        self.angular_vel = finite_or_zero(self.angular_vel);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearBrakeConfig {
    pub decel: f32,
    pub deadband: f32,
    pub auto_brake: bool,
    pub active: bool,
}

impl Default for LinearBrakeConfig {
    fn default() -> Self {
        Self {
            decel: 45.0,
            deadband: 2.5,
            auto_brake: true,
            active: false,
        }
    }
}

impl LinearBrakeConfig {
    pub fn from_map(config: &RhaiMap) -> Self {
        Self {
            decel: rhai_map_get_f32(config, "decel", 45.0),
            deadband: rhai_map_get_f32(config, "deadband", 2.5),
            auto_brake: rhai_map_get_bool(config, "auto_brake", true),
            active: false,
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.decel = non_negative(self.decel);
        self.deadband = non_negative(self.deadband);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThrusterRampConfig {
    pub thrust_delay_ms: f32,
    pub thrust_ramp_ms: f32,
    pub no_input_threshold_ms: f32,
    pub rot_factor_max_vel: f32,
    pub burst_speed_threshold: f32,
    pub burst_wave_interval_ms: f32,
    pub burst_wave_count: u8,
    pub rot_deadband: f32,
    pub move_deadband: f32,
}

impl Default for ThrusterRampConfig {
    fn default() -> Self {
        Self {
            thrust_delay_ms: 8.0,
            thrust_ramp_ms: 12.0,
            no_input_threshold_ms: 30.0,
            rot_factor_max_vel: 7.0,
            burst_speed_threshold: 15.0,
            burst_wave_interval_ms: 150.0,
            burst_wave_count: 3,
            rot_deadband: 0.10,
            move_deadband: 2.5,
        }
    }
}

impl ThrusterRampConfig {
    pub fn from_map(config: &RhaiMap) -> Self {
        Self {
            thrust_delay_ms: rhai_map_get_f32(config, "thrust_delay_ms", 8.0),
            thrust_ramp_ms: rhai_map_get_f32(config, "thrust_ramp_ms", 12.0),
            no_input_threshold_ms: rhai_map_get_f32(config, "no_input_threshold_ms", 30.0),
            rot_factor_max_vel: rhai_map_get_f32(config, "rot_factor_max_vel", 7.0),
            burst_speed_threshold: rhai_map_get_f32(config, "burst_speed_threshold", 15.0),
            burst_wave_interval_ms: rhai_map_get_f32(config, "burst_wave_interval_ms", 150.0),
            burst_wave_count: rhai_map_get_u8(config, "burst_wave_count", 3),
            rot_deadband: rhai_map_get_f32(config, "rot_deadband", 0.10),
            move_deadband: rhai_map_get_f32(config, "move_deadband", 2.5),
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.thrust_delay_ms = non_negative(self.thrust_delay_ms);
        self.thrust_ramp_ms = non_negative(self.thrust_ramp_ms);
        self.no_input_threshold_ms = non_negative(self.no_input_threshold_ms);
        self.rot_factor_max_vel = non_negative(self.rot_factor_max_vel);
        self.burst_speed_threshold = non_negative(self.burst_speed_threshold);
        self.burst_wave_interval_ms = non_negative(self.burst_wave_interval_ms);
        self.rot_deadband = non_negative(self.rot_deadband);
        self.move_deadband = non_negative(self.move_deadband);
    }
}

/// Neutral sink for attaching a parsed assembly plan to a runtime/store.
pub trait VehicleAssemblySink {
    type Error;

    fn attach_arcade(&mut self, arcade: ArcadeConfig) -> Result<(), Self::Error>;
    fn attach_angular_body(&mut self, angular_body: AngularBodyConfig) -> Result<(), Self::Error>;
    fn attach_linear_brake(&mut self, linear_brake: LinearBrakeConfig) -> Result<(), Self::Error>;
    fn attach_thruster_ramp(
        &mut self,
        thruster_ramp: ThrusterRampConfig,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VehicleAssemblyError {
    EmptyAssembly,
    InvalidSubmap(&'static str),
    InvalidTextField(&'static str),
    MissingArcadeField(&'static str),
}

fn rhai_map_get_present_map(
    map: &RhaiMap,
    key: &'static str,
) -> Result<Option<RhaiMap>, VehicleAssemblyError> {
    let Some(value) = map.get(key) else {
        return Ok(None);
    };
    if value.is_unit() {
        return Ok(None);
    }
    value
        .clone()
        .try_cast::<RhaiMap>()
        .map(Some)
        .ok_or(VehicleAssemblyError::InvalidSubmap(key))
}

fn rhai_map_get_first_present_string(
    map: &RhaiMap,
    keys: &[&'static str],
) -> Result<Option<String>, VehicleAssemblyError> {
    for key in keys {
        if let Some(value) = rhai_map_get_present_string(map, key)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn rhai_map_get_present_string(
    map: &RhaiMap,
    key: &'static str,
) -> Result<Option<String>, VehicleAssemblyError> {
    let Some(value) = map.get(key) else {
        return Ok(None);
    };
    if value.is_unit() {
        return Ok(None);
    }
    value
        .clone()
        .try_cast::<String>()
        .or_else(|| {
            value
                .clone()
                .try_cast::<ImmutableString>()
                .map(|value| value.to_string())
        })
        .map(Some)
        .ok_or(VehicleAssemblyError::InvalidTextField(key))
}

fn rhai_map_get_f32(map: &RhaiMap, key: &str, default: f32) -> f32 {
    rhai_map_get_f32_optional(map, key).unwrap_or(default)
}

fn rhai_map_get_f32_optional(map: &RhaiMap, key: &str) -> Option<f32> {
    map.get(key).and_then(dynamic_to_f32)
}

fn rhai_map_get_f32_required(
    map: &RhaiMap,
    key: &'static str,
) -> Result<f32, VehicleAssemblyError> {
    rhai_map_get_f32_optional(map, key).ok_or(VehicleAssemblyError::MissingArcadeField(key))
}

fn rhai_map_get_bool(map: &RhaiMap, key: &str, default: bool) -> bool {
    map.get(key)
        .and_then(|value| value.as_bool().ok())
        .unwrap_or(default)
}

fn rhai_map_get_u8(map: &RhaiMap, key: &str, default: u8) -> u8 {
    map.get(key)
        .and_then(|value| value.as_int().ok())
        .map(|value| value as u8)
        .unwrap_or(default)
}

fn rhai_map_get_u8_required(map: &RhaiMap, key: &'static str) -> Result<u8, VehicleAssemblyError> {
    map.get(key)
        .and_then(|value| value.as_int().ok())
        .map(|value| value as u8)
        .ok_or(VehicleAssemblyError::MissingArcadeField(key))
}

fn rhai_map_get_u32_required(
    map: &RhaiMap,
    key: &'static str,
) -> Result<u32, VehicleAssemblyError> {
    map.get(key)
        .and_then(|value| value.as_int().ok())
        .map(|value| value as u32)
        .ok_or(VehicleAssemblyError::MissingArcadeField(key))
}

fn dynamic_to_f32(value: &Dynamic) -> Option<f32> {
    value
        .as_float()
        .ok()
        .map(|value| value as f32)
        .or_else(|| value.as_int().ok().map(|value| value as f32))
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn non_negative(value: f32) -> f32 {
    finite_or_zero(value).max(0.0)
}

fn finite_option(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

fn trim_to_option(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;

    #[test]
    fn parses_flat_arcade_vehicle_assembly() {
        let mut map = RhaiMap::new();
        map.insert("turn_step_ms".into(), 90_i64.into());
        map.insert("thrust_power".into(), 12.5_f64.into());
        map.insert("max_speed".into(), 18.0_f64.into());
        map.insert("heading_bits".into(), 16_i64.into());

        let assembly = VehicleAssembly::from_rhai_map(&map).expect("flat assembly");
        assert_eq!(
            assembly.arcade,
            Some(ArcadeConfig {
                turn_step_ms: 90,
                thrust_power: 12.5,
                max_speed: 18.0,
                heading_bits: 16,
                initial_heading: None,
            })
        );
        assert!(assembly.angular_body.is_none());
    }

    #[test]
    fn parses_nested_vehicle_assembly_with_defaults() {
        let mut root = RhaiMap::new();

        let mut angular = RhaiMap::new();
        angular.insert("accel".into(), 2.5_f64.into());
        angular.insert("auto_brake".into(), false.into());
        root.insert("angular_body".into(), angular.into());

        let mut ramp = RhaiMap::new();
        ramp.insert("burst_wave_count".into(), 5_i64.into());
        root.insert("thruster_ramp".into(), ramp.into());

        let assembly = VehicleAssembly::from_rhai_map(&root).expect("nested assembly");
        assert!(assembly.arcade.is_none());
        assert_eq!(
            assembly.angular_body,
            Some(AngularBodyConfig {
                accel: 2.5,
                auto_brake: false,
                ..AngularBodyConfig::default()
            })
        );
        assert_eq!(
            assembly.thruster_ramp,
            Some(ThrusterRampConfig {
                burst_wave_count: 5,
                ..ThrusterRampConfig::default()
            })
        );
    }

    #[test]
    fn rejects_invalid_submap_shape() {
        let mut root = RhaiMap::new();
        root.insert("angular_body".into(), 1_i64.into());

        let err = VehicleAssembly::from_rhai_map(&root).expect_err("invalid submap");
        assert_eq!(err, VehicleAssemblyError::InvalidSubmap("angular_body"));
    }

    #[test]
    fn rejects_invalid_descriptor_text_shape() {
        let mut root = RhaiMap::new();
        root.insert("kind".into(), 1_i64.into());

        let mut angular = RhaiMap::new();
        angular.insert("accel".into(), 2.0_f64.into());
        root.insert("angular_body".into(), angular.into());

        let err = VehicleAssemblyPlan::from_rhai_map(&root).expect_err("invalid descriptor");
        assert_eq!(err, VehicleAssemblyError::InvalidTextField("kind"));
    }

    #[test]
    fn applies_context_and_builds_profile_input() {
        let assembly = VehicleAssembly {
            arcade: Some(ArcadeConfig {
                turn_step_ms: 60,
                thrust_power: 8.0,
                max_speed: 20.0,
                heading_bits: 32,
                initial_heading: None,
            }),
            angular_body: Some(AngularBodyConfig::default()),
            linear_brake: Some(LinearBrakeConfig::default()),
            thruster_ramp: Some(ThrusterRampConfig::default()),
        }
        .with_context(VehicleAssemblyContext {
            heading: Some(std::f32::consts::FRAC_PI_2),
        });

        assert_eq!(
            assembly.arcade.as_ref().and_then(|cfg| cfg.initial_heading),
            Some(std::f32::consts::FRAC_PI_2)
        );

        let profile = assembly.to_profile_input().expect("profile input");
        assert_eq!(profile.heading_bits, Some(32));
        assert_eq!(profile.turn_step_ms, Some(60));
        assert_eq!(profile.thrust_power, 8.0);
        assert!(profile.thruster_ramp_enabled);
    }

    #[test]
    fn parses_with_context_in_one_step() {
        let mut map = RhaiMap::new();
        map.insert("turn_step_ms".into(), 60_i64.into());
        map.insert("thrust_power".into(), 8.0_f64.into());
        map.insert("max_speed".into(), 20.0_f64.into());
        map.insert("heading_bits".into(), 32_i64.into());

        let assembly = VehicleAssembly::from_rhai_map_with_context(
            &map,
            VehicleAssemblyContext {
                heading: Some(1.25),
            },
        )
        .expect("assembly");

        assert_eq!(
            assembly.arcade.as_ref().and_then(|cfg| cfg.initial_heading),
            Some(1.25)
        );
    }

    #[test]
    fn context_none_preserves_authored_heading() {
        let assembly = VehicleAssembly {
            arcade: Some(ArcadeConfig {
                turn_step_ms: 60,
                thrust_power: 8.0,
                max_speed: 20.0,
                heading_bits: 32,
                initial_heading: Some(0.75),
            }),
            ..VehicleAssembly::default()
        }
        .with_context(VehicleAssemblyContext { heading: None });

        assert_eq!(
            assembly.arcade.as_ref().and_then(|cfg| cfg.initial_heading),
            Some(0.75)
        );
    }

    #[test]
    fn assembly_plan_normalizes_descriptor_and_builds_profile() {
        let mut root = RhaiMap::new();
        root.insert("kind".into(), " ship ".into());
        root.insert("label".into(), " Surveyor ".into());

        let mut arcade = RhaiMap::new();
        arcade.insert("turn_step_ms".into(), 60_i64.into());
        arcade.insert("thrust_power".into(), 8.0_f64.into());
        arcade.insert("max_speed".into(), 20.0_f64.into());
        arcade.insert("heading_bits".into(), 32_i64.into());
        root.insert("arcade".into(), arcade.into());

        let plan = VehicleAssemblyPlan::from_rhai_map_with_context(
            &root,
            VehicleAssemblyContext {
                heading: Some(1.25),
            },
        )
        .expect("assembly plan");

        assert_eq!(plan.descriptor.kind, Some(VehicleKind::Ship));
        assert_eq!(plan.descriptor.profile_id.as_deref(), Some("arcade"));
        assert_eq!(plan.descriptor.label.as_deref(), Some("Surveyor"));

        let profile = plan.to_profile().expect("vehicle profile");
        assert_eq!(profile.profile_id, "arcade");
        assert_eq!(profile.label.as_deref(), Some("Surveyor"));
        assert_eq!(profile.turn_step_ms, Some(60));
        assert_eq!(
            plan.resolved_assembly()
                .arcade
                .as_ref()
                .and_then(|cfg| cfg.initial_heading),
            Some(1.25)
        );
    }

    #[test]
    fn assembly_plan_infers_arcade_descriptor_for_legacy_arcade_stack() {
        let mut root = RhaiMap::new();
        root.insert("turn_step_ms".into(), 60_i64.into());
        root.insert("thrust_power".into(), 8.0_f64.into());
        root.insert("max_speed".into(), 20.0_f64.into());
        root.insert("heading_bits".into(), 32_i64.into());

        let plan = VehicleAssemblyPlan::from_rhai_map(&root).expect("assembly plan");
        assert_eq!(plan.descriptor.kind, Some(VehicleKind::Ship));
        assert_eq!(plan.descriptor.profile_id.as_deref(), Some("arcade"));

        let profile = plan.to_profile().expect("vehicle profile");
        assert_eq!(profile.profile_id, "arcade");
    }

    #[test]
    fn apply_normalizes_configs_before_sink_attach() {
        #[derive(Default)]
        struct NormalizingSink {
            saw_arcade: bool,
        }

        impl VehicleAssemblySink for NormalizingSink {
            type Error = Infallible;

            fn attach_arcade(&mut self, arcade: ArcadeConfig) -> Result<(), Self::Error> {
                self.saw_arcade = true;
                assert_eq!(arcade.turn_step_ms, 1);
                assert_eq!(arcade.heading_bits, 1);
                assert_eq!(arcade.thrust_power, 0.0);
                assert_eq!(arcade.max_speed, 0.0);
                assert_eq!(arcade.initial_heading, None);
                Ok(())
            }

            fn attach_angular_body(
                &mut self,
                _angular_body: AngularBodyConfig,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn attach_linear_brake(
                &mut self,
                _linear_brake: LinearBrakeConfig,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn attach_thruster_ramp(
                &mut self,
                _thruster_ramp: ThrusterRampConfig,
            ) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        let assembly = VehicleAssembly {
            arcade: Some(ArcadeConfig {
                turn_step_ms: 0,
                thrust_power: -5.0,
                max_speed: -10.0,
                heading_bits: 0,
                initial_heading: Some(f32::NAN),
            }),
            ..VehicleAssembly::default()
        };

        let mut sink = NormalizingSink::default();
        let attached = assembly.apply(&mut sink).expect("apply");
        assert!(attached);
        assert!(sink.saw_arcade);
    }

    #[test]
    fn applies_to_sink_in_component_order() {
        #[derive(Default)]
        struct RecordingSink {
            calls: Vec<&'static str>,
        }

        impl VehicleAssemblySink for RecordingSink {
            type Error = Infallible;

            fn attach_arcade(&mut self, _arcade: ArcadeConfig) -> Result<(), Self::Error> {
                self.calls.push("arcade");
                Ok(())
            }

            fn attach_angular_body(
                &mut self,
                _angular_body: AngularBodyConfig,
            ) -> Result<(), Self::Error> {
                self.calls.push("angular_body");
                Ok(())
            }

            fn attach_linear_brake(
                &mut self,
                _linear_brake: LinearBrakeConfig,
            ) -> Result<(), Self::Error> {
                self.calls.push("linear_brake");
                Ok(())
            }

            fn attach_thruster_ramp(
                &mut self,
                _thruster_ramp: ThrusterRampConfig,
            ) -> Result<(), Self::Error> {
                self.calls.push("thruster_ramp");
                Ok(())
            }
        }

        let assembly = VehicleAssembly {
            arcade: Some(ArcadeConfig {
                turn_step_ms: 60,
                thrust_power: 8.0,
                max_speed: 20.0,
                heading_bits: 32,
                initial_heading: None,
            }),
            angular_body: Some(AngularBodyConfig::default()),
            linear_brake: Some(LinearBrakeConfig::default()),
            thruster_ramp: Some(ThrusterRampConfig::default()),
        };

        let mut sink = RecordingSink::default();
        let attached = assembly.apply(&mut sink).expect("apply");
        assert!(attached);
        assert_eq!(
            sink.calls,
            vec!["arcade", "angular_body", "linear_brake", "thruster_ramp"]
        );
    }
}
