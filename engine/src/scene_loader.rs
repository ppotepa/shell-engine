use std::path::{Path, PathBuf};

use crate::repositories::{create_scene_repository, AnySceneRepository, SceneRepository};
use crate::{scene::Scene, EngineError};

/// Standalone function — used at startup to load the entrypoint scene.
pub fn load_scene(mod_source: &Path, scene_path: &str) -> Result<Scene, EngineError> {
    create_scene_repository(mod_source)?.load_scene(scene_path)
}

/// World singleton — used at runtime to load scenes by id or path.
pub struct SceneLoader {
    repo: AnySceneRepository,
}

impl SceneLoader {
    pub fn new(mod_source: PathBuf) -> Result<Self, EngineError> {
        Ok(Self {
            repo: create_scene_repository(&mod_source)?,
        })
    }

    /// Load a scene by file path (e.g. "/scenes/intro.yml").
    pub fn load_by_path(&self, path: &str) -> Result<Scene, EngineError> {
        self.repo.load_scene(path)
    }

    /// Load a scene by id using convention: id → scenes/{id}.yml
    pub fn load_by_id(&self, id: &str) -> Result<Scene, EngineError> {
        let path = format!("/scenes/{id}.yml");
        self.repo.load_scene(&path)
    }

    /// Load a scene from a generic reference:
    /// - "/scenes/name.yml" => treated as explicit path
    /// - "scene-id"         => treated as scene id (convention lookup)
    pub fn load_by_ref(&self, scene_ref: &str) -> Result<Scene, EngineError> {
        let trimmed = scene_ref.trim();
        if trimmed.starts_with('/') {
            return self.load_by_path(trimmed);
        }
        self.load_by_id(trimmed)
    }
}
