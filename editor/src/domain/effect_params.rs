//! Effect parameter specifications, UI controls, and runtime value helpers.

use engine_core::scene::{Easing, EffectParams, TermColour};
use std::collections::HashMap;

/// Describes the UI control type and bounds for a single adjustable effect parameter.
#[derive(Debug, Clone, Copy)]
pub enum EffectParamControl {
    Slider {
        min: f32,
        max: f32,
        step: f32,
        unit: &'static str,
    },
}

impl EffectParamControl {
    pub const fn slider(min: f32, max: f32, step: f32, unit: &'static str) -> Self {
        Self::Slider {
            min,
            max,
            step,
            unit,
        }
    }

    pub fn bounds(&self) -> (f32, f32) {
        match *self {
            EffectParamControl::Slider { min, max, .. } => (min, max),
        }
    }

    pub fn step(&self) -> f32 {
        match *self {
            EffectParamControl::Slider { step, .. } => step,
        }
    }

    pub fn unit(&self) -> &'static str {
        match *self {
            EffectParamControl::Slider { unit, .. } => unit,
        }
    }

    pub fn clamp(&self, value: f32) -> f32 {
        let (min, max) = self.bounds();
        value.clamp(min, max)
    }
}

/// Metadata for a single effect parameter: its name, display label, and control specification.
#[derive(Debug, Clone, Copy)]
pub struct EffectParamSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub control: EffectParamControl,
}

impl EffectParamSpec {
    pub const fn slider(
        name: &'static str,
        label: &'static str,
        min: f32,
        max: f32,
        step: f32,
        unit: &'static str,
    ) -> Self {
        Self {
            name,
            label,
            control: EffectParamControl::slider(min, max, step, unit),
        }
    }

    pub fn default_value(&self) -> EffectParamValue {
        EffectParamValue(self.control.bounds().0)
    }

    pub fn adjust(&self, current: f32, delta_dir: f32) -> EffectParamValue {
        let delta = self.control.step() * delta_dir;
        let next = self.control.clamp(current + delta);
        EffectParamValue(next)
    }

    pub fn render_value(&self, value: f32) -> String {
        let track_length: usize = 12;
        let (min, max) = self.control.bounds();
        let normalized = if max - min <= 0.0 {
            0.0
        } else {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        };
        let filled = (normalized * track_length as f32).round() as usize;
        let empty = track_length.saturating_sub(filled);
        let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));
        if self.control.unit().is_empty() {
            format!("{bar} {:5.2}", value)
        } else {
            format!("{bar} {:5.1} {}", value, self.control.unit())
        }
    }
}

/// A resolved single-float value for an effect parameter.
#[derive(Debug, Clone, Copy)]
pub struct EffectParamValue(pub f32);

impl EffectParamValue {
    pub fn as_float(self) -> f32 {
        self.0
    }
}

type ParamReader = fn(&EffectParams) -> Option<f32>;
type ParamWriter = fn(&mut EffectParams, f32);

/// Central descriptor for a numeric effect parameter used by UI and override application.
#[derive(Debug, Clone, Copy)]
pub struct EffectParamDescriptor {
    pub name: &'static str,
    pub label: &'static str,
    pub read: ParamReader,
    pub write: ParamWriter,
}

macro_rules! spec {
    ($name:expr, $label:expr, $min:expr, $max:expr, $step:expr, $unit:expr) => {
        EffectParamSpec::slider($name, $label, $min, $max, $step, $unit)
    };
}

const SPEC_INTENSITY: EffectParamSpec = spec!("intensity", "Intensity", 0.0, 2.0, 0.05, "");
const SPEC_ANGLE: EffectParamSpec = spec!("angle", "Angle", 0.0, 90.0, 2.0, "deg");
const SPEC_WIDTH: EffectParamSpec = spec!("width", "Width", 1.0, 12.0, 0.5, "cols");
const SPEC_FALLOFF: EffectParamSpec = spec!("falloff", "Falloff", 0.0, 5.0, 0.2, "");
const SPEC_AMPLITUDE_X: EffectParamSpec = spec!("amplitude_x", "X Amplitude", 0.0, 3.0, 0.1, "");
const SPEC_AMPLITUDE_Y: EffectParamSpec = spec!("amplitude_y", "Y Amplitude", 0.0, 3.0, 0.1, "");
const SPEC_FREQUENCY: EffectParamSpec = spec!("frequency", "Frequency", 0.0, 20.0, 0.5, "");
const SPEC_STRIKES: EffectParamSpec = spec!("strikes", "Strikes", 1.0, 10.0, 1.0, "");
const SPEC_THICKNESS: EffectParamSpec = spec!("thickness", "Thickness", 0.1, 3.0, 0.1, "");
const SPEC_SPEED: EffectParamSpec = spec!("speed", "Speed", 0.0, 2.0, 0.1, "");
const SPEC_GLOW: EffectParamSpec = spec!("glow", "Glow", 0.0, 1.0, 1.0, "");
const SPEC_OCTAVE_COUNT: EffectParamSpec = spec!("octave_count", "Octaves", 1.0, 8.0, 1.0, "");

