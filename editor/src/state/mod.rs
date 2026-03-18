//! Application state: mode, cursor positions, project index, and all UI-level state.

pub mod filters;
pub mod focus;
pub mod selection;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use engine::repositories::{create_scene_repository, SceneRepository};
use engine::scene::Scene;

use crate::domain::asset_index::AssetIndex;
use crate::domain::effect_params::{self, EffectParamSpec, EffectParamValue};
use crate::domain::effects_catalog;
use crate::domain::effects_preview_scene;
use crate::input::commands::Command;
use crate::io::fs_scan::{
    collect_files, collect_game_yaml_files, collect_schema_project_yml_files,
    infer_mod_root_from_project_yml, validate_project_dir,
};
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
    Settings,
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
    pub start_focus: StartFocus,
    pub recent_cursor: usize,
    pub action_cursor: usize,
    pub start_cursor: usize,
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
    pub tree_cursor: usize,
    pub tree_items: Vec<TreeItem>,
    pub editing_file: Option<String>,
    pub edit_content: String,
    pub sidebar_active: SidebarItem,
    pub sidebar_visible: bool,
    pub builtin_effects: Vec<String>,
    pub effect_cursor: usize,
    pub effect_param_cursor: usize,
    pub effect_param_overrides: HashMap<String, EffectParamValue>,
    pub effects_live_preview: bool,
    pub effects_preview_started_at_ms: u64,
    pub effects_preview_scene_yaml: String,
    pub scene_cursor: usize,
    pub scene_display_names: Vec<String>,
    pub scene_layer_cursor: usize,
    pub scene_layer_visibility: Vec<bool>,
    pub scene_preview_layers: Vec<String>,
    pub scene_preview_scene: Option<Scene>,
    pub scene_preview_started_at_ms: u64,
    pub scene_preview_fullscreen_hold: bool,
    pub scene_preview_fullscreen_toggle: bool,
    pub project_watch_interval_ms: u64,
    pub project_watch_last_scan_ms: u64,
    pub project_watch_stamp: u64,
    /// Scroll offset for the YAML code pane in the effects browser.
    pub effects_code_scroll: u16,
    /// Active tab in the code pane.
    pub effects_code_tab: EffectsCodeTab,
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
            start_focus: StartFocus::Recents,
            recent_cursor: 0,
            action_cursor: 0,
            start_cursor: 0,
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
            tree_cursor: 0,
            tree_items: Vec::new(),
            editing_file: None,
            edit_content: String::new(),
            sidebar_active: SidebarItem::Explorer,
            sidebar_visible: true,
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
            scene_cursor: 0,
            scene_display_names: Vec::new(),
            scene_layer_cursor: 0,
            scene_layer_visibility: Vec::new(),
            scene_preview_layers: Vec::new(),
            scene_preview_scene: None,
            scene_preview_started_at_ms: 0,
            scene_preview_fullscreen_hold: false,
            scene_preview_fullscreen_toggle: false,
            project_watch_interval_ms: 1200,
            project_watch_last_scan_ms: 0,
            project_watch_stamp: 0,
            effects_code_scroll: 0,
            effects_code_tab: EffectsCodeTab::Info,
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
        self.tree_items.get(self.tree_cursor)
    }

    /// Returns the name of the currently selected built-in effect, if any.
    pub fn selected_builtin_effect(&self) -> Option<&str> {
        self.builtin_effects
            .get(self.effect_cursor)
            .map(String::as_str)
    }

    /// Returns the currently selected scene path from the indexed scene list, if any.
    pub fn selected_scene_path(&self) -> Option<&str> {
        self.index
            .scenes
            .scene_paths
            .get(self.scene_cursor)
            .map(String::as_str)
    }

    /// Returns display label for scene index based on authored YAML metadata.
    pub fn scene_display_name(&self, idx: usize) -> String {
        self.scene_display_names
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
            Some(self.scene_display_name(self.scene_cursor))
        }
    }

    /// Returns the currently selected layer name for the scene preview, if any.
    pub fn selected_scene_layer(&self) -> Option<&str> {
        self.scene_preview_layers
            .get(self.scene_layer_cursor)
            .map(String::as_str)
    }

    /// Returns whether the given layer index is enabled for preview.
    pub fn scene_layer_enabled(&self, idx: usize) -> bool {
        self.scene_layer_visibility
            .get(idx)
            .copied()
            .unwrap_or(true)
    }

    /// Returns the normalised playback progress (0.0–1.0) of the live effects preview.
    pub fn effect_preview_progress(&self) -> f32 {
        if !self.effects_live_preview {
            return 0.0;
        }
        let start = self.effects_preview_started_at_ms;
        if start == 0 {
            return 0.0;
        }
        let elapsed_ms = now_millis().saturating_sub(start);
        ((elapsed_ms % 1600) as f32) / 1600.0
    }

    /// Returns the normalised playback progress (0.0–1.0) of the scene live preview.
    pub fn scene_preview_progress(&self) -> f32 {
        let start = self.scene_preview_started_at_ms;
        if start == 0 {
            return 0.0;
        }
        let elapsed_ms = now_millis().saturating_sub(start);
        ((elapsed_ms % 3000) as f32) / 3000.0
    }

    /// Returns whether scene preview should be shown as fullscreen in browser mode.
    pub fn scene_preview_fullscreen_active(&self) -> bool {
        self.scene_preview_fullscreen_hold || self.scene_preview_fullscreen_toggle
    }

    fn compute_project_watch_stamp(mod_source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        mod_source.hash(&mut hasher);
        let root = Path::new(mod_source);

        let mut watched_files = collect_game_yaml_files(root);
        watched_files.extend(collect_files(root, "assets/images", "png"));
        watched_files.extend(collect_files(root, "assets/fonts", "yaml"));
        watched_files.sort();
        watched_files.dedup();
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

    fn poll_project_refresh(&mut self) {
        if self.mode == AppMode::Start || self.mod_source.is_empty() {
            return;
        }

        let now = now_millis();
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

        self.index = build_project_index(&self.mod_source);
        self.scene_display_names =
            Self::build_scene_display_names(&self.mod_source, &self.index.scenes.scene_paths);
        self.tree_items = self.build_tree_items();

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
                self.scene_cursor = pos;
            } else {
                self.scene_cursor = self
                    .scene_cursor
                    .min(self.index.scenes.scene_paths.len().saturating_sub(1));
            }
        } else {
            self.scene_cursor = self
                .scene_cursor
                .min(self.index.scenes.scene_paths.len().saturating_sub(1));
        }

        self.sync_scene_preview_selection();
        self.status = "Detected file changes: project lists refreshed".to_string();
    }

    fn restart_effect_preview_clock(&mut self) {
        self.effects_preview_started_at_ms = now_millis();
    }

    /// Returns the parameter specifications for the currently selected effect.
    pub fn effect_param_specs(&self) -> &'static [EffectParamSpec] {
        self.selected_builtin_effect()
            .map(effect_params::effect_param_specs)
            .unwrap_or(&[])
    }

    /// Returns the spec for the currently focused effect parameter, if any.
    pub fn selected_effect_param_spec(&self) -> Option<&'static EffectParamSpec> {
        self.effect_param_specs().get(self.effect_param_cursor)
    }

    /// Returns the current value for the given parameter, preferring user overrides.
    pub fn effect_param_value(&self, spec: &EffectParamSpec) -> EffectParamValue {
        if let Some(value) = self.effect_param_overrides.get(spec.name) {
            return *value;
        }

        self.selected_builtin_effect()
            .map(effect_params::default_effect_params)
            .and_then(|params| effect_params::effect_param_value(&params, spec.name))
            .unwrap_or_else(|| spec.default_value())
    }

    fn sync_effect_param_cursor(&mut self) {
        let len = self.effect_param_specs().len();
        if len == 0 {
            self.effect_param_cursor = 0;
        } else {
            self.effect_param_cursor = self.effect_param_cursor.min(len - 1);
        }
    }

    fn reset_selected_effect_preview(&mut self) {
        self.effect_param_cursor = 0;
        self.effect_param_overrides.clear();
        self.effects_code_scroll = 0;
        self.effects_code_tab = EffectsCodeTab::Info;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
    }

    fn move_effect_selection(&mut self, next_cursor: usize) {
        if next_cursor != self.effect_cursor {
            self.effect_cursor = next_cursor;
            self.reset_selected_effect_preview();
        } else {
            self.restart_effect_preview_clock();
        }
    }

    fn move_effect_param_cursor(&mut self, delta: isize) {
        let len = self.effect_param_specs().len();
        if len == 0 {
            self.effect_param_cursor = 0;
            return;
        }

        let next = (self.effect_param_cursor as isize + delta).clamp(0, (len - 1) as isize);
        self.effect_param_cursor = next as usize;
    }

    fn adjust_selected_effect_param(&mut self, delta_dir: f32) {
        let Some(spec) = self.selected_effect_param_spec().copied() else {
            return;
        };

        let current = self.effect_param_value(&spec).as_float();
        let next = spec.adjust(current, delta_dir);
        self.effect_param_overrides
            .insert(spec.name.to_string(), next);
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.status = format!("{}: {}", spec.label, spec.render_value(next.as_float()));
    }

    fn activate_effects_browser(&mut self) {
        self.reset_scene_fullscreen_state();
        self.sidebar_active = SidebarItem::Search;
        self.sidebar_visible = true;
        self.effects_live_preview = true;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.sync_effect_param_cursor();
        self.status = "Effects Browser: LIVE preview ON | Tab focus | j/k effect | Enter controls"
            .to_string();
    }

    fn activate_scenes_browser(&mut self) {
        self.reset_scene_fullscreen_state();
        self.sidebar_active = SidebarItem::Scenes;
        self.sidebar_visible = false;
        self.focus = FocusPane::ProjectTree;
        self.sync_scene_preview_selection();
        self.scene_preview_started_at_ms = now_millis();
        self.status = if self.index.scenes.scene_paths.is_empty() {
            "Scenes Browser: no discoverable scenes found".to_string()
        } else {
            "Scenes Browser: j/k scenes | Tab focus | Space toggle | Enter solo | Ctrl+F fullscreen"
                .to_string()
        };
    }

    fn reset_scene_fullscreen_state(&mut self) {
        self.scene_preview_fullscreen_hold = false;
        self.scene_preview_fullscreen_toggle = false;
    }

    fn set_scene_fullscreen_hold(&mut self, enabled: bool) {
        if self.sidebar_active != SidebarItem::Scenes {
            return;
        }
        self.scene_preview_fullscreen_hold = enabled;
        if enabled {
            self.status = "Scenes Browser: fullscreen hold (release F to exit)".to_string();
        } else if self.scene_preview_fullscreen_toggle {
            self.status = "Scenes Browser: fullscreen toggle ON (Ctrl+F to exit)".to_string();
        } else {
            self.status =
                "Scenes Browser: j/k scenes | Tab focus | Space toggle | Enter solo".to_string();
        }
    }

    fn toggle_scene_fullscreen(&mut self) {
        if self.sidebar_active != SidebarItem::Scenes {
            return;
        }
        self.scene_preview_fullscreen_toggle = !self.scene_preview_fullscreen_toggle;
        self.scene_preview_fullscreen_hold = false;
        self.status = if self.scene_preview_fullscreen_toggle {
            "Scenes Browser: fullscreen toggle ON (Ctrl+F to exit)".to_string()
        } else {
            "Scenes Browser: fullscreen toggle OFF".to_string()
        };
    }

    fn move_scene_selection(&mut self, next_cursor: usize) {
        if self.index.scenes.scene_paths.is_empty() {
            self.scene_cursor = 0;
            self.scene_layer_cursor = 0;
            self.scene_preview_layers.clear();
            self.scene_preview_scene = None;
            return;
        }
        self.scene_cursor = next_cursor.min(self.index.scenes.scene_paths.len().saturating_sub(1));
        self.sync_scene_preview_selection();
        self.scene_preview_started_at_ms = now_millis();
    }

    fn move_scene_layer_cursor(&mut self, delta: isize) {
        let len = self.scene_preview_layers.len();
        if len == 0 {
            self.scene_layer_cursor = 0;
            return;
        }
        let next = (self.scene_layer_cursor as isize + delta).clamp(0, (len - 1) as isize);
        self.scene_layer_cursor = next as usize;
    }

    fn toggle_selected_scene_layer(&mut self) {
        if self.scene_preview_layers.is_empty() {
            return;
        }
        if self.scene_layer_visibility.len() != self.scene_preview_layers.len() {
            self.scene_layer_visibility = vec![true; self.scene_preview_layers.len()];
        }
        let idx = self
            .scene_layer_cursor
            .min(self.scene_layer_visibility.len().saturating_sub(1));
        self.scene_layer_visibility[idx] = !self.scene_layer_visibility[idx];
        let enabled = self
            .scene_layer_visibility
            .iter()
            .filter(|enabled| **enabled)
            .count();
        self.status = format!(
            "Scenes Browser: layer '{}' {} (visible: {enabled}/{})",
            self.scene_preview_layers
                .get(idx)
                .map(String::as_str)
                .unwrap_or("-"),
            if self.scene_layer_visibility[idx] {
                "enabled"
            } else {
                "disabled"
            },
            self.scene_preview_layers.len()
        );
    }

    fn isolate_selected_scene_layer(&mut self) {
        if self.scene_preview_layers.is_empty() {
            return;
        }
        if self.scene_layer_visibility.len() != self.scene_preview_layers.len() {
            self.scene_layer_visibility = vec![true; self.scene_preview_layers.len()];
        }
        let idx = self
            .scene_layer_cursor
            .min(self.scene_layer_visibility.len().saturating_sub(1));
        for visible in &mut self.scene_layer_visibility {
            *visible = false;
        }
        self.scene_layer_visibility[idx] = true;
        self.status = format!(
            "Scenes Browser: solo layer '{}'",
            self.scene_preview_layers
                .get(idx)
                .map(String::as_str)
                .unwrap_or("-")
        );
    }

    fn normalize_scene_ref_path(&self, scene_path: &str) -> String {
        Self::normalize_scene_ref_path_static(&self.mod_source, scene_path)
    }

    fn normalize_scene_ref_path_static(mod_source: &str, scene_path: &str) -> String {
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

    fn sync_scene_preview_selection(&mut self) {
        let Some(scene_path) = self.selected_scene_path().map(str::to_string) else {
            self.scene_preview_layers.clear();
            self.scene_layer_visibility.clear();
            self.scene_layer_cursor = 0;
            self.scene_preview_scene = None;
            return;
        };

        if self.mod_source.is_empty() {
            self.scene_preview_layers.clear();
            self.scene_layer_visibility.clear();
            self.scene_layer_cursor = 0;
            self.scene_preview_scene = None;
            return;
        }

        let scene_ref = self.normalize_scene_ref_path(&scene_path);
        match create_scene_repository(Path::new(&self.mod_source))
            .and_then(|repo| repo.load_scene(&scene_ref))
        {
            Ok(scene) => {
                self.scene_preview_layers = scene
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
                self.scene_layer_visibility = vec![true; scene.layers.len()];
                self.scene_layer_cursor = self
                    .scene_layer_cursor
                    .min(self.scene_preview_layers.len().saturating_sub(1));
                self.scene_preview_scene = Some(scene);
            }
            Err(err) => {
                self.scene_preview_layers.clear();
                self.scene_layer_visibility.clear();
                self.scene_layer_cursor = 0;
                self.scene_preview_scene = None;
                self.status = format!("Scenes Browser: failed to compile scene ({err})");
            }
        }
    }

    fn toggle_effects_preview(&mut self) {
        if self.sidebar_active != SidebarItem::Search {
            return;
        }

        self.effects_live_preview = !self.effects_live_preview;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.sync_effect_param_cursor();

        self.status = if self.effects_live_preview {
            "Effects Browser: LIVE preview ON | Tab focus | ↑/↓ param | ←/→ adjust".to_string()
        } else {
            "Effects Browser: LIVE preview OFF | F enables live buffer preview".to_string()
        };
    }

    fn sync_effect_preview_scene_yaml(&mut self) {
        let Some(effect_name) = self.selected_builtin_effect() else {
            self.effects_preview_scene_yaml.clear();
            return;
        };

        let mut params = effect_params::default_effect_params(effect_name);
        effect_params::apply_overrides(effect_name, &self.effect_param_overrides, &mut params);
        self.effects_preview_scene_yaml =
            effects_preview_scene::build_preview_scene_yaml_default(effect_name, &params);
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
        self.project_watch_stamp = Self::compute_project_watch_stamp(&self.mod_source);
        self.project_watch_last_scan_ms = now_millis();
        self.tree_items = self.build_tree_items();
        self.tree_cursor = 0;
        self.scene_cursor = 0;
        self.scene_display_names =
            Self::build_scene_display_names(&self.mod_source, &self.index.scenes.scene_paths);
        self.scene_layer_cursor = 0;
        self.scene_layer_visibility.clear();
        self.scene_preview_started_at_ms = now_millis();
        self.sync_scene_preview_selection();
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
        self.start_cursor = self
            .start_cursor
            .min(self.start_items().len().saturating_sub(1));
        self.status = format!("Removed {removed} stale recent entrie(s)");
    }

    fn close_project(&mut self) {
        self.mode = AppMode::Start;
        self.start_dialog = StartDialog::RecentMenu;
        self.mod_source.clear();
        self.index = AssetIndex::default();
        self.start_cursor = 0;
        self.schema_cursor = 0;
        self.dir_cursor = 0;
        self.dir_preview_path.clear();
        self.dir_preview_index = None;
        self.dir_preview_popup = false;
        self.dir_preview_started_at_ms = 0;
        self.scene_cursor = 0;
        self.scene_display_names.clear();
        self.scene_layer_cursor = 0;
        self.scene_layer_visibility.clear();
        self.scene_preview_layers.clear();
        self.scene_preview_scene = None;
        self.scene_preview_started_at_ms = 0;
        self.reset_scene_fullscreen_state();
        self.project_watch_last_scan_ms = 0;
        self.project_watch_stamp = 0;
        self.status =
            "Start: j/k move | Enter select | f schema scan | x prune stale | q quit".to_string();
    }

    fn open_schema_picker(&mut self) {
        self.schema_candidates = collect_schema_project_yml_files(Path::new("."));
        self.schema_cursor = 0;
        self.start_dialog = StartDialog::SchemaPicker;
        if self.schema_candidates.is_empty() {
            self.status = "No schema-tagged .yml files found in current workspace".to_string();
        } else {
            self.status = "Select schema .yml and Enter to open project".to_string();
        }
    }

    fn open_directory_browser(&mut self, initial: &str) {
        self.start_dialog = StartDialog::DirectoryBrowser;
        self.dir_cursor = 0;
        self.dir_preview_popup = false;
        self.dir_preview_started_at_ms = 0;
        self.refresh_directory_items(initial);
        self.status = "Directory browser: Enter open, F5 preview, Esc back, j/k move".to_string();
    }

    fn refresh_directory_items(&mut self, base: &str) {
        let canonical = fs::canonicalize(base).unwrap_or_else(|_| PathBuf::from(base));
        self.dir_browser_path = canonical.display().to_string();
        let root_validation = validate_project_dir(&canonical);
        self.dir_can_open = root_validation.valid;
        self.dir_validation_code = root_validation.code.to_string();
        self.dir_validation_message = root_validation.message;
        self.dir_browser_items.clear();

        self.dir_browser_items.push(DirBrowserItem::OpenHere);
        if canonical.parent().is_some() {
            self.dir_browser_items.push(DirBrowserItem::Parent);
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
        self.dir_browser_items
            .extend(dirs.into_iter().map(|(path, valid_project, code)| {
                DirBrowserItem::Directory {
                    path,
                    valid_project,
                    code,
                }
            }));
        self.dir_cursor = self
            .dir_cursor
            .min(self.dir_browser_items.len().saturating_sub(1));
        self.refresh_dir_preview();
    }

    fn selected_directory_path(&self) -> Option<String> {
        match self.dir_browser_items.get(self.dir_cursor)? {
            DirBrowserItem::OpenHere => Some(self.dir_browser_path.clone()),
            DirBrowserItem::Parent => Path::new(&self.dir_browser_path)
                .parent()
                .map(|p| p.display().to_string()),
            DirBrowserItem::Directory { path, .. } => Some(path.clone()),
        }
    }

    fn refresh_dir_preview(&mut self) {
        let Some(path) = self.selected_directory_path() else {
            self.dir_preview_path.clear();
            self.dir_preview_index = None;
            return;
        };
        self.dir_preview_path = path.clone();
        let validation = validate_project_dir(Path::new(&path));
        self.dir_preview_index = if validation.valid {
            Some(build_project_index(&path))
        } else {
            None
        };
        if self.dir_preview_index.is_none() {
            self.dir_preview_popup = false;
            self.dir_preview_started_at_ms = 0;
        }
    }

    fn toggle_dir_preview_popup(&mut self) {
        if self.dir_preview_index.is_some() {
            self.dir_preview_popup = !self.dir_preview_popup;
            self.status = if self.dir_preview_popup {
                self.dir_preview_started_at_ms = now_millis();
                format!("Live preview x{} running", self.dir_preview_speed_mult)
            } else {
                self.dir_preview_started_at_ms = 0;
                "Preview closed".to_string()
            };
        } else {
            self.dir_preview_popup = false;
            self.dir_preview_started_at_ms = 0;
            self.status = "Preview unavailable for this folder".to_string();
        }
    }

    fn enter_directory_item(&mut self) {
        let Some(item) = self.dir_browser_items.get(self.dir_cursor).cloned() else {
            return;
        };
        match item {
            DirBrowserItem::OpenHere => {
                if self.dir_can_open {
                    let path = self.dir_browser_path.clone();
                    self.open_project(&path);
                } else {
                    self.status = "Cannot open this directory".to_string();
                }
            }
            DirBrowserItem::Parent => {
                let parent = Path::new(&self.dir_browser_path)
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| self.dir_browser_path.clone());
                self.refresh_directory_items(&parent);
            }
            DirBrowserItem::Directory { path, .. } => self.refresh_directory_items(&path),
        }
    }

    /// Applies the given command for the current mode; returns `true` if the app should quit.
    pub fn apply_command(&mut self, cmd: Command) -> bool {
        match self.mode {
            AppMode::Start => self.apply_start_command(cmd),
            AppMode::Browser => self.apply_browser_command(cmd),
            AppMode::EditMode => self.apply_edit_command(cmd),
        }
    }

    fn apply_start_command(&mut self, cmd: Command) -> bool {
        match self.start_dialog {
            StartDialog::RecentMenu => self.apply_start_recent_menu(cmd),
            StartDialog::SchemaPicker => self.apply_start_schema_picker(cmd),
            StartDialog::DirectoryBrowser => self.apply_start_directory_browser(cmd),
        }
    }

    fn apply_start_recent_menu(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::NextPane => {
                self.start_focus = match self.start_focus {
                    StartFocus::Recents => StartFocus::Actions,
                    StartFocus::Actions => StartFocus::Recents,
                };
            }
            Command::PrevPane => {
                self.start_focus = match self.start_focus {
                    StartFocus::Recents => StartFocus::Actions,
                    StartFocus::Actions => StartFocus::Recents,
                };
            }
            Command::Up => match self.start_focus {
                StartFocus::Recents => {
                    self.recent_cursor = self.recent_cursor.saturating_sub(1);
                }
                StartFocus::Actions => {
                    self.action_cursor = self.action_cursor.saturating_sub(1);
                }
            },
            Command::Down => match self.start_focus {
                StartFocus::Recents => {
                    let max = self.recent_projects.len().saturating_sub(1);
                    self.recent_cursor = (self.recent_cursor + 1).min(max);
                }
                StartFocus::Actions => {
                    self.action_cursor = (self.action_cursor + 1).min(3); // 4 actions (0-3)
                }
            },
            Command::OpenProject => {
                let path = self.launch_mod_source.clone();
                self.open_directory_browser(&path);
            }
            Command::PruneRecents => self.prune_stale_recents(),
            Command::OpenSchemaPicker => self.open_schema_picker(),
            Command::Enter => match self.start_focus {
                StartFocus::Recents => {
                    if let Some(path) = self.recent_projects.get(self.recent_cursor).cloned() {
                        self.open_project(&path);
                    }
                }
                StartFocus::Actions => match self.action_cursor {
                    0 => {
                        // Open Project
                        let path = self.launch_mod_source.clone();
                        self.open_directory_browser(&path);
                    }
                    1 => {
                        // Find Schema YML
                        self.open_schema_picker();
                    }
                    2 => {
                        // New Project (coming soon)
                        self.status = "New Project: coming soon (MVP browser)".to_string();
                    }
                    3 => {
                        // Quit
                        return true;
                    }
                    _ => {}
                },
            },
            Command::Back
            | Command::Left
            | Command::Right
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
            | Command::Noop => {}
        }
        false
    }

    fn apply_start_schema_picker(&mut self, cmd: Command) -> bool {
        let max = self.schema_candidates.len().saturating_sub(1);
        match cmd {
            Command::Quit => return true,
            Command::Back => {
                self.start_dialog = StartDialog::RecentMenu;
                self.status =
                    "Start: j/k move | Enter select | f schema scan | x prune stale | q quit"
                        .to_string();
            }
            Command::Up => self.schema_cursor = self.schema_cursor.saturating_sub(1),
            Command::Down => self.schema_cursor = (self.schema_cursor + 1).min(max),
            Command::Enter => {
                if let Some(path) = self.schema_candidates.get(self.schema_cursor).cloned() {
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
            | Command::Noop => {}
        }
        false
    }

    fn apply_start_directory_browser(&mut self, cmd: Command) -> bool {
        let max = self.dir_browser_items.len().saturating_sub(1);
        if self.dir_preview_popup {
            match cmd {
                Command::Quit => return true,
                Command::Back | Command::TogglePreview => {
                    self.dir_preview_popup = false;
                    self.dir_preview_started_at_ms = 0;
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
            Command::Up => self.dir_cursor = self.dir_cursor.saturating_sub(1),
            Command::Down => self.dir_cursor = (self.dir_cursor + 1).min(max),
            Command::Enter => self.enter_directory_item(),
            Command::OpenProject => {
                let path = self.dir_browser_path.clone();
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
            | Command::Noop => {}
        }
        if matches!(cmd, Command::Up | Command::Down) {
            self.refresh_dir_preview();
        }
        false
    }

    fn apply_browser_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::CloseProject => self.close_project(),
            Command::Up => {
                if self.sidebar_active == SidebarItem::Search {
                    if self.effects_live_preview && self.focus == FocusPane::Inspector {
                        self.move_effect_param_cursor(-1);
                    } else if self.effects_live_preview && self.focus == FocusPane::Browser {
                        self.effects_code_scroll = self.effects_code_scroll.saturating_sub(1);
                    } else {
                        self.move_effect_selection(self.effect_cursor.saturating_sub(1));
                    }
                } else if self.sidebar_active == SidebarItem::Scenes {
                    if self.focus == FocusPane::ProjectTree {
                        self.move_scene_selection(self.scene_cursor.saturating_sub(1));
                    } else if self.focus == FocusPane::Browser {
                        self.move_scene_layer_cursor(-1);
                    }
                } else if self.focus == FocusPane::ProjectTree {
                    self.tree_cursor = self.tree_cursor.saturating_sub(1);
                }
            }
            Command::Down => {
                if self.sidebar_active == SidebarItem::Search {
                    if self.effects_live_preview && self.focus == FocusPane::Inspector {
                        self.move_effect_param_cursor(1);
                    } else if self.effects_live_preview && self.focus == FocusPane::Browser {
                        self.effects_code_scroll = self.effects_code_scroll.saturating_add(1);
                    } else {
                        let max = self.builtin_effects.len().saturating_sub(1);
                        self.move_effect_selection((self.effect_cursor + 1).min(max));
                    }
                } else if self.sidebar_active == SidebarItem::Scenes {
                    if self.focus == FocusPane::ProjectTree {
                        let max = self.index.scenes.scene_paths.len().saturating_sub(1);
                        self.move_scene_selection((self.scene_cursor + 1).min(max));
                    } else if self.focus == FocusPane::Browser {
                        self.move_scene_layer_cursor(1);
                    }
                } else if self.focus == FocusPane::ProjectTree {
                    let max = self.tree_items.len().saturating_sub(1);
                    self.tree_cursor = (self.tree_cursor + 1).min(max);
                }
            }
            Command::Left => {
                if self.sidebar_active == SidebarItem::Search
                    && self.effects_live_preview
                    && self.focus == FocusPane::Inspector
                {
                    self.adjust_selected_effect_param(-1.0);
                }
            }
            Command::Right => {
                if self.sidebar_active == SidebarItem::Search
                    && self.effects_live_preview
                    && self.focus == FocusPane::Inspector
                {
                    self.adjust_selected_effect_param(1.0);
                }
            }
            Command::NextPane => self.focus = self.focus.next(),
            Command::PrevPane => self.focus = self.focus.prev(),
            Command::EnterFile => {
                if self.sidebar_active == SidebarItem::Explorer {
                    self.enter_edit_mode();
                } else if self.sidebar_active == SidebarItem::Search {
                    if !self.effects_live_preview {
                        self.effects_live_preview = true;
                    }
                    self.focus = FocusPane::Inspector;
                    self.sync_effect_param_cursor();
                    self.restart_effect_preview_clock();
                    self.status =
                        "Effects Browser: controls focused | ↑/↓ param | ←/→ adjust | F toggle"
                            .to_string();
                } else if self.sidebar_active == SidebarItem::Scenes
                    && self.focus == FocusPane::Browser
                {
                    self.isolate_selected_scene_layer();
                }
            }
            Command::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            Command::SelectPanel1 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Explorer;
                self.sidebar_visible = true;
            }
            Command::SelectPanel2 => self.activate_effects_browser(),
            Command::SelectPanel3 => self.activate_scenes_browser(),
            Command::SelectPanel4 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Settings;
                self.sidebar_visible = true;
            }
            Command::PruneRecents => {}
            Command::TogglePreview => {}
            Command::ToggleEffectsPreview => {
                if self.sidebar_active == SidebarItem::Scenes {
                    self.set_scene_fullscreen_hold(true);
                } else {
                    self.toggle_effects_preview();
                }
            }
            Command::SceneFullscreenHoldStart => {
                if self.sidebar_active == SidebarItem::Scenes {
                    if self.scene_preview_fullscreen_hold && !self.scene_preview_fullscreen_toggle {
                        self.set_scene_fullscreen_hold(false);
                    } else {
                        self.set_scene_fullscreen_hold(true);
                    }
                } else if self.sidebar_active == SidebarItem::Search {
                    self.toggle_effects_preview();
                }
            }
            Command::SceneFullscreenHoldEnd => {
                if self.sidebar_active == SidebarItem::Scenes {
                    self.set_scene_fullscreen_hold(false);
                }
            }
            Command::ToggleSceneFullscreen => self.toggle_scene_fullscreen(),
            Command::ToggleSceneLayer => {
                if self.sidebar_active == SidebarItem::Scenes && self.focus == FocusPane::Browser {
                    self.toggle_selected_scene_layer();
                }
            }
            Command::NextCodeTab => {
                if self.sidebar_active == SidebarItem::Search {
                    self.effects_code_scroll = 0;
                    self.effects_code_tab = self.effects_code_tab.next();
                }
            }
            Command::PrevCodeTab => {
                if self.sidebar_active == SidebarItem::Search {
                    self.effects_code_scroll = 0;
                    self.effects_code_tab = self.effects_code_tab.prev();
                }
            }
            Command::Back
            | Command::Noop
            | Command::Enter
            | Command::OpenProject
            | Command::OpenSchemaPicker
            | Command::ExitEditor => {}
        }
        false
    }

    fn apply_edit_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Quit => return true,
            Command::ExitEditor => self.exit_edit_mode(),
            Command::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            Command::ToggleEffectsPreview => self.toggle_effects_preview(),
            Command::SelectPanel1 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Explorer;
                self.sidebar_visible = true;
            }
            Command::SelectPanel2 => self.activate_effects_browser(),
            Command::SelectPanel3 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Scenes;
                self.sidebar_visible = true;
            }
            Command::SelectPanel4 => {
                self.reset_scene_fullscreen_state();
                self.sidebar_active = SidebarItem::Settings;
                self.sidebar_visible = true;
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
            | Command::ToggleSceneLayer => {}
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
                        self.editing_file = Some(path.clone());
                        self.edit_content = content;
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
        self.editing_file = None;
        self.edit_content.clear();
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
