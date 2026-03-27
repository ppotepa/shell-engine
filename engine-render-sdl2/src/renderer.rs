use engine_render::{OutputBackend, OverlayData, RenderError};
use engine_runtime::PresentationPolicy;

use crate::input::Sdl2InputBackend;
use crate::runtime::{RuntimeCommand, RuntimeResponse, Sdl2RuntimeClient};
use std::sync::{Arc, Mutex};

pub const DEFAULT_PIXEL_SCALE: u32 = 8;
pub const LOGICAL_CELL_WIDTH: u32 = 1;
pub const LOGICAL_CELL_HEIGHT: u32 = 2;

pub struct Sdl2Backend {
    client: Arc<Mutex<Sdl2RuntimeClient>>,
    width: u16,
    height: u16,
    pending_overlay: Option<OverlayData>,
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
}

impl OutputBackend for Sdl2Backend {
    fn present_buffer(&mut self, buffer: &engine_core::buffer::Buffer) {
        let overlay = self.pending_overlay.take();
        let _ = self.request(RuntimeCommand::Present(buffer.clone(), overlay));
    }

    fn present_overlay(&mut self, overlay: &OverlayData) {
        self.pending_overlay = Some(overlay.clone());
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
        };
        assert_eq!(backend.output_size(), (120, 40));
    }
}
