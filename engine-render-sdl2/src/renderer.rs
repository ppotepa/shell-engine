use engine_render::{OutputBackend, OverlayData, RenderError, VectorOverlay};
use engine_runtime::PresentationPolicy;

use crate::input::Sdl2InputBackend;
use crate::runtime::{
    sdl_profile_enabled, CellPatch, RuntimeCommand, RuntimeResponse, Sdl2RuntimeClient,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_PIXEL_SCALE: u32 = 8;
pub const LOGICAL_CELL_WIDTH: u32 = 1;
pub const LOGICAL_CELL_HEIGHT: u32 = 2;

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

impl OutputBackend for Sdl2Backend {
    fn present_buffer(&mut self, buffer: &engine_core::buffer::Buffer) {
        let overlay = self.pending_overlay.take();
        let vectors = self.pending_vectors.take();
        let t_diff = Instant::now();
        let mut patches = Vec::<CellPatch>::new();
        buffer.diff_into(&mut patches);
        let diff_dur = t_diff.elapsed();
        if patches.is_empty() && overlay.is_none() && vectors.is_none() {
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
    fn sdl_scaling_constants_are_non_zero() {
        assert!(DEFAULT_PIXEL_SCALE > 0);
        assert!(LOGICAL_CELL_WIDTH > 0);
        assert!(LOGICAL_CELL_HEIGHT > 0);
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
