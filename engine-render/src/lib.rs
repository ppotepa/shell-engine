//! Abstract render backend trait enabling pluggable rendering (Terminal, OpenGL, D3D, Vulkan, WebGL).
//!
//! Core types:
//! - `RenderBackend` trait: present frames, query capabilities, shutdown
//! - `DisplaySink` trait: queue and flush frames (may be async)
//! - `RenderFrame`: per-frame data (buffer, virtual size, present mode)
//! - `RenderCaps`: capabilities (resolution, color depth, FPS)

use engine_core::buffer::Buffer;

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
    pub virtual_size: (u16, u16),
    pub present_mode: PresentMode,
}

/// Abstract trait for render backends (Terminal, OpenGL, D3D, Vulkan, WebGL, etc).
/// 
/// Implementations handle all rendering details; the engine calls these methods
/// each frame without knowing which backend is active.
pub trait RenderBackend: Send {
    /// Present the frame buffer to the render target.
    /// Backends may batch, async-queue, or block depending on implementation.
    fn present(&self, frame: &RenderFrame) -> Result<(), RenderError>;

    /// Query render capabilities (resolution, color depth, FPS limits).
    fn capabilities(&self) -> RenderCaps;

    /// Gracefully shut down the backend, cleaning up resources and restoring state.
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
