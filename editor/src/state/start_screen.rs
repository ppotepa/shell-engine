//! Start screen command dispatch.

use std::path::Path;

use crate::input::commands::Command;
use crate::io::fs_scan::infer_mod_root_from_project_yml;

use super::{AppState, StartDialog};

impl AppState {
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
