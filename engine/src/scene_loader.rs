use std::{fs, path::Path};

use crate::{scene::Scene, EngineError};

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
