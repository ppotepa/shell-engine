//! Mod manifest loader and startup validation pipeline.
//!
//! Reads and validates `mod.yaml` from a directory or `.zip` archive, and provides
//! the pre-run startup check framework.

pub mod output_backend;
pub mod startup;
pub mod display_config;

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde_yaml::Value;
use zip::ZipArchive;

use engine_error::EngineError;

pub use output_backend::StartupOutputSetting;

/// Reads, parses, and validates `mod.yaml` from `mod_source` (directory or `.zip`), returning the manifest value.
pub fn load_mod_manifest(mod_source: &Path) -> Result<Value, EngineError> {
    if !mod_source.exists() {
        return Err(EngineError::SourceNotFound(mod_source.to_path_buf()));
    }

    if mod_source.is_dir() {
        load_from_directory(mod_source)
    } else if is_zip_file(mod_source) {
        load_from_zip(mod_source)
    } else {
        Err(EngineError::UnsupportedSource(mod_source.to_path_buf()))
    }
}

fn load_from_directory(mod_source: &Path) -> Result<Value, EngineError> {
    let manifest_path = mod_source.join("mod.yaml");
    if !manifest_path.exists() {
        return Err(EngineError::MissingModEntrypoint(mod_source.to_path_buf()));
    }

    let content =
        fs::read_to_string(&manifest_path).map_err(|source| EngineError::ManifestRead {
            path: manifest_path.clone(),
            source,
        })?;

    let manifest =
        serde_yaml::from_str::<Value>(&content).map_err(|source| EngineError::InvalidModYaml {
            path: manifest_path.clone(),
            source,
        })?;
    validate_directory_entrypoint(mod_source, &manifest, &manifest_path)?;

    Ok(manifest)
}

fn load_from_zip(mod_source: &Path) -> Result<Value, EngineError> {
    let file = fs::File::open(mod_source).map_err(|source| EngineError::ManifestRead {
        path: mod_source.to_path_buf(),
        source,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|source| EngineError::ZipArchive {
        path: mod_source.to_path_buf(),
        source,
    })?;

    let mut manifest_file = archive
        .by_name("mod.yaml")
        .map_err(|_| EngineError::MissingModEntrypoint(PathBuf::from(mod_source)))?;
    let mut content = String::new();
    manifest_file
        .read_to_string(&mut content)
        .map_err(|source| EngineError::ManifestRead {
            path: mod_source.to_path_buf(),
            source,
        })?;
    drop(manifest_file);

    let manifest =
        serde_yaml::from_str::<Value>(&content).map_err(|source| EngineError::InvalidModYaml {
            path: mod_source.to_path_buf(),
            source,
        })?;
    validate_zip_entrypoint(mod_source, &manifest, &mut archive)?;

    Ok(manifest)
}

fn validate_directory_entrypoint(
    mod_source: &Path,
    manifest: &Value,
    manifest_path: &Path,
) -> Result<(), EngineError> {
    let entrypoint = extract_entrypoint(manifest, manifest_path)?;
    let normalized = normalize_entrypoint(&entrypoint);
    let scene_path = mod_source.join(normalized);
    if !scene_path.exists() {
        return Err(EngineError::MissingSceneEntrypoint {
            mod_source: mod_source.to_path_buf(),
            entrypoint,
        });
    }
    Ok(())
}

fn validate_zip_entrypoint(
    mod_source: &Path,
    manifest: &Value,
    archive: &mut ZipArchive<fs::File>,
) -> Result<(), EngineError> {
    let entrypoint = extract_entrypoint(manifest, mod_source)?;
    let normalized = normalize_entrypoint(&entrypoint);
    archive
        .by_name(normalized)
        .map_err(|_| EngineError::MissingSceneEntrypoint {
            mod_source: mod_source.to_path_buf(),
            entrypoint,
        })?;
    Ok(())
}

fn extract_entrypoint(manifest: &Value, source_path: &Path) -> Result<String, EngineError> {
    let mapping = manifest
        .as_mapping()
        .ok_or_else(|| EngineError::InvalidManifestFieldType {
            path: source_path.to_path_buf(),
            field: "root".to_string(),
            expected: "YAML mapping/object".to_string(),
        })?;
    let key = Value::String("entrypoint".to_string());
    let entrypoint = mapping
        .get(&key)
        .ok_or_else(|| EngineError::MissingManifestField {
            path: source_path.to_path_buf(),
            field: "entrypoint".to_string(),
        })?;
    let entrypoint = entrypoint
        .as_str()
        .ok_or_else(|| EngineError::InvalidManifestFieldType {
            path: source_path.to_path_buf(),
            field: "entrypoint".to_string(),
            expected: "string".to_string(),
        })?;
    if normalize_entrypoint(entrypoint).is_empty() {
        return Err(EngineError::InvalidManifestFieldType {
            path: source_path.to_path_buf(),
            field: "entrypoint".to_string(),
            expected: "non-empty path string".to_string(),
        });
    }
    Ok(entrypoint.to_string())
}

fn normalize_entrypoint(entrypoint: &str) -> &str {
    entrypoint.trim_start_matches('/')
}

fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}
