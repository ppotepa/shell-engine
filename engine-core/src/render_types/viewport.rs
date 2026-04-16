use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}
