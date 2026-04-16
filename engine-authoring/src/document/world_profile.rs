use serde::{Deserialize, Serialize};

/// Authored world profile for 3D viewport documents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldProfileDocument {
    #[serde(default)]
    pub ambient_light: Option<f32>,
    #[serde(default)]
    pub gravity: Option<f32>,
    #[serde(default)]
    pub origin: Option<[f32; 3]>,
}
