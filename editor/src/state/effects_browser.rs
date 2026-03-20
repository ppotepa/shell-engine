//! Effects browser command dispatch.

use crate::input::commands::Command;

use super::{focus::FocusPane, AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_effects_browser_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Search {
            return false;
        }
        match cmd {
            Command::Up => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.move_effect_param_cursor(-1);
                } else if self.effects.effects_live_preview && self.focus == FocusPane::Browser {
                    self.effects.effects_code_scroll =
                        self.effects.effects_code_scroll.saturating_sub(1);
                } else {
                    self.move_effect_selection(self.effects.effect_cursor.saturating_sub(1));
                }
                true
            }
            Command::Down => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.move_effect_param_cursor(1);
                } else if self.effects.effects_live_preview && self.focus == FocusPane::Browser {
                    self.effects.effects_code_scroll =
                        self.effects.effects_code_scroll.saturating_add(1);
                } else {
                    let max = self.effects.builtin_effects.len().saturating_sub(1);
                    self.move_effect_selection((self.effects.effect_cursor + 1).min(max));
                }
                true
            }
            Command::Left => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.adjust_selected_effect_param(-1.0);
                }
                true
            }
            Command::Right => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.adjust_selected_effect_param(1.0);
                }
                true
            }
            Command::EnterFile => {
                if !self.effects.effects_live_preview {
                    self.effects.effects_live_preview = true;
                }
                self.focus = FocusPane::Inspector;
                self.sync_effect_param_cursor();
                self.restart_effect_preview_clock();
                self.status =
                    "Effects Browser: controls focused | ↑/↓ param | ←/→ adjust | F toggle"
                        .to_string();
                true
            }
            Command::ToggleEffectsPreview | Command::SceneFullscreenHoldStart => {
                self.toggle_effects_preview();
                true
            }
            Command::NextCodeTab => {
                self.effects.effects_code_scroll = 0;
                self.effects.effects_code_tab = self.effects.effects_code_tab.next();
                true
            }
            Command::PrevCodeTab => {
                self.effects.effects_code_scroll = 0;
                self.effects.effects_code_tab = self.effects.effects_code_tab.prev();
                true
            }
            _ => false,
        }
    }
}
