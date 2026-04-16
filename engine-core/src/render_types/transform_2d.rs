use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub x: i32,
    pub y: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_deg: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_deg: 0.0,
        }
    }
}
