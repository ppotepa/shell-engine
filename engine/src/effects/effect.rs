use crate::buffer::Buffer;
use crate::scene::EffectParams;

/// A region of the screen the effect operates on.
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Region {
    pub fn full(buffer: &Buffer) -> Self {
        Self {
            x: 0,
            y: 0,
            width: buffer.width,
            height: buffer.height,
        }
    }

    pub fn row(y: u16, x: u16, width: u16) -> Self {
        Self {
            x,
            y,
            width,
            height: 1,
        }
    }
}

/// Core effect abstraction.
pub trait Effect: Send + Sync {
    /// Apply this effect to `buffer` within `region`.
    /// `progress` is 0.0–1.0 normalized time within the effect duration.
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer);
}
