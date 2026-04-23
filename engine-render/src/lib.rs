//! Abstract render backend traits and helpers for backend-neutral frame presentation.
//!
//! Also exports rasterizer for font rendering across crates.
//!
//! Core types:
//! - `RenderBackend` trait: present frames, query capabilities, shutdown
//! - `RenderFrame`: per-frame data (frame buffer, render canvas size, present mode)
//! - `PreparedWorld` / `PreparedUi` / `PreparedOverlay`: backend-neutral render payloads
//! - `FrameSubmission`: backend-neutral submission envelope for hardware-capable paths
//! - `RenderCaps`: capabilities (resolution, color depth, FPS)

use engine_core::buffer::Buffer;

mod font_loader;
pub mod generic;
pub mod overlay;
pub mod rasterizer;
pub mod simd_text;
mod types;
pub mod vector_overlay;

pub use generic::*;
pub use overlay::{OverlayData, OverlayLine};
pub use rasterizer::{blit, has_font_assets, missing_glyphs, rasterize, rasterize_cached};
pub use simd_text::{rasterize_staged_glyphs, stage_glyph_placement, GlyphBatch};
pub use vector_overlay::{VectorOverlay, VectorPrimitive};

/// High-level render backend family selected at engine startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderBackendKind {
    /// Current CPU/compositor-driven presentation path.
    #[default]
    Software,
    /// Future GPU-first path (planned: winit + wgpu).
    Hardware,
}

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

/// Per-frame render data passed to `RenderBackend::present()`.
pub struct RenderFrame<'a> {
    pub buffer: &'a Buffer,
    /// Logical frame size before backend-specific presentation is applied.
    pub virtual_size: (u16, u16),
    pub present_mode: PresentMode,
}

/// Minimal backend-neutral world payload for hardware-capable submission paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedWorld {
    pub ready: bool,
}

/// Minimal backend-neutral UI payload for hardware-capable submission paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedUi {
    pub ready: bool,
}

/// Minimal backend-neutral overlay payload for hardware-capable submission paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedOverlay {
    pub ready: bool,
    pub line_count: usize,
    pub primitive_count: usize,
}

/// Backend-neutral frame envelope consumed by hardware-capable runtime backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSubmission {
    pub output_size: (u16, u16),
    pub present_mode: PresentMode,
    pub world: PreparedWorld,
    pub ui: PreparedUi,
    pub overlay: PreparedOverlay,
}

/// Minimal metadata handed to a future hardware presentation path.
///
/// Stage 1 keeps this intentionally small. The real GPU path will grow this
/// into world/UI/postfx attachments once `wgpu` is wired in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareFrame {
    pub output_size: (u16, u16),
    pub world_ready: bool,
    pub ui_ready: bool,
}

impl From<HardwareFrame> for FrameSubmission {
    fn from(frame: HardwareFrame) -> Self {
        Self {
            output_size: frame.output_size,
            // Compatibility default for legacy callers that never provided present mode.
            present_mode: PresentMode::VSync,
            world: PreparedWorld {
                ready: frame.world_ready,
            },
            ui: PreparedUi {
                ready: frame.ui_ready,
            },
            overlay: PreparedOverlay {
                ready: false,
                line_count: 0,
                primitive_count: 0,
            },
        }
    }
}

impl From<&HardwareFrame> for FrameSubmission {
    fn from(frame: &HardwareFrame) -> Self {
        (*frame).into()
    }
}

impl From<FrameSubmission> for HardwareFrame {
    fn from(submission: FrameSubmission) -> Self {
        Self {
            output_size: submission.output_size,
            world_ready: submission.world.ready,
            ui_ready: submission.ui.ready,
        }
    }
}

impl From<&FrameSubmission> for HardwareFrame {
    fn from(submission: &FrameSubmission) -> Self {
        (*submission).into()
    }
}

/// Abstract trait for full render backends (software or hardware).
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

/// Common presentation-layer operations shared by software and hardware paths.
///
/// This trait is additive in Stage 1. Existing software backends continue to
/// implement `RendererBackend`; blanket impls below expose them through the new
/// split without breaking the current engine loop.
pub trait PresentationBackend: Send {
    fn backend_kind(&self) -> RenderBackendKind;
    fn submit_frame(&mut self, _submission: &FrameSubmission) -> Result<(), RenderError> {
        Err(RenderError::PresentFailed(
            "frame submission path is not supported by this backend".to_string(),
        ))
    }
    fn present_overlay(&mut self, overlay: &OverlayData);
    fn present_vectors(&mut self, vectors: &VectorOverlay);
    fn output_size(&self) -> (u16, u16);
    fn copy_to_clipboard(&mut self, text: &str) -> bool;
    fn clear(&mut self) -> Result<(), RenderError>;
    fn shutdown(&mut self) -> Result<(), RenderError>;
}

impl<T: RendererBackend + ?Sized> PresentationBackend for T {
    fn backend_kind(&self) -> RenderBackendKind {
        RenderBackendKind::Software
    }

