use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{scene::Scene, EngineError};

/// Standalone function — used at startup to load the entrypoint scene.
pub fn load_scene(mod_source: &Path, scene_path: &str) -> Result<Scene, EngineError> {
    let normalized = scene_path.trim_start_matches('/');
    let full_path = mod_source.join(normalized);

    let content = fs::read_to_string(&full_path).map_err(|source| EngineError::ManifestRead {
        path: full_path.clone(),
        source,
    })?;

    serde_yaml::from_str::<Scene>(&content).map_err(|source| EngineError::InvalidModYaml {
        path: full_path,
        source,
    })
}

/// World singleton — used at runtime to load scenes by id or path.
pub struct SceneLoader {
    mod_source: PathBuf,
}

impl SceneLoader {
    pub fn new(mod_source: PathBuf) -> Self {
        Self { mod_source }
    }

    /// Load a scene by file path (e.g. "/scenes/intro.yml").
    pub fn load_by_path(&self, path: &str) -> Result<Scene, EngineError> {
        load_scene(&self.mod_source, path)
    }

    /// Load a scene by id using convention: id → scenes/{id}.yml
    pub fn load_by_id(&self, id: &str) -> Result<Scene, EngineError> {
        let path = format!("/scenes/{id}.yml");
        load_scene(&self.mod_source, &path)
    }
}
