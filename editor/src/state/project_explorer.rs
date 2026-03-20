//! Project explorer command dispatch.

use crate::input::commands::Command;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_project_explorer_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Explorer {
            return false;
        }

        match cmd {
            Command::Up => {
                self.explorer.cursor = self.explorer.cursor.saturating_sub(1);
                true
            }
            Command::Down => {
                let max = self.explorer.items.len().saturating_sub(1);
                self.explorer.cursor = (self.explorer.cursor + 1).min(max);
                true
            }
            Command::EnterFile => {
                self.enter_edit_mode();
                true
            }
            _ => false,
        }
    }
}
