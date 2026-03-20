//! Edit-mode command dispatch.

use crate::input::commands::Command;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_editor_pane_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::ExitEditor => {
                self.exit_edit_mode();
                true
            }
            Command::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                true
            }
            Command::ToggleEffectsPreview => {
                self.toggle_effects_preview();
                true
            }
            Command::SelectPanel1 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Explorer;
                self.sidebar_visible = true;
                true
            }
            Command::SelectPanel2 => {
                self.activate_effects_browser();
                true
            }
            Command::SelectPanel3 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Scenes;
                self.sidebar_visible = true;
                true
            }
            Command::SelectPanel4 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Cutscene;
                self.sidebar_visible = true;
                self.refresh_cutscene_source_folder();
                true
            }
            _ => false,
        }
    }
}
