//! Overlay data model for backend-agnostic debug console rendering.
//!
//! The overlay is rendered **after** the game buffer is presented, directly
//! onto the output surface (terminal or window), so it is always readable
//! regardless of game resolution or scaling.

use engine_core::color::Color;

/// A single line of overlay text with foreground and background colors.
///
/// `bg_alpha` controls background opacity (0 = transparent, 255 = opaque).
/// SDL2 uses this for alpha blending; terminal ignores it (no alpha support).
#[derive(Debug, Clone)]
pub struct OverlayLine {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub bg_alpha: u8,
}

impl OverlayLine {
    pub fn new(text: impl Into<String>, fg: Color, bg: Color) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            bg_alpha: 255,
        }
    }

    pub fn with_alpha(text: impl Into<String>, fg: Color, bg: Color, bg_alpha: u8) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            bg_alpha,
        }
    }
}

/// Complete overlay state to be rendered by the backend.
///
/// When `dim_scene` is true, backends should darken the game scene behind the
/// overlay to make it visually clear the console is active.
#[derive(Debug, Clone, Default)]
pub struct OverlayData {
    pub lines: Vec<OverlayLine>,
    pub dim_scene: bool,
}

impl OverlayData {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
