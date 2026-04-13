use engine_render::{OverlayData, RenderError, RendererBackend, VectorOverlay};
use engine_runtime::PresentationPolicy;

use crate::input::Sdl2InputBackend;
use crate::runtime::{
    sdl_profile_enabled, GlyphPatch, PixelCanvasData, RuntimeCommand, RuntimeResponse,
    Sdl2RuntimeClient,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_PIXEL_SCALE: u32 = 1;

pub struct Sdl2Backend {
    client: Arc<Mutex<Sdl2RuntimeClient>>,
    width: u16,
    height: u16,
    pending_overlay: Option<OverlayData>,
    pending_vectors: Option<VectorOverlay>,
    profile: Option<BackendProfile>,
}

struct BackendProfile {
    frames: u64,
    sent_frames: u64,
    total_patches: u64,
    max_patches: usize,
    total_diff: Duration,
    total_request: Duration,
    last_emit: Instant,
}

impl BackendProfile {
    fn new() -> Self {
        Self {
            frames: 0,
            sent_frames: 0,
            total_patches: 0,
            max_patches: 0,
            total_diff: Duration::ZERO,
            total_request: Duration::ZERO,
            last_emit: Instant::now(),
        }
    }

    fn record(&mut self, patch_count: usize, diff: Duration, request: Duration, sent: bool) {
        self.frames = self.frames.saturating_add(1);
        if sent {
            self.sent_frames = self.sent_frames.saturating_add(1);
        }
        self.total_patches = self.total_patches.saturating_add(patch_count as u64);
        self.max_patches = self.max_patches.max(patch_count);
        self.total_diff += diff;
        self.total_request += request;

        if self.last_emit.elapsed() < Duration::from_secs(1) {
            return;
        }
        let frames = self.frames.max(1);
        let avg_patches = self.total_patches as f64 / frames as f64;
        let avg_diff_us = self.total_diff.as_micros() as f64 / frames as f64;
        let avg_req_us = self.total_request.as_micros() as f64 / frames as f64;
        let sent_ratio = self.sent_frames as f64 * 100.0 / frames as f64;
        engine_core::logging::debug(
            "sdl2.backend",
            format!(
                "fps_window={} sent={:.1}% avg_patches={:.1} max_patches={} avg_us(diff/request)={:.0}/{:.0}",
                frames, sent_ratio, avg_patches, self.max_patches, avg_diff_us, avg_req_us
            ),
        );

        self.frames = 0;
        self.sent_frames = 0;
        self.total_patches = 0;
        self.max_patches = 0;
        self.total_diff = Duration::ZERO;
        self.total_request = Duration::ZERO;
        self.last_emit = Instant::now();
    }
}

impl Sdl2Backend {
    pub fn new_pair(
        width: u16,
        height: u16,
        presentation_policy: PresentationPolicy,
        window_ratio: Option<(u32, u32)>,
        pixel_scale: u32,
        vsync: bool,
    ) -> Result<(Self, Sdl2InputBackend), String> {
        let client = Arc::new(Mutex::new(Sdl2RuntimeClient::spawn(
            width,
            height,
            presentation_policy,
            window_ratio,
            pixel_scale,
            vsync,
        )?));
        Ok((
            Self {
                client: Arc::clone(&client),
                width,
                height,
                pending_overlay: None,
                pending_vectors: None,
                profile: if sdl_profile_enabled() {
                    Some(BackendProfile::new())
                } else {
                    None
                },
            },
            Sdl2InputBackend::from_client(client),
        ))
    }

    fn request(&self, command: RuntimeCommand) -> Result<RuntimeResponse, RenderError> {
        self.client
            .lock()
            .expect("sdl2 runtime client poisoned")
            .request(command)
            .map_err(RenderError::PresentFailed)
    }

    /// Enable/disable splash presentation mode.
    ///
    /// When enabled, runtime presents using aspect-preserving `Fit` policy
    /// regardless of the scene's configured presentation policy.
    pub fn set_splash_mode(&self, enabled: bool) -> Result<(), RenderError> {
        match self.request(RuntimeCommand::SetSplashMode(enabled))? {
            RuntimeResponse::Ack | RuntimeResponse::Input(_) => Ok(()),
        }
    }
}

impl RendererBackend for Sdl2Backend {
    fn present_frame(&mut self, buffer: &engine_core::buffer::Buffer) {
        let overlay = self.pending_overlay.take();
        let vectors = self.pending_vectors.take();

        // ── Extract pixel canvas data for direct SDL2 upload ─────────────
        let pixel_canvas_data = buffer
            .pixel_canvas
            .as_ref()
            .filter(|pc| pc.dirty)
            .map(|pc| PixelCanvasData {
                data: pc.data.clone(),
                width: pc.width as u32,
                height: pc.height as u32,
            });

        let t_diff = Instant::now();
        let mut patches = Vec::<GlyphPatch>::new();
        buffer.diff_into(&mut patches);
        let diff_dur = t_diff.elapsed();

        // ── Flicker diagnostic: log patch counts at ~1 Hz ────────────────────
        {
            static DIAG_FRAME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            static DIAG_ZERO_RUNS: std::sync::atomic::AtomicU64 =
                std::sync::atomic::AtomicU64::new(0);
            static DIAG_NONZERO_RUNS: std::sync::atomic::AtomicU64 =
                std::sync::atomic::AtomicU64::new(0);
            static DIAG_MAX_PATCH: std::sync::atomic::AtomicU64 =
                std::sync::atomic::AtomicU64::new(0);
            static DIAG_MIN_PATCH: std::sync::atomic::AtomicU64 =
                std::sync::atomic::AtomicU64::new(u64::MAX);
            let fnum = DIAG_FRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let plen = patches.len() as u64;
            if patches.is_empty() {
                DIAG_ZERO_RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                DIAG_NONZERO_RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                DIAG_MAX_PATCH.fetch_max(plen, std::sync::atomic::Ordering::Relaxed);
                DIAG_MIN_PATCH.fetch_min(plen, std::sync::atomic::Ordering::Relaxed);
            }
            if fnum % 60 == 0 && fnum > 0 {
                let zr = DIAG_ZERO_RUNS.swap(0, std::sync::atomic::Ordering::Relaxed);
                let nz = DIAG_NONZERO_RUNS.swap(0, std::sync::atomic::Ordering::Relaxed);
                let mx = DIAG_MAX_PATCH.swap(0, std::sync::atomic::Ordering::Relaxed);
                let mn = DIAG_MIN_PATCH.swap(u64::MAX, std::sync::atomic::Ordering::Relaxed);
                let mn_display = if mn == u64::MAX { 0 } else { mn };
                engine_core::logging::info(
                    "sdl2.flicker_diag",
                    format!(
                        "f={fnum} zero={zr} nonzero={nz} patch_min={mn_display} patch_max={mx}"
                    ),
                );
            }
        }

        let has_pixel_canvas = pixel_canvas_data.is_some();
        if patches.is_empty() && overlay.is_none() && vectors.is_none() && !has_pixel_canvas {
            if let Some(profile) = self.profile.as_mut() {
                profile.record(0, diff_dur, Duration::ZERO, false);
            }
            return;
        }
        let patch_count = patches.len();
        let t_req = Instant::now();
        let _ = self.request(RuntimeCommand::Present {
            width: buffer.width,
            height: buffer.height,
            patches,
            overlay,
            vectors,
            pixel_canvas: pixel_canvas_data,
        });
        if let Some(profile) = self.profile.as_mut() {
            profile.record(patch_count, diff_dur, t_req.elapsed(), true);
        }
    }

    fn present_overlay(&mut self, overlay: &OverlayData) {
        self.pending_overlay = Some(overlay.clone());
    }

    fn present_vectors(&mut self, vectors: &VectorOverlay) {
        self.pending_vectors = Some(vectors.clone());
    }

    fn output_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        match self.request(RuntimeCommand::Clear)? {
            RuntimeResponse::Ack => Ok(()),
            RuntimeResponse::Input(_) => Ok(()),
        }
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        match self.request(RuntimeCommand::Shutdown)? {
            RuntimeResponse::Ack => Ok(()),
            RuntimeResponse::Input(_) => Ok(()),
        }
    }
}

impl Drop for Sdl2Backend {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_scale_is_non_zero() {
        assert!(DEFAULT_PIXEL_SCALE > 0);
    }

    #[test]
    fn backend_reports_requested_size() {
        let backend = Sdl2Backend {
            client: Arc::new(Mutex::new(Sdl2RuntimeClient::disconnected_for_tests())),
            width: 120,
            height: 40,
            pending_overlay: None,
            pending_vectors: None,
            profile: None,
        };
        assert_eq!(backend.output_size(), (120, 40));
    }
}
