//! Cutscene maker command dispatch.

use crate::input::commands::Command;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_cutscene_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Cutscene {
            return false;
        }

        match cmd {
            Command::TogglePreview => {
                self.refresh_cutscene_source_folder();
                self.status = self.cutscene_status_message();
                true
            }
            _ => false,
        }
    }
}
