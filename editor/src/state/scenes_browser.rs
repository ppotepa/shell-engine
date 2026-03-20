//! Scenes browser command dispatch.

use std::path::Path;

use crate::input::commands::Command;
use engine::repositories::{create_scene_repository, SceneRepository};
use engine_core::logging;

use super::{focus::FocusPane, now_millis, AppState, SidebarItem};

impl AppState {
    pub(super) fn build_scene_display_names(
        mod_source: &str,
        scene_paths: &[String],
    ) -> Vec<String> {
        let repo = create_scene_repository(Path::new(mod_source)).ok();
        scene_paths
            .iter()
            .map(|scene_path| {
                let scene_ref = Self::normalize_scene_ref_path_static(mod_source, scene_path);
                if let Some(scene) = repo.as_ref().and_then(|r| r.load_scene(&scene_ref).ok()) {
                    if !scene.title.trim().is_empty() {
                        return scene.title;
                    }
                    if !scene.id.trim().is_empty() {
                        return scene.id;
                    }
                }

                Path::new(scene_path)
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or(scene_path)
                    .to_string()
            })
            .collect()
    }

    pub(super) fn activate_scenes_browser(&mut self) {
        self.reset_scene_fullscreen_state();
        self.refresh_project_index_now();
        self.sidebar.active = SidebarItem::Scenes;
        self.sidebar.visible = false;
        self.focus = FocusPane::ProjectTree;
        self.sync_scene_preview_selection();
        self.scenes.scene_preview_started_at_ms = now_millis();
        self.status = self.scene_browser_status_message();
        logging::info(
            "editor.scenes",
            format!(
                "scenes browser active: scenes={} mod={}",
                self.index.scenes.scene_paths.len(),
                self.mod_source
            ),
        );
    }

    pub(super) fn scene_browser_status_message(&self) -> String {
        if self.index.scenes.scene_paths.is_empty() {
            return "Scenes Browser: no discoverable scenes found".to_string();
        }
        let scene_name = self
            .selected_scene_display_name()
            .unwrap_or_else(|| "<unknown-scene>".to_string());
        let scene_path = self
            .selected_scene_path()
            .map(|path| self.normalize_scene_ref_path(path))
            .unwrap_or_else(|| "<none>".to_string());
        format!(
            "Scenes Browser: F5 soft-run | F6 run | scene={} | path={} | mod={}",
            scene_name, scene_path, self.mod_source
        )
    }

    pub(super) fn reset_scene_fullscreen_state(&mut self) {
        self.scenes.scene_preview_fullscreen_hold = false;
        self.scenes.scene_preview_fullscreen_toggle = false;
    }

    pub(super) fn set_scene_fullscreen_hold(&mut self, enabled: bool) {
        if self.sidebar.active != SidebarItem::Scenes {
            return;
        }
        self.scenes.scene_preview_fullscreen_hold = enabled;
        if enabled {
            self.status = "Scenes Browser: fullscreen hold (release F to exit)".to_string();
        } else if self.scenes.scene_preview_fullscreen_toggle {
            self.status = "Scenes Browser: fullscreen toggle ON (Ctrl+F to exit)".to_string();
        } else {
            self.status = self.scene_browser_status_message();
        }
    }

    pub(super) fn toggle_scene_fullscreen(&mut self) {
        if self.sidebar.active != SidebarItem::Scenes {
            return;
        }
        self.scenes.scene_preview_fullscreen_toggle = !self.scenes.scene_preview_fullscreen_toggle;
        self.scenes.scene_preview_fullscreen_hold = false;
        self.status = if self.scenes.scene_preview_fullscreen_toggle {
            "Scenes Browser: fullscreen toggle ON (Ctrl+F to exit)".to_string()
        } else {
            "Scenes Browser: fullscreen toggle OFF".to_string()
        };
    }

    pub(super) fn move_scene_selection(&mut self, next_cursor: usize) {
        if self.index.scenes.scene_paths.is_empty() {
            self.scenes.scene_cursor = 0;
            self.scenes.scene_layer_cursor = 0;
            self.scenes.scene_preview_layers.clear();
            self.scenes.scene_preview_scene = None;
            return;
        }
        self.scenes.scene_cursor =
            next_cursor.min(self.index.scenes.scene_paths.len().saturating_sub(1));
        self.sync_scene_preview_selection();
        self.scenes.scene_preview_started_at_ms = now_millis();
        self.status = self.scene_browser_status_message();
        if let Some(name) = self.selected_scene_display_name() {
            logging::debug(
                "editor.scenes",
                format!(
                    "scene selection changed: index={} name={} path={}",
                    self.scenes.scene_cursor,
                    name,
                    self.selected_scene_path().unwrap_or("<none>")
                ),
            );
        }
    }

    pub(super) fn move_scene_layer_cursor(&mut self, delta: isize) {
        let len = self.scenes.scene_preview_layers.len();
        if len == 0 {
            self.scenes.scene_layer_cursor = 0;
            return;
        }
        let next = (self.scenes.scene_layer_cursor as isize + delta).clamp(0, (len - 1) as isize);
        self.scenes.scene_layer_cursor = next as usize;
    }