const DEFAULT_SPECS: &[EffectParamSpec] = &[SPEC_INTENSITY];
const SHINE_SPECS: &[EffectParamSpec] = &[
    SPEC_ANGLE,
    SPEC_WIDTH,
    SPEC_FALLOFF,
    SPEC_INTENSITY,
    SPEC_SPEED,
];
const SHAKE_SPECS: &[EffectParamSpec] = &[SPEC_AMPLITUDE_X, SPEC_AMPLITUDE_Y, SPEC_FREQUENCY];
const LIGHTNING_SPECS: &[EffectParamSpec] = &[
    SPEC_INTENSITY,
    SPEC_STRIKES,
    SPEC_THICKNESS,
    SPEC_SPEED,
    SPEC_GLOW,
    SPEC_OCTAVE_COUNT,
];

fn read_intensity(p: &EffectParams) -> Option<f32> {
    p.intensity
}
fn write_intensity(p: &mut EffectParams, v: f32) {
    p.intensity = Some(v);
}
fn read_angle(p: &EffectParams) -> Option<f32> {
    p.angle
}
fn write_angle(p: &mut EffectParams, v: f32) {
    p.angle = Some(v);
}
fn read_width(p: &EffectParams) -> Option<f32> {
    p.width
}
fn write_width(p: &mut EffectParams, v: f32) {
    p.width = Some(v);
}
fn read_falloff(p: &EffectParams) -> Option<f32> {
    p.falloff
}
fn write_falloff(p: &mut EffectParams, v: f32) {
    p.falloff = Some(v);
}
fn read_amplitude_x(p: &EffectParams) -> Option<f32> {
    p.amplitude_x
}
fn write_amplitude_x(p: &mut EffectParams, v: f32) {
    p.amplitude_x = Some(v);
}
fn read_amplitude_y(p: &EffectParams) -> Option<f32> {
    p.amplitude_y
}
fn write_amplitude_y(p: &mut EffectParams, v: f32) {
    p.amplitude_y = Some(v);
}
fn read_frequency(p: &EffectParams) -> Option<f32> {
    p.frequency
}
fn write_frequency(p: &mut EffectParams, v: f32) {
    p.frequency = Some(v);
}
fn read_strikes(p: &EffectParams) -> Option<f32> {
    p.strikes.map(|v| v as f32)
}
fn write_strikes(p: &mut EffectParams, v: f32) {
    p.strikes = Some(v.round() as u16);
}
fn read_thickness(p: &EffectParams) -> Option<f32> {
    p.thickness
}
fn write_thickness(p: &mut EffectParams, v: f32) {
    p.thickness = Some(v);
}
fn read_speed(p: &EffectParams) -> Option<f32> {
    p.speed
}
fn write_speed(p: &mut EffectParams, v: f32) {
    p.speed = Some(v);
}
fn read_glow(p: &EffectParams) -> Option<f32> {
    p.glow.map(|v| if v { 1.0 } else { 0.0 })
}
fn write_glow(p: &mut EffectParams, v: f32) {
    p.glow = Some(v >= 0.5);
}
fn read_octave_count(p: &EffectParams) -> Option<f32> {
    p.octave_count.map(|v| v as f32)
}
fn write_octave_count(p: &mut EffectParams, v: f32) {
    p.octave_count = Some(v.round() as u8);
}

const PARAM_DESCRIPTORS: &[EffectParamDescriptor] = &[
    EffectParamDescriptor {
        name: "intensity",
        label: "Intensity",
        read: read_intensity,
        write: write_intensity,
    },
    EffectParamDescriptor {
        name: "angle",
        label: "Angle",
        read: read_angle,
        write: write_angle,
    },
    EffectParamDescriptor {
        name: "width",
        label: "Width",
        read: read_width,
        write: write_width,
    },
    EffectParamDescriptor {
        name: "falloff",
        label: "Falloff",
        read: read_falloff,
        write: write_falloff,
    },
    EffectParamDescriptor {
        name: "amplitude_x",
        label: "X Amplitude",
        read: read_amplitude_x,
        write: write_amplitude_x,
    },
    EffectParamDescriptor {
        name: "amplitude_y",
        label: "Y Amplitude",
        read: read_amplitude_y,
        write: write_amplitude_y,
    },
    EffectParamDescriptor {
        name: "frequency",
        label: "Frequency",
        read: read_frequency,
        write: write_frequency,
    },
    EffectParamDescriptor {
        name: "strikes",
        label: "Strikes",
        read: read_strikes,
        write: write_strikes,
    },
    EffectParamDescriptor {
        name: "thickness",
        label: "Thickness",
        read: read_thickness,
        write: write_thickness,
    },
    EffectParamDescriptor {
        name: "speed",
        label: "Speed",
        read: read_speed,
        write: write_speed,
    },
    EffectParamDescriptor {
        name: "glow",
        label: "Glow",
        read: read_glow,
        write: write_glow,
    },
    EffectParamDescriptor {
        name: "octave_count",
        label: "Octaves",
        read: read_octave_count,
        write: write_octave_count,
    },
];

