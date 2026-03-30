//! Application state: mode, cursor positions, project index, and all UI-level state.

pub mod cutscene;
pub mod editor_pane;
pub mod effects_browser;
pub mod filters;
pub mod focus;
pub mod project_explorer;
pub mod scene_run;
pub mod scenes_browser;
pub mod selection;
pub mod start_screen;
pub mod watch;

use std::collections::HashMap;
use std::path::Path;

use engine::scene::Scene;
use engine_core::logging;

use crate::domain::asset_index::AssetIndex;
use crate::domain::effect_params;
use crate::domain::effect_params::EffectParamValue;
use crate::domain::effects_catalog;
use crate::domain::effects_preview_scene;
use crate::input::commands::Command;
use crate::io::fs_scan::validate_project_dir;
use focus::FocusPane;

pub mod types;
pub use types::*;

/// Complete runtime state for the editor application.
#[derive(Debug)]
pub struct AppState {
    pub launch_mod_source: String,
    pub mode: AppMode,
    pub start_dialog: StartDialog,
    pub mod_source: String,
    pub focus: FocusPane,
    pub index: AssetIndex,
    pub status: String,
    pub recent_projects: Vec<String>,
    pub start: StartScreenState,
    pub picker: StartPickerState,
    pub explorer: ProjectExplorerState,
    pub editor: EditorPaneState,
    pub sidebar: SidebarState,
    pub effects: EffectsBrowserState,
    pub scenes: SceneBrowserState,
    pub scene_run: SceneRunState,
    pub cutscene: CutsceneMakerState,
    pub watch: ProjectWatchState,
    /// Whether the current-screen help overlay is visible.
    pub help_overlay_active: bool,
}

impl AppState {
    /// Creates a new [`AppState`] with the given mod source path and recent project list.
    pub fn new(launch_mod_source: String, recent_projects: Vec<String>) -> Self {
        let builtin_effects = effects_catalog::builtin_effect_names();
        let initial_effect = builtin_effects
            .first()
            .cloned()
            .unwrap_or_else(|| "shine".to_string());
        let initial_params = effect_params::default_effect_params(&initial_effect);

        Self {
            launch_mod_source,
            mode: AppMode::Start,
            start_dialog: StartDialog::RecentMenu,
            mod_source: String::new(),
            focus: FocusPane::ProjectTree,
            index: AssetIndex::default(),
            status: "Start: j/k move | Enter select | f schema scan | x prune stale | q quit"
                .to_string(),
            recent_projects,
            start: StartScreenState {
                focus: StartFocus::Recents,
                recent_cursor: 0,
                action_cursor: 0,
                cursor: 0,
            },
            picker: StartPickerState {
                schema_candidates: Vec::new(),
                schema_cursor: 0,
                dir_browser_path: ".".to_string(),
                dir_browser_items: Vec::new(),
                dir_cursor: 0,
                dir_can_open: false,
                dir_validation_code: "E_MOD_MISSING".to_string(),
                dir_validation_message: "mod.yaml not found".to_string(),
                dir_preview_path: String::new(),
                dir_preview_index: None,
                dir_preview_popup: false,
                dir_preview_speed_mult: 5,
                dir_preview_started_at_ms: 0,
            },
            explorer: ProjectExplorerState {
                cursor: 0,
                items: Vec::new(),
            },
            editor: EditorPaneState {
                file: None,
                content: String::new(),
            },
            sidebar: SidebarState {
                active: SidebarItem::Explorer,
                visible: true,
            },
            effects: EffectsBrowserState {
                builtin_effects,
                effect_cursor: 0,
                effect_param_cursor: 0,
                effect_param_overrides: HashMap::new(),
                effects_live_preview: false,
                effects_preview_started_at_ms: 0,
                effects_preview_scene_yaml: effects_preview_scene::build_preview_scene_yaml_default(
                    &initial_effect,
                    &initial_params,
                ),
                effects_code_scroll: 0,
                effects_code_tab: EffectsCodeTab::Info,
            },
            scenes: SceneBrowserState {
                scene_cursor: 0,
                scene_display_names: Vec::new(),
                scene_layer_cursor: 0,
                scene_layer_visibility: Vec::new(),
                scene_preview_layers: Vec::new(),
                scene_preview_scene: None,
                scene_preview_started_at_ms: 0,
                scene_preview_fullscreen_hold: false,
                scene_preview_fullscreen_toggle: false,
            },
            scene_run: SceneRunState {
                kind: SceneRunKind::Soft,
                scene_path: String::new(),
                scene_name: String::new(),
                world: None,
                last_tick_ms: 0,
            },
            cutscene: CutsceneMakerState {
                source_dir: "assets/raw".to_string(),
                output_gif: "assets/images/intro/cutscene.gif".to_string(),
                default_frame_ms: 250,
                frames: Vec::new(),
                missing_frames: Vec::new(),
                validation_error: None,
            },
            watch: ProjectWatchState {
                interval_ms: 1200,
                last_scan_ms: 0,
                stamp: 0,
            },
            help_overlay_active: false,
        }
    }

