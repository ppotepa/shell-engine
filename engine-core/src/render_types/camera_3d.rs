use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera3DState {
    pub eye: [f32; 3],
    pub look_at: [f32; 3],
    pub up: [f32; 3],
    pub fov_deg: f32,
}

impl Default for Camera3DState {
    fn default() -> Self {
        Self {
            eye: [0.0, 0.0, -5.0],
            look_at: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov_deg: 60.0,
        }
    }
}
