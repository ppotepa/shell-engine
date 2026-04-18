//! GUI update system — processes input events and mutates [`GuiRuntimeState`].
//!
//! Call [`GuiSystem::update`] once per frame with all input events collected
//! since the last frame. It updates hover, pressed, clicked, and value fields.

use engine_events::{InputEvent, MouseButton};

use crate::control::GuiControl;
use crate::state::GuiRuntimeState;

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
                .or_insert_with(|| w.initial_state());
        }

        // Clear per-frame transient flags.
        state.last_changed = None;
        for ws in state.widgets.values_mut() {
            ws.changed = false;
            ws.clicked = false;
            ws.submitted = false;
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
                        let hit = widgets.iter().find(|w| {
                            state
                                .widgets
                                .get(w.id())
                                .and_then(|ws| w.bounds(ws))
                                .map(|b| b.hit_test(*x, *y))
                                .unwrap_or(false)
                        });
                        if let Some(w) = hit {
                            let id = w.id().to_string();
                            state.drag_widget = Some(id.clone());
                            if w.wants_keyboard_focus() {
                                state.focused_widget = Some(id.clone());
                            } else {
                                state.focused_widget = None;
                            }
                            if let Some(ws) = state.widgets.get_mut(&id) {
                                let prev_value = ws.value;
                                let prev_text = ws.text.clone();
                                w.on_mouse_down(ws, *x, *y);
                                if (ws.value - prev_value).abs() > f64::EPSILON
                                    || ws.text != prev_text
                                {
                                    ws.changed = true;
                                    state.last_changed = Some(id);
                                }
                            }
                        } else {
                            state.focused_widget = None;
                            for ws in state.widgets.values_mut() {
                                ws.open = false;
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
                                .and_then(|w| {
                                    state.widgets.get(&drag_id).and_then(|ws| w.bounds(ws))
                                })
                                .map(|b| b.hit_test(*x, *y))
                                .unwrap_or(false);
                            if let Some(w) = widgets.iter().find(|w| w.id() == drag_id) {
                                if let Some(ws) = state.widgets.get_mut(&drag_id) {
                                    let prev_value = ws.value;
                                    let prev_text = ws.text.clone();
                                    w.on_mouse_up(ws, *x, *y, still_hovered);
                                    if (ws.value - prev_value).abs() > f64::EPSILON
                                        || ws.text != prev_text
                                        || ws.submitted
                                    {
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
                InputEvent::KeyDown { key, .. } => {
                    if let Some(focused_id) = state.focused_widget.clone() {
                        if let Some(w) = widgets.iter().find(|w| w.id() == focused_id) {
                            if let Some(ws) = state.widgets.get_mut(&focused_id) {
                                let prev_value = ws.value;
                                let prev_text = ws.text.clone();
                                w.on_key_down(ws, key);
                                if (ws.value - prev_value).abs() > f64::EPSILON
                                    || ws.text != prev_text
                                    || ws.submitted
                                {
                                    ws.changed = true;
                                    state.last_changed = Some(focused_id);
                                }
                            }
                        }
                    }
                }
                InputEvent::FocusLost => {
                    state.focused_widget = None;
                    for ws in state.widgets.values_mut() {
                        ws.pressed = false;
                        ws.open = false;
                    }
                }
                InputEvent::KeyUp { .. } | InputEvent::MouseWheel { .. } => {}
            }
        }
    }

    fn update_hover(widgets: &[Box<dyn GuiControl>], state: &mut GuiRuntimeState) {
        let mx = state.mouse_x;
        let my = state.mouse_y;
        for w in widgets {
            let hovered = state
                .widgets
                .get(w.id())
                .and_then(|ws| w.bounds(ws))
                .map(|b| b.hit_test(mx, my))
                .unwrap_or(false);
            if let Some(ws) = state.widgets.get_mut(w.id()) {
                ws.hovered = hovered;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{
        ButtonControl, ChoiceOption, DropdownControl, NumberInputControl, SliderControl,
    };
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
            follow_layout: true,
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
            follow_layout: true,
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

    #[test]
    fn dropdown_click_opens_then_selects_option() {
        let widgets: Vec<Box<dyn GuiControl>> = vec![Box::new(DropdownControl {
            id: "dd".to_string(),
            sprite: "trigger".to_string(),
            x: 10,
            y: 10,
            w: 100,
            h: 20,
            options: vec![
                ChoiceOption::new("one".to_string(), "One".to_string()),
                ChoiceOption::new("two".to_string(), "Two".to_string()),
            ],
            selected: 0,
            popup_sprite: "popup".to_string(),
            label_sprite: "label".to_string(),
            option_sprites: Vec::new(),
            popup_above: false,
            follow_layout: true,
        })];
        let mut state = GuiRuntimeState::new();

        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseDown {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseUp {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        assert!(state.is_open("dd"));

        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseDown {
                x: 20.0,
                y: 45.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseUp {
                x: 20.0,
                y: 45.0,
                button: MouseButton::Left,
            }],
        );
        assert_eq!(state.selected_index("dd"), Some(0));
        assert!(!state.is_open("dd"));
    }

    #[test]
    fn text_input_focuses_and_accepts_keyboard_input() {
        let widgets: Vec<Box<dyn GuiControl>> = vec![Box::new(crate::control::TextInputControl {
            id: "name".to_string(),
            sprite: "input-box".to_string(),
            x: 10,
            y: 10,
            w: 120,
            h: 20,
            text_sprite: "input-text".to_string(),
            placeholder: "Name".to_string(),
            value: String::new(),
            max_length: 16,
            follow_layout: true,
        })];
        let mut state = GuiRuntimeState::new();

        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseDown {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseUp {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::KeyDown {
                key: engine_events::KeyEvent::new(
                    engine_events::KeyCode::Char('A'),
                    engine_events::KeyModifiers::NONE,
                ),
                repeat: false,
            }],
        );
        assert_eq!(state.text("name"), "A");

        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::KeyDown {
                key: engine_events::KeyEvent::new(
                    engine_events::KeyCode::Enter,
                    engine_events::KeyModifiers::NONE,
                ),
                repeat: false,
            }],
        );
        assert!(state.submitted("name"));
    }

    #[test]
    fn number_input_filters_chars_and_submits_clamped_value() {
        let widgets: Vec<Box<dyn GuiControl>> = vec![Box::new(NumberInputControl {
            id: "num".to_string(),
            sprite: "input-box".to_string(),
            x: 10,
            y: 10,
            w: 120,
            h: 20,
            text_sprite: "input-text".to_string(),
            placeholder: "0".to_string(),
            value: String::new(),
            min: Some(0.0),
            max: Some(10.0),
            step: Some(0.5),
            max_length: 16,
            follow_layout: true,
        })];
        let mut state = GuiRuntimeState::new();

        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseDown {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[InputEvent::MouseUp {
                x: 20.0,
                y: 15.0,
                button: MouseButton::Left,
            }],
        );
        GuiSystem::update(
            &widgets,
            &mut state,
            &[
                InputEvent::KeyDown {
                    key: engine_events::KeyEvent::new(
                        engine_events::KeyCode::Char('1'),
                        engine_events::KeyModifiers::NONE,
                    ),
                    repeat: false,
                },
                InputEvent::KeyDown {
                    key: engine_events::KeyEvent::new(
                        engine_events::KeyCode::Char('2'),
                        engine_events::KeyModifiers::NONE,
                    ),
                    repeat: false,
                },
                InputEvent::KeyDown {
                    key: engine_events::KeyEvent::new(
                        engine_events::KeyCode::Char('a'),
                        engine_events::KeyModifiers::NONE,
                    ),
                    repeat: false,
                },
                InputEvent::KeyDown {
                    key: engine_events::KeyEvent::new(
                        engine_events::KeyCode::Enter,
                        engine_events::KeyModifiers::NONE,
                    ),
                    repeat: false,
                },
            ],
        );

        assert_eq!(state.text("num"), "10");
        assert!((state.value("num") - 10.0).abs() < f64::EPSILON);
        assert!(state.submitted("num"));
    }
}