    /// Returns the name of the currently selected built-in effect, if any.
    pub fn selected_builtin_effect(&self) -> Option<&str> {
        self.effects
            .builtin_effects
            .get(self.effects.effect_cursor)
            .map(String::as_str)
    }

    /// Returns the currently selected scene path from the indexed scene list, if any.
    pub fn selected_scene_path(&self) -> Option<&str> {
        self.index
            .scenes
            .scene_paths
            .get(self.scenes.scene_cursor)
            .map(String::as_str)
    }

    /// Returns selected scene path normalized to a scene reference (`/scenes/...`).
    pub fn selected_scene_ref_path(&self) -> Option<String> {
        self.selected_scene_path()
            .map(|scene_path| self.normalize_scene_ref_path(scene_path))
    }

    /// Returns display label for scene index based on authored YAML metadata.
    pub fn scene_display_name(&self, idx: usize) -> String {
        self.scenes
            .scene_display_names
            .get(idx)
            .cloned()
            .or_else(|| self.index.scenes.scene_paths.get(idx).cloned())
            .unwrap_or_else(|| "<unknown-scene>".to_string())
    }

    /// Returns display label for currently selected scene.
    pub fn selected_scene_display_name(&self) -> Option<String> {
        if self.index.scenes.scene_paths.is_empty() {
            None
        } else {
            Some(self.scene_display_name(self.scenes.scene_cursor))
        }
    }

    /// Returns whether the given layer index is enabled for preview.
    pub fn scene_layer_enabled(&self, idx: usize) -> bool {
        self.scenes
            .scene_layer_visibility
            .get(idx)
            .copied()
            .unwrap_or(true)
    }

    /// Returns the normalised playback progress (0.0–1.0) of the live effects preview.
    pub fn effect_preview_progress(&self) -> f32 {
        if !self.effects.effects_live_preview {
            return 0.0;
        }
        let start = self.effects.effects_preview_started_at_ms;
        if start == 0 {
            return 0.0;
        }
        let elapsed_ms = now_millis().saturating_sub(start);
        ((elapsed_ms % 1600) as f32) / 1600.0
    }

    /// Returns the normalised playback progress (0.0–1.0) of the scene live preview.
    pub fn scene_preview_progress(&self) -> f32 {
        let start = self.scenes.scene_preview_started_at_ms;
        if start == 0 {
            return 0.0;
        }
        let elapsed_ms = now_millis().saturating_sub(start);
        ((elapsed_ms % 3000) as f32) / 3000.0
    }

    /// Returns whether scene preview should be shown as fullscreen in browser mode.
    pub fn scene_preview_fullscreen_active(&self) -> bool {
        self.scenes.scene_preview_fullscreen_hold || self.scenes.scene_preview_fullscreen_toggle
    }

