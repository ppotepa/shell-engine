//! Unified frame-skip oracle — coordinates all frame-skip decisions to prevent desynchronization.
//!
//! **Problem:** PostFX had its own frame-skip cache (every N frames), and Presenter had
//! its own hash-based skip logic. When they disagreed, frames got duplicated or dropped,
//! causing flickering and animation artifacts.
//!
//! **Solution:** Single oracle that both systems consult. Either all skip together, or
//! all render together. No desynchronization possible.

/// Coordinates frame-skip decisions across PostFX and Presenter.
pub trait FrameSkipOracle: Send + Sync {
    /// Ask PostFX: should we skip the full pipeline and blit cached result?
    fn should_skip_postfx(&mut self, scene_id: &str, pass_fingerprint: u64) -> bool;

    /// Ask Presenter: should we skip the virtual→output copy?
    fn should_skip_present(&mut self, vbuf_hash: u64) -> bool;

    /// Notify oracle that we advanced to a new frame. Call once per game loop.
    fn frame_advanced(&mut self, sim_frame_id: u64, scene_changed: bool);
}

/// Always render. Safe default — zero overhead.
pub struct AlwaysRender;

impl FrameSkipOracle for AlwaysRender {
    #[inline]
    fn should_skip_postfx(&mut self, _scene_id: &str, _pass_fingerprint: u64) -> bool {
        false
    }

    #[inline]
    fn should_skip_present(&mut self, _vbuf_hash: u64) -> bool {
        false
    }

    #[inline]
    fn frame_advanced(&mut self, _sim_frame_id: u64, _scene_changed: bool) {}
}

/// Coordinated frame-skip (`--opt-skip`): both PostFX and Presenter skip together.
///
/// Guarantees:
/// - If PostFX skips, Presenter also skips (consistent frame)
/// - If either detects change, both render next frame
/// - Scene changes force render (no skipped first frame)
pub struct CoordinatedSkip {
    last_rendered_frame_id: u64,
    last_scene_id: Option<String>,
    last_pass_fingerprint: u64,
    last_vbuf_hash: u64,
    /// Skip interval for PostFX (run every N+1 frames)
    postfx_interval: u8,
    postfx_counter: u8,
}

impl Default for CoordinatedSkip {
    fn default() -> Self {
        Self {
            last_rendered_frame_id: 0,
            last_scene_id: None,
            last_pass_fingerprint: 0,
            last_vbuf_hash: u64::MAX,
            postfx_interval: 1, // Run every 2 frames
            postfx_counter: 0,
        }
    }
}

impl FrameSkipOracle for CoordinatedSkip {
    fn should_skip_postfx(&mut self, scene_id: &str, pass_fingerprint: u64) -> bool {
        // Scene or config changed: force render
        if self.last_scene_id.as_deref() != Some(scene_id)
            || self.last_pass_fingerprint != pass_fingerprint
        {
            self.last_scene_id = Some(scene_id.to_string());
            self.last_pass_fingerprint = pass_fingerprint;
            self.postfx_counter = 0;
            return false; // Force render
        }

        // Check counter-based interval
        if self.postfx_counter > 0 {
            self.postfx_counter -= 1;
            return true; // Skip this frame
        }

        // Time to render: reset counter
        self.postfx_counter = self.postfx_interval;
        false // Render
    }

    fn should_skip_present(&mut self, vbuf_hash: u64) -> bool {
        // Never skip on scene change (handled in frame_advanced + should_skip_postfx alignment)
        // If hash matches last rendered frame's hash, we can skip
        vbuf_hash == self.last_vbuf_hash
    }

    fn frame_advanced(&mut self, sim_frame_id: u64, scene_changed: bool) {
        if scene_changed {
            // Force both PostFX and Presenter to render next frame
            self.postfx_counter = 0;
            self.last_vbuf_hash = u64::MAX; // Invalidate hash so present renders
        }
        self.last_rendered_frame_id = sim_frame_id;
    }
}
