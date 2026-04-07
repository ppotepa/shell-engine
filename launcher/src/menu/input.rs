use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use super::state::{MenuState, MenuAction};

pub fn wait_for_input(state: &mut MenuState) -> Result<MenuAction> {
    loop {
        let ev = event::read()?;
        match ev {
            Event::Key(key) => {
                // Only handle Press events — ignore Release/Repeat to prevent double-jump
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                return handle_key(state, key);
            }
            Event::Resize(_, _) => {
                // Signal a re-render on resize
                return Ok(MenuAction::Redraw);
            }
            _ => continue,
        }
    }
}

fn handle_key(state: &mut MenuState, key: KeyEvent) -> Result<MenuAction> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(MenuAction::Quit);
    }

    if state.search_mode {
        return handle_search_key(state, key);
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.navigate(-1);
            Ok(MenuAction::Redraw)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.navigate(1);
            Ok(MenuAction::Redraw)
        }
        KeyCode::Right => {
            let action = state.expand_or_launch();
            Ok(action)
        }
        KeyCode::Enter => {
            let action = state.enter_action();
            Ok(action)
        }
        KeyCode::Left => {
            state.collapse_current();
            Ok(MenuAction::Redraw)
        }
        KeyCode::Esc => {
            state.collapse_current();
            Ok(MenuAction::Redraw)
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => Ok(MenuAction::Quit),
        KeyCode::Char('/') => {
            state.search_mode = true;
            Ok(MenuAction::Redraw)
        }
        KeyCode::Char(c) if c.is_ascii_digit() && ('1'..='7').contains(&c) => {
            let n = (c as u8) - b'0';
            Ok(state.toggle_flag(n))
        }
        // Backspace outside search — collapse or noop
        KeyCode::Backspace => {
            state.collapse_current();
            Ok(MenuAction::Redraw)
        }
        _ => Ok(MenuAction::None),
    }
}

fn handle_search_key(state: &mut MenuState, key: KeyEvent) -> Result<MenuAction> {
    match key.code {
        KeyCode::Esc => {
            state.search_mode = false;
            state.search_clear();
            Ok(MenuAction::Redraw)
        }
        KeyCode::Enter => {
            // Confirm search, exit search mode but keep filter active, and launch if 1 result
            state.search_mode = false;
            // If there's exactly one scene visible, launch it
            let scene_count = state.filtered_indices.iter()
                .filter(|(_, s)| s.is_some()).count();
            if scene_count == 1 {
                state.cursor = state.filtered_indices.iter()
                    .position(|(_, s)| s.is_some()).unwrap_or(0);
                return Ok(MenuAction::Launch);
            }
            Ok(MenuAction::Redraw)
        }
        KeyCode::Backspace => {
            state.search_backspace();
            Ok(MenuAction::Redraw)
        }
        KeyCode::Char(c) => {
            state.search_add_char(c);
            Ok(MenuAction::Redraw)
        }
        _ => Ok(MenuAction::None),
    }
}