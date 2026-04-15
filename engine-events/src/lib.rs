//! Engine event types and the per-frame [`EventQueue`] that shuttles them between systems.

pub mod access;
pub mod input_backend;
pub mod key;
pub mod mouse;

pub use input_backend::InputBackend;
pub use key::{KeyCode, KeyEvent, KeyModifiers};
pub use mouse::MouseButton;

/// Represents a discrete engine event produced by input, the game loop, or scene transitions.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Tick,
    /// A key was pressed. `repeat` is true when the platform is auto-repeating a held key.
    KeyDown { key: KeyEvent, repeat: bool },
    /// A key was released.
    KeyUp { key: KeyEvent },
    InputFocusLost,
    /// Mouse cursor moved to virtual-space coordinates `(x, y)` in `[0, width) × [0, height)`.
    MouseMoved { x: f32, y: f32 },
    MouseButtonDown { button: MouseButton, x: f32, y: f32 },
    MouseButtonUp { button: MouseButton, x: f32, y: f32 },
    /// Mouse scroll wheel moved. Positive `delta_y` = scroll up, negative = scroll down.
    MouseWheel { delta_y: f32 },
    SceneLoaded { scene_id: String },
    SceneTransition { to_scene_id: String },
    AudioCue { cue: String, volume: Option<f32> },
    OutputResized { width: u16, height: u16 },
    Quit,
}

impl EngineEvent {
    /// Returns the [`InputEvent`] representation of this event, if it is an input event.
    /// Non-input events (Tick, SceneTransition, etc.) return `None`.
    pub fn as_input_event(&self) -> Option<InputEvent> {
        match self {
            Self::KeyDown { key, repeat } => Some(InputEvent::KeyDown { key: *key, repeat: *repeat }),
            Self::KeyUp { key } => Some(InputEvent::KeyUp { key: *key }),
            Self::MouseMoved { x, y } => Some(InputEvent::MouseMoved { x: *x, y: *y }),
            Self::MouseButtonDown { button, x, y } => Some(InputEvent::MouseDown { button: *button, x: *x, y: *y }),
            Self::MouseButtonUp { button, x, y } => Some(InputEvent::MouseUp { button: *button, x: *x, y: *y }),
            Self::MouseWheel { delta_y } => Some(InputEvent::MouseWheel { delta_y: *delta_y }),
            Self::InputFocusLost => Some(InputEvent::FocusLost),
            _ => None,
        }
    }
}

/// Input-only subset of [`EngineEvent`].
///
/// Systems that only care about player input (GUI, camera, scripting) accept `&[InputEvent]`
/// instead of `&[EngineEvent]`, keeping them decoupled from game-lifecycle events.
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyDown { key: KeyEvent, repeat: bool },
    KeyUp { key: KeyEvent },
    MouseMoved { x: f32, y: f32 },
    MouseDown { x: f32, y: f32, button: MouseButton },
    MouseUp { x: f32, y: f32, button: MouseButton },
    MouseWheel { delta_y: f32 },
    FocusLost,
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
