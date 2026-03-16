use std::path::Path;
use std::sync::OnceLock;

use serde_yaml::Value;

use crate::repositories::{create_scene_repository, SceneRepository};
use crate::scene::Scene;
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