    fn submit_frame(&mut self, submission: &FrameSubmission) -> Result<(), RenderError> {
        RendererBackend::submit_frame(self, submission)
    }

    fn present_overlay(&mut self, overlay: &OverlayData) {
        RendererBackend::present_overlay(self, overlay);
    }

    fn present_vectors(&mut self, vectors: &VectorOverlay) {
        RendererBackend::present_vectors(self, vectors);
    }

    fn output_size(&self) -> (u16, u16) {
        RendererBackend::output_size(self)
    }

    fn copy_to_clipboard(&mut self, text: &str) -> bool {
        RendererBackend::copy_to_clipboard(self, text)
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        RendererBackend::clear(self)
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        RendererBackend::shutdown(self)
    }
}

/// Software presentation path backed by a composed CPU [`Buffer`].
pub trait SoftwareRendererBackend: PresentationBackend {
    fn present_software_frame(&mut self, buffer: &Buffer);
}

impl<T: RendererBackend + ?Sized> SoftwareRendererBackend for T {
    fn present_software_frame(&mut self, buffer: &Buffer) {
        RendererBackend::present_frame(self, buffer);
    }
}

/// Hardware presentation path backed by GPU-rendered attachments or command buffers.
pub trait HardwareRendererBackend: PresentationBackend {
    fn present_hardware_frame(&mut self, frame: &HardwareFrame) -> Result<(), RenderError>;

    fn submit_frame(&mut self, submission: &FrameSubmission) -> Result<(), RenderError> {
        self.present_hardware_frame(&HardwareFrame::from(submission))
    }
}

/// Minimal presentation backend interface used by the live engine loop.
///
/// Unlike `RenderBackend`, this works on the engine's composed frame data and is the
/// abstraction point for interchangeable runtime backends.
///
/// Stage 1 keeps this as the legacy software interface so the existing software path
/// remains untouched while a parallel hardware path is scaffolded.
pub trait RendererBackend: Send {
    fn present_frame(&mut self, buffer: &Buffer);
    /// Report which runtime backend family this renderer represents.
    ///
    /// Default keeps legacy renderers in software mode.
    fn backend_kind(&self) -> RenderBackendKind {
        RenderBackendKind::Software
    }
    /// Present a hardware frame payload.
    ///
    /// Software renderers can ignore this path and keep using `present_frame`.
    fn present_hardware_frame(&mut self, _frame: &HardwareFrame) -> Result<(), RenderError> {
        Err(RenderError::PresentFailed(
            "hardware frame path is not supported by this renderer".to_string(),
        ))
    }
    /// Present a backend-neutral frame submission payload.
    ///
    /// Default compatibility implementation maps to `present_hardware_frame`.
    fn submit_frame(&mut self, submission: &FrameSubmission) -> Result<(), RenderError> {
        self.present_hardware_frame(&HardwareFrame::from(submission))
    }
    /// Render a debug overlay on top of the last presented frame.
    ///
    /// Called after `present_frame`. Lines are drawn directly onto the output
    /// surface at native resolution so text stays readable regardless of frame scaling.
    fn present_overlay(&mut self, overlay: &OverlayData);
    /// Stage vector primitives for native-resolution rendering on the next present.
    ///
    /// Backends with native vector support may draw these directly on the
    /// presentation surface. Backends without vector support may ignore this hook.
    fn present_vectors(&mut self, _vectors: &VectorOverlay) {}
    /// Returns the logical output size before backend-specific window/display presentation
    /// is applied.
    fn output_size(&self) -> (u16, u16);
    /// Copy text to the host clipboard if the backend supports it.
    ///
    /// Returns `true` when the operation succeeded.
    fn copy_to_clipboard(&mut self, _text: &str) -> bool {
        false
    }
    fn clear(&mut self) -> Result<(), RenderError>;
    fn shutdown(&mut self) -> Result<(), RenderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::color::Color;

    #[derive(Default)]
    struct DummyRenderer {
        presented: bool,
        overlay_calls: usize,
        vector_calls: usize,
        copied: Option<String>,
        cleared: bool,
    }

    impl RendererBackend for DummyRenderer {
        fn present_frame(&mut self, _buffer: &Buffer) {
            self.presented = true;
        }

        fn present_overlay(&mut self, _overlay: &OverlayData) {
            self.overlay_calls += 1;
        }

        fn present_vectors(&mut self, _vectors: &VectorOverlay) {
            self.vector_calls += 1;
        }

        fn output_size(&self) -> (u16, u16) {
            (64, 36)
        }

        fn copy_to_clipboard(&mut self, text: &str) -> bool {
            self.copied = Some(text.to_string());
            true
        }

        fn clear(&mut self) -> Result<(), RenderError> {
            self.cleared = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), RenderError> {
            Ok(())
        }
    }

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

    #[test]
    fn render_backend_kind_defaults_to_software() {
        assert_eq!(RenderBackendKind::default(), RenderBackendKind::Software);
    }

