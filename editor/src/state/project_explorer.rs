//! Project explorer command dispatch.

use crate::input::commands::Command;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_project_explorer_command(&mut self, cmd: Command) -> bool {
        if self.sidebar_active != SidebarItem::Explorer {
            return false;
        }

        match cmd {
            Command::Up => {
                self.tree_cursor = self.tree_cursor.saturating_sub(1);
                true
            }
            Command::Down => {
                let max = self.tree_items.len().saturating_sub(1);
                self.tree_cursor = (self.tree_cursor + 1).min(max);
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
