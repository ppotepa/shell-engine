//! Project index builder: scans a mod root directory and assembles an [`AssetIndex`].

use std::path::Path;

use crate::domain::asset_index::AssetIndex;
use crate::domain::diagnostics::Diagnostics;
use crate::domain::mod_manifest::ModManifestSummary;
use crate::domain::scene_index::SceneIndex;

use super::fs_scan::{
    collect_files, collect_game_yaml_files, collect_scene_entry_files, validate_project_dir,
};
use super::yaml::load_yaml;

/// Scans `mod_source` and returns a fully populated [`AssetIndex`] for the project.
pub fn build_project_index(mod_source: &str) -> AssetIndex {
    let root = Path::new(mod_source);
    let manifest = load_yaml::<ModManifestSummary>(&root.join("mod.yaml"));

    let scenes = collect_scene_entry_files(root);
    let images = collect_files(root, "assets/images", "png");
    let fonts = collect_files(root, "assets/fonts", "yaml");
    let project_yamls = collect_game_yaml_files(root);
    let validation = validate_project_dir(root);
    let is_valid_project = validation.valid;

    let mut diagnostics = Diagnostics::default();
    if !is_valid_project {
        diagnostics
            .warnings
            .push(format!("{}: {}", validation.code, validation.message));
    }

    AssetIndex {
        manifest,
        project_yamls,
        is_valid_project,
        scenes: SceneIndex {
            scene_paths: scenes,
        },
        images,
        fonts,
        effects: vec![
            "crt-on".into(),
            "lightning-optical-80s".into(),
            "lightning-fbm".into(),
            "lightning-natural".into(),
            "random-spark".into(),
            "tesla-orb".into(),
        ],
        diagnostics,
    }
}
