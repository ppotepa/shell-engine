//! Pre-baked OBJ frame cache: stores rendered canvases keyed by (source, wireframe, yaw_step).

use std::collections::HashMap;

/// Yaw quantisation step in degrees. 72 steps per full rotation.
pub const YAW_STEP_DEG: u16 = 5;

/// Cache key: OBJ source path + wireframe flag + snapped yaw angle.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BakeCacheKey {
    pub source: String,
    pub wireframe: bool,
    /// Snapped yaw in degrees: 0, 5, 10, ..., 355.
    pub yaw_step: u16,
}

/// A flat RGBA canvas: one `Option<[u8; 3]>` per virtual pixel (row-major, width × height).
pub type BakedCanvas = Vec<Option<[u8; 3]>>;

/// Holds pre-baked canvases for static OBJ sprites.
pub struct ObjFrameCache {
    frames: HashMap<BakeCacheKey, Arc<BakedCanvas>>,
}

impl ObjFrameCache {
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: BakeCacheKey, canvas: BakedCanvas) {
        self.frames.insert(key, Arc::new(canvas));
    }

    pub fn get(&self, key: &BakeCacheKey) -> Option<&Arc<BakedCanvas>> {
        self.frames.get(key)
    }

    /// Snap an arbitrary yaw angle (degrees) to the nearest `YAW_STEP_DEG` multiple.
    pub fn snap_yaw(yaw_deg: f32) -> u16 {
        let normalized = ((yaw_deg % 360.0) + 360.0) % 360.0;
        let step = (normalized / YAW_STEP_DEG as f32).round() as u16 % (360 / YAW_STEP_DEG);
        step * YAW_STEP_DEG
    }
}

impl Default for ObjFrameCache {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::Arc;

/// World resource tracking status of the prerender pass.
pub enum ObjBakeStatus {
    /// No prerender scheduled.
    Idle,
    /// Prerender complete — cache is populated and ready.
    Ready,
}
