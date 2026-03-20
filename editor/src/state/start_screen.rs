//! Start screen command dispatch.

use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::asset_index::AssetIndex;
use crate::input::commands::Command;
use crate::io::fs_scan::{
    collect_schema_project_yml_files, infer_mod_root_from_project_yml, validate_project_dir,
};
use crate::io::indexer::build_project_index;
use crate::io::recent::push_recent;

use super::{now_millis, AppMode, AppState, DirBrowserItem, StartDialog};

impl AppState {
    pub(super) fn open_project(&mut self, path: &str) {
        self.reset_scene_fullscreen_state();
        let validation = validate_project_dir(Path::new(path));
        if !validation.valid {
            self.mode = AppMode::Start;
            self.start_dialog = StartDialog::RecentMenu;
            self.status = format!("Cannot open project: {path}");
            return;
        }
        let index = build_project_index(path);
        self.mode = AppMode::Browser;
        self.mod_source = path.to_string();
        self.index = index;
        self.watch.stamp = Self::compute_project_watch_stamp(&self.mod_source);
        self.watch.last_scan_ms = now_millis();
        self.explorer.items = self.build_tree_items();
        self.explorer.cursor = 0;
        self.scenes.scene_cursor = 0;
        self.scenes.scene_display_names =
            Self::build_scene_display_names(&self.mod_source, &self.index.scenes.scene_paths);
        self.scenes.scene_layer_cursor = 0;
        self.scenes.scene_layer_visibility.clear();
        self.scenes.scene_preview_started_at_ms = now_millis();
        self.sync_scene_preview_selection();
        self.refresh_cutscene_source_folder();
        push_recent(&mut self.recent_projects, path);
        self.status = format!("Opened: {path} | Ctrl+W close project");
    }

    pub(super) fn prune_stale_recents(&mut self) {
        let before = self.recent_projects.len();
        self.recent_projects.retain(|path| Path::new(path).exists());
        let removed = before.saturating_sub(self.recent_projects.len());
        self.start.cursor = self
            .start
            .cursor
            .min(self.start_items().len().saturating_sub(1));
        self.status = format!("Removed {removed} stale recent entrie(s)");
    }

    pub(super) fn close_project(&mut self) {
        self.mode = AppMode::Start;
        self.start_dialog = StartDialog::RecentMenu;
        self.mod_source.clear();
        self.index = AssetIndex::default();
        self.start.cursor = 0;
        self.picker.schema_cursor = 0;
        self.picker.dir_cursor = 0;
        self.picker.dir_preview_path.clear();
        self.picker.dir_preview_index = None;
        self.picker.dir_preview_popup = false;
        self.picker.dir_preview_started_at_ms = 0;
        self.scenes.scene_cursor = 0;
        self.scenes.scene_display_names.clear();
        self.scenes.scene_layer_cursor = 0;
        self.scenes.scene_layer_visibility.clear();
        self.scenes.scene_preview_layers.clear();
        self.scenes.scene_preview_scene = None;
        self.scenes.scene_preview_started_at_ms = 0;
        self.cutscene.source_dir = "assets/raw".to_string();
        self.cutscene.frames.clear();
        self.cutscene.missing_frames.clear();
        self.cutscene.validation_error = None;
        self.reset_scene_fullscreen_state();
        self.watch.last_scan_ms = 0;
        self.watch.stamp = 0;
        self.status =
            "Start: j/k move | Enter select | f schema scan | x prune stale | q quit".to_string();
    }

    fn open_schema_picker(&mut self) {
        self.picker.schema_candidates = collect_schema_project_yml_files(Path::new("."));
        self.picker.schema_cursor = 0;
        self.start_dialog = StartDialog::SchemaPicker;
        if self.picker.schema_candidates.is_empty() {
            self.status = "No schema-tagged .yml files found in current workspace".to_string();
        } else {
            self.status = "Select schema .yml and Enter to open project".to_string();
        }
    }

