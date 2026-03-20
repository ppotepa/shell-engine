//! Application state: mode, cursor positions, project index, and all UI-level state.

pub mod filters;
pub mod focus;
pub mod cutscene;
pub mod editor_pane;
pub mod effects_browser;
pub mod project_explorer;
pub mod scenes_browser;
pub mod selection;
pub mod start_screen;
pub mod watch;

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use engine::repositories::{create_scene_repository, SceneRepository};
use engine::scene::Scene;

use crate::domain::asset_index::AssetIndex;
use crate::domain::effect_params;
use crate::domain::effect_params::EffectParamValue;
use crate::domain::effects_catalog;
use crate::domain::effects_preview_scene;
use crate::input::commands::Command;
use crate::io::fs_scan::validate_project_dir;
use crate::io::indexer::build_project_index;
use crate::io::recent::push_recent;
use focus::FocusPane;

/// Available tabs in the effects browser code pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectsCodeTab {
    Info,
    Schema,
    Yaml,
    Rust,
}

impl EffectsCodeTab {
    pub const ALL: &'static [Self] = &[Self::Info, Self::Schema, Self::Yaml, Self::Rust];

    /// Returns the display label for this tab.
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Schema => "Schema",
            Self::Yaml => "YAML",
            Self::Rust => "Rust",
        }
    }

    /// Returns the next tab in cyclic order.
    pub fn next(self) -> Self {
        match self {
            Self::Info => Self::Schema,
            Self::Schema => Self::Yaml,
            Self::Yaml => Self::Rust,
            Self::Rust => Self::Info,
        }
    }

    /// Returns the previous tab in cyclic order.
    pub fn prev(self) -> Self {
        match self {
            Self::Info => Self::Rust,
            Self::Schema => Self::Info,
            Self::Yaml => Self::Schema,
            Self::Rust => Self::Yaml,
        }
    }
}

/// Top-level application mode controlling which screen and input bindings are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Start,
    Browser,
    EditMode,
}

/// Which overlay dialog is active on the start screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartDialog {
    RecentMenu,
    SchemaPicker,
    DirectoryBrowser,
}

/// Which column of the start screen (recents or actions) currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartFocus {
    Recents,
    Actions,
}

/// An action item selectable from the start screen actions column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartAction {
    OpenProject,
    OpenSchemaYml,
    NewProject,
    Quit,
}

/// A single selectable row in the start screen, either a recent project or an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartItem {
    Recent(usize),
    Action(StartAction),
}

/// An item in the directory browser list, representing a navigable entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirBrowserItem {
    OpenHere,
    Parent,
    Directory {
        path: String,
        valid_project: bool,
        code: String,
    },
}

/// A single entry in the project tree sidebar, representing a file or folder node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeItem {
    ModYaml,
    ScenesFolder,
    Scene(String),
    ImagesFolder,
    Image(String),
    FontsFolder,
    Font(String),
}

/// The panel currently shown in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarItem {
    Explorer,
    Search,
    Scenes,
    Cutscene,
}

/// State owned by the Effects Browser feature.
#[derive(Debug, Clone)]
pub struct EffectsBrowserState {
    pub builtin_effects: Vec<String>,
    pub effect_cursor: usize,
    pub effect_param_cursor: usize,
    pub effect_param_overrides: HashMap<String, EffectParamValue>,
    pub effects_live_preview: bool,
    pub effects_preview_started_at_ms: u64,
    pub effects_preview_scene_yaml: String,
    /// Scroll offset for the YAML/code pane in the effects browser.
    pub effects_code_scroll: u16,
    /// Active tab in the effects code pane.
    pub effects_code_tab: EffectsCodeTab,
}

/// State owned by the start screen.
#[derive(Debug, Clone)]
pub struct StartScreenState {
    pub focus: StartFocus,
    pub recent_cursor: usize,
    pub action_cursor: usize,
    pub cursor: usize,
}

/// State owned by start-screen project/schema pickers.
#[derive(Debug, Clone)]
pub struct StartPickerState {
    pub schema_candidates: Vec<String>,
    pub schema_cursor: usize,
    pub dir_browser_path: String,
    pub dir_browser_items: Vec<DirBrowserItem>,
    pub dir_cursor: usize,
    pub dir_can_open: bool,
    pub dir_validation_code: String,
    pub dir_validation_message: String,
    pub dir_preview_path: String,
    pub dir_preview_index: Option<AssetIndex>,
    pub dir_preview_popup: bool,
    pub dir_preview_speed_mult: u8,
    pub dir_preview_started_at_ms: u64,
}

/// State owned by the Scenes Browser feature.
#[derive(Debug, Clone)]
pub struct SceneBrowserState {
    pub scene_cursor: usize,
    pub scene_display_names: Vec<String>,
    pub scene_layer_cursor: usize,
    pub scene_layer_visibility: Vec<bool>,
    pub scene_preview_layers: Vec<String>,
    pub scene_preview_scene: Option<Scene>,
    pub scene_preview_started_at_ms: u64,
    pub scene_preview_fullscreen_hold: bool,
    pub scene_preview_fullscreen_toggle: bool,
}

