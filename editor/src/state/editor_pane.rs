//! Edit-mode command dispatch.

use std::fs;
use std::path::Path;

use crate::input::commands::Command;

use super::{AppMode, AppState, SidebarItem, TreeItem};

impl AppState {
    pub(super) fn enter_edit_mode(&mut self) {
        if let Some(item) = self.selected_tree_item() {
            let file_path = match item {
                TreeItem::ModYaml => Some("mod.yaml".to_string()),
                TreeItem::Scene(path) => Some(path.clone()),
                TreeItem::Image(path) => Some(path.clone()),
                TreeItem::Font(path) => Some(path.clone()),
                _ => None,
            };

            if let Some(path) = file_path {
                let full_path = Path::new(&self.mod_source).join(&path);
                match fs::read_to_string(&full_path) {
                    Ok(content) => {
                        self.editor.file = Some(path.clone());
                        self.editor.content = content;
                        self.mode = AppMode::EditMode;
                        self.status = format!("Editing: {} | ESC to exit", path);
                    }
                    Err(e) => {
                        self.status = format!("Cannot open file: {}", e);
                    }
                }
            }
        }
    }

    pub(super) fn exit_edit_mode(&mut self) {
        self.mode = AppMode::Browser;
        self.editor.file = None;
        self.editor.content.clear();
        self.status =
            "Browser: j/k navigate | Enter edit | Tab switch pane | Ctrl+W close | q quit"
                .to_string();
    }

    pub(super) fn handle_editor_pane_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::ExitEditor => {
                self.exit_edit_mode();
                true
            }
            Command::ToggleSidebar => {
                self.sidebar.visible = !self.sidebar.visible;
                true
            }
            Command::ToggleEffectsPreview => {
                self.toggle_effects_preview();
                true
            }
            Command::SelectPanel1 => {
                self.reset_scene_fullscreen_state();
                self.sidebar.active = SidebarItem::Scenes;
                self.sidebar.visible = false;
                true
            }
            Command::SelectPanel2 => {
                self.reset_scene_fullscreen_state();
                self.sidebar.active = SidebarItem::Explorer;
                self.sidebar.visible = true;
                true
            }
            Command::SelectPanel3 => {
                self.activate_effects_browser();
                true
            }
            Command::SelectPanel4 => {
                self.reset_scene_fullscreen_state();
                self.sidebar.active = SidebarItem::Cutscene;
                self.sidebar.visible = true;
                self.refresh_cutscene_source_folder();
                true
            }
            _ => false,
        }
    }
}
