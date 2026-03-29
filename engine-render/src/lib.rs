//! Abstract render backend trait enabling pluggable rendering (Terminal, OpenGL, D3D, Vulkan, WebGL).
//!
//! Also exports rasterizer for font rendering across crates.
//!
//! Core types:
//! - `RenderBackend` trait: present frames, query capabilities, shutdown
//! - `DisplaySink` trait: queue and flush frames (may be async)
//! - `RenderFrame`: per-frame data (buffer, render canvas size, present mode)
//! - `RenderCaps`: capabilities (resolution, color depth, FPS)

use engine_core::buffer::Buffer;

mod font_loader;
pub mod generic;
pub mod image_loader;
pub mod overlay;
pub mod rasterizer;
pub mod simd_text;
mod types;
pub mod vector_overlay;

pub use generic::*;
pub use overlay::{OverlayData, OverlayLine};
pub use rasterizer::{blit, has_font_assets, missing_glyphs, rasterize, rasterize_cached};
pub use simd_text::{stage_glyph_placement, rasterize_staged_glyphs, GlyphBatch};
pub use vector_overlay::{VectorOverlay, VectorPrimitive};

/// Error type for render backend operations
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("render initialization failed: {0}")]
    InitFailed(String),
    #[error("render present failed: {0}")]
    PresentFailed(String),
    #[error("render shutdown failed: {0}")]
    ShutdownFailed(String),
    #[error("render capability query failed: {0}")]
    CapabilityError(String),
}

/// Color depth capability of the render target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    EightColor,
    SixteenColor,
    TwoFiftySix,
    TrueColor,
}

/// Frame presentation mode (VSync, immediate, or mailbox)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    /// No synchronization; present immediately
    Immediate,
    /// Wait for monitor refresh cycle
    VSync,
    /// Triple-buffer mailbox (discards old frames if present hasn't consumed)
    Mailbox,
}

/// Capabilities reported by the render backend
#[derive(Debug, Clone)]
pub struct RenderCaps {
    pub width: u16,
    pub height: u16,
    pub color_depth: ColorDepth,
    pub vsync_capable: bool,
    pub max_fps: u16,
}

/// Per-frame render data passed to `RenderBackend::present()`
pub struct RenderFrame<'a> {
    pub buffer: &'a Buffer,
    /// Render canvas size. The field name is legacy terminology.
    pub virtual_size: (u16, u16),
    pub present_mode: PresentMode,
}

/// Abstract trait for render backends (Terminal, OpenGL, D3D, Vulkan, WebGL, etc).
///
/// Implementations handle all rendering details; the engine calls these methods
/// each frame without knowing which backend is active.
pub trait RenderBackend {
    /// Present the frame buffer to the render target.
    /// Backends may batch, async-queue, or block depending on implementation.
    fn present(&self, frame: &RenderFrame) -> Result<(), RenderError>;

    /// Query render capabilities (resolution, color depth, FPS limits).
    fn capabilities(&self) -> RenderCaps;

    /// Gracefully shut down the backend, cleaning up resources and restoring state.
    fn shutdown(&mut self) -> Result<(), RenderError>;
}

/// Minimal backend interface used by the live engine loop.
///
/// Unlike `RenderBackend`, this works on already-diffed cell output and is the
/// abstraction point for interchangeable runtime backends.
pub trait OutputBackend: Send {
    fn present_buffer(&mut self, buffer: &Buffer);
    /// Render a debug overlay on top of the last presented frame.
    ///
    /// Called after `present_buffer`. Lines are drawn directly onto the output
    /// surface (terminal or window) at native resolution, bypassing the game
    /// buffer so text is always readable regardless of game scaling.
    fn present_overlay(&mut self, overlay: &OverlayData);
    /// Stage vector primitives for native-resolution rendering on the next present.
    ///
    /// Pixel backends (SDL2) draw these directly on the canvas, bypassing the
    /// character-cell buffer for smooth polygon/line output. Terminal backends
    /// ignore this (vectors are already rasterized to glyphs by the compositor).
    fn present_vectors(&mut self, _vectors: &VectorOverlay) {}
    /// Returns the logical output grid size that the engine composes into before
    /// backend-specific window/display presentation is applied.
    fn output_size(&self) -> (u16, u16);
    fn clear(&mut self) -> Result<(), RenderError>;
    fn shutdown(&mut self) -> Result<(), RenderError>;
}

/// Trait for display sinks (may buffer/batch/async-queue frames).
pub trait DisplaySink: Send {
    /// Queue a frame for output (may batch internally)
    fn queue_frame(&self, buffer: &Buffer, timestamp_ns: u64) -> Result<(), RenderError>;

    /// Flush queued frames (called each frame by game loop)
    fn flush(&self) -> Result<(), RenderError>;

    /// Drain and close (called during shutdown)
    fn drain(&mut self) -> Result<(), RenderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_caps_has_reasonable_defaults() {
        let caps = RenderCaps {
            width: 256,
            height: 100,
            color_depth: ColorDepth::TrueColor,
            vsync_capable: true,
            max_fps: 120,
        };
        assert_eq!(caps.width, 256);
    }

    #[test]
    fn present_modes_are_distinct() {
        let a = PresentMode::Immediate;
        let b = PresentMode::VSync;
        assert_ne!(a, b);
    }

    #[test]
    fn color_depths_are_distinct() {
        let a = ColorDepth::TrueColor;
        let b = ColorDepth::TwoFiftySix;
        assert_ne!(a, b);
    }
}