    #[test]
    fn legacy_renderer_is_exposed_as_software_backend() {
        let mut renderer = DummyRenderer::default();
        let mut buffer = Buffer::new(2, 2);
        buffer.fill(Color::Black);

        assert_eq!(PresentationBackend::backend_kind(&renderer), RenderBackendKind::Software);
        SoftwareRendererBackend::present_software_frame(&mut renderer, &buffer);

        assert!(renderer.presented);
    }

    #[test]
    fn renderer_backend_defaults_to_software_kind() {
        let renderer = DummyRenderer::default();
        assert_eq!(
            RendererBackend::backend_kind(&renderer),
            RenderBackendKind::Software
        );
    }

    #[test]
    fn renderer_backend_default_hardware_frame_returns_error() {
        let mut renderer = DummyRenderer::default();
        let err = RendererBackend::present_hardware_frame(
            &mut renderer,
            &HardwareFrame {
                output_size: (64, 36),
                world_ready: true,
                ui_ready: true,
            },
        )
        .expect_err("software default should reject hardware frame payload");

        match err {
            RenderError::PresentFailed(msg) => {
                assert!(msg.contains("not supported"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn hardware_frame_converts_to_frame_submission() {
        let frame = HardwareFrame {
            output_size: (100, 50),
            world_ready: true,
            ui_ready: false,
        };
        let submission: FrameSubmission = frame.into();
        assert_eq!(submission.output_size, (100, 50));
        assert_eq!(submission.present_mode, PresentMode::VSync);
        assert!(submission.world.ready);
        assert!(!submission.ui.ready);
    }

    #[test]
    fn frame_submission_converts_to_hardware_frame() {
        let submission = FrameSubmission {
            output_size: (128, 72),
            present_mode: PresentMode::Immediate,
            world: PreparedWorld { ready: true },
            ui: PreparedUi { ready: true },
            overlay: PreparedOverlay {
                ready: true,
                line_count: 5,
                primitive_count: 7,
            },
        };
        let frame: HardwareFrame = submission.into();
        assert_eq!(frame.output_size, (128, 72));
        assert!(frame.world_ready);
        assert!(frame.ui_ready);
    }

    #[derive(Default)]
    struct DummyHardwareRenderer {
        last_frame: Option<HardwareFrame>,
    }

    impl RendererBackend for DummyHardwareRenderer {
        fn present_frame(&mut self, _buffer: &Buffer) {}

        fn backend_kind(&self) -> RenderBackendKind {
            RenderBackendKind::Hardware
        }

        fn present_hardware_frame(&mut self, frame: &HardwareFrame) -> Result<(), RenderError> {
            self.last_frame = Some(*frame);
            Ok(())
        }

        fn present_overlay(&mut self, _overlay: &OverlayData) {}

        fn output_size(&self) -> (u16, u16) {
            (64, 36)
        }

        fn clear(&mut self) -> Result<(), RenderError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), RenderError> {
            Ok(())
        }
    }

    #[test]
    fn renderer_backend_submit_frame_defaults_to_hardware_path() {
        let mut renderer = DummyHardwareRenderer::default();
        let submission = FrameSubmission {
            output_size: (64, 36),
            present_mode: PresentMode::VSync,
            world: PreparedWorld { ready: true },
            ui: PreparedUi { ready: true },
            overlay: PreparedOverlay {
                ready: false,
                line_count: 0,
                primitive_count: 0,
            },
        };
        RendererBackend::submit_frame(&mut renderer, &submission).expect("submit frame");
        assert_eq!(
            renderer.last_frame,
            Some(HardwareFrame {
                output_size: (64, 36),
                world_ready: true,
                ui_ready: true,
            })
        );
    }

    #[test]
    fn presentation_backend_submit_frame_delegates() {
        let mut renderer = DummyHardwareRenderer::default();
        let submission = FrameSubmission {
            output_size: (80, 45),
            present_mode: PresentMode::VSync,
            world: PreparedWorld { ready: false },
            ui: PreparedUi { ready: true },
            overlay: PreparedOverlay {
                ready: false,
                line_count: 0,
                primitive_count: 0,
            },
        };
        PresentationBackend::submit_frame(&mut renderer, &submission).expect("submit frame");
        assert_eq!(
            renderer.last_frame,
            Some(HardwareFrame {
                output_size: (80, 45),
                world_ready: false,
                ui_ready: true,
            })
        );
    }

    #[test]
    fn software_renderer_submit_frame_returns_error() {
        let mut renderer = DummyRenderer::default();
        let submission = FrameSubmission {
            output_size: (64, 36),
            present_mode: PresentMode::VSync,
            world: PreparedWorld { ready: true },
            ui: PreparedUi { ready: true },
            overlay: PreparedOverlay {
                ready: false,
                line_count: 0,
                primitive_count: 0,
            },
        };
        let err = RendererBackend::submit_frame(&mut renderer, &submission)
            .expect_err("software backend should reject submission by default");
        match err {
            RenderError::PresentFailed(msg) => assert!(msg.contains("not supported")),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
