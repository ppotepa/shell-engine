use serde::Deserialize;
use super::{color::TermColour, LayerStages};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

/// Glow halo effect for a text sprite.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Glow {
    /// Glow colour. If None, uses a dimmed version of the sprite's fg_colour.
    pub colour: Option<TermColour>,
    /// Radius in cells. 1 = 8-neighbor halo. Default 1.
    #[serde(default = "default_glow_radius")]
    pub radius: u16,
}

fn default_glow_radius() -> u16 { 1 }

/// A drawable object within a layer. Has its own local bitmap position and lifecycle.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sprite {
    /// Terminal-native text sprite.
    /// If `font` is None: characters written directly to buffer (fast path).
    /// If `font` is Some: text is rasterized using a bitmap font definition (rasterizer path).
    Text {
        /// Optional stable identifier for script-side lookups (e.g. intro.rs logic hooks).
        #[serde(default)]
        id: Option<String>,
        content: String,
        #[serde(default)]
        x: u16,
        #[serde(default)]
        y: u16,
        #[serde(default)]
        z_index: i32,
        font: Option<String>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        /// Delay from scene start (ms) before sprite becomes visible.
        #[serde(default)]
        appear_at_ms: Option<u64>,
        /// Optional absolute scene time (ms) after which sprite is hidden.
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        /// Optional reveal duration (ms). When set, text appears left->right.
        #[serde(default)]
        reveal_ms: Option<u64>,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        /// Optional glow halo rendered behind the sprite.
        #[serde(default)]
        glow: Option<Glow>,
    },
}
