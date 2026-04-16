use serde::{Deserialize, Serialize};

/// Authored atmosphere profile for 3D viewport documents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AtmosphereProfileDocument {
    #[serde(default)]
    pub haze: Option<f32>,
    #[serde(default)]
    pub exposure: Option<f32>,
    #[serde(default)]
    pub rim_intensity: Option<f32>,
    #[serde(default)]
    pub sky_colour: Option<String>,
}
