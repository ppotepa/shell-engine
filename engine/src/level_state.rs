//! Level-state resource wiring for engine startup.
//!
//! Loads level payloads from `/levels/*.yml|*.yaml|*.json` in the active mod
//! source and selects an initial active level.

pub use engine_core::level_state::LevelState;

use engine_asset::{create_asset_repository, AssetRepository};
use engine_core::logging;
use serde_yaml::{Mapping, Value};
use std::path::Path;

/// Loads level payloads from the active mod and returns initialized [`LevelState`].
///
/// Manifest selection rules (first match):
/// 1. `level.initial`
/// 2. `level.default`
/// 3. `level` (string)
/// 4. first discovered level id (lexicographic)
pub fn load_level_state(mod_source: &Path, manifest: &Value) -> LevelState {
    let state = LevelState::new();
    let Ok(repo) = create_asset_repository(mod_source) else {
        return state;
    };
    let Ok(paths) = repo.list_assets_under("/levels") else {
        return state;
    };

    for path in paths {
        if !is_level_path(&path) {
            continue;
        }
        let Ok(raw) = repo.read_asset_bytes(&path) else {
            logging::warn("engine.level", format!("failed to read level file: {path}"));
            continue;
        };
        let Ok(text) = String::from_utf8(raw) else {
            logging::warn("engine.level", format!("level file is not utf8: {path}"));
            continue;
        };
        let Ok(doc) = serde_yaml::from_str::<Value>(&text) else {
            logging::warn("engine.level", format!("level parse failed: {path}"));
            continue;
        };
        let Ok(payload) = serde_json::to_value(&doc) else {
            logging::warn("engine.level", format!("level conversion failed: {path}"));
            continue;
        };
        let level_id = level_id_from_doc(&doc, &path);
        if !state.register_level(&level_id, payload) {
            logging::warn(
                "engine.level",
                format!("skipped invalid level id from path={path}"),
            );
        }
    }

    let configured = configured_initial_level_id(manifest);
    if let Some(configured_id) = configured {
        if !state.select(&configured_id) {
            logging::warn(
                "engine.level",
                format!("configured initial level not found: {configured_id}"),
            );
        }
    } else if let Some(first_id) = state.ids().first().cloned() {
        let _ = state.select(&first_id);
    }

    if let Some(active) = state.current_id() {
        logging::info("engine.level", format!("active level: {active}"));
    }

    state
}

fn is_level_path(path: &str) -> bool {
    path.ends_with(".yml") || path.ends_with(".yaml") || path.ends_with(".json")
}

fn level_id_from_doc(doc: &Value, path: &str) -> String {
    if let Some(id) = doc
        .as_mapping()
        .and_then(|map| map_get(map, "id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        return id.to_string();
    }
    level_id_from_path(path)
}

fn level_id_from_path(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    let rel = trimmed.strip_prefix("levels/").unwrap_or(trimmed);
    let rel = rel
        .strip_suffix(".yml")
        .or_else(|| rel.strip_suffix(".yaml"))
        .or_else(|| rel.strip_suffix(".json"))
        .unwrap_or(rel);
    if rel.trim().is_empty() {
        return "default".to_string();
    }
    rel.split('/')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn configured_initial_level_id(manifest: &Value) -> Option<String> {
    let root = manifest.as_mapping()?;
    let level = map_get(root, "level")?;
    if let Some(value) = level.as_str().map(str::trim).filter(|s| !s.is_empty()) {
        return Some(value.to_string());
    }
    let level_map = level.as_mapping()?;
    map_get(level_map, "initial")
        .or_else(|| map_get(level_map, "default"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn map_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{configured_initial_level_id, level_id_from_path};

    #[test]
    fn derives_level_id_from_path() {
        assert_eq!(
            level_id_from_path("/levels/game/default.yml"),
            "game.default"
        );
        assert_eq!(level_id_from_path("/levels/default.yaml"), "default");
    }

    #[test]
    fn reads_initial_level_from_manifest() {
        let manifest: serde_yaml::Value = serde_yaml::from_str(
            r#"
name: test
version: 0.1.0
entrypoint: /scenes/main.yml
level:
  initial: game.default
"#,
        )
        .expect("manifest");
        assert_eq!(
            configured_initial_level_id(&manifest).as_deref(),
            Some("game.default")
        );
    }
}