    /// Returns the top-level application mode label for the header and status bar.
    pub fn current_mode_label(&self) -> &'static str {
        match self.mode {
            AppMode::Start => "START",
            AppMode::Browser => "NORMAL",
            AppMode::EditMode => "EDIT",
            AppMode::SceneRun => match self.scene_run.kind {
                SceneRunKind::Soft => "SOFT",
                SceneRunKind::Hard => "RUN",
            },
        }
    }

    /// Returns the current screen name, refined by the active pane when useful.
    pub fn current_screen_name(&self) -> String {
        match self.mode {
            AppMode::Start => match self.start_dialog {
                StartDialog::RecentMenu => "Start / Project Launcher".to_string(),
                StartDialog::SchemaPicker => "Start / Schema Picker".to_string(),
                StartDialog::DirectoryBrowser => "Start / Directory Browser".to_string(),
            },
            AppMode::Browser => match self.sidebar.active {
                SidebarItem::Explorer => "Project Explorer".to_string(),
                SidebarItem::Search => match self.focus {
                    FocusPane::ProjectTree => "Effects Browser / Effect List".to_string(),
                    FocusPane::Browser => "Effects Browser / Docs".to_string(),
                    FocusPane::Inspector => "Effects Browser / Parameters".to_string(),
                },
                SidebarItem::Scenes => match self.focus {
                    FocusPane::ProjectTree => "Scenes Browser / Scene List".to_string(),
                    FocusPane::Browser => "Scenes Browser / Layers Explorer".to_string(),
                    FocusPane::Inspector => "Scenes Browser / Live Preview".to_string(),
                },
                SidebarItem::Cutscene => match self.focus {
                    FocusPane::ProjectTree => "Cutscene Maker / Source".to_string(),
                    FocusPane::Browser => "Cutscene Maker / Validation".to_string(),
                    FocusPane::Inspector => "Cutscene Maker / Export".to_string(),
                },
            },
            AppMode::EditMode => self
                .editor
                .file
                .as_ref()
                .map(|path| format!("Edit Mode / {path}"))
                .unwrap_or_else(|| "Edit Mode / File Editor".to_string()),
            AppMode::SceneRun => {
                if self.scene_run.scene_name.is_empty() {
                    "Scene Run / Active Scene".to_string()
                } else {
                    format!("Scene Run / {}", self.scene_run.scene_name)
                }
            }
        }
    }

    fn help_toggle_label(&self) -> &'static str {
        if self.help_overlay_active {
            "F1 hide help"
        } else {
            "F1 help"
        }
    }

    /// Returns the shortcut legend for the currently visible screen.
    pub fn current_shortcuts(&self) -> String {
        let help = self.help_toggle_label();
        match self.mode {
            AppMode::Start => match self.start_dialog {
                StartDialog::RecentMenu => {
                    format!("Tab panels | j/k move | Enter select | {help} | q quit")
                }
                StartDialog::SchemaPicker => {
                    format!("j/k move | Enter open | Esc back | {help} | q quit")
                }
                StartDialog::DirectoryBrowser => {
                    format!("j/k move | Enter open | F5 preview | Esc back | {help}")
                }
            },
            AppMode::Browser => match self.sidebar.active {
                SidebarItem::Explorer => {
                    format!("1 scenes | 2 explorer | 3 effects | 4 cutscene | Enter edit | {help}")
                }
                SidebarItem::Search => match self.focus {
                    FocusPane::ProjectTree => {
                        format!("j/k effect | Enter controls | [/] tabs | F live | {help}")
                    }
                    FocusPane::Browser => {
                        format!("↑/↓ scroll | [/] tabs | Tab pane | F live | {help}")
                    }
                    FocusPane::Inspector => {
                        format!("↑/↓ param | ←/→ adjust | Tab pane | F live | {help}")
                    }
                },
                SidebarItem::Scenes => match self.focus {
                    FocusPane::ProjectTree => {
                        format!(
                            "j/k scenes | F5 soft-run | F6 run | Tab pane | F/Ctrl+F fullscreen | {help}"
                        )
                    }
                    FocusPane::Browser => {
                        format!("j/k layers | Space toggle | Enter solo | F5 soft-run | F6 run | {help}")
                    }
                    FocusPane::Inspector => {
                        format!("F5 soft-run | F6 run | F/Ctrl+F fullscreen | Tab pane | {help}")
                    }
                },
                SidebarItem::Cutscene => {
                    format!("F5 rescan | 1 scenes | 2 explorer | 3 effects | 4 cutscene | {help}")
                }
            },
            AppMode::EditMode => {
                if self.sidebar.active == SidebarItem::Search {
                    format!("Esc editor | F live | T sidebar | {help} | Ctrl+Q quit")
                } else {
                    format!("Esc editor | 1 scenes | 2 explorer | 3 effects | 4 cutscene | {help}")
                }
            }
            AppMode::SceneRun => format!("Esc back to editor | Ctrl+Q quit | {help}"),
        }
    }

    /// Returns the help text shown after toggling `F1` on the current screen.
    pub fn current_help(&self) -> Vec<String> {
        let mut lines = match self.mode {
            AppMode::Start => match self.start_dialog {
                StartDialog::RecentMenu => vec![
                    "Open a Shell Quest project from recents or from the action list.".to_string(),
                    "Tab switches between Recent Projects and Actions.".to_string(),
                    "Use j/k to move and Enter to open the highlighted item.".to_string(),
                    "Press f to scan schema-tagged YAML files or o to browse directories."
                        .to_string(),
                ],
                StartDialog::SchemaPicker => vec![
                    "Pick a schema-tagged YAML file and the editor will infer the mod root."
                        .to_string(),
                    "Use j/k to move through the result list.".to_string(),
                    "Press Enter to open the inferred project or Esc to return.".to_string(),
                ],
                StartDialog::DirectoryBrowser => vec![
                    "Browse directories and open a valid Shell Quest mod root.".to_string(),
                    "The left column is the navigator, the right side previews the selected folder."
                        .to_string(),
                    "Press F5 to toggle the live preview popup for the selected folder."
                        .to_string(),
                ],
            },
            AppMode::Browser => match self.sidebar.active {
                SidebarItem::Explorer => vec![
                    "Project Explorer shows the project tree on the left and a content preview in the center."
                        .to_string(),
                    "Use 1/2/3/4 to switch screens (1 Scenes, 2 Explorer, 3 Effects, 4 Cutscene)."
                        .to_string(),
                    "Press Enter on a file to open it in Edit Mode.".to_string(),
                    "Press T to hide or show the sidebar.".to_string(),
                ],
                SidebarItem::Search => match self.focus {
                    FocusPane::ProjectTree => vec![
                        "Effect List: choose a builtin effect to inspect and preview.".to_string(),
                        "Use j/k to move through effects.".to_string(),
                        "Press Enter to jump to the live controls pane.".to_string(),
                    ],
                    FocusPane::Browser => vec![
                        "Docs pane: browse Info, Schema, YAML, and Rust source tabs for the selected effect."
                            .to_string(),
                        "Use [ and ] to switch tabs.".to_string(),
                        "When live preview is on, use ↑/↓ to scroll the current tab.".to_string(),
                    ],
                    FocusPane::Inspector => vec![
                        "Parameters pane: tweak live effect controls.".to_string(),
                        "Use ↑/↓ to select a parameter and ←/→ to adjust it.".to_string(),
                        "Press F to toggle the live preview.".to_string(),
                    ],
                },
                SidebarItem::Scenes => match self.focus {
                    FocusPane::ProjectTree => vec![
                        "Scene List: choose which authored scene to preview.".to_string(),
                        "Use j/k to move through scenes.".to_string(),
                        "Press F5 for SOFT RUN (single-scene loop, no scene transitions)."
                            .to_string(),
                        "Press F6 for RUN (hard run with scene transitions).".to_string(),
                        "Press Tab to move into Layers Explorer or Live Preview.".to_string(),
                    ],
                    FocusPane::Browser => vec![
                        "Layers Explorer: enable, disable, or isolate layers of the selected scene."
                            .to_string(),
                        "Use j/k to move through layers.".to_string(),
                        "Press F5 for SOFT RUN; press F6 for RUN.".to_string(),
                        "Press Space to toggle a layer and Enter to solo it.".to_string(),
                    ],
                    FocusPane::Inspector => vec![
                        "Live Preview renders the selected scene inside the editor.".to_string(),
                        "Hold F for temporary fullscreen preview.".to_string(),
                        "Press Ctrl+F to toggle fullscreen until you press it again.".to_string(),
                    ],
                },
                SidebarItem::Cutscene => vec![
                    "Cutscene Maker validates stop-action source folders.".to_string(),
                    "Expected naming is strict: 1.png, 2.png, 3.png ... without gaps."
                        .to_string(),
                    "Use F5 to rescan assets/raw after adding or renaming frames.".to_string(),
                ],
            },
            AppMode::EditMode => vec![
                "File Editor shows the selected file in the center pane.".to_string(),
                "Edit Mode is visually marked with green borders.".to_string(),
                "Press Esc to return to Browser mode.".to_string(),
                "Use 1/2/3/4 to switch helper screens without leaving the editor."
                    .to_string(),
            ],
            AppMode::SceneRun => match self.scene_run.kind {
                SceneRunKind::Soft => vec![
                    "SOFT RUN plays only the selected scene.".to_string(),
                    "Scene transitions are ignored in this mode.".to_string(),
                    "Press Esc to stop playback and return to the editor.".to_string(),
                ],
                SceneRunKind::Hard => vec![
                    "RUN plays the selected scene with normal scene transitions.".to_string(),
                    "Use this for realistic end-to-end flow checks.".to_string(),
                    "Press Esc to stop playback and return to the editor.".to_string(),
                ],
            },
        };

        lines.push(String::new());
        lines.push(format!("Shortcuts: {}", self.current_shortcuts()));
        lines
    }

    /// Returns the flat ordered list of start screen items (recents then actions).
    pub fn start_items(&self) -> Vec<StartItem> {
        let mut items = (0..self.recent_projects.len())
            .map(StartItem::Recent)
            .collect::<Vec<_>>();
        items.extend([
            StartItem::Action(StartAction::OpenProject),
            StartItem::Action(StartAction::OpenSchemaYml),
            StartItem::Action(StartAction::NewProject),
            StartItem::Action(StartAction::Quit),
        ]);
        items
    }

    /// Returns the validation status label and valid flag for a recent project by index.
    pub fn recent_project_status(&self, idx: usize) -> (String, bool) {
        let Some(path) = self.recent_projects.get(idx) else {
            return ("MISSING".to_string(), false);
        };
        if !Path::new(path).exists() {
            return ("STALE".to_string(), false);
        }
        let v = validate_project_dir(Path::new(path));
        (v.code.to_string(), v.valid)
    }

    /// Applies the given command for the current mode; returns `true` if the app should quit.
    pub fn apply_command(&mut self, cmd: Command) -> bool {
        let mode_before = self.mode;
        let sidebar_before = self.sidebar.active;
        let focus_before = self.focus;
        logging::debug(
            "editor.command",
            format!(
                "dispatch: mode={:?} sidebar={:?} focus={:?} cmd={:?}",
                mode_before, sidebar_before, focus_before, cmd
            ),
        );

        if matches!(cmd, Command::ToggleHelp) {
            self.help_overlay_active = !self.help_overlay_active;
            logging::info(
                "editor.state",
                format!("help overlay toggled: active={}", self.help_overlay_active),
            );
            return false;
        }

        let should_quit = match self.mode {
            AppMode::Start => self.apply_start_command(cmd),
            AppMode::Browser => self.apply_browser_command(cmd),
            AppMode::EditMode => self.apply_edit_command(cmd),
            AppMode::SceneRun => self.handle_scene_run_command(cmd),
        };

        if should_quit {
            logging::info(
                "editor.state",
                format!("quit requested by command={cmd:?} in mode={mode_before:?}"),
            );
        }
        if self.mode != mode_before {
            logging::info(
                "editor.state",
                format!("mode changed: {:?} -> {:?}", mode_before, self.mode),
            );
        }
        if self.sidebar.active != sidebar_before {
            logging::info(
                "editor.state",
                format!(
                    "sidebar changed: {:?} -> {:?}",
                    sidebar_before, self.sidebar.active
                ),
            );
        }
        if self.focus != focus_before {
            logging::debug(
                "editor.state",
                format!("focus changed: {:?} -> {:?}", focus_before, self.focus),
            );
        }

        should_quit
    }

    fn apply_start_command(&mut self, cmd: Command) -> bool {
        self.handle_start_screen_command(cmd)
    }

    fn apply_browser_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::CloseProject => self.close_project(),
            Command::Up => {
                if !self.handle_project_explorer_command(cmd)
                    && !self.handle_effects_browser_command(cmd)
                    && !self.handle_scenes_browser_command(cmd)
                    && self.focus == FocusPane::ProjectTree
                {
                    self.explorer.cursor = self.explorer.cursor.saturating_sub(1);
                }
            }
            Command::Down => {
                if !self.handle_project_explorer_command(cmd)
                    && !self.handle_effects_browser_command(cmd)
                    && !self.handle_scenes_browser_command(cmd)
                    && self.focus == FocusPane::ProjectTree
                {
                    let max = self.explorer.items.len().saturating_sub(1);
                    self.explorer.cursor = (self.explorer.cursor + 1).min(max);
                }
            }
            Command::Left => {
                let _ = self.handle_effects_browser_command(cmd);
            }
            Command::Right => {
                let _ = self.handle_effects_browser_command(cmd);
            }
            Command::NextPane => self.focus = self.focus.next(),
            Command::PrevPane => self.focus = self.focus.prev(),
            Command::EnterFile => {
                if !self.handle_project_explorer_command(cmd) {
                    let _ = self.handle_effects_browser_command(cmd);
                    let _ = self.handle_scenes_browser_command(cmd);
                }
            }
            Command::ToggleSidebar => self.sidebar.visible = !self.sidebar.visible,
            Command::SelectPanel1 => {
                self.reset_scene_fullscreen_state();
                self.activate_scenes_browser();
            }
            Command::SelectPanel2 => {
                self.reset_scene_fullscreen_state();
                self.sidebar.active = SidebarItem::Explorer;
                self.sidebar.visible = true;
            }
            Command::SelectPanel3 => self.activate_effects_browser(),
            Command::SelectPanel4 => self.activate_cutscene_maker(),
            Command::PruneRecents => {}
            Command::TogglePreview => {
                let _ =
                    self.handle_scenes_browser_command(cmd) || self.handle_cutscene_command(cmd);
            }
            Command::RunHard => {
                let _ = self.handle_scenes_browser_command(cmd);
            }
            Command::ToggleEffectsPreview => {
                let _ = self.handle_scenes_browser_command(cmd)
                    || self.handle_effects_browser_command(cmd);
            }
            Command::SceneFullscreenHoldStart => {
                let _ = self.handle_scenes_browser_command(cmd)
                    || self.handle_effects_browser_command(cmd);
            }
            Command::SceneFullscreenHoldEnd => {
                let _ = self.handle_scenes_browser_command(cmd);
            }
            Command::ToggleSceneFullscreen => self.toggle_scene_fullscreen(),
            Command::ToggleSceneLayer => {
                let _ = self.handle_scenes_browser_command(cmd);
            }
            Command::NextCodeTab => {
                let _ = self.handle_effects_browser_command(cmd);
            }
            Command::PrevCodeTab => {
                let _ = self.handle_effects_browser_command(cmd);
            }
            Command::Back
            | Command::Noop
            | Command::Enter
            | Command::OpenProject
            | Command::OpenSchemaPicker
            | Command::ExitEditor
            | Command::ToggleHelp => {}
        }
        false
    }

    fn apply_edit_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::ExitEditor
            | Command::ToggleSidebar
            | Command::ToggleEffectsPreview
            | Command::SelectPanel1
            | Command::SelectPanel2
            | Command::SelectPanel3
            | Command::SelectPanel4 => {
                let _ = self.handle_editor_pane_command(cmd);
            }
            Command::Left
            | Command::Right
            | Command::Up
            | Command::Down
            | Command::EnterFile
            | Command::Back
            | Command::Noop
            | Command::Enter
            | Command::OpenProject
            | Command::OpenSchemaPicker
            | Command::PruneRecents
            | Command::TogglePreview
            | Command::RunHard
            | Command::NextCodeTab
            | Command::PrevCodeTab
            | Command::CloseProject
            | Command::NextPane
            | Command::PrevPane
            | Command::SceneFullscreenHoldStart
            | Command::SceneFullscreenHoldEnd
            | Command::ToggleSceneFullscreen
            | Command::ToggleSceneLayer
            | Command::ToggleHelp => {}
        }
        false
    }

    /// Advances any in-progress transition animations by `dt_secs` seconds.
    pub fn update_transition(&mut self, dt_secs: f32) {
        self.tick_scene_run(dt_secs);
        if self.mode != AppMode::SceneRun {
            self.poll_project_refresh();
        }
    }
}

fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
