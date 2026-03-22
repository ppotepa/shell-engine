//! Scene3D prerendered frame atlas — holds one Buffer per named frame per scene3d asset.
//!
//! Populated by [`Scene3DPrerenderStep`] during scene transition (before first frame).
//! Accessed at render time via a thread-local pointer set by the compositor for the duration
//! of each frame (same pattern as `PRERENDER_FRAMES_PTR` in obj_render.rs).

use crate::buffer::Buffer;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

/// Key: `"{src}::{frame_id}"` — unique per (scene3d file path, named frame).
pub type AtlasKey = String;

/// World resource: all prerendered Scene3D buffers for the active scene.
#[derive(Default)]
pub struct Scene3DAtlas {
    frames: HashMap<AtlasKey, Arc<Buffer>>,
}

impl Scene3DAtlas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, src: &str, frame_id: &str, buf: Buffer) {
        self.frames.insert(Self::key(src, frame_id), Arc::new(buf));
    }

    pub fn get(&self, src: &str, frame_id: &str) -> Option<Arc<Buffer>> {
        self.frames.get(&Self::key(src, frame_id)).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    fn key(src: &str, frame_id: &str) -> AtlasKey {
        format!("{src}::{frame_id}")
    }
}

// ── Thread-local pointer for zero-overhead atlas access inside compositor ────

thread_local! {
    static ATLAS_PTR: Cell<*const Scene3DAtlas> = Cell::new(std::ptr::null());
}

/// Set the atlas pointer for the duration of `f`, then clear it.
/// Called once per frame by the compositor before rendering sprites.
///
/// # Safety
/// `atlas` must remain valid for the entire duration of `f`.
pub fn with_atlas<R>(atlas: Option<&Scene3DAtlas>, f: impl FnOnce() -> R) -> R {
    let ptr = atlas.map(|a| a as *const _).unwrap_or(std::ptr::null());
    ATLAS_PTR.with(|cell| cell.set(ptr));
    let result = f();
    ATLAS_PTR.with(|cell| cell.set(std::ptr::null()));
    result
}

impl Scene3DAtlas {
    /// Look up a frame from the thread-local atlas pointer (zero borrow overhead).
    /// Returns `None` if no atlas is set or the frame does not exist.
    pub fn current_get(src: &str, frame_id: &str) -> Option<Arc<Buffer>> {
        ATLAS_PTR.with(|cell| {
            let ptr = cell.get();
            if ptr.is_null() {
                return None;
            }
            // SAFETY: ptr was set from a valid reference in `with_atlas`, still live.
            let atlas = unsafe { &*ptr };
            atlas.get(src, frame_id)
        })
    }
}
