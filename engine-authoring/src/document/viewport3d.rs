use serde::{Deserialize, Serialize};

/// Authored 3D viewport settings used by Scene3D-oriented documents.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport3dDocument {
    #[serde(default)]
    pub width: Option<u16>,
    #[serde(default)]
    pub height: Option<u16>,
}
