//! Start screen command dispatch.

use crate::input::commands::Command;

use super::{AppState, StartDialog};

impl AppState {
    pub(super) fn handle_start_screen_command(&mut self, cmd: Command) -> bool {
        match self.start_dialog {
            StartDialog::RecentMenu => self.apply_start_recent_menu(cmd),
            StartDialog::SchemaPicker => self.apply_start_schema_picker(cmd),
            StartDialog::DirectoryBrowser => self.apply_start_directory_browser(cmd),
        }
    }
}
