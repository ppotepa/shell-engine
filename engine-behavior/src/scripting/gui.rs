//! GUI domain API: ScriptGuiApi exposes widget state to Rhai scripts.
//!
//! Scripts read values via `gui.slider_value("id")`, `gui.button_clicked("id")`, etc.
//! Rendering is handled by existing Panel/Vector/Text sprites in scene YAML.
//! Hit-testing and state tracking are handled by `engine-gui`.

use std::sync::{Arc, Mutex};

use engine_gui::GuiRuntimeState;
use rhai::Engine as RhaiEngine;

use crate::{BehaviorCommand, BehaviorContext};

#[derive(Clone)]
pub(crate) struct ScriptGuiApi {
    state: Option<Arc<GuiRuntimeState>>,
    mouse_x: f32,
    mouse_y: f32,
    mouse_left_down: bool,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptGuiApi {
    pub(crate) fn new(ctx: &BehaviorContext, queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        let mouse_left_down = ctx.gui_state.as_ref().map(|s| {
            use engine_events::MouseButton;
            s.drag_button == Some(MouseButton::Left) && s.drag_widget.is_none()
        }).unwrap_or(false);
        Self {
            state: ctx.gui_state.clone(),
            mouse_x: ctx.mouse_x,
            mouse_y: ctx.mouse_y,
            mouse_left_down,
            queue,
        }
    }

    fn slider_value(&mut self, id: &str) -> f64 {
        self.state.as_ref().map(|s| s.value(id)).unwrap_or(0.0)
    }

    fn button_clicked(&mut self, id: &str) -> bool {
        self.state.as_ref().map(|s| s.clicked(id)).unwrap_or(false)
    }

    fn toggle_on(&mut self, id: &str) -> bool {
        self.state.as_ref().map(|s| s.toggle_on(id)).unwrap_or(false)
    }

    fn has_change(&mut self) -> bool {
        self.state.as_ref().map(|s| s.has_change()).unwrap_or(false)
    }

    fn changed_widget(&mut self) -> String {
        self.state
            .as_ref()
            .and_then(|s| s.last_changed.clone())
            .unwrap_or_default()
    }

    fn widget_value(&mut self, id: &str) -> f64 {
        self.slider_value(id)
    }

    fn widget_hovered(&mut self, id: &str) -> bool {
        self.state
            .as_ref()
            .and_then(|s| s.widgets.get(id))
            .map(|w| w.hovered)
            .unwrap_or(false)
    }

    fn widget_pressed(&mut self, id: &str) -> bool {
        self.state
            .as_ref()
            .and_then(|s| s.widgets.get(id))
            .map(|w| w.pressed)
            .unwrap_or(false)
    }

    fn mouse_x(&mut self) -> rhai::INT {
        self.mouse_x as rhai::INT
    }

    fn mouse_y(&mut self) -> rhai::INT {
        self.mouse_y as rhai::INT
    }

    fn mouse_x_f(&mut self) -> rhai::FLOAT {
        self.mouse_x as rhai::FLOAT
    }

    fn mouse_y_f(&mut self) -> rhai::FLOAT {
        self.mouse_y as rhai::FLOAT
    }

    fn mouse_left_down(&mut self) -> bool {
        self.mouse_left_down
    }

    fn set_panel_visible(&mut self, id: &str, visible: bool) -> bool {
        if let Ok(mut q) = self.queue.lock() {
            q.push(BehaviorCommand::SetProperty {
                target: id.to_string(),
                path: "visible".to_string(),
                value: serde_json::Value::Bool(visible),
            });
            return true;
        }
        false
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptGuiApi>("GuiApi");

    engine.register_fn("slider_value", |gui: &mut ScriptGuiApi, id: &str| {
        gui.slider_value(id)
    });
    engine.register_fn("button_clicked", |gui: &mut ScriptGuiApi, id: &str| {
        gui.button_clicked(id)
    });
    engine.register_fn("toggle_on", |gui: &mut ScriptGuiApi, id: &str| {
        gui.toggle_on(id)
    });
    engine.register_fn("has_change", |gui: &mut ScriptGuiApi| gui.has_change());
    engine.register_fn("changed_widget", |gui: &mut ScriptGuiApi| {
        gui.changed_widget()
    });
    engine.register_fn("widget_value", |gui: &mut ScriptGuiApi, id: &str| {
        gui.widget_value(id)
    });
    engine.register_fn("widget_hovered", |gui: &mut ScriptGuiApi, id: &str| {
        gui.widget_hovered(id)
    });
    engine.register_fn("widget_pressed", |gui: &mut ScriptGuiApi, id: &str| {
        gui.widget_pressed(id)
    });
    engine.register_get("mouse_x", |gui: &mut ScriptGuiApi| gui.mouse_x());
    engine.register_get("mouse_y", |gui: &mut ScriptGuiApi| gui.mouse_y());
    engine.register_get("mouse_x_f", |gui: &mut ScriptGuiApi| gui.mouse_x_f());
    engine.register_get("mouse_y_f", |gui: &mut ScriptGuiApi| gui.mouse_y_f());
    engine.register_get("mouse_left_down", |gui: &mut ScriptGuiApi| gui.mouse_left_down());
    engine.register_fn(
        "set_panel_visible",
        |gui: &mut ScriptGuiApi, id: &str, visible: bool| gui.set_panel_visible(id, visible),
    );
}
