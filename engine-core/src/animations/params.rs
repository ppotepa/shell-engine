//! Animation parameter types shared by sprites and objects.

use crate::scene::Easing;
use serde::Deserialize;

/// Axis along which an animation displacement is applied.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationAxis {
    /// Animate along the vertical axis (default).
    #[default]
    Y,
    /// Animate along the horizontal axis.
    X,
}

/// Parameters controlling the behaviour of a sprite or object animation.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnimationParams {
    /// Axis the animation moves along.
    #[serde(default)]
    pub axis: AnimationAxis,
    /// Peak displacement in terminal cells.
    #[serde(default = "default_amplitude")]
    pub amplitude: u16,
    /// Full cycle duration in milliseconds.
    #[serde(default = "default_period")]
    pub period_ms: u64,
    /// Easing function applied to the animation curve.
    #[serde(default)]
    pub easing: Easing,
}

fn default_amplitude() -> u16 {
    1
}
fn default_period() -> u64 {
    2000
}
