//! Vector overlay data for high-fidelity polygon rendering on pixel backends.
//!
//! Terminal backend renders vectors as character glyphs via `engine_vector`.
//! SDL2 backend draws vectors directly on the canvas at native resolution,
//! bypassing the character-cell buffer for smooth, anti-alias-ready shapes.

/// A single resolved vector shape ready for pixel-backend rendering.
#[derive(Debug, Clone)]
pub struct VectorPrimitive {
    /// Points in buffer cell coordinates (sprite-local + origin already applied).
    pub points: Vec<[f32; 2]>,
    /// Whether the shape is closed (last point connects to first).
    pub closed: bool,
    /// Foreground (stroke) color as RGB.
    pub fg: (u8, u8, u8),
    /// Background (fill) color as RGB. `None` means outline-only.
    pub bg: Option<(u8, u8, u8)>,
}

/// Collected vector primitives for a single frame.
#[derive(Debug, Clone, Default)]
pub struct VectorOverlay {
    pub primitives: Vec<VectorPrimitive>,
    pub buffer_width: u16,
    pub buffer_height: u16,
}

impl VectorOverlay {
    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }
}
