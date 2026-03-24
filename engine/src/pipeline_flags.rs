//! `PipelineFlags` — explicit feature flags for risky render pipeline optimizations.
//!
//! All experimental or unstable paths are off by default. Stable defaults are set here
//! so no code needs to guess what is safe. To re-enable an optimization, flip the flag
//! and verify that no regressions appear in the full scene flow.
//!
//! Registered as a World resource at startup. Read by game_loop and compositor.

/// Feature flags for the render pipeline.
///
/// Designed so `PipelineFlags::default()` is always the safe, stable configuration.
#[derive(Debug, Clone, Copy)]
pub struct PipelineFlags {
    /// Enable async postfx offload to the render thread.
    /// When `false`, every frame renders synchronously on the simulation thread.
    /// Default: `false` (synchronous — stable, predictable frame delivery).
    /// Enable via `--async-render` CLI flag for performance experiments.
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

    /// Allow dirty-region partial composite in Cell/QuadBlock/Braille mode.
    /// When `false`, every compositor pass does a full buffer fill + full layer walk.
    /// Default: `false` (full redraw — stable).
    pub experimental_dirty_cell: bool,

    /// Allow dirty-region partial repack in HalfBlock mode.
    /// When `false`, always runs `pack_halfblock_buffer()` (full repack).
    /// Default: `false` (full repack — stable).
    pub experimental_dirty_halfblock: bool,

    /// Allow adaptive virtual-present sampling (skip unchanged output cells).
    /// Default: `false` (always full virtual present — stable).
    pub adaptive_virtual_present: bool,
}

impl Default for PipelineFlags {
    fn default() -> Self {
        Self {
            async_render_enabled: false,
            full_redraw_on_scene_change: true,
            sync_guard_frame_count: 2,
            lock_renderer_mode_to_scene: true,
            experimental_dirty_cell: false,
            experimental_dirty_halfblock: false,
            adaptive_virtual_present: false,
        }
    }
}
