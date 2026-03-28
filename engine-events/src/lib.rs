//! Engine event types and the per-frame [`EventQueue`] that shuttles them between systems.

pub mod access;
pub mod input_backend;
pub mod key;

pub use input_backend::InputBackend;
pub use key::{KeyCode, KeyEvent, KeyModifiers};

/// Represents a discrete engine event produced by input, the game loop, or scene transitions.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Tick,
    KeyPressed(KeyEvent),
    KeyReleased(KeyEvent),
    InputFocusLost,
    MouseMoved { column: u16, row: u16 },
    SceneLoaded { scene_id: String },
    SceneTransition { to_scene_id: String },
    AudioCue { cue: String, volume: Option<f32> },
    OutputResized { width: u16, height: u16 },
    Quit,
}

/// Collects [`EngineEvent`]s during a frame and delivers them to systems via [`drain`](Self::drain).
#[derive(Debug, Default)]
pub struct EventQueue {
    events: Vec<EngineEvent>,
}

impl EventQueue {
    /// Creates an empty [`EventQueue`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends `event` to the queue.
    pub fn push(&mut self, event: EngineEvent) {
        self.events.push(event);
    }

    /// Removes and returns all queued events, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }

    /// Returns `true` if no events are waiting.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
