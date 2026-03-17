//! Authored repository path helpers.
//!
//! This module keeps authored discovery rules in one place so filesystem, zip,
//! editor, and tooling code can share the same idea of which YAML files are
//! real scenes and which ones are scene-package partials.

/// Returns `true` when the path points to a scene document discoverable by the
/// runtime and editor.
pub fn is_discoverable_scene_path(path: &str) -> bool {
    path.starts_with("scenes/") && is_yaml_path(path) && !is_reserved_scene_partial_path(path)
}

/// Returns `true` when the path is a folder-scene composition root manifest.
pub fn is_scene_package_manifest(path: &str) -> bool {
    let trimmed = path.trim_start_matches('/');
    trimmed.starts_with("scenes/") && trimmed.ends_with("/scene.yml")
}

/// Returns `true` when a path belongs to a reserved scene partial location and
/// therefore should not be indexed as a standalone scene.
pub fn is_reserved_scene_partial_path(path: &str) -> bool {
    let trimmed = path.trim_start_matches('/');
    let segments: Vec<&str> = trimmed.split('/').collect();
    if segments.first() != Some(&"scenes") {
        return false;
    }
    if segments.get(1) == Some(&"shared") {
        return true;
    }
    if segments.len() < 4 {
        return false;
    }
    matches!(
        segments[2],
        "layers" | "sprites" | "templates" | "objects" | "effects"
    )
}

/// Returns `true` for `.yml` and `.yaml` files.
pub fn is_yaml_path(path: &str) -> bool {
    path.ends_with(".yml") || path.ends_with(".yaml")
}

#[cfg(test)]
mod tests {
    use super::{
        is_discoverable_scene_path, is_reserved_scene_partial_path, is_scene_package_manifest,
        is_yaml_path,
    };

    #[test]
    fn detects_yaml_extensions() {
        assert!(is_yaml_path("scene.yml"));
        assert!(is_yaml_path("scene.yaml"));
        assert!(!is_yaml_path("scene.txt"));
    }

    #[test]
    fn excludes_shared_and_partial_paths_from_scene_discovery() {
        assert!(is_reserved_scene_partial_path("scenes/shared/banner.yml"));
        assert!(is_reserved_scene_partial_path("scenes/intro/layers/base.yml"));
        assert!(!is_reserved_scene_partial_path("scenes/intro/scene.yml"));
        assert!(!is_discoverable_scene_path("scenes/shared/banner.yml"));
        assert!(!is_discoverable_scene_path("scenes/intro/layers/base.yml"));
        assert!(is_discoverable_scene_path("scenes/intro/scene.yml"));
    }

    #[test]
    fn recognizes_scene_package_manifests() {
        assert!(is_scene_package_manifest("/scenes/intro/scene.yml"));
        assert!(is_scene_package_manifest("scenes/intro/scene.yml"));
        assert!(!is_scene_package_manifest("scenes/intro.yml"));
    }
}
