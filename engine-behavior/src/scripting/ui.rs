//! UI domain API: ScriptUiApi for user interface state and interactions.

use std::sync::{Arc, Mutex};

use engine_core::game_state::GameState;
use rhai::Engine as RhaiEngine;
use serde_json::{Number as JsonNumber, Value as JsonValue};

use crate::{BehaviorCommand, BehaviorContext};

#[derive(Clone)]
pub(crate) struct ScriptUiApi {
    focused_target: String,
    theme: String,
    has_submit: bool,
    submit_target: String,
    submit_text: String,
    has_change: bool,
    change_target: String,
    change_text: String,
    scene_elapsed_ms: u64,
    game_state: Option<GameState>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptUiApi {
    pub(crate) const FLASH_TEXT_PATH: &'static str = "/__ui/game_message/text";
    pub(crate) const FLASH_UNTIL_MS_PATH: &'static str = "/__ui/game_message/until_ms";
    pub(crate) const FLASH_TARGET: &'static str = "game-message";

    pub(crate) fn new(ctx: &BehaviorContext, queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self {
            focused_target: ctx
                .ui_focused_target_id
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            theme: ctx.ui_theme_id.as_deref().unwrap_or_default().to_string(),
            has_submit: ctx.ui_last_submit_target_id.is_some(),
            submit_target: ctx
                .ui_last_submit_target_id
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            submit_text: ctx
                .ui_last_submit_text
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            has_change: ctx.ui_last_change_target_id.is_some(),
            change_target: ctx
                .ui_last_change_target_id
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            change_text: ctx
                .ui_last_change_text
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            scene_elapsed_ms: ctx.scene_elapsed_ms,
            game_state: ctx.game_state.clone(),
            queue,
        }
    }

    fn focused_target(&mut self) -> String {
        self.focused_target.clone()
    }
    fn theme(&mut self) -> String {
        self.theme.clone()
    }
    fn has_submit(&mut self) -> bool {
        self.has_submit
    }
    fn submit_target(&mut self) -> String {
        self.submit_target.clone()
    }
    fn submit_text(&mut self) -> String {
        self.submit_text.clone()
    }
    fn has_change(&mut self) -> bool {
        self.has_change
    }
    fn change_target(&mut self) -> String {
        self.change_target.clone()
    }
    fn change_text(&mut self) -> String {
        self.change_text.clone()
    }

    fn flash_message(&mut self, text: &str, ttl_ms: rhai::INT) -> bool {
        let Some(state) = self.game_state.as_ref() else {
            return false;
        };
        let trimmed = text.trim();
        let ttl_ms = ttl_ms.max(0) as u64;
        let until_ms = self.scene_elapsed_ms.saturating_add(ttl_ms) as i64;

        if !state.set(
            Self::FLASH_TEXT_PATH,
            JsonValue::String(trimmed.to_string()),
        ) {
            return false;
        }
        if !state.set(
            Self::FLASH_UNTIL_MS_PATH,
            JsonValue::Number(JsonNumber::from(until_ms)),
        ) {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::SetText {
            target: Self::FLASH_TARGET.to_string(),
            text: trimmed.to_string(),
        });
        true
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptUiApi>("UiApi");

    engine.register_get("focused_target", |ui: &mut ScriptUiApi| ui.focused_target());
    engine.register_get("theme", |ui: &mut ScriptUiApi| ui.theme());
    engine.register_get("has_submit", |ui: &mut ScriptUiApi| ui.has_submit());
    engine.register_get("submit_target", |ui: &mut ScriptUiApi| ui.submit_target());
    engine.register_get("submit_text", |ui: &mut ScriptUiApi| ui.submit_text());
    engine.register_get("has_change", |ui: &mut ScriptUiApi| ui.has_change());
    engine.register_get("change_target", |ui: &mut ScriptUiApi| ui.change_target());
    engine.register_get("change_text", |ui: &mut ScriptUiApi| ui.change_text());
    engine.register_fn(
        "flash_message",
        |ui: &mut ScriptUiApi, text: &str, ttl_ms: rhai::INT| ui.flash_message(text, ttl_ms),
    );
}
