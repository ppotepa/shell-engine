//! Validates level configuration assets under `/levels` and manifest selection.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use engine_error::EngineError;
use serde_yaml::{Mapping, Value};
use zip::ZipArchive;

use super::asset_utils::{is_zip_file, normalize_relative_asset_path};
use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that validates `/levels` payload parsing and configured
/// `level.initial` / `level.default` references in `mod.yaml`.
pub struct LevelConfigCheck;

impl StartupCheck for LevelConfigCheck {
    fn name(&self) -> &'static str {
        "level-config"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let assets = discover_level_assets(ctx.mod_source())?;
        let file_count = assets.len();
        let mut ids = BTreeMap::new();
        let mut parsed_count = 0usize;

        for asset in assets {
            let text = match String::from_utf8(asset.bytes) {
                Ok(value) => value,
                Err(_) => {
                    report.add_warning(
                        self.name(),
                        format!("level file is not utf8: {}", asset.path),
                    );
                    continue;
                }
            };

            let doc = match serde_yaml::from_str::<Value>(&text) {
                Ok(value) => value,
                Err(error) => {
                    report.add_warning(
                        self.name(),
                        format!("level parse failed: {} ({error})", asset.path),
                    );
                    continue;
                }
            };

            parsed_count += 1;
            let level_id = level_id_from_doc(&doc, &asset.path);
            if let Some(existing_path) = ids.insert(level_id.clone(), asset.path.clone()) {
                report.add_warning(
                    self.name(),
                    format!(
                        "duplicate level id `{level_id}` in `{existing_path}` and `{}`",
                        asset.path
                    ),
                );
            }
        }

        if let Some(configured) = configured_initial_level_id(ctx.manifest()) {
            if !ids.contains_key(&configured) {
                report.add_warning(
                    self.name(),
                    format!(
                        "configured initial level `{configured}` was not found in /levels assets"
                    ),
                );
            }
        }

        report.add_info(
            self.name(),
            format!("level config checked ({file_count} files, {parsed_count} parsed)"),
        );
        Ok(())
    }
}

struct LevelAsset {
    path: String,
    bytes: Vec<u8>,
}

fn discover_level_assets(mod_source: &Path) -> Result<Vec<LevelAsset>, EngineError> {
    if mod_source.is_dir() {
        discover_level_assets_from_dir(mod_source)
    } else if is_zip_file(mod_source) {
        discover_level_assets_from_zip(mod_source)
    } else {
        Ok(Vec::new())
    }
}

fn discover_level_assets_from_dir(mod_source: &Path) -> Result<Vec<LevelAsset>, EngineError> {
    let levels_root = mod_source.join("levels");
    if !levels_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_level_dir(mod_source, &levels_root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_level_dir(
    mod_source: &Path,
    dir: &Path,
    out: &mut Vec<LevelAsset>,
) -> Result<(), EngineError> {
    let entries = fs::read_dir(dir).map_err(|source| EngineError::ManifestRead {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| EngineError::ManifestRead {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk_level_dir(mod_source, &path, out)?;
            continue;
        }
        let rel = normalize_relative_asset_path(mod_source, &path);
        if !is_level_asset_path(&rel) {
            continue;
        }
        let bytes = fs::read(&path).map_err(|source| EngineError::ManifestRead {
            path: path.clone(),
            source,
        })?;
        out.push(LevelAsset { path: rel, bytes });
    }
    Ok(())
}

fn discover_level_assets_from_zip(mod_source: &Path) -> Result<Vec<LevelAsset>, EngineError> {
    let file = fs::File::open(mod_source).map_err(|source| EngineError::ManifestRead {
        path: mod_source.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| EngineError::ZipArchive {
        path: mod_source.to_path_buf(),
        source,
    })?;

    let mut out = Vec::new();
    for idx in 0..archive.len() {
        let mut entry = archive
            .by_index(idx)
            .map_err(|source| EngineError::ZipArchive {
                path: mod_source.to_path_buf(),
                source,
            })?;
        if !entry.is_file() {
            continue;
        }
        let normalized = format!(
            "/{}",
            entry.name().trim_start_matches('/').replace('\\', "/")
        );
        if !is_level_asset_path(&normalized) {
            continue;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| EngineError::ManifestRead {
                path: mod_source.to_path_buf(),
                source,
            })?;
        out.push(LevelAsset {
            path: normalized,
            bytes,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn is_level_asset_path(path: &str) -> bool {
    let normalized = path.trim_start_matches('/').to_ascii_lowercase();
    if !normalized.starts_with("levels/") {
        return false;
    }
    normalized.ends_with(".yml") || normalized.ends_with(".yaml") || normalized.ends_with(".json")
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
    use super::LevelConfigCheck;
    use crate::startup::{StartupCheck, StartupContext, StartupIssueLevel, StartupReport};
    use engine_error::EngineError;
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    fn empty_scene_loader(
        _mod_source: &std::path::Path,
    ) -> Result<Vec<crate::startup::StartupSceneFile>, EngineError> {
        Ok(Vec::new())
    }

    #[test]
    fn validates_level_assets_and_reports_info() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("levels")).expect("create levels");
        fs::write(
            mod_dir.join("levels/default.yml"),
            r#"
id: asteroids.default
player:
  lives: 3
"#,
        )
        .expect("write level");

        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nlevel:\n  initial: asteroids.default\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &empty_scene_loader);
        let mut report = StartupReport::default();
        LevelConfigCheck
            .run(&ctx, &mut report)
            .expect("level config check should pass");

        assert!(report
            .issues()
            .iter()
            .any(|issue| issue.level == StartupIssueLevel::Info
                && issue.check == "level-config"
                && issue.message.contains("level config checked")));
        assert!(!report
            .issues()
            .iter()
            .any(|issue| issue.level == StartupIssueLevel::Warning));
    }

    #[test]
    fn warns_when_manifest_points_to_missing_level() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("levels")).expect("create levels");
        fs::write(
            mod_dir.join("levels/default.yml"),
            r#"
id: asteroids.default
player:
  lives: 3
"#,
        )
        .expect("write level");

        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nlevel:\n  initial: asteroids.hard\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/main.yml", &empty_scene_loader);
        let mut report = StartupReport::default();
        LevelConfigCheck
            .run(&ctx, &mut report)
            .expect("level config check should pass");

        assert!(report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.check == "level-config"
                && issue.message.contains("configured initial level")
        }));
    }
}
