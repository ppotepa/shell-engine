use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightKind3D {
    Directional,
    Point,
    Ambient,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Light3D {
    pub kind: LightKind3D,
    pub color: [u8; 3],
    pub intensity: f32,
    pub vector: [f32; 3],
}

impl Default for Light3D {
    fn default() -> Self {
        Self {
            kind: LightKind3D::Directional,
            color: [255, 255, 255],
            intensity: 1.0,
            vector: [0.0, -1.0, 0.0],
        }
    }
}
