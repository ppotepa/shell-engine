//! Scene3D runtime definition store — holds parsed `Scene3DDefinition` objects for real-time
//! rendering, as opposed to the [`Scene3DAtlas`] which holds pre-baked `Buffer` snapshots.
//!
//! Real-time rendering path:
//! 1. At scene preparation time [`Scene3DPrerenderStep`] builds this store alongside the atlas.
//! 2. The compositor injects it via [`with_runtime_store`] for the duration of each frame.
//! 3. [`render_scene_clip_sprite`] checks if the requested `frame` is a live clip name in this store;
//!    if so it calls [`render_scene3d_frame_at`] to render on demand at the current `elapsed_ms`.
//!
//! Thread-local pointer pattern mirrors `Scene3DAtlas` — zero allocation / zero borrow on lookup.

use std::cell::Cell;
use std::collections::HashMap;

use engine_3d::scene3d_format::Scene3DDefinition;

pub struct Scene3DRuntimeEntry {
    pub def: Scene3DDefinition,
}

/// World resource: parsed Scene3D definitions for all `.scene3d.yml` sources referenced by the
/// active scene. Used by the real-time rendering path in [`render_scene_clip_sprite`].
#[derive(Default)]
pub struct Scene3DRuntimeStore {
    entries: HashMap<String, Scene3DRuntimeEntry>,
}

impl Scene3DRuntimeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, src: impl Into<String>, entry: Scene3DRuntimeEntry) {
        self.entries.insert(src.into(), entry);
    }

    pub fn get(&self, src: &str) -> Option<&Scene3DRuntimeEntry> {
        self.entries.get(src)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Thread-local pointer for zero-overhead access inside compositor ────────────

thread_local! {
    static STORE_PTR: Cell<*const Scene3DRuntimeStore> = const { Cell::new(std::ptr::null()) };
}

/// Inject the runtime store pointer for the duration of `f`, then clear it.
///
/// # Safety
/// `store` must remain valid for the entire duration of `f`.
pub fn with_runtime_store<R>(store: Option<&Scene3DRuntimeStore>, f: impl FnOnce() -> R) -> R {
    let ptr = store.map(|s| s as *const _).unwrap_or(std::ptr::null());
    STORE_PTR.with(|cell| cell.set(ptr));
    let result = f();
    STORE_PTR.with(|cell| cell.set(std::ptr::null()));
    result
}

impl Scene3DRuntimeStore {
    /// Look up a runtime entry from the thread-local store pointer (zero allocation on hot path).
    /// Returns `None` if no store is set or the source is not registered.
    pub fn current_get(src: &str) -> Option<&'static Scene3DRuntimeEntry> {
        STORE_PTR.with(|cell| {
            let ptr = cell.get();
            if ptr.is_null() {
                return None;
            }
            // SAFETY: ptr was set from a valid reference in `with_runtime_store`; the reference
            // remains live for the entire duration of the compositor frame. We extend the lifetime
            // to 'static here, but only expose it within the same call stack as `with_runtime_store`.
            let store = unsafe { &*ptr };
            let entry = store.get(src)?;
            Some(unsafe { &*(entry as *const Scene3DRuntimeEntry) })
        })
    }
}
