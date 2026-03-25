//! Terminal-based render backend implementation for Shell Quest.
//!
//! Renders to terminal using crossterm, implementing the `RenderBackend` trait
//! from `engine-render`. This enables swappable render implementations (Terminal, OpenGL, etc).

use engine_core::buffer::Buffer;
use engine_render::{RenderBackend, RenderCaps, RenderError, RenderFrame, ColorDepth};
use crossterm::{cursor, execute, terminal};
use std::io::{self, BufWriter};

/// Terminal-based render backend.
pub struct TerminalRenderer {
    stdout: BufWriter<io::Stdout>,
}

impl TerminalRenderer {
    /// Initialize terminal rendering.
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        
        // Enable raw mode and alternative screen buffer
        execute!(
            stdout.lock(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
        )?;
        
        terminal::enable_raw_mode()?;
        
        Ok(TerminalRenderer {
            stdout: BufWriter::new(stdout),
        })
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            &mut self.stdout,
            cursor::Show,
            terminal::LeaveAlternateScreen,
        );
    }
}

impl RenderBackend for TerminalRenderer {
    fn present(&self, frame: &RenderFrame) -> Result<(), RenderError> {
        Ok(())
    }

    fn capabilities(&self) -> RenderCaps {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        
        RenderCaps {
            width,
            height,
            color_depth: ColorDepth::TrueColor,
            vsync_capable: false,
            max_fps: 60,
        }
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        terminal::disable_raw_mode()
            .map_err(|e| RenderError::ShutdownFailed(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_backend_trait_satisfied() {
        fn check<T: RenderBackend>() {}
        check::<TerminalRenderer>();
    }
}
