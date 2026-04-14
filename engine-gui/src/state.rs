//! Per-widget runtime state, accumulated each frame by [`GuiSystem`].

use std::collections::HashMap;

/// Runtime state for a single widget.
#[derive(Debug, Clone, Default)]
pub struct GuiWidgetState {
    /// Current value: [min..max] for sliders, 0.0/1.0 for toggles, 0.0 for buttons/panels.
    pub value: f64,
    /// True if the mouse cursor is inside this widget's bounds.
    pub hovered: bool,
    /// True while the primary mouse button is held down over this widget.
    pub pressed: bool,
    /// True for exactly one frame when the value changed.
    pub changed: bool,
    /// True for exactly one frame when a button was clicked (fires on mouse-up inside bounds).
    pub clicked: bool,
}

/// All GUI widget state for a scene, keyed by widget id.
#[derive(Debug, Clone, Default)]
pub struct GuiRuntimeState {
    pub widgets: HashMap<String, GuiWidgetState>,
    /// Id of the last widget that changed this frame (if any).
    pub last_changed: Option<String>,
    /// Current mouse position in virtual screen coordinates.
    pub mouse_x: f32,
    pub mouse_y: f32,
    /// Which button is currently held (if any).
    pub drag_button: Option<engine_events::MouseButton>,
    /// Which widget id is being dragged (if any).
    pub drag_widget: Option<String>,
}

impl GuiRuntimeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current value for a widget, or 0.0 if not found.
    pub fn value(&self, id: &str) -> f64 {
        self.widgets.get(id).map(|s| s.value).unwrap_or(0.0)
    }

    /// Returns true if a button widget was clicked this frame.
    pub fn clicked(&self, id: &str) -> bool {
        self.widgets.get(id).map(|s| s.clicked).unwrap_or(false)
    }

    /// Returns true if a toggle widget is currently on.
    pub fn toggle_on(&self, id: &str) -> bool {
        self.widgets.get(id).map(|s| s.value > 0.5).unwrap_or(false)
    }

    /// Returns true if any widget changed this frame.
    pub fn has_change(&self) -> bool {
        self.last_changed.is_some()
    }
}
