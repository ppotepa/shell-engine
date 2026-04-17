//! GUI update system — processes input events and mutates [`GuiRuntimeState`].
//!
//! Call [`GuiSystem::update`] once per frame with all input events collected
//! since the last frame. It updates hover, pressed, clicked, and value fields.

use engine_events::{InputEvent, MouseButton};

use crate::control::GuiControl;
use crate::state::{GuiRuntimeState, GuiWidgetState};

/// Stateless system — all mutable state lives in [`GuiRuntimeState`].
pub struct GuiSystem;

impl GuiSystem {
    /// Update GUI state for one frame.
    ///
    /// - Initializes any widgets not yet in `state.widgets`.
    /// - Processes all `events` in order.
    /// - Clears per-frame flags (changed, clicked) before processing.
    pub fn update(
        widgets: &[Box<dyn GuiControl>],
        state: &mut GuiRuntimeState,
        events: &[InputEvent],
    ) {
        // Initialize any new widgets.
        for w in widgets {
            state
                .widgets
                .entry(w.id().to_string())
                .or_insert_with(|| GuiWidgetState {
                    value: w.initial_value(),
                    ..Default::default()
                });
        }

        // Clear per-frame transient flags.
        state.last_changed = None;
        for ws in state.widgets.values_mut() {
            ws.changed = false;
            ws.clicked = false;
        }

        // Process events in order, dispatching to each widget's own methods.
        for event in events {
            match event {
                InputEvent::MouseMoved { x, y } => {
                    state.mouse_x = *x;
                    state.mouse_y = *y;
                    Self::update_hover(widgets, state);
                    if state.drag_button == Some(MouseButton::Left) {
                        if let Some(drag_id) = state.drag_widget.clone() {
                            if let Some(w) = widgets.iter().find(|w| w.id() == drag_id) {
                                if let Some(ws) = state.widgets.get_mut(&drag_id) {
                                    let prev = ws.value;
                                    w.on_drag(ws, *x, *y);
                                    if (ws.value - prev).abs() > f64::EPSILON {
                                        ws.changed = true;
                                        state.last_changed = Some(drag_id);
                                    }
                                }
                            }
                        }
                    }
                }
                InputEvent::MouseDown { x, y, button } => {
                    state.mouse_x = *x;
                    state.mouse_y = *y;
                    Self::update_hover(widgets, state);
                    if *button == MouseButton::Left {
                        state.drag_button = Some(MouseButton::Left);
                        let hit = widgets
                            .iter()
                            .find(|w| w.bounds().map(|b| b.hit_test(*x, *y)).unwrap_or(false));
                        if let Some(w) = hit {
                            let id = w.id().to_string();
                            state.drag_widget = Some(id.clone());
                            if let Some(ws) = state.widgets.get_mut(&id) {
                                let prev = ws.value;
                                w.on_mouse_down(ws, *x, *y);
                                if (ws.value - prev).abs() > f64::EPSILON {
                                    ws.changed = true;
                                    state.last_changed = Some(id);
                                }
                            }
                        }
                    }
                }
                InputEvent::MouseUp { x, y, button } => {
                    state.mouse_x = *x;
                    state.mouse_y = *y;
                    if *button == MouseButton::Left {
                        state.drag_button = None;
                        if let Some(drag_id) = state.drag_widget.take() {
                            let still_hovered = widgets
                                .iter()
                                .find(|w| w.id() == drag_id)
                                .and_then(|w| w.bounds())
                                .map(|b| b.hit_test(*x, *y))
                                .unwrap_or(false);
                            if let Some(w) = widgets.iter().find(|w| w.id() == drag_id) {
                                if let Some(ws) = state.widgets.get_mut(&drag_id) {
                                    let prev = ws.value;
                                    w.on_mouse_up(ws, still_hovered);
                                    if (ws.value - prev).abs() > f64::EPSILON {
                                        ws.changed = true;
                                        state.last_changed = Some(drag_id.clone());
                                    }
                                }
                            }
                        }
                        for ws in state.widgets.values_mut() {
                            ws.pressed = false;
                        }
                    }
                }
                // Keyboard events are accepted but not yet consumed here.
                // Future: route to focused widget for text input / slider adjustment.
                InputEvent::KeyDown { .. }
                | InputEvent::KeyUp { .. }
                | InputEvent::FocusLost
                | InputEvent::MouseWheel { .. } => {}
            }
        }
    }

    fn update_hover(widgets: &[Box<dyn GuiControl>], state: &mut GuiRuntimeState) {
        let mx = state.mouse_x;
        let my = state.mouse_y;
        for w in widgets {
            let hovered = w.bounds().map(|b| b.hit_test(mx, my)).unwrap_or(false);
            if let Some(ws) = state.widgets.get_mut(w.id()) {
                ws.hovered = hovered;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ButtonControl, SliderControl};
    use engine_events::{InputEvent, MouseButton};

    fn make_slider() -> Box<dyn GuiControl> {
        Box::new(SliderControl {
            id: "s".to_string(),
            sprite: "track".to_string(),
            x: 10,
            y: 10,
            w: 100,
            h: 12,
            min: 0.0,
            max: 1.0,
            value: 0.5,
            hit_padding: 0,
            handle: String::new(),
        })
    }

    #[test]
    fn slider_drag_updates_value() {
        let widgets: Vec<Box<dyn GuiControl>> = vec![make_slider()];
        let mut state = GuiRuntimeState::new();
        GuiSystem::update(&widgets, &mut state, &[]);
        assert!((state.value("s") - 0.5).abs() < 1e-9);

        let events = vec![InputEvent::MouseDown {
            x: 60.0,
            y: 15.0,
            button: MouseButton::Left,
        }];
        GuiSystem::update(&widgets, &mut state, &events);
        assert!((state.value("s") - 0.5).abs() < 0.01);

        let events = vec![InputEvent::MouseMoved { x: 110.0, y: 15.0 }];
        GuiSystem::update(&widgets, &mut state, &events);
        assert!((state.value("s") - 1.0).abs() < 0.01);
    }

    #[test]
    fn button_clicked_fires_once() {
        let widgets: Vec<Box<dyn GuiControl>> = vec![Box::new(ButtonControl {
            id: "btn".to_string(),
            sprite: "b".to_string(),
            x: 0,
            y: 0,
            w: 50,
            h: 20,
        })];
        let mut state = GuiRuntimeState::new();
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseDown {
                x: 10.0,
                y: 5.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseUp {
                x: 10.0,
                y: 5.0,
                button: MouseButton::Left,
            }],
        );
        assert!(state.clicked("btn"));
        GuiSystem::update(&widgets, &mut state, &[]);
        assert!(!state.clicked("btn"));
    }
}
