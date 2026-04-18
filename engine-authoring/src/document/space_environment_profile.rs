use serde::{Deserialize, Serialize};

/// Authored space environment profile for reusable 3D scene backgrounds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpaceEnvironmentProfileDocument {
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub background_floor: Option<f32>,
    #[serde(default)]
    pub starfield_density: Option<f32>,
    #[serde(default)]
    pub starfield_brightness: Option<f32>,
    #[serde(default)]
    pub primary_star_glare_strength: Option<f32>,
}
