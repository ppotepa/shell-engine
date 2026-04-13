//! Render pipeline configuration: flags, strategy traits, and strategy aggregation.
//!
//! `PipelineFlags` holds startup-time CLI options. Strategy traits define the
//! abstract interfaces for each pipeline stage. `PipelineStrategies` aggregates
//! one concrete strategy per stage into a single World resource.

pub mod strategies;

pub use strategies::{
    AlwaysPresenter,
    AlwaysRender,
    CoordinatedSkip,
    // Simple impls (no engine/ deps)
    DirectLayerCompositor,
    FrameSkipOracle,
    HashSkipPresenter,
    // Trait definitions
    LayerCompositor,
    // Aggregation
    PipelineStrategies,
    ScratchLayerCompositor,
    VirtualPresenter,
};

/// Feature flags for the render pipeline.
///
/// `PipelineFlags::default()` is always the safe, stable configuration.
/// The `opt_*` fields are read once at startup to construct `PipelineStrategies`.
/// Runtime systems use `PipelineStrategies`, not these booleans directly.
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

    /// `--opt-comp`: Compositor optimizations.
    /// Gates #4 (skip scratch buffer for effectless layers) and
    /// #5 (dirty-region narrowing in compositing).
    /// Default: `false` (full redraw every frame — stable).
    pub opt_comp: bool,

    /// `--opt-present`: Virtual-to-output present optimizations.
    /// Gates #13 (hash-based frame skip when virtual buffer is unchanged).
    /// **DEPRECATED:** Use `--opt-skip` instead for unified frame-skip coordination.
    /// Default: `false` (always full present — stable).
    pub opt_present: bool,

    /// `--opt-diff`: Use dirty-region scan in diff_into instead of full-buffer scan.
    /// ONLY safe when fill() is guaranteed before every diff and reset_dirty() is not
    /// called after fill(). Experimental — off by default to avoid artifact bugs.
    /// Default: `false` (always full-buffer scan — stable).
    pub opt_diff: bool,

    /// `--opt-skip`: Unified frame-skip oracle (coordinated PostFX cache + Presenter hash).
    /// Prevents desynchronization between independent skip mechanisms that caused animation
    /// flickering. PostFX cache and Presenter skip decisions are now atomic — both skip or
    /// both render, never disagree.
    /// Default: `false` (always full render — stable).
    pub opt_skip: bool,

    /// `--opt-rowdiff`: Row-level dirty skip in diff scan.
    /// Skips entire rows marked not dirty, avoiding per-cell comparisons.
    /// Safe: dirty_rows only set to true during frame, reset after swap().
    /// Up to ~10-20% faster on frames with static regions (e.g., UI background).
    /// Default: `false` (always full-buffer scan — stable).
    pub opt_rowdiff: bool,

    /// `--opt-async`: Async display sink for I/O offload.
    /// Decouple main thread from output write/flush latency.
    /// Renderer submits immutable frame data to a background thread; main thread starts
    /// the next frame immediately.
    /// Expected: 1-5ms/frame unblocked on slower output backends.
    /// Default: `false` (sync flush — no latency decoupling).
    pub opt_async_display: bool,
}

impl Default for PipelineFlags {
    fn default() -> Self {
        Self {
            async_render_enabled: false,
            full_redraw_on_scene_change: true,
            sync_guard_frame_count: 2,
            opt_comp: false,
            opt_present: false,
            opt_diff: false,
            opt_skip: false,
            opt_rowdiff: false,
            opt_async_display: false,
        }
    }
}
