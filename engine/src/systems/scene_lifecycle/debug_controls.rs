use crate::debug_features::{DebugFeatures, DebugOverlayMode};
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_core::logging;
use engine_events::{KeyCode, KeyEvent};

use super::{begin_leave, is_scene_idle};

fn toggle_debug_overlay(world: &mut World) -> bool {
    let Some(debug) = world.get_mut::<DebugFeatures>() else {
        return false;
    };
    if !debug.enabled {
        debug.enabled = true;
    }
    debug.overlay_visible = !debug.overlay_visible;
    logging::debug(
        "engine.debug.input",
        &format!(
            "console toggled: {}",
            if debug.overlay_visible {
                "visible"
            } else {
                "hidden"
            }
        ),
    );
    true
}

fn cycle_debug_overlay_mode(world: &mut World) -> bool {
    let Some(debug) = world.get_mut::<DebugFeatures>() else {
        return false;
    };
    if !debug.enabled || !debug.overlay_visible {
        return false;
    }
    debug.overlay_mode = match debug.overlay_mode {
        DebugOverlayMode::Stats => DebugOverlayMode::Logs,
        DebugOverlayMode::Logs => DebugOverlayMode::Stats,
    };
    logging::debug(
        "engine.debug.input",
        &format!("console tab: {:?}", debug.overlay_mode),
    );
    true
}

fn debug_target_scene(world: &World, key: &KeyEvent) -> Option<String> {
    let debug_enabled = world
        .get::<DebugFeatures>()
        .map(|debug| debug.enabled)
        .unwrap_or(false);
    if !debug_enabled || !is_scene_idle(world) {
        return None;
    }
    let current_scene_id = world
        .scene_runtime()
        .map(|runtime| runtime.scene().id.clone())?;
    let loader = world.scene_loader()?;
    let candidate = if matches!(key.code, KeyCode::F(3)) {
        loader.prev_scene_id(&current_scene_id)
    } else {
        loader.next_scene_id(&current_scene_id)
    };
    match candidate {
        Some(scene_id) if scene_id != current_scene_id => Some(scene_id),
        _ => None,
    }
}

fn handle_debug_scene_nav(world: &mut World, key: &KeyEvent) -> bool {
    let Some(scene_id) = debug_target_scene(world, key) else {
        return false;
    };
    let Some(animator) = world.animator_mut() else {
        return false;
    };
    animator.next_scene_override = Some(scene_id);
    begin_leave(animator);
    true
}

pub(super) fn handle_debug_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    let mut handled = false;
    for key in key_presses {
        match key.code {
            // ~ / ` toggles the debug console on/off.
            KeyCode::Char('~') | KeyCode::Char('`') => {
                handled |= toggle_debug_overlay(world);
            }
            // Tab switches between Stats and Logs panels while console is open.
            KeyCode::Tab => {
                handled |= cycle_debug_overlay_mode(world);
            }
            KeyCode::F(3) | KeyCode::F(4) => {
                handled |= handle_debug_scene_nav(world, key);
            }
            _ => {}
        }
    }
    handled
}
