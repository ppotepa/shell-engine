//! GUI update system — processes input events and mutates [`GuiRuntimeState`].
//!
//! Call [`GuiSystem::update`] once per frame with all input events collected
//! since the last frame. It updates hover, pressed, clicked, and value fields.

use engine_events::{InputEvent, MouseButton};

use crate::state::{GuiRuntimeState, GuiWidgetState};
use crate::widget::GuiWidgetDef;

/// Stateless system — all mutable state lives in [`GuiRuntimeState`].
pub struct GuiSystem;

impl GuiSystem {
    /// Update GUI state for one frame.
    ///
    /// - Initializes any widgets not yet in `state.widgets`.
    /// - Processes all `events` in order.
    /// - Clears per-frame flags (changed, clicked) before processing.
    pub fn update(
        widgets: &[GuiWidgetDef],
        state: &mut GuiRuntimeState,
        events: &[InputEvent],
    ) {
        // Initialize any new widgets.
        for w in widgets {
            state.widgets.entry(w.id().to_string()).or_insert_with(|| {
                GuiWidgetState {
                    value: w.initial_value(),
                    ..Default::default()
                }
            });
        }

        // Clear per-frame transient flags.
        state.last_changed = None;
        for ws in state.widgets.values_mut() {
            ws.changed = false;
            ws.clicked = false;
        }

        // Process events in order.
        for event in events {
            match event {
                InputEvent::MouseMoved { x, y } => {
                    state.mouse_x = *x;
                    state.mouse_y = *y;
                    Self::update_hover(widgets, state);
                    if state.drag_button == Some(MouseButton::Left) {
                        if let Some(drag_id) = state.drag_widget.clone() {
                            if let Some(w) = widgets.iter().find(|w| w.id() == drag_id) {
                                Self::apply_slider_drag(w, state, *x);
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
                        let hit = widgets.iter().find(|w| {
                            w.bounds()
                                .map(|(wx, wy, ww, wh)| hit_test(wx, wy, ww, wh, *x, *y))
                                .unwrap_or(false)
                        });
                        if let Some(w) = hit {
                            let id = w.id().to_string();
                            if let Some(ws) = state.widgets.get_mut(&id) {
                                ws.pressed = true;
                            }
                            state.drag_widget = Some(id.clone());
                            Self::apply_slider_drag(w, state, *x);
                        }
                    }
                }
                InputEvent::MouseUp { x, y, button } => {
                    state.mouse_x = *x;
                    state.mouse_y = *y;
                    if *button == MouseButton::Left {
                        state.drag_button = None;
                        if let Some(drag_id) = state.drag_widget.take() {
                            let still_hovered = widgets.iter().find(|w| w.id() == drag_id)
                                .and_then(|w| w.bounds())
                                .map(|(wx, wy, ww, wh)| hit_test(wx, wy, ww, wh, *x, *y))
                                .unwrap_or(false);
                            if let Some(ws) = state.widgets.get_mut(&drag_id) {
                                ws.pressed = false;
                                if still_hovered {
                                    ws.clicked = true;
                                    if let Some(w) = widgets.iter().find(|w| w.id() == drag_id) {
                                        if matches!(w, GuiWidgetDef::Toggle { .. }) {
                                            ws.value = if ws.value > 0.5 { 0.0 } else { 1.0 };
                                            ws.changed = true;
                                            state.last_changed = Some(drag_id.clone());
                                        }
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
                InputEvent::KeyDown { .. } | InputEvent::KeyUp { .. } | InputEvent::FocusLost => {}
            }
        }
    }

    fn update_hover(widgets: &[GuiWidgetDef], state: &mut GuiRuntimeState) {
        let mx = state.mouse_x;
        let my = state.mouse_y;
        for w in widgets {
            let hovered = w.bounds()
                .map(|(wx, wy, ww, wh)| hit_test(wx, wy, ww, wh, mx, my))
                .unwrap_or(false);
            if let Some(ws) = state.widgets.get_mut(w.id()) {
                ws.hovered = hovered;
            }
        }
    }

    fn apply_slider_drag(widget: &GuiWidgetDef, state: &mut GuiRuntimeState, mouse_x: f32) {
        let GuiWidgetDef::Slider { id, x, w, min, max, .. } = widget else {
            return;
        };
        let t = ((mouse_x - *x as f32) / (*w).max(1) as f32).clamp(0.0, 1.0) as f64;
        let new_value = min + t * (max - min);
        if let Some(ws) = state.widgets.get_mut(id) {
            let prev = ws.value;
            ws.value = new_value;
            if (ws.value - prev).abs() > f64::EPSILON {
                ws.changed = true;
                state.last_changed = Some(id.clone());
            }
        }
    }
}

#[inline]
fn hit_test(wx: i32, wy: i32, ww: i32, wh: i32, mx: f32, my: f32) -> bool {
    mx >= wx as f32 && mx < (wx + ww) as f32 && my >= wy as f32 && my < (wy + wh) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_events::{InputEvent, MouseButton};
    use crate::widget::GuiWidgetDef;

    fn make_slider() -> GuiWidgetDef {
        GuiWidgetDef::Slider {
            id: "s".to_string(),
            sprite: "track".to_string(),
            x: 10, y: 10, w: 100, h: 12,
            min: 0.0, max: 1.0, value: 0.5,
        }
    }

    #[test]
    fn slider_drag_updates_value() {
        let widgets = vec![make_slider()];
        let mut state = GuiRuntimeState::new();
        GuiSystem::update(&widgets, &mut state, &[]);
        assert!((state.value("s") - 0.5).abs() < 1e-9);

        let events = vec![
            InputEvent::MouseDown { x: 60.0, y: 15.0, button: MouseButton::Left },
        ];
        GuiSystem::update(&widgets, &mut state, &events);
        assert!((state.value("s") - 0.5).abs() < 0.01);

        let events = vec![
            InputEvent::MouseMoved { x: 110.0, y: 15.0 },
        ];
        GuiSystem::update(&widgets, &mut state, &events);
        assert!((state.value("s") - 1.0).abs() < 0.01);
    }

    #[test]
    fn button_clicked_fires_once() {
        let widgets = vec![GuiWidgetDef::Button {
            id: "btn".to_string(), sprite: "b".to_string(),
            x: 0, y: 0, w: 50, h: 20,
        }];
        let mut state = GuiRuntimeState::new();
        GuiSystem::update(&widgets, &mut state, &[
            InputEvent::MouseDown { x: 10.0, y: 5.0, button: MouseButton::Left },
        ]);
        GuiSystem::update(&widgets, &mut state, &[
            InputEvent::MouseUp { x: 10.0, y: 5.0, button: MouseButton::Left },
        ]);
        assert!(state.clicked("btn"));
        GuiSystem::update(&widgets, &mut state, &[]);
        assert!(!state.clicked("btn"));
    }
}

