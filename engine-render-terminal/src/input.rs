//! Terminal input backend: crossterm event polling and key conversion.

use crossterm::event::{self, Event, KeyEventKind};
use engine_events::{EngineEvent, InputBackend, KeyCode, KeyEvent, KeyModifiers};

/// Crossterm-based terminal input backend.
pub struct TerminalInputBackend;

impl TerminalInputBackend {
    pub fn new(_debug_feature: bool) -> std::io::Result<Self> {
        Ok(Self)
    }
}

impl InputBackend for TerminalInputBackend {
    fn poll_events(&mut self) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        while event::poll(std::time::Duration::ZERO).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if let Some(engine_key) = crossterm_key_to_engine(key) {
                        match key.kind {
                            KeyEventKind::Release => {
                                events.push(EngineEvent::KeyReleased(engine_key));
                            }
                            KeyEventKind::Press | KeyEventKind::Repeat => {
                                if is_quit_key(engine_key.code, engine_key.modifiers) {
                                    events.push(EngineEvent::Quit);
                                } else {
                                    events.push(EngineEvent::KeyPressed(engine_key));
                                }
                            }
                        }
                    }
                }
                Ok(Event::Resize(w, h)) => {
                    events.push(EngineEvent::OutputResized {
                        width: w,
                        height: h,
                    });
                }
                Ok(Event::Mouse(mouse)) => {
                    events.push(EngineEvent::MouseMoved {
                        column: mouse.column,
                        row: mouse.row,
                    });
                }
                _ => {}
            }
        }
        events
    }
}

/// Converts a crossterm key event into an engine key event.
/// Returns `None` for key events that have no engine mapping.
pub fn crossterm_key_to_engine(key: crossterm::event::KeyEvent) -> Option<KeyEvent> {
    let code = match key.code {
        event::KeyCode::Char(c) => KeyCode::Char(c),
        event::KeyCode::Enter => KeyCode::Enter,
        event::KeyCode::Backspace => KeyCode::Backspace,
        event::KeyCode::Tab => KeyCode::Tab,
        event::KeyCode::Esc => KeyCode::Esc,
        event::KeyCode::Up => KeyCode::Up,
        event::KeyCode::Down => KeyCode::Down,
        event::KeyCode::Left => KeyCode::Left,
        event::KeyCode::Right => KeyCode::Right,
        event::KeyCode::Home => KeyCode::Home,
        event::KeyCode::End => KeyCode::End,
        event::KeyCode::PageUp => KeyCode::PageUp,
        event::KeyCode::PageDown => KeyCode::PageDown,
        event::KeyCode::Delete => KeyCode::Delete,
        event::KeyCode::Insert => KeyCode::Insert,
        event::KeyCode::F(n) => KeyCode::F(n),
        event::KeyCode::BackTab => KeyCode::BackTab,
        _ => return None,
    };
    let mut modifiers = KeyModifiers::NONE;
    if key
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT)
    {
        modifiers |= KeyModifiers::SHIFT;
    }
    if key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        modifiers |= KeyModifiers::CONTROL;
    }
    if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
        modifiers |= KeyModifiers::ALT;
    }
    Some(KeyEvent::new(code, modifiers))
}

/// Returns `true` when the key combination is Ctrl+F5 (debug fast-forward toggle).
pub fn is_debug_fast_forward_toggle(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::F(5)
}

fn is_quit_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
        && matches!(
            code,
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('q') | KeyCode::Char('Q')
        )
}
