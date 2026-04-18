use serde::{Deserialize, Serialize};

/// Authored lighting profile for reusable 3D scene lighting behavior.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LightingProfileDocument {
    #[serde(default)]
    pub ambient_intensity: Option<f32>,
    #[serde(default)]
    pub black_level: Option<f32>,
    #[serde(default)]
    pub exposure: Option<f32>,
    #[serde(default)]
    pub tonemap: Option<String>,
    #[serde(default)]
    pub gamma: Option<f32>,
}
