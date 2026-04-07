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
        format!(
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
        format!("console tab: {:?}", debug.overlay_mode),
    );
    true
}

/// Switch active palette by 0-based index (debug only).
/// Writes the selected palette id to the persistence store so Rhai picks it
/// up on the next frame via `palette.get()`.
fn switch_palette_by_index(world: &mut World, index: usize) -> bool {
    let debug_enabled = world
        .get::<DebugFeatures>()
        .map(|d| d.enabled)
        .unwrap_or(false);
    if !debug_enabled {
        return false;
    }

    let (id, name) = {
        let Some(store) = world.get::<engine_behavior::palette::PaletteStore>() else {
            return false;
        };
        let Some(id) = store.order.get(index) else {
            logging::debug(
                "engine.debug.palette",
                format!("palette index {} out of range (have {})", index, store.len()),
            );
            return false;
        };
        let name = store
            .palettes
            .get(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| id.clone());
        (id.clone(), name)
    };

    let Some(persist) = world.get::<engine_persistence::PersistenceStore>() else {
        return false;
    };
    persist.set("/__palette__", serde_json::Value::String(id.clone()));
    logging::debug(
        "engine.debug.palette",
        format!("[{}] palette → {} ({})", index + 1, name, id),
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
            // 1–9: switch active palette by index (debug mode only).
            KeyCode::Char(c @ '1'..='9') => {
                let index = (c as usize) - ('1' as usize);
                handled |= switch_palette_by_index(world, index);
            }
            _ => {}
        }
    }
    handled
}
