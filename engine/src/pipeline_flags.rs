//! `PipelineFlags` — explicit feature flags for risky render pipeline optimizations.
//!
//! All experimental or unstable paths are off by default. Stable defaults are set here
//! so no code needs to guess what is safe. To re-enable an optimization, flip the flag
//! and verify that no regressions appear in the full scene flow.
//!
//! Registered as a World resource at startup. Read by compositor and renderer.

/// Feature flags for the render pipeline.
///
/// Designed so `PipelineFlags::default()` is always the safe, stable configuration.
#[derive(Debug, Clone, Copy)]
pub struct PipelineFlags {
    /// Enable async postfx offload to the render thread.
    /// When `false`, every frame renders synchronously on the simulation thread.
    /// Default: `false` (synchronous — stable, predictable frame delivery).
    pub async_render_enabled: bool,

    /// Invalidate buffers and force N sync frames on scene transition or resize.
    /// Prevents stale content from leaking into a new scene.
    /// Default: `true`.
    pub full_redraw_on_scene_change: bool,

    /// Number of synchronous frames forced after a scene/resize barrier.
    /// Default: `2`.
    pub sync_guard_frame_count: u8,

    /// Lock the renderer mode (`Cell`/`HalfBlock`/etc.) to the value active at scene entry.
    /// Prevents mid-scene renderer mode drift from adaptive or hot-reload changes.
    /// Default: `true`.
    pub lock_renderer_mode_to_scene: bool,

    /// `--opt-comp`: Compositor optimizations.
    /// Gates #4 (skip scratch buffer for effectless layers) and
    /// #5 (dirty-region narrowing in halfblock packing).
    /// Default: `false` (full redraw every frame — stable).
    pub opt_comp: bool,

    /// `--opt-present`: Virtual-to-output present optimizations.
    /// Gates #13 (hash-based frame skip when virtual buffer is unchanged).
    /// Default: `false` (always full present — stable).
    pub opt_present: bool,

    /// `--opt-diff`: Use dirty-region scan in diff_into instead of full-buffer scan.
    /// ONLY safe when fill() is guaranteed before every diff and reset_dirty() is not
    /// called after fill(). Experimental — off by default to avoid artifact bugs.
    /// Default: `false` (always full-buffer scan — stable).
    pub opt_diff: bool,
}

impl Default for PipelineFlags {
    fn default() -> Self {
        Self {
            async_render_enabled: false,
            full_redraw_on_scene_change: true,
            sync_guard_frame_count: 2,
            lock_renderer_mode_to_scene: true,
            opt_comp: false,
            opt_present: false,
            opt_diff: false,
        }
    }
}
