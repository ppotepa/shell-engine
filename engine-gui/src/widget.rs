//! GUI widget definitions — authored once per scene, describe logical widgets.
//!
//! These are pure data; no rendering, no Rhai. The compositor renders the
//! corresponding visual sprites; engine-gui handles hit-testing and state.

use serde::{Deserialize, Serialize};

/// A logical GUI widget bound to a visual sprite in the scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GuiWidgetDef {
    /// Horizontal drag slider mapped to a [min, max] range.
    Slider {
        id: String,
        /// Sprite id whose screen bounds define the draggable track.
        sprite: String,
        /// Left edge of the track in screen pixels.
        x: i32,
        /// Top edge of the track in screen pixels.
        y: i32,
        /// Width of the track in screen pixels.
        w: i32,
        /// Height of the hit zone in screen pixels.
        h: i32,
        min: f64,
        max: f64,
        /// Initial value (clamped to [min, max]).
        #[serde(default)]
        value: f64,
        /// Extra pixels added around the track rect for hit-testing only.
        /// The value calculation still uses (x, w) so the slider range is unaffected.
        #[serde(default)]
        hit_padding: i32,
        /// Sprite id of the handle/thumb element. When set, the engine automatically
        /// positions this sprite's offset_x based on the current slider value.
        #[serde(default)]
        handle: String,
    },
    /// Clickable button — fires once per press.
    Button {
        id: String,
        sprite: String,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    /// Boolean toggle — flips on each click.
    Toggle {
        id: String,
        sprite: String,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        /// Initial state.
        #[serde(default)]
        on: bool,
    },
    /// Visibility group — controls whether a named panel layer is shown.
    Panel {
        id: String,
        /// Sprite id of the panel container.
        sprite: String,
        #[serde(default)]
        visible: bool,
    },
}

impl GuiWidgetDef {
    pub fn id(&self) -> &str {
        match self {
            Self::Slider { id, .. } => id,
            Self::Button { id, .. } => id,
            Self::Toggle { id, .. } => id,
            Self::Panel { id, .. } => id,
        }
    }

    /// Returns the hit-test bounding rect (x, y, w, h) if this widget has one.
    /// For sliders, the rect is expanded by `hit_padding` for easier grabbing.
    pub fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        match self {
            Self::Slider {
                x,
                y,
                w,
                h,
                hit_padding,
                ..
            } => {
                let p = *hit_padding;
                Some((*x - p, *y - p, *w + 2 * p, *h + 2 * p))
            }
            Self::Button { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            Self::Toggle { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            Self::Panel { .. } => None,
        }
    }

    pub fn initial_value(&self) -> f64 {
        match self {
            Self::Slider { value, min, .. } => value.max(*min),
            Self::Toggle { on, .. } => {
                if *on {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    /// For sliders with a `handle` sprite, returns (handle_sprite_id, track_w, frac)
    /// where frac = (value - min) / (max - min) so the caller can set offset_x = frac * w.
    pub fn handle_offset(&self, current_value: f64) -> Option<(&str, f64)> {
        match self {
            Self::Slider {
                handle,
                w,
                min,
                max,
                ..
            } if !handle.is_empty() => {
                let range = max - min;
                let frac = if range.abs() > f64::EPSILON {
                    ((current_value - min) / range).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                Some((handle.as_str(), frac * *w as f64))
            }
            _ => None,
        }
    }
}
