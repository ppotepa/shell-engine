//! Pre-rendered OBJ frame store — one canvas per sprite, keyed by sprite ID.

use std::collections::HashMap;
use std::sync::Arc;

/// Flat RGB canvas: `None` = transparent, `Some([r,g,b])` = opaque pixel.
/// Row-major, width × height virtual pixels.
pub type PrerenderedCanvas = Vec<Option<[u8; 3]>>;

/// Yaw quantisation step in degrees. 72 evenly-spaced keyframes per full rotation.
pub const YAW_STEP_DEG: u16 = 5;
/// Total number of yaw keyframes per animated sprite.
pub const YAW_FRAME_COUNT: usize = (360 / YAW_STEP_DEG) as usize;

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

/// 72 pre-baked rotation keyframes (every 5°) for an animated OBJ sprite.
///
/// Index = `(snapped_yaw / YAW_STEP_DEG) % YAW_FRAME_COUNT`.
pub struct AnimSpriteFrames {
    /// Canvases indexed by yaw step (0 = 0°, 1 = 5°, …, 71 = 355°).
    pub canvases: Vec<Arc<PrerenderedCanvas>>,
    pub virtual_w: u16,
    pub virtual_h: u16,
    pub target_w: u16,
    pub target_h: u16,
}

/// World resource: holds pre-rendered canvases for all eligible OBJ sprites in the active scene.
#[derive(Default)]
pub struct ObjPrerenderedFrames {
    frames: HashMap<String, PrerenderedFrame>,
    /// Animated sprites: 72 yaw keyframes per sprite ID.
    anim: HashMap<String, AnimSpriteFrames>,
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

    pub fn insert_anim(&mut self, sprite_id: String, anim: AnimSpriteFrames) {
        self.anim.insert(sprite_id, anim);
    }

    /// Look up the pre-baked canvas closest to `live_yaw_deg` for an animated sprite.
    /// Returns `(canvas, virtual_w, virtual_h, target_w, target_h)` or `None` if not cached.
    pub fn get_anim_canvas(
        &self,
        sprite_id: &str,
        live_yaw_deg: f32,
    ) -> Option<(&Arc<PrerenderedCanvas>, u16, u16, u16, u16)> {
        let entry = self.anim.get(sprite_id)?;
        let normalized = ((live_yaw_deg % 360.0) + 360.0) % 360.0;
        let index = ((normalized / YAW_STEP_DEG as f32).round() as usize) % YAW_FRAME_COUNT;
        let canvas = entry.canvases.get(index)?;
        Some((canvas, entry.virtual_w, entry.virtual_h, entry.target_w, entry.target_h))
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty() && self.anim.is_empty()
    }

    pub fn len(&self) -> usize {
        self.frames.len() + self.anim.len()
    }
}

/// World resource tracking status of the prerender pass.
pub enum ObjPrerenderStatus {
    /// No prerender scheduled or not yet run.
    Idle,
    /// Prerender complete — cache is populated and ready.
    Ready,
}