    pub(super) fn toggle_selected_scene_layer(&mut self) {
        if self.scenes.scene_preview_layers.is_empty() {
            return;
        }
        if self.scenes.scene_layer_visibility.len() != self.scenes.scene_preview_layers.len() {
            self.scenes.scene_layer_visibility = vec![true; self.scenes.scene_preview_layers.len()];
        }
        let idx = self
            .scenes
            .scene_layer_cursor
            .min(self.scenes.scene_layer_visibility.len().saturating_sub(1));
        self.scenes.scene_layer_visibility[idx] = !self.scenes.scene_layer_visibility[idx];
        let enabled = self
            .scenes
            .scene_layer_visibility
            .iter()
            .filter(|enabled| **enabled)
            .count();
        self.status = format!(
            "Scenes Browser: layer '{}' {} (visible: {enabled}/{})",
            self.scenes
                .scene_preview_layers
                .get(idx)
                .map(String::as_str)
                .unwrap_or("-"),
            if self.scenes.scene_layer_visibility[idx] {
                "enabled"
            } else {
                "disabled"
            },
            self.scenes.scene_preview_layers.len()
        );
    }

    pub(super) fn isolate_selected_scene_layer(&mut self) {
        if self.scenes.scene_preview_layers.is_empty() {
            return;
        }
        if self.scenes.scene_layer_visibility.len() != self.scenes.scene_preview_layers.len() {
            self.scenes.scene_layer_visibility = vec![true; self.scenes.scene_preview_layers.len()];
        }
        let idx = self
            .scenes
            .scene_layer_cursor
            .min(self.scenes.scene_layer_visibility.len().saturating_sub(1));
        for visible in &mut self.scenes.scene_layer_visibility {
            *visible = false;
        }
        self.scenes.scene_layer_visibility[idx] = true;
        self.status = format!(
            "Scenes Browser: solo layer '{}'",
            self.scenes
                .scene_preview_layers
                .get(idx)
                .map(String::as_str)
                .unwrap_or("-")
        );
    }

    pub(super) fn normalize_scene_ref_path(&self, scene_path: &str) -> String {
        Self::normalize_scene_ref_path_static(&self.mod_source, scene_path)
    }

    pub(super) fn normalize_scene_ref_path_static(mod_source: &str, scene_path: &str) -> String {
        let mut normalized = scene_path.replace('\\', "/");
        let mod_source = mod_source.replace('\\', "/");

        if normalized.starts_with(&mod_source) {
            normalized = normalized[mod_source.len()..].to_string();
        } else if let Some(idx) = normalized.find("/scenes/") {
            normalized = normalized[idx..].to_string();
        }

        if !normalized.starts_with('/') {
            normalized = format!("/{}", normalized.trim_start_matches('/'));
        }
        normalized
    }

    pub(super) fn sync_scene_preview_selection(&mut self) {
        let Some(scene_path) = self.selected_scene_path().map(str::to_string) else {
            self.scenes.scene_preview_layers.clear();
            self.scenes.scene_layer_visibility.clear();
            self.scenes.scene_layer_cursor = 0;
            self.scenes.scene_preview_scene = None;
            return;
        };

        if self.mod_source.is_empty() {
            self.scenes.scene_preview_layers.clear();
            self.scenes.scene_layer_visibility.clear();
            self.scenes.scene_layer_cursor = 0;
            self.scenes.scene_preview_scene = None;
            return;
        }

        let scene_ref = self.normalize_scene_ref_path(&scene_path);
        match create_scene_repository(Path::new(&self.mod_source))
            .and_then(|repo| repo.load_scene(&scene_ref))
        {
            Ok(scene) => {
                self.scenes.scene_preview_layers = scene
                    .layers
                    .iter()
                    .enumerate()
                    .map(|(idx, layer)| {
                        if layer.name.trim().is_empty() {
                            format!("layer-{idx}")
                        } else {
                            layer.name.clone()
                        }
                    })
                    .collect();
                self.scenes.scene_layer_visibility = vec![true; scene.layers.len()];
                self.scenes.scene_layer_cursor = self
                    .scenes
                    .scene_layer_cursor
                    .min(self.scenes.scene_preview_layers.len().saturating_sub(1));
                self.scenes.scene_preview_scene = Some(scene);
            }
            Err(err) => {
                self.scenes.scene_preview_layers.clear();
                self.scenes.scene_layer_visibility.clear();
                self.scenes.scene_layer_cursor = 0;
                self.scenes.scene_preview_scene = None;
                self.status = format!("Scenes Browser: failed to load preview scene ({err})");
            }
        }
    }

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
            Command::TogglePreview => {
                self.start_scene_soft_run();
                true
            }
            Command::RunHard => {
                self.start_scene_hard_run();
                true
            }
            Command::SceneFullscreenHoldStart => {
                if self.scenes.scene_preview_fullscreen_hold
                    && !self.scenes.scene_preview_fullscreen_toggle
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