fn descriptor(name: &str) -> Option<&'static EffectParamDescriptor> {
    PARAM_DESCRIPTORS.iter().find(|d| d.name == name)
}

/// Returns the parameter specifications for the named effect.
pub fn effect_param_specs(effect_name: &str) -> &'static [EffectParamSpec] {
    match effect_name {
        "shine" => SHINE_SPECS,
        "screen-shake" => SHAKE_SPECS,
        "lightning-flash"
        | "lightning-branch"
        | "lightning-optical-80s"
        | "lightning-fbm"
        | "lightning-growth"
        | "lightning-ambient"
        | "lightning-natural"
        | "tesla-orb" => LIGHTNING_SPECS,
        _ => DEFAULT_SPECS,
    }
}

/// Builds a default [`EffectParams`] populated with sensible values for the named effect.
#[allow(clippy::field_reassign_with_default)]
pub fn default_effect_params(effect_name: &str) -> EffectParams {
    let mut params = EffectParams::default();
    params.intensity = Some(1.0);

    match effect_name {
        "screen-shake" => {
            params.amplitude_x = Some(1.4);
            params.amplitude_y = Some(0.8);
            params.frequency = Some(10.0);
        }
        "shine" => {
            params.angle = Some(22.0);
            params.width = Some(5.0);
            params.falloff = Some(1.2);
            params.intensity = Some(1.0);
            params.speed = Some(0.8);
        }
        "clear-to-colour" => {
            params.colour = Some(TermColour::Rgb(8, 12, 24));
        }
        name if name.starts_with("lightning-") || name == "tesla-orb" => {
            params.intensity = Some(1.1);
            params.glow = Some(true);
            params.strikes = Some(3);
            params.thickness = Some(1.1);
            params.speed = Some(0.8);
        }
        _ => {}
    }

    params
}

/// Reads a named float parameter from an [`EffectParams`] instance.
pub fn effect_param_value(params: &EffectParams, name: &str) -> Option<EffectParamValue> {
    let value = descriptor(name).and_then(|d| (d.read)(params))?;
    Some(EffectParamValue(value))
}

/// Returns a human-readable display label for the given parameter name.
pub fn param_label(name: &str) -> &'static str {
    if let Some(d) = descriptor(name) {
        return d.label;
    }
    match name {
        "colour" => "Colour",
        "easing" => "Easing",
        "coverage" => "Coverage",
        "orientation" => "Orientation",
        "target" => "Target",
        "start_x" => "Start X",
        "end_x" => "End X",
        "amp_start" => "Amp Start",
        "amp_coeff" => "Amp Coeff",
        "freq_coeff" => "Freq Coeff",
        _ => "Param",
    }
}

/// Formats the current value of a named parameter as a display string.
pub fn param_text_value(params: &EffectParams, name: &str) -> Option<String> {
    match name {
        "colour" => params.colour.as_ref().map(render_colour),
        "easing" => Some(render_easing(&params.easing).to_string()),
        "coverage" => params.coverage.clone(),
        "orientation" => params.orientation.clone(),
        "target" => params.target.clone(),
        "start_x" => params.start_x.clone(),
        "end_x" => params.end_x.clone(),
        "glow" => params
            .glow
            .map(|v| if v { "true" } else { "false" }.to_string()),
        other => effect_param_value(params, other).map(|v| format!("{:.2}", v.as_float())),
    }
}

fn render_easing(easing: &Easing) -> &'static str {
    match easing {
        Easing::Linear => "linear",
        Easing::EaseIn => "ease-in",
        Easing::EaseOut => "ease-out",
        Easing::EaseInOut => "ease-in-out",
    }
}

fn render_colour(colour: &TermColour) -> String {
    match colour {
        TermColour::Black => "black".to_string(),
        TermColour::White => "white".to_string(),
        TermColour::Silver => "silver".to_string(),
        TermColour::Gray => "grey".to_string(),
        TermColour::Red => "red".to_string(),
        TermColour::Green => "green".to_string(),
        TermColour::Blue => "blue".to_string(),
        TermColour::Yellow => "yellow".to_string(),
        TermColour::Cyan => "cyan".to_string(),
        TermColour::Magenta => "magenta".to_string(),
        TermColour::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

/// Applies a map of named float overrides to an [`EffectParams`] instance in-place.
pub fn apply_overrides(
    _effect_name: &str,
    overrides: &HashMap<String, EffectParamValue>,
    params: &mut EffectParams,
) {
    for (key, value) in overrides {
        if let Some(d) = descriptor(key) {
            (d.write)(params, value.as_float());
        }
    }
}
