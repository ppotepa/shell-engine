use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera2DState {
    pub x: i32,
    pub y: i32,
    pub zoom: f32,
}

impl Default for Camera2DState {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            zoom: 1.0,
        }
    }
}
