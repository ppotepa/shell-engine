//! Scenes browser command dispatch.

use crate::input::commands::Command;

use super::{focus::FocusPane, AppState, SidebarItem};

impl AppState {
    pub(super) fn handle_scenes_browser_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Scenes {
            return false;
        }
        match cmd {
            Command::Up => {
                if self.focus == FocusPane::ProjectTree {
                    self.move_scene_selection(self.scenes.scene_cursor.saturating_sub(1));
                } else if self.focus == FocusPane::Browser {
                    self.move_scene_layer_cursor(-1);
                }
                true
            }
            Command::Down => {
                if self.focus == FocusPane::ProjectTree {
                    let max = self.index.scenes.scene_paths.len().saturating_sub(1);
                    self.move_scene_selection((self.scenes.scene_cursor + 1).min(max));
                } else if self.focus == FocusPane::Browser {
                    self.move_scene_layer_cursor(1);
                }
                true
            }
            Command::EnterFile => {
                if self.focus == FocusPane::Browser {
                    self.isolate_selected_scene_layer();
                }
                true
            }
            Command::ToggleEffectsPreview => {
                self.set_scene_fullscreen_hold(true);
                true
            }
            Command::SceneFullscreenHoldStart => {
                if self.scenes.scene_preview_fullscreen_hold && !self.scenes.scene_preview_fullscreen_toggle
                {
                    self.set_scene_fullscreen_hold(false);
                } else {
                    self.set_scene_fullscreen_hold(true);
                }
                true
            }
            Command::SceneFullscreenHoldEnd => {
                self.set_scene_fullscreen_hold(false);
                true
            }
            Command::ToggleSceneLayer => {
                if self.focus == FocusPane::Browser {
                    self.toggle_selected_scene_layer();
                }
                true
            }
            _ => false,
        }
    }
}
