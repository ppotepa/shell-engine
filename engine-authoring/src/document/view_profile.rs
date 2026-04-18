use serde::{Deserialize, Serialize};

/// Authored top-level 3D view profile that composes lower-level reusable profiles.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewProfileDocument {
    #[serde(default)]
    pub lighting_profile: Option<String>,
    #[serde(default)]
    pub space_environment_profile: Option<String>,
    #[serde(default)]
    pub exposure_override: Option<f32>,
    #[serde(default)]
    pub black_level_override: Option<f32>,
}
