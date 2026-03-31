//! Sub-state structs and enums owned by [`super::AppState`].

use std::collections::HashMap;

use engine::scene::Scene;

use crate::domain::asset_index::AssetIndex;
use crate::domain::effect_params::EffectParamValue;
use crate::state::focus::FocusPane;

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
    SceneRun,
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

/// Scene run flavor selected from Scenes Browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneRunKind {
    Soft,
    Hard,
}

/// State owned by the scene-run player (F5 from Scenes Browser).
pub struct SceneRunState {
    pub kind: SceneRunKind,
    pub scene_path: String,
    pub scene_name: String,
    pub world: Option<engine::world::World>,
    pub last_tick_ms: u64,
}

impl std::fmt::Debug for SceneRunState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneRunState")
            .field("kind", &self.kind)
            .field("scene_path", &self.scene_path)
            .field("scene_name", &self.scene_name)
            .field("world_active", &self.world.is_some())
            .field("last_tick_ms", &self.last_tick_ms)
            .finish()
    }
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
