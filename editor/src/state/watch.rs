//! Project watch and refresh state.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::io::fs_scan::collect_project_files;
use crate::io::indexer::build_project_index;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn poll_project_refresh(&mut self) {
        if self.mode == super::AppMode::Start || self.mod_source.is_empty() {
            return;
        }

        let now = super::now_millis();
        if now.saturating_sub(self.project_watch_last_scan_ms) < self.project_watch_interval_ms {
            return;
        }
        self.project_watch_last_scan_ms = now;

        let new_stamp = Self::compute_project_watch_stamp(&self.mod_source);
        if new_stamp == self.project_watch_stamp {
            return;
        }
        self.project_watch_stamp = new_stamp;
        self.reload_project_index_after_fs_change();
    }

    fn reload_project_index_after_fs_change(&mut self) {
        let previous_tree_item = self.selected_tree_item().cloned();
        let previous_scene_path = self.selected_scene_path().map(str::to_string);
        let refreshed_editor = self.reload_open_file_after_fs_change();

        self.index = build_project_index(&self.mod_source);
        self.scenes.scene_display_names =
            Self::build_scene_display_names(&self.mod_source, &self.index.scenes.scene_paths);
        self.tree_items = self.build_tree_items();
        self.refresh_cutscene_source_folder();

        if let Some(item) = previous_tree_item {
            if let Some(pos) = self
                .tree_items
                .iter()
                .position(|candidate| candidate == &item)
            {
                self.tree_cursor = pos;
            } else {
                self.tree_cursor = self
                    .tree_cursor
                    .min(self.tree_items.len().saturating_sub(1));
            }
        } else {
            self.tree_cursor = self
                .tree_cursor
                .min(self.tree_items.len().saturating_sub(1));
        }

        if let Some(scene_path) = previous_scene_path {
            if let Some(pos) = self
                .index
                .scenes
                .scene_paths
                .iter()
                .position(|path| path == &scene_path)
            {
                self.scenes.scene_cursor = pos;
            } else {
                self.scenes.scene_cursor = self
                    .scenes
                    .scene_cursor
                    .min(self.index.scenes.scene_paths.len().saturating_sub(1));
            }
        } else {
            self.scenes.scene_cursor = self
                .scenes
                .scene_cursor
                .min(self.index.scenes.scene_paths.len().saturating_sub(1));
        }

        self.sync_scene_preview_selection();
        self.scenes.scene_preview_started_at_ms = super::now_millis();
        self.status = if refreshed_editor {
            "Detected file changes: project lists, previews, and editor buffer refreshed"
                .to_string()
        } else {
            "Detected file changes: project lists and previews refreshed".to_string()
        };
        if self.sidebar_active == SidebarItem::Cutscene {
            self.status = self.cutscene_status_message();
        }
    }

    fn reload_open_file_after_fs_change(&mut self) -> bool {
        let Some(path) = self.editing_file.clone() else {
            return false;
        };

        let full_path = Path::new(&self.mod_source).join(&path);
        match fs::read_to_string(&full_path) {
            Ok(content) => {
                let changed = self.edit_content != content;
                self.edit_content = content;
                changed
            }
            Err(_) => {
                let missing_notice = format!(
                    "File is no longer available on disk:\n{}\n\nClose the editor pane or restore the file.",
                    full_path.display()
                );
                let changed = self.edit_content != missing_notice;
                self.edit_content = missing_notice;
                changed
            }
        }
    }

    pub(super) fn compute_project_watch_stamp(mod_source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        mod_source.hash(&mut hasher);
        let root = Path::new(mod_source);

        let watched_files = collect_project_files(root);
        watched_files.len().hash(&mut hasher);

        for path in watched_files {
            path.hash(&mut hasher);
            match fs::metadata(&path) {
                Ok(meta) => {
                    meta.len().hash(&mut hasher);
                    match meta.modified() {
                        Ok(modified) => {
                            if let Ok(delta) = modified.duration_since(UNIX_EPOCH) {
                                delta.as_secs().hash(&mut hasher);
                                delta.subsec_nanos().hash(&mut hasher);
                            }
                        }
                        Err(_) => 0_u8.hash(&mut hasher),
                    }
                }
                Err(_) => 255_u8.hash(&mut hasher),
            }
        }

        hasher.finish()
    }
}
