use std::collections::HashMap;
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
    scene_path_by_id: HashMap<String, String>,
}

impl SceneLoader {
    pub fn new(mod_source: PathBuf) -> Result<Self, EngineError> {
        let repo = create_scene_repository(&mod_source)?;
        let scene_path_by_id = build_scene_id_index(&repo)?;
        Ok(Self {
            repo,
            scene_path_by_id,
        })
    }

    /// Load a scene by file path (e.g. "/scenes/intro.yml").
    pub fn load_by_path(&self, path: &str) -> Result<Scene, EngineError> {
        self.repo.load_scene(path)
    }

    /// Load a scene by id using convention: id → scenes/{id}.yml
    pub fn load_by_id(&self, id: &str) -> Result<Scene, EngineError> {
        if let Some(path) = self.scene_path_by_id.get(id) {
            return self.repo.load_scene(path);
        }
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

fn build_scene_id_index(repo: &AnySceneRepository) -> Result<HashMap<String, String>, EngineError> {
    let mut scene_path_by_id = HashMap::new();
    for path in repo.discover_scene_paths()? {
        let Ok(scene) = repo.load_scene(&path) else {
            continue;
        };
        scene_path_by_id.entry(scene.id).or_insert(path);
    }
    Ok(scene_path_by_id)
}

#[cfg(test)]
mod tests {
    use super::SceneLoader;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_by_ref_resolves_scene_id_not_matching_filename() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_dir.join("scenes/3d-scene.yml"),
            r#"
id: playground-3d-scene
title: 3D
bg_colour: black
layers: []
"#,
        )
        .expect("write scene");

        let loader = SceneLoader::new(mod_dir).expect("create scene loader");
        let scene = loader
            .load_by_ref("playground-3d-scene")
            .expect("resolve by id");
        assert_eq!(scene.id, "playground-3d-scene");
    }
}