/// State owned by the project explorer tree.
#[derive(Debug, Clone)]
pub struct ProjectExplorerState {
    pub cursor: usize,
    pub items: Vec<TreeItem>,
}

/// State owned by browser sidebar shell.
#[derive(Debug, Clone)]
pub struct SidebarState {
    pub active: SidebarItem,
    pub visible: bool,
}

/// State owned by the Cutscene Maker feature.
#[derive(Debug, Clone)]
pub struct CutsceneMakerState {
    pub source_dir: String,
    pub output_gif: String,
    pub default_frame_ms: u32,
    pub frames: Vec<String>,
    pub missing_frames: Vec<u32>,
    pub validation_error: Option<String>,
}

/// State owned by the project filesystem watcher.
#[derive(Debug, Clone)]
pub struct ProjectWatchState {
    pub interval_ms: u64,
    pub last_scan_ms: u64,
    pub stamp: u64,
}

/// State owned by the edit pane.
#[derive(Debug, Clone)]
pub struct EditorPaneState {
    pub file: Option<String>,
    pub content: String,
}

/// Complete runtime state for the editor application.
#[derive(Debug, Clone)]
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

    /// Builds the flat ordered list of tree items from the current project index.
    pub fn build_tree_items(&self) -> Vec<TreeItem> {
        let mut items = Vec::new();

        // mod.yaml always first
        items.push(TreeItem::ModYaml);

        // Scenes folder + scenes
        if !self.index.scenes.scene_paths.is_empty() {
            items.push(TreeItem::ScenesFolder);
            for scene in &self.index.scenes.scene_paths {
                items.push(TreeItem::Scene(scene.clone()));
            }
        }

        // Images folder + images
        if !self.index.images.is_empty() {
            items.push(TreeItem::ImagesFolder);
            for image in &self.index.images {
                items.push(TreeItem::Image(image.clone()));
            }
        }

        // Fonts folder + fonts
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

    /// Returns the name of the currently selected built-in effect, if any.
    pub fn selected_builtin_effect(&self) -> Option<&str> {
        self.effects.builtin_effects
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

    /// Returns display label for scene index based on authored YAML metadata.
    pub fn scene_display_name(&self, idx: usize) -> String {
        self.scenes.scene_display_names
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
        self.scenes.scene_layer_visibility
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
                    format!("1-4 screens | Tab pane | Enter edit | T sidebar | {help}")
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
                        format!("j/k scenes | Tab pane | F/Ctrl+F fullscreen | {help}")
                    }
                    FocusPane::Browser => {
                        format!("j/k layers | Space toggle | Enter solo | Tab pane | {help}")
                    }
                    FocusPane::Inspector => {
                        format!("F/Ctrl+F fullscreen | Tab pane | T sidebar | {help}")
                    }
                },
                SidebarItem::Cutscene => {
                    format!("F5 rescan | 1-4 screens | Tab pane | T sidebar | {help}")
                }
            },
            AppMode::EditMode => {
                if self.sidebar.active == SidebarItem::Search {
                    format!("Esc editor | F live | T sidebar | {help} | Ctrl+Q quit")
                } else {
                    format!("Esc editor | T sidebar | 1-4 screens | {help} | Ctrl+Q quit")
                }
            }
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
                    "Use 1-4 to switch screens and Tab to cycle focus.".to_string(),
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
                        "Press Tab to move into Layers Explorer or Live Preview.".to_string(),
                    ],
                    FocusPane::Browser => vec![
                        "Layers Explorer: enable, disable, or isolate layers of the selected scene."
                            .to_string(),
                        "Use j/k to move through layers.".to_string(),
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
                "Use 1-4 and T to switch helper side panels without leaving the editor."
                    .to_string(),
            ],
        };

        lines.push(String::new());
        lines.push(format!("Shortcuts: {}", self.current_shortcuts()));
        lines
    }

    fn build_scene_display_names(mod_source: &str, scene_paths: &[String]) -> Vec<String> {
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

    fn refresh_cutscene_source_folder(&mut self) {
        self.cutscene.frames.clear();
        self.cutscene.missing_frames.clear();
        self.cutscene.validation_error = None;

        if self.mod_source.is_empty() {
            self.cutscene.source_dir = "assets/raw".to_string();
            self.cutscene.validation_error = Some("Open a mod project first.".to_string());
            return;
        }

        let source_dir = Path::new(&self.mod_source).join("assets/raw");
        self.cutscene.source_dir = source_dir.display().to_string();
        if !source_dir.exists() {
            self.cutscene.validation_error = Some("Missing folder: assets/raw".to_string());
            return;
        }
        if !source_dir.is_dir() {
            self.cutscene.validation_error =
                Some("assets/raw exists but is not a directory".to_string());
            return;
        }

        let mut numbered_frames: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        let mut invalid_named_frames: Vec<String> = Vec::new();
        let mut read_failed = false;

        const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp"];

        match fs::read_dir(&source_dir) {
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        read_failed = true;
                        continue;
                    };
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
                        continue;
                    };
                    let ext = ext.to_ascii_lowercase();
                    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                        continue;
                    }

                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("<invalid-name>")
                        .to_string();
                    let stem = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("");
                    match stem.parse::<u32>() {
                        Ok(number) if number > 0 => {
                            numbered_frames.entry(number).or_default().push(file_name);
                        }
                        _ => invalid_named_frames.push(file_name),
                    }
                }
            }
            Err(err) => {
                self.cutscene.validation_error =
                    Some(format!("Cannot read assets/raw folder: {err}"));
                return;
            }
        }

        if read_failed {
            self.cutscene.validation_error =
                Some("Could not read some files from assets/raw".to_string());
            return;
        }

        if !invalid_named_frames.is_empty() {
            invalid_named_frames.sort();
            self.cutscene.validation_error = Some(format!(
                "Invalid frame names (must be numeric): {}",
                invalid_named_frames.join(", ")
            ));
            return;
        }

        if numbered_frames.is_empty() {
            self.cutscene.validation_error = Some(
                "No numbered image frames found in assets/raw (expected 1.png, 2.png, ...)"
                    .to_string(),
            );
            return;
        }

        let mut duplicate_numbers = Vec::new();
        for (number, names) in &numbered_frames {
            if names.len() > 1 {
                duplicate_numbers.push(format!("{number}"));
            }
        }
        if !duplicate_numbers.is_empty() {
            self.cutscene.validation_error = Some(format!(
                "Duplicate frame numbers detected: {}",
                duplicate_numbers.join(", ")
            ));
            return;
        }

        if let Some(max_number) = numbered_frames.keys().copied().max() {
            for expected in 1..=max_number {
                if !numbered_frames.contains_key(&expected) {
                    self.cutscene.missing_frames.push(expected);
                }
            }
        }

        for names in numbered_frames.values_mut() {
            names.sort();
        }
        self.cutscene.frames = numbered_frames
            .values()
            .filter_map(|names| names.first().cloned())
            .collect();

        if !self.cutscene.missing_frames.is_empty() {
            let preview = self
                .cutscene
                .missing_frames
                .iter()
                .take(12)
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = if self.cutscene.missing_frames.len() > 12 {
                ", ..."
            } else {
                ""
            };
            self.cutscene.validation_error =
                Some(format!("Missing frame numbers: {preview}{suffix}"));
        }
    }

    fn cutscene_status_message(&self) -> String {
        if let Some(err) = &self.cutscene.validation_error {
            return format!("Cutscene Maker: invalid source ({err})");
        }
        if self.cutscene.frames.is_empty() {
            "Cutscene Maker: no frames detected in assets/raw".to_string()
        } else {
            format!(
                "Cutscene Maker: {} frame(s) ready | default {}ms/frame",
                self.cutscene.frames.len(),
                self.cutscene.default_frame_ms
            )
        }
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

    fn open_project(&mut self, path: &str) {
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

    fn prune_stale_recents(&mut self) {
        let before = self.recent_projects.len();
        self.recent_projects.retain(|path| Path::new(path).exists());
        let removed = before.saturating_sub(self.recent_projects.len());
        self.start.cursor = self
            .start
            .cursor
            .min(self.start_items().len().saturating_sub(1));
        self.status = format!("Removed {removed} stale recent entrie(s)");
    }

    fn close_project(&mut self) {
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

    /// Applies the given command for the current mode; returns `true` if the app should quit.
    pub fn apply_command(&mut self, cmd: Command) -> bool {
        if matches!(cmd, Command::ToggleHelp) {
            self.help_overlay_active = !self.help_overlay_active;
            return false;
        }

        match self.mode {
            AppMode::Start => self.apply_start_command(cmd),
            AppMode::Browser => self.apply_browser_command(cmd),
            AppMode::EditMode => self.apply_edit_command(cmd),
        }
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
                self.sidebar.active = SidebarItem::Explorer;
                self.sidebar.visible = true;
            }
            Command::SelectPanel2 => self.activate_effects_browser(),
            Command::SelectPanel3 => self.activate_scenes_browser(),
            Command::SelectPanel4 => self.activate_cutscene_maker(),
            Command::PruneRecents => {}
            Command::TogglePreview => {
                let _ = self.handle_cutscene_command(cmd);
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

    fn enter_edit_mode(&mut self) {
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

    fn exit_edit_mode(&mut self) {
        self.mode = AppMode::Browser;
        self.editor.file = None;
        self.editor.content.clear();
        self.status =
            "Browser: j/k navigate | Enter edit | Tab switch pane | Ctrl+W close | q quit"
                .to_string();
    }

    /// Advances any in-progress transition animations by `dt_secs` seconds.
    pub fn update_transition(&mut self, _dt_secs: f32) {
        self.poll_project_refresh();
    }
}

fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
