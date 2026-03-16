use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_yaml::Value;

use crate::scene::Scene;
use crate::scene_loader::load_scene;
use crate::EngineError;

#[derive(Debug, Clone)]
pub struct StartupSceneFile {
    pub path: String,
    pub scene: Scene,
}

pub struct StartupContext<'a> {
    mod_source: &'a Path,
    manifest: &'a Value,
    entrypoint: &'a str,
    scene_cache: OnceLock<Vec<StartupSceneFile>>,
}

impl<'a> StartupContext<'a> {
    pub fn new(mod_source: &'a Path, manifest: &'a Value, entrypoint: &'a str) -> Self {
        Self {
            mod_source,
            manifest,
            entrypoint,
            scene_cache: OnceLock::new(),
        }
    }

    pub fn mod_source(&self) -> &Path {
        self.mod_source
    }

    pub fn manifest(&self) -> &Value {
        self.manifest
    }

    pub fn entrypoint(&self) -> &str {
        self.entrypoint
    }

    pub fn all_scenes(&self) -> Result<&[StartupSceneFile], EngineError> {
        if let Some(cached) = self.scene_cache.get() {
            return Ok(cached.as_slice());
        }
        let loaded = load_all_scenes(self.mod_source)?;
        let _ = self.scene_cache.set(loaded);
        Ok(self
            .scene_cache
            .get()
            .map(Vec::as_slice)
            .unwrap_or(&[]))
    }
}

fn load_all_scenes(mod_source: &Path) -> Result<Vec<StartupSceneFile>, EngineError> {
    let paths = discover_scene_paths(mod_source)?;
    let mut scenes = Vec::with_capacity(paths.len());
    for path in paths {
        let scene = load_scene(mod_source, &path)?;
        scenes.push(StartupSceneFile { path, scene });
    }
    Ok(scenes)
}

fn discover_scene_paths(mod_source: &Path) -> Result<Vec<String>, EngineError> {
    let root = mod_source.join("scenes");
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    walk_scene_paths(&root, &mut paths).map_err(|source| EngineError::ManifestRead {
        path: root.clone(),
        source,
    })?;
    paths.sort();

    let mut normalized = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = path
            .strip_prefix(mod_source)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        normalized.push(format!("/{rel}"));
    }
    Ok(normalized)
}

fn walk_scene_paths(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_scene_paths(&path, out)?;
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml") {
            out.push(path);
        }
    }
    Ok(())
}

