//! Emit helpers and wave math for behavior commands.

use std::f32::consts::TAU;
use std::sync::Mutex;

use engine_core::game_object::GameObject;
use engine_core::game_state::GameState;

use crate::{BehaviorCommand, ScriptUiApi};

pub(crate) fn emit_audio(commands: &mut Vec<BehaviorCommand>, cue: String, volume: Option<f32>) {
    commands.push(BehaviorCommand::PlayAudioCue { cue, volume });
}

pub(crate) fn emit_visibility(commands: &mut Vec<BehaviorCommand>, target: String, visible: bool) {
    commands.push(BehaviorCommand::SetVisibility { target, visible });
}

pub(crate) fn emit_offset(commands: &mut Vec<BehaviorCommand>, target: String, dx: i32, dy: i32) {
    commands.push(BehaviorCommand::SetOffset { target, dx, dy });
}

pub(crate) fn emit_text(commands: &mut Vec<BehaviorCommand>, target: String, text: String) {
    commands.push(BehaviorCommand::SetText { target, text });
}

pub(crate) fn resolve_target(target: &Option<String>, object: &GameObject) -> String {
    target
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| object.id.clone())
}

pub(crate) fn sine_wave(elapsed_ms: u64, phase_ms: u64, period_ms: u64) -> f32 {
    let phase = (elapsed_ms.saturating_add(phase_ms) % period_ms) as f32 / period_ms as f32;
    (phase * TAU).sin()
}

pub(crate) fn rounded_sine_wave(elapsed_ms: u64, phase_ms: u64, period_ms: u64) -> i32 {
    sine_wave(elapsed_ms, phase_ms, period_ms).round() as i32
}

pub(crate) fn wrapped_menu_distance(index: usize, selected: usize, total: usize) -> i32 {
    let raw = index as i32 - selected as i32;
    if total <= 1 {
        return raw;
    }
    let total_i = total as i32;
    [raw, raw - total_i, raw + total_i]
        .into_iter()
        .min_by_key(|value| value.abs())
        .unwrap_or(raw)
}

pub(crate) fn expire_ui_flash_message(
    scene_elapsed_ms: u64,
    game_state: Option<&GameState>,
    helper_commands: &Mutex<Vec<BehaviorCommand>>,
) {
    let Some(state) = game_state else {
        return;
    };
    let until_ms = state
        .get(ScriptUiApi::FLASH_UNTIL_MS_PATH)
        .and_then(|value| value.as_i64());
    let Some(until_ms) = until_ms else {
        return;
    };
    if (scene_elapsed_ms as i64) < until_ms {
        return;
    }
    let current_text = state
        .get(ScriptUiApi::FLASH_TEXT_PATH)
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_default();
    state.remove(ScriptUiApi::FLASH_UNTIL_MS_PATH);
    state.remove(ScriptUiApi::FLASH_TEXT_PATH);
    if current_text.is_empty() {
        return;
    }
    if let Ok(mut queue) = helper_commands.lock() {
        queue.push(BehaviorCommand::SetText {
            target: ScriptUiApi::FLASH_TARGET.to_string(),
            text: String::new(),
        });
    }
}
