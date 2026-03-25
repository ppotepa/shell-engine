//! Pre-rendered OBJ frame store — one canvas per sprite, keyed by sprite ID.

use std::collections::HashMap;
use std::sync::Arc;

/// Flat RGB canvas: `None` = transparent, `Some([r,g,b])` = opaque pixel.
/// Row-major, width × height virtual pixels.
pub type PrerenderedCanvas = Vec<Option<[u8; 3]>>;

/// One pre-rendered sprite frame with its virtual dimensions and the pose it was rendered at.
pub struct PrerenderedFrame {
    pub canvas: Arc<PrerenderedCanvas>,
    /// Virtual pixel dimensions used when blitting.
    pub virtual_w: u16,
    pub virtual_h: u16,
    /// Terminal cell dimensions.
    pub target_w: u16,
    pub target_h: u16,
    /// Total yaw at render time (rotation_y + yaw_deg) — for cache-hit check.
    pub rendered_yaw: f32,
    /// Pitch at render time — for cache-hit check.
    pub rendered_pitch: f32,
}

/// World resource: holds pre-rendered canvases for all eligible OBJ sprites in the active scene.
#[derive(Default)]
pub struct ObjPrerenderedFrames {
    frames: HashMap<String, PrerenderedFrame>,
}

impl ObjPrerenderedFrames {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sprite_id: String, frame: PrerenderedFrame) {
        self.frames.insert(sprite_id, frame);
    }

    pub fn get(&self, sprite_id: &str) -> Option<&PrerenderedFrame> {
        self.frames.get(sprite_id)
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }
}

/// World resource tracking status of the prerender pass.
pub enum ObjPrerenderStatus {
    /// No prerender scheduled or not yet run.
    Idle,
    /// Prerender complete — cache is populated and ready.
    Ready,
}
