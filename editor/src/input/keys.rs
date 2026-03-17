//! Crossterm key-event mapping to editor [`Command`]s.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::commands::Command;
use crate::state::AppMode;

/// Maps a raw key event and the current application mode to a high-level [`Command`].
pub fn map_key_event(key: KeyEvent, mode: AppMode) -> Command {
    // EditMode: ESC exits, T toggles sidebar
    if mode == AppMode::EditMode {
        return match key.code {
            KeyCode::Esc => Command::ExitEditor,
            KeyCode::Char('t') | KeyCode::Char('T') => Command::ToggleSidebar,
            KeyCode::Char('f') | KeyCode::Char('F') => Command::ToggleEffectsPreview,
            KeyCode::Char('1') => Command::SelectPanel1,
            KeyCode::Char('2') => Command::SelectPanel2,
            KeyCode::Char('3') => Command::SelectPanel3,
            KeyCode::Char('4') => Command::SelectPanel4,
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Quit,
            _ => Command::Noop,
        };
    }

    // Browser mode: Enter opens file, T toggles sidebar, 1-4 select panels
    if mode == AppMode::Browser {
        match key.code {
            KeyCode::Enter => return Command::EnterFile,
            KeyCode::Char('t') | KeyCode::Char('T') => return Command::ToggleSidebar,
            KeyCode::Char('f') | KeyCode::Char('F') => return Command::ToggleEffectsPreview,
            KeyCode::Char('1') => return Command::SelectPanel1,
            KeyCode::Char('2') => return Command::SelectPanel2,
            KeyCode::Char('3') => return Command::SelectPanel3,
            KeyCode::Char('4') => return Command::SelectPanel4,
            KeyCode::Char(']') => return Command::NextCodeTab,
            KeyCode::Char('[') => return Command::PrevCodeTab,
            _ => {}
        }
    }

    // Standard bindings
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Command::Quit,
        (KeyCode::Esc, _) => Command::Back,
        (KeyCode::Left, _) | (KeyCode::Char('h'), _) => Command::Left,
        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => Command::Right,
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Command::Up,
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Command::Down,
        (KeyCode::Enter, _) => Command::Enter,
        (KeyCode::Char('o'), _) => Command::OpenProject,
        (KeyCode::Char('f'), _) | (KeyCode::Char('/'), _) => Command::OpenSchemaPicker,
        (KeyCode::Char('x'), _) => Command::PruneRecents,
        (KeyCode::F(5), _) => Command::TogglePreview,
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => Command::CloseProject,
        (KeyCode::Tab, KeyModifiers::NONE) => Command::NextPane,
        (KeyCode::BackTab, _) => Command::PrevPane,
        _ => Command::Noop,
    }
}
