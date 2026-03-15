use serde::Deserialize;
use crate::scene::Easing;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationAxis {
    #[default]
    Y,
    X,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnimationParams {
    #[serde(default)]
    pub axis: AnimationAxis,
    #[serde(default = "default_amplitude")]
    pub amplitude: u16,
    #[serde(default = "default_period")]
    pub period_ms: u64,
    #[serde(default)]
    pub easing: Easing,
}

fn default_amplitude() -> u16 { 1 }
fn default_period() -> u64 { 2000 }
