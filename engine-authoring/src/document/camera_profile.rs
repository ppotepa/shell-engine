use serde::{Deserialize, Serialize};

/// Authored camera profile for 3D viewport documents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CameraProfileDocument {
    #[serde(default)]
    pub distance: Option<f32>,
    #[serde(default)]
    pub fov_degrees: Option<f32>,
    #[serde(default)]
    pub near_clip: Option<f32>,
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    #[serde(default)]
    pub look_at: Option<[f32; 3]>,
}
