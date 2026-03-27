//! Abstract input backend trait for pluggable input sources (terminal, SDL2, etc.).

use crate::EngineEvent;

/// Trait implemented by each input backend to poll platform events into engine events.
pub trait InputBackend: Send {
    /// Polls the platform for pending input events and returns them as engine events.
    fn poll_events(&mut self) -> Vec<EngineEvent>;
}
