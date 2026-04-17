//! Deprecated legacy GUI input events — kept for backward compatibility.
//!
//! New code should use [`engine_events::InputEvent`] directly.

/// Deprecated. Use [`engine_events::InputEvent`] instead.
#[deprecated(since = "0.0.0", note = "Use engine_events::InputEvent instead")]
#[derive(Debug, Clone)]
pub enum GuiInputEvent {
    MouseMoved {
        x: i32,
        y: i32,
    },
    MouseDown {
        x: i32,
        y: i32,
        button: engine_events::MouseButton,
    },
    MouseUp {
        x: i32,
        y: i32,
        button: engine_events::MouseButton,
    },
}
