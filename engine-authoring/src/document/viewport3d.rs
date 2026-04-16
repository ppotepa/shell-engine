use serde::{Deserialize, Serialize};

/// Authored 3D viewport settings used by Scene3D-oriented documents.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport3dDocument {
    #[serde(default)]
    pub width: Option<u16>,
    #[serde(default)]
    pub height: Option<u16>,
}

/// Sprite-backed 3D viewport reference in the intermediate render-scene model.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport3dSpriteDocument {
    #[serde(default)]
    pub id: Option<String>,
    pub layer_index: usize,
    #[serde(default)]
    pub sprite_path: Vec<usize>,
    #[serde(default)]
    pub sprite_id: Option<String>,
}
