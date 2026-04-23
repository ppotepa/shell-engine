//! Raw-terminal key input for the launcher menu — supports arrow keys, Enter, Escape.

use super::state::{MenuAction, MenuState};
use anyhow::Result;
use console::{Key, Term};

pub fn wait_for_input(state: &mut MenuState) -> Result<MenuAction> {
    let term = Term::stdout();
    let key = term.read_key()?;

    let action = match key {
        Key::ArrowUp | Key::Char('k') => {
            state.navigate(-1);
            MenuAction::Redraw
        }
        Key::ArrowDown | Key::Char('j') => {
            state.navigate(1);
            MenuAction::Redraw
        }
        Key::ArrowLeft | Key::Char('h') | Key::Char('b') => {
            state.collapse_current();
            MenuAction::Redraw
        }
        Key::ArrowRight | Key::Enter | Key::Char('l') => state.enter_action(),
        Key::Escape | Key::Char('q') => MenuAction::Quit,
        Key::Char(c) if c.is_ascii_digit() && c != '0' => {
            let n = (c as u8 - b'0') as usize;
            if n <= 7 {
                // Flag toggles (1-7)
                state.toggle_flag(n as u8)
            } else if n <= state.filtered_indices.len() {
                state.cursor = n - 1;
                state.enter_action()
            } else {
                MenuAction::Redraw
            }
        }
        _ => MenuAction::None,
    };

    Ok(action)
}
