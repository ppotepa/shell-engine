//! Project explorer command dispatch.

use crate::input::commands::Command;

use super::{AppState, SidebarItem, TreeItem};

impl AppState {
    /// Builds the flat ordered list of tree items from the current project index.
    pub(super) fn build_tree_items(&self) -> Vec<TreeItem> {
        let mut items = Vec::new();

        items.push(TreeItem::ModYaml);

        if !self.index.scenes.scene_paths.is_empty() {
            items.push(TreeItem::ScenesFolder);
            for scene in &self.index.scenes.scene_paths {
                items.push(TreeItem::Scene(scene.clone()));
            }
        }

        if !self.index.images.is_empty() {
            items.push(TreeItem::ImagesFolder);
            for image in &self.index.images {
                items.push(TreeItem::Image(image.clone()));
            }
        }

        if !self.index.fonts.is_empty() {
            items.push(TreeItem::FontsFolder);
            for font in &self.index.fonts {
                items.push(TreeItem::Font(font.clone()));
            }
        }

        items
    }

    /// Returns the tree item at the current cursor position, if any.
    pub fn selected_tree_item(&self) -> Option<&TreeItem> {
        self.explorer.items.get(self.explorer.cursor)
    }

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