    fn open_directory_browser(&mut self, initial: &str) {
        self.start_dialog = StartDialog::DirectoryBrowser;
        self.picker.dir_cursor = 0;
        self.picker.dir_preview_popup = false;
        self.picker.dir_preview_started_at_ms = 0;
        self.refresh_directory_items(initial);
        self.status = "Directory browser: Enter open, F5 preview, Esc back, j/k move".to_string();
    }

    fn refresh_directory_items(&mut self, base: &str) {
        let canonical = fs::canonicalize(base).unwrap_or_else(|_| PathBuf::from(base));
        self.picker.dir_browser_path = canonical.display().to_string();
        let root_validation = validate_project_dir(&canonical);
        self.picker.dir_can_open = root_validation.valid;
        self.picker.dir_validation_code = root_validation.code.to_string();
        self.picker.dir_validation_message = root_validation.message;
        self.picker.dir_browser_items.clear();

        self.picker.dir_browser_items.push(DirBrowserItem::OpenHere);
        if canonical.parent().is_some() {
            self.picker.dir_browser_items.push(DirBrowserItem::Parent);
        }

        let mut dirs = Vec::new();
        if let Ok(entries) = fs::read_dir(&canonical) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let v = validate_project_dir(&p);
                    dirs.push((p.display().to_string(), v.valid, v.code.to_string()));
                }
            }
        }
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        self.picker
            .dir_browser_items
            .extend(dirs.into_iter().map(|(path, valid_project, code)| {
                DirBrowserItem::Directory {
                    path,
                    valid_project,
                    code,
                }
            }));
        self.picker.dir_cursor = self
            .picker
            .dir_cursor
            .min(self.picker.dir_browser_items.len().saturating_sub(1));
        self.refresh_dir_preview();
    }

    fn selected_directory_path(&self) -> Option<String> {
        match self.picker.dir_browser_items.get(self.picker.dir_cursor)? {
            DirBrowserItem::OpenHere => Some(self.picker.dir_browser_path.clone()),
            DirBrowserItem::Parent => Path::new(&self.picker.dir_browser_path)
                .parent()
                .map(|p| p.display().to_string()),
            DirBrowserItem::Directory { path, .. } => Some(path.clone()),
        }
    }

    fn refresh_dir_preview(&mut self) {
        let Some(path) = self.selected_directory_path() else {
            self.picker.dir_preview_path.clear();
            self.picker.dir_preview_index = None;
            return;
        };
        self.picker.dir_preview_path = path.clone();
        let validation = validate_project_dir(Path::new(&path));
        self.picker.dir_preview_index = if validation.valid {
            Some(build_project_index(&path))
        } else {
            None
        };
        if self.picker.dir_preview_index.is_none() {
            self.picker.dir_preview_popup = false;
            self.picker.dir_preview_started_at_ms = 0;
        }
    }

    fn toggle_dir_preview_popup(&mut self) {
        if self.picker.dir_preview_index.is_some() {
            self.picker.dir_preview_popup = !self.picker.dir_preview_popup;
            self.status = if self.picker.dir_preview_popup {
                self.picker.dir_preview_started_at_ms = now_millis();
                format!("Live preview x{} running", self.picker.dir_preview_speed_mult)
            } else {
                self.picker.dir_preview_started_at_ms = 0;
                "Preview closed".to_string()
            };
        } else {
            self.picker.dir_preview_popup = false;
            self.picker.dir_preview_started_at_ms = 0;
            self.status = "Preview unavailable for this folder".to_string();
        }
    }

    fn enter_directory_item(&mut self) {
        let Some(item) = self
            .picker
            .dir_browser_items
            .get(self.picker.dir_cursor)
            .cloned()
        else {
            return;
        };
        match item {
            DirBrowserItem::OpenHere => {
                if self.picker.dir_can_open {
                    let path = self.picker.dir_browser_path.clone();
                    self.open_project(&path);
                } else {
                    self.status = "Cannot open this directory".to_string();
                }
            }
            DirBrowserItem::Parent => {
                let parent = Path::new(&self.picker.dir_browser_path)
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| self.picker.dir_browser_path.clone());
                self.refresh_directory_items(&parent);
            }
            DirBrowserItem::Directory { path, .. } => self.refresh_directory_items(&path),
        }
    }

    pub(super) fn handle_start_screen_command(&mut self, cmd: Command) -> bool {
        match self.start_dialog {
            StartDialog::RecentMenu => self.apply_start_recent_menu(cmd),
            StartDialog::SchemaPicker => self.apply_start_schema_picker(cmd),
            StartDialog::DirectoryBrowser => self.apply_start_directory_browser(cmd),
        }
    }

    pub(super) fn apply_start_recent_menu(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::Left => {
                self.start.focus = match self.start.focus {
                    super::StartFocus::Recents => super::StartFocus::Actions,
                    super::StartFocus::Actions => super::StartFocus::Recents,
                };
            }
            Command::Right => {
                self.start.focus = match self.start.focus {
                    super::StartFocus::Recents => super::StartFocus::Actions,
                    super::StartFocus::Actions => super::StartFocus::Recents,
                };
            }
            Command::Up => match self.start.focus {
                super::StartFocus::Recents => {
                    self.start.recent_cursor = self.start.recent_cursor.saturating_sub(1);
                }
                super::StartFocus::Actions => {
                    self.start.action_cursor = self.start.action_cursor.saturating_sub(1);
                }
            },
            Command::Down => match self.start.focus {
                super::StartFocus::Recents => {
                    let max = self.recent_projects.len().saturating_sub(1);
                    self.start.recent_cursor = (self.start.recent_cursor + 1).min(max);
                }
                super::StartFocus::Actions => {
                    self.start.action_cursor = (self.start.action_cursor + 1).min(3); // 4 actions (0-3)
                }
            },
            Command::OpenProject => {
                let path = self.launch_mod_source.clone();
                self.open_directory_browser(&path);
            }
            Command::PruneRecents => self.prune_stale_recents(),
            Command::OpenSchemaPicker => self.open_schema_picker(),
            Command::Enter => match self.start.focus {
                super::StartFocus::Recents => {
                    if let Some(path) = self
                        .recent_projects
                        .get(self.start.recent_cursor)
                        .cloned()
                    {
                        self.open_project(&path);
                    }
                }
                super::StartFocus::Actions => match self.start.action_cursor {
                    0 => {
                        let path = self.launch_mod_source.clone();
                        self.open_directory_browser(&path);
                    }
                    1 => {
                        self.open_schema_picker();
                    }
                    2 => {
                        self.status = "New Project: coming soon (MVP browser)".to_string();
                    }
                    3 => {
                        return true;
                    }
                    _ => {}
                },
            },
            Command::Back
            | Command::CloseProject
            | Command::TogglePreview
            | Command::ToggleEffectsPreview
            | Command::EnterFile
            | Command::ExitEditor
            | Command::ToggleSidebar
            | Command::SelectPanel1
            | Command::SelectPanel2
            | Command::SelectPanel3
            | Command::SelectPanel4
            | Command::NextCodeTab
            | Command::PrevCodeTab
            | Command::SceneFullscreenHoldStart
            | Command::SceneFullscreenHoldEnd
            | Command::ToggleSceneFullscreen
            | Command::ToggleSceneLayer
            | Command::ToggleHelp
            | Command::Noop
            | Command::NextPane
            | Command::PrevPane => {}
        }
        false
    }

    pub(super) fn apply_start_schema_picker(&mut self, cmd: Command) -> bool {
        let max = self.picker.schema_candidates.len().saturating_sub(1);
        match cmd {
            Command::Quit => return true,
            Command::Back => {
                self.start_dialog = StartDialog::RecentMenu;
                self.status =
                    "Start: j/k move | Enter select | f schema scan | x prune stale | q quit"
                        .to_string();
            }
            Command::Up => self.picker.schema_cursor = self.picker.schema_cursor.saturating_sub(1),
            Command::Down => {
                self.picker.schema_cursor = (self.picker.schema_cursor + 1).min(max)
            }
            Command::Enter => {
                if let Some(path) = self
                    .picker
                    .schema_candidates
                    .get(self.picker.schema_cursor)
                    .cloned()
                {
                    if let Some(mod_root) = infer_mod_root_from_project_yml(Path::new(&path)) {
                        self.open_project(&mod_root);
                    } else {
                        self.status =
                            format!("Could not infer mod root from selected scene file: {path}");
                    }
                }
            }
            Command::OpenSchemaPicker => self.open_schema_picker(),
            Command::OpenProject => {
                let path = self.launch_mod_source.clone();
                self.open_directory_browser(&path);
            }
            Command::PruneRecents => self.prune_stale_recents(),
            Command::CloseProject
            | Command::Left
            | Command::Right
            | Command::NextPane
            | Command::PrevPane
            | Command::TogglePreview
            | Command::ToggleEffectsPreview
            | Command::EnterFile
            | Command::ExitEditor
            | Command::ToggleSidebar
            | Command::SelectPanel1
            | Command::SelectPanel2
            | Command::SelectPanel3
            | Command::SelectPanel4
            | Command::NextCodeTab
            | Command::PrevCodeTab
            | Command::SceneFullscreenHoldStart
            | Command::SceneFullscreenHoldEnd
            | Command::ToggleSceneFullscreen
            | Command::ToggleSceneLayer
            | Command::ToggleHelp
            | Command::Noop => {}
        }
        false
    }

    pub(super) fn apply_start_directory_browser(&mut self, cmd: Command) -> bool {
        let max = self.picker.dir_browser_items.len().saturating_sub(1);
        if self.picker.dir_preview_popup {
            match cmd {
                Command::Quit => return true,
                Command::Back | Command::TogglePreview => {
                    self.picker.dir_preview_popup = false;
                    self.picker.dir_preview_started_at_ms = 0;
                    self.status = "Preview closed".to_string();
                }
                Command::Up
                | Command::Down
                | Command::Left
                | Command::Right
                | Command::Enter
                | Command::OpenProject
                | Command::OpenSchemaPicker
                | Command::PruneRecents
                | Command::CloseProject
                | Command::NextPane
                | Command::PrevPane
                | Command::EnterFile
                | Command::ExitEditor
                | Command::ToggleSidebar
                | Command::ToggleEffectsPreview
                | Command::SelectPanel1
                | Command::SelectPanel2
                | Command::SelectPanel3
                | Command::SelectPanel4
                | Command::NextCodeTab
                | Command::PrevCodeTab
                | Command::SceneFullscreenHoldStart
                | Command::SceneFullscreenHoldEnd
                | Command::ToggleSceneFullscreen
                | Command::ToggleSceneLayer
                | Command::ToggleHelp
                | Command::Noop => {}
            }
            return false;
        }
        match cmd {
            Command::Quit => return true,
            Command::Back => {
                self.start_dialog = StartDialog::RecentMenu;
                self.status =
                    "Start: j/k move | Enter select | f schema scan | x prune stale | q quit"
                        .to_string();
            }
            Command::Up => self.picker.dir_cursor = self.picker.dir_cursor.saturating_sub(1),
            Command::Down => self.picker.dir_cursor = (self.picker.dir_cursor + 1).min(max),
            Command::Enter => self.enter_directory_item(),
            Command::OpenProject => {
                let path = self.picker.dir_browser_path.clone();
                self.refresh_directory_items(&path);
            }
            Command::PruneRecents => {}
            Command::TogglePreview => self.toggle_dir_preview_popup(),
            Command::OpenSchemaPicker => self.open_schema_picker(),
            Command::CloseProject
            | Command::Left
            | Command::Right
            | Command::NextPane
            | Command::PrevPane
            | Command::EnterFile
            | Command::ExitEditor
            | Command::ToggleSidebar
            | Command::ToggleEffectsPreview
            | Command::SelectPanel1
            | Command::SelectPanel2
            | Command::SelectPanel3
            | Command::SelectPanel4
            | Command::NextCodeTab
            | Command::PrevCodeTab
            | Command::SceneFullscreenHoldStart
            | Command::SceneFullscreenHoldEnd
            | Command::ToggleSceneFullscreen
            | Command::ToggleSceneLayer
            | Command::ToggleHelp
            | Command::Noop => {}
        }
        if matches!(cmd, Command::Up | Command::Down) {
            self.refresh_dir_preview();
        }
        false
    }
}
