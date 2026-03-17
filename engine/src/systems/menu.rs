//! Menu input system — maps key presses to navigation and activation actions for scene menus.

use crate::scene::MenuOption;
use crossterm::event::KeyCode;

/// The outcome of evaluating menu input for a single frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    None,
    Navigate(usize),
    Activate(String),
}

/// Evaluates `key_presses` against `options` to produce a [`MenuAction`] for the current frame.
pub fn evaluate_menu_action(
    options: &[MenuOption],
    selected_index: usize,
    key_presses: &[KeyCode],
) -> MenuAction {
    if options.is_empty() {
        return MenuAction::None;
    }
    let mut index = selected_index.min(options.len().saturating_sub(1));

    for key_code in key_presses {
        for (option_idx, option) in options.iter().enumerate() {
            if key_matches_binding(key_code, &option.key) {
                if let Some(target) = resolve_menu_target(option) {
                    return MenuAction::Activate(target);
                }
            }
            if option_idx == index && matches_confirm_key(key_code) {
                if let Some(target) = resolve_menu_target(option) {
                    return MenuAction::Activate(target);
                }
            }
        }

        if is_prev_key(key_code) {
            index = if index == 0 {
                options.len().saturating_sub(1)
            } else {
                index - 1
            };
            continue;
        }

        if is_next_key(key_code) {
            index = (index + 1) % options.len();
        }
    }

    if index != selected_index {
        return MenuAction::Navigate(index);
    }
    MenuAction::None
}

fn resolve_menu_target(option: &MenuOption) -> Option<String> {
    option.scene.clone().or_else(|| Some(option.next.clone()))
}

fn key_matches_binding(key_code: &KeyCode, binding: &str) -> bool {
    let b = binding.trim().to_ascii_lowercase();
    match key_code {
        KeyCode::Char(c) => b == c.to_ascii_lowercase().to_string() || (*c == ' ' && b == "space"),
        KeyCode::Enter => b == "enter",
        KeyCode::Esc => b == "esc" || b == "escape",
        KeyCode::Tab => b == "tab",
        KeyCode::Backspace => b == "backspace",
        KeyCode::Left => b == "left",
        KeyCode::Right => b == "right",
        KeyCode::Up => b == "up",
        KeyCode::Down => b == "down",
        KeyCode::Home => b == "home",
        KeyCode::End => b == "end",
        KeyCode::PageUp => b == "pageup" || b == "page-up",
        KeyCode::PageDown => b == "pagedown" || b == "page-down",
        KeyCode::Delete => b == "delete" || b == "del",
        KeyCode::Insert => b == "insert" || b == "ins",
        KeyCode::F(n) => b == format!("f{n}"),
        KeyCode::Null => b == "null",
        _ => false,
    }
}

fn is_prev_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Up | KeyCode::Left)
}

fn is_next_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Down | KeyCode::Right)
}

fn matches_confirm_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Enter | KeyCode::Char(' '))
}
