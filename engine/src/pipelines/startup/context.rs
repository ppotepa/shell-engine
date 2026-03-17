//! Startup context — lazily loads and caches all scene files for use by [`StartupCheck`](super::check::StartupCheck) implementations.

use std::path::Path;
use std::sync::OnceLock;

use serde_yaml::Value;

use crate::repositories::{create_scene_repository, SceneRepository};
use crate::scene::Scene;
use crate::EngineError;

/// A parsed scene file alongside its path, used during startup validation.
#[derive(Debug, Clone)]
pub struct StartupSceneFile {
    pub path: String,
    pub scene: Scene,
}

/// Read-only view of the mod under validation, with lazy-loaded scene cache.
pub struct StartupContext<'a> {
    mod_source: &'a Path,
    manifest: &'a Value,
    entrypoint: &'a str,
    scene_cache: OnceLock<Vec<StartupSceneFile>>,
}

impl<'a> StartupContext<'a> {
    /// Creates a new context for the given mod source, manifest, and entrypoint.
    pub fn new(mod_source: &'a Path, manifest: &'a Value, entrypoint: &'a str) -> Self {
        Self {
            mod_source,
            manifest,
            entrypoint,
            scene_cache: OnceLock::new(),
        }
    }

    /// Returns the path to the mod source directory or archive.
    pub fn mod_source(&self) -> &Path {
        self.mod_source
    }

    /// Returns the parsed `mod.yaml` manifest value.
    pub fn manifest(&self) -> &Value {
        self.manifest
    }

    /// Returns the entrypoint scene path declared in the manifest.
    pub fn entrypoint(&self) -> &str {
        self.entrypoint
    }

    /// Returns (and caches) every parsed scene in the mod, loading them on first call.
    pub fn all_scenes(&self) -> Result<&[StartupSceneFile], EngineError> {
        if let Some(cached) = self.scene_cache.get() {
            return Ok(cached.as_slice());
        }
        let loaded = load_all_scenes(self.mod_source)?;
        let _ = self.scene_cache.set(loaded);
        Ok(self.scene_cache.get().map(Vec::as_slice).unwrap_or(&[]))
    }
}

fn load_all_scenes(mod_source: &Path) -> Result<Vec<StartupSceneFile>, EngineError> {
    let repo = create_scene_repository(mod_source)?;
    let paths = repo.discover_scene_paths()?;
    let mut scenes = Vec::with_capacity(paths.len());
    for path in paths {
        let scene = repo.load_scene(&path)?;
        scenes.push(StartupSceneFile { path, scene });
    }
    Ok(scenes)
}
