//! Compile-time metadata for builtin effects.
//!
//! Each builtin declares a static `EffectMetadata` that describes its parameters,
//! their types, defaults, and ranges — the single source of truth consumed by
//! any tool (editor, documentation generator, schema validator).

use crate::effects::effect::EffectTargetMask;
use crate::authoring::metadata as authored;

/// How a parameter value is expressed and controlled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamControl {
    /// Numeric slider: min, max, step (all f32).
    Slider {
        min: f32,
        max: f32,
        step: f32,
        unit: &'static str,
    },
    /// Fixed set of named string values.
    Select {
        options: &'static [&'static str],
        default: &'static str,
    },
    /// Boolean toggle (true/false).
    Toggle { default: bool },
    /// Free-form string (e.g. `start_x: "random"` or `"32"`).
    Text { default: &'static str },
    /// Colour string — named or `#rrggbb`.
    Colour { default: &'static str },
}

impl ParamControl {
    pub const fn default_str(&self) -> &'static str {
        match self {
            ParamControl::Slider { .. } => "",
            ParamControl::Select { default, .. } => default,
            ParamControl::Toggle { default } => {
                if *default {
                    "true"
                } else {
                    "false"
                }
            }
            ParamControl::Text { default } => default,
            ParamControl::Colour { default } => default,
        }
    }

    pub const fn default_float(&self) -> f32 {
        match self {
            ParamControl::Slider { min, .. } => *min,
            ParamControl::Toggle { default } => {
                if *default {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    pub fn is_slider(&self) -> bool {
        matches!(self, ParamControl::Slider { .. })
    }

    pub const fn as_value_kind(&self) -> authored::ValueKind {
        match self {
            ParamControl::Slider { .. } => authored::ValueKind::Number,
            ParamControl::Select { .. } => authored::ValueKind::Select,
            ParamControl::Toggle { .. } => authored::ValueKind::Boolean,
            ParamControl::Text { .. } => authored::ValueKind::Text,
            ParamControl::Colour { .. } => authored::ValueKind::Colour,
        }
    }
}

/// Metadata for a single effect parameter.
#[derive(Debug, Clone, Copy)]
pub struct ParamMetadata {
    /// Internal key matching `EffectParams` field name.
    pub name: &'static str,
    /// Human-readable label shown in editor.
    pub label: &'static str,
    /// Tooltip / one-line description.
    pub description: &'static str,
    /// How the value is represented and controlled.
    pub control: ParamControl,
}

impl ParamMetadata {
    /// Convert effect-specific parameter metadata into shared authored field metadata.
    pub const fn as_authored_field(&self) -> authored::FieldMetadata {
        authored::FieldMetadata {
            target: authored::TargetKind::Effect,
            name: self.name,
            value_kind: self.control.as_value_kind(),
            requirement: authored::Requirement::Optional,
            description: self.description,
            default_text: Some(self.control.default_str()),
            default_number: Some(self.control.default_float()),
            enum_options: match self.control {
                ParamControl::Select { options, .. } => Some(options),
                _ => None,
            },
            min: match self.control {
                ParamControl::Slider { min, .. } => Some(min),
                _ => None,
            },
            max: match self.control {
                ParamControl::Slider { max, .. } => Some(max),
                _ => None,
            },
            step: match self.control {
                ParamControl::Slider { step, .. } => Some(step),
                _ => None,
            },
            unit: match self.control {
                ParamControl::Slider { unit, .. } => Some(unit),
                _ => None,
            },
            sources: &[authored::ValueSource::Literal],
        }
    }
}

/// Complete metadata for a builtin effect.
#[derive(Debug, Clone, Copy)]
pub struct EffectMetadata {
    /// Internal effect name, matches registry key.
    pub name: &'static str,
    /// Human-readable display name.
    pub display_name: &'static str,
    /// Short description shown in the effects browser.
    pub summary: &'static str,
    /// Functional category string (fade / lightning / crt / colour / motion / distortion).
    pub category: &'static str,
    /// Which target kinds this effect can safely operate on.
    pub compatible_targets: EffectTargetMask,
    /// Ordered list of parameters this effect uses.
    pub params: &'static [ParamMetadata],
    /// Example YAML snippet shown in editor docs pane.
    pub sample: &'static str,
}

impl EffectMetadata {
    /// Returns only params that have a slider control — useful for live controls panel.
    pub fn slider_params(&self) -> impl Iterator<Item = &ParamMetadata> {
        self.params.iter().filter(|p| p.control.is_slider())
    }

    /// Returns all params — useful for docs pane.
    pub fn all_params(&self) -> &[ParamMetadata] {
        self.params
    }
}

// ─── Param shorthands ────────────────────────────────────────────────────────

pub const fn slider(
    name: &'static str,
    label: &'static str,
    desc: &'static str,
    min: f32,
    max: f32,
    step: f32,
    unit: &'static str,
) -> ParamMetadata {
    ParamMetadata {
        name,
        label,
        description: desc,
        control: ParamControl::Slider {
            min,
            max,
            step,
            unit,
        },
    }
}

pub const fn select(
    name: &'static str,
    label: &'static str,
    desc: &'static str,
    options: &'static [&'static str],
    default: &'static str,
) -> ParamMetadata {
    ParamMetadata {
        name,
        label,
        description: desc,
        control: ParamControl::Select { options, default },
    }
}

pub const fn toggle(
    name: &'static str,
    label: &'static str,
    desc: &'static str,
    default: bool,
) -> ParamMetadata {
    ParamMetadata {
        name,
        label,
        description: desc,
        control: ParamControl::Toggle { default },
    }
}

pub const fn text(
    name: &'static str,
    label: &'static str,
    desc: &'static str,
    default: &'static str,
) -> ParamMetadata {
    ParamMetadata {
        name,
        label,
        description: desc,
        control: ParamControl::Text { default },
    }
}

// ─── Shared param constants ───────────────────────────────────────────────────

pub const EASINGS: &[&str] = &["linear", "ease-in", "ease-out", "ease-in-out"];

pub const P_EASING: ParamMetadata = select(
    "easing",
    "Easing",
    "Progress curve applied to the effect.",
    EASINGS,
    "linear",
);
pub const P_INTENSITY: ParamMetadata = slider(
    "intensity",
    "Intensity",
    "Overall strength multiplier.",
    0.0,
    2.0,
    0.05,
    "",
);
pub const P_SPEED: ParamMetadata = slider(
    "speed",
    "Speed",
    "Animation speed multiplier.",
    0.0,
    2.0,
    0.1,
    "",
);
pub const P_STRIKES: ParamMetadata = slider(
    "strikes",
    "Strikes",
    "Number of primary arcs / branches.",
    1.0,
    10.0,
    1.0,
    "",
);
pub const P_THICKNESS: ParamMetadata = slider(
    "thickness",
    "Thickness",
    "Branch / bolt thickness multiplier.",
    0.1,
    3.0,
    0.1,
    "",
);
pub const P_GLOW: ParamMetadata = toggle("glow", "Glow", "Draw halo glow around the bolt.", true);
pub const P_ORIENTATION: ParamMetadata = select(
    "orientation",
    "Orientation",
    "Directional axis for the effect.",
    &["horizontal", "vertical"],
    "horizontal",
);
pub const P_OCTAVES: ParamMetadata = slider(
    "octave_count",
    "Octaves",
    "FBM complexity octaves.",
    1.0,
    8.0,
    1.0,
    "",
);

pub static META_UNKNOWN: EffectMetadata = EffectMetadata {
    name: "unknown",
    display_name: "Unknown Effect",
    summary: "Builtin effect registered in engine dispatcher.",
    category: "other",
    compatible_targets: EffectTargetMask::ANY,
    params: &[P_INTENSITY, P_EASING],
    sample: "- name: EFFECT_NAME\n  duration: 600\n  params:\n    easing: linear",
};

#[cfg(test)]
mod tests {
    use super::{ParamControl, P_EASING, P_INTENSITY};
    use crate::authoring::metadata::{TargetKind, ValueKind};

    #[test]
    fn converts_slider_to_shared_field_metadata() {
        let field = P_INTENSITY.as_authored_field();
        assert_eq!(field.target, TargetKind::Effect);
        assert_eq!(field.name, "intensity");
        assert_eq!(field.value_kind, ValueKind::Number);
        assert_eq!(field.min, Some(0.0));
        assert_eq!(field.max, Some(2.0));
    }

    #[test]
    fn converts_select_to_shared_field_metadata() {
        let field = P_EASING.as_authored_field();
        assert_eq!(field.value_kind, ValueKind::Select);
        assert!(field.enum_options.is_some());
        match P_EASING.control {
            ParamControl::Select { default, .. } => {
                assert_eq!(field.default_text, Some(default));
            }
            _ => panic!("expected select"),
        }
    }
}
