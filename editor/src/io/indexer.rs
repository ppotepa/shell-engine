//! Project index builder: scans a mod root directory and assembles an [`AssetIndex`].

use std::path::Path;

use crate::domain::asset_index::AssetIndex;
use crate::domain::diagnostics::Diagnostics;
use crate::domain::scene_index::SceneIndex;

use super::fs_scan::{
    collect_files, collect_game_yaml_files, collect_scene_entry_files,
    validate_project_dir_with_manifest,
};

/// Scans `mod_source` and returns a fully populated [`AssetIndex`] for the project.
pub fn build_project_index(mod_source: &str) -> AssetIndex {
    let root = Path::new(mod_source);
    let (manifest, validation) = validate_project_dir_with_manifest(root);

    let scenes = collect_scene_entry_files(root);
    let images = collect_files(root, "assets/images", "png");
    let fonts = collect_files(root, "assets/fonts", "yaml");
    let project_yamls = collect_game_yaml_files(root);
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
        effects: engine_effects::EffectDispatcher::builtin_names()
            .iter()
            .map(|&name| name.to_string())
            .collect(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_uses_typed_manifest_and_effect_catalog() {
        let temp_dir = std::env::temp_dir().join("editor-test-indexer");
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(
            temp_dir.join("mod.yaml"),
            "name: test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n",
        )
        .unwrap();
        std::fs::create_dir_all(temp_dir.join("scenes")).unwrap();
        std::fs::write(temp_dir.join("scenes/main.yml"), "id: main\ntitle: Main\n").unwrap();

        let index = build_project_index(temp_dir.to_str().unwrap());

        let catalog_effects: Vec<String> = engine_effects::EffectDispatcher::builtin_names()
            .iter()
            .map(|&name| name.to_string())
            .collect();

        assert_eq!(
            index.effects, catalog_effects,
            "Effects should match catalog"
        );
        assert!(!index.effects.is_empty(), "Effects should not be empty");
        assert!(index.is_valid_project);
        let manifest = index.manifest.expect("typed manifest");
        assert_eq!(manifest.name.as_deref(), Some("test"));
        assert_eq!(manifest.version.as_deref(), Some("0.1.0"));
        assert_eq!(manifest.entrypoint.as_deref(), Some("/scenes/main.yml"));
        assert!(index.diagnostics.warnings.is_empty());

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
