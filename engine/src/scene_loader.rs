//! High-level scene loading entry points used at startup and during runtime
//! scene transitions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::repositories::{create_scene_repository, AnySceneRepository, SceneRepository};
use crate::{scene::Scene, EngineError};

/// Loads a scene from the active mod source without constructing a long-lived
/// loader.
///
/// This is the startup path for the initial authored scene before the runtime
/// world has a [`SceneLoader`].
pub fn load_scene(mod_source: &Path, scene_path: &str) -> Result<Scene, EngineError> {
    create_scene_repository(mod_source)?.load_scene(scene_path)
}

/// Runtime scene loader that resolves authored scene ids to packaged or flat
/// scene manifests.
pub struct SceneLoader {
    repo: AnySceneRepository,
    scene_path_by_id: HashMap<String, String>,
    scene_ids_in_order: Vec<String>,
}

impl SceneLoader {
    /// Builds a loader and indexes discoverable scene ids for later lookups.
    pub fn new(mod_source: PathBuf) -> Result<Self, EngineError> {
        let repo = create_scene_repository(&mod_source)?;
        let (scene_path_by_id, scene_ids_in_order) = build_scene_id_index(&repo)?;
        Ok(Self {
            repo,
            scene_path_by_id,
            scene_ids_in_order,
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

    /// Loads a scene from the authored transition reference format:
    /// - "/scenes/name.yml" => treated as explicit path
    /// - "scene-id"         => treated as scene id (convention lookup)
    pub fn load_by_ref(&self, scene_ref: &str) -> Result<Scene, EngineError> {
        let trimmed = scene_ref.trim();
        if trimmed.starts_with('/') {
            return self.load_by_path(trimmed);
        }
        self.load_by_id(trimmed)
    }

    /// Returns the previous scene id in repository discovery order (wrap-around).
    pub fn prev_scene_id(&self, current_scene_id: &str) -> Option<String> {
        let len = self.scene_ids_in_order.len();
        if len == 0 {
            return None;
        }
        let idx = self
            .scene_ids_in_order
            .iter()
            .position(|id| id == current_scene_id)?;
        let prev_idx = if idx == 0 { len - 1 } else { idx - 1 };
        self.scene_ids_in_order.get(prev_idx).cloned()
    }

    /// Returns the next scene id in repository discovery order (wrap-around).
    pub fn next_scene_id(&self, current_scene_id: &str) -> Option<String> {
        let len = self.scene_ids_in_order.len();
        if len == 0 {
            return None;
        }
        let idx = self
            .scene_ids_in_order
            .iter()
            .position(|id| id == current_scene_id)?;
        let next_idx = (idx + 1) % len;
        self.scene_ids_in_order.get(next_idx).cloned()
    }
}

fn build_scene_id_index(
    repo: &AnySceneRepository,
) -> Result<(HashMap<String, String>, Vec<String>), EngineError> {
    let mut scene_path_by_id = HashMap::new();
    let mut scene_ids_in_order = Vec::new();
    for path in repo.discover_scene_paths()? {
        let Ok(scene) = repo.load_scene(&path) else {
            continue;
        };
        if !scene_path_by_id.contains_key(&scene.id) {
            scene_ids_in_order.push(scene.id.clone());
        }
        scene_path_by_id.entry(scene.id).or_insert(path);
    }
    Ok((scene_path_by_id, scene_ids_in_order))
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

    #[test]
    fn load_by_ref_resolves_scene_package_id() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes/intro/layers")).expect("create scene package");
        fs::write(
            mod_dir.join("scenes/intro/scene.yml"),
            r#"
id: packaged-intro
title: Package
next: null
"#,
        )
        .expect("write scene root");
        fs::write(
            mod_dir.join("scenes/intro/layers/base.yml"),
            r#"
- name: base
  sprites:
    - type: text
      content: HI
"#,
        )
        .expect("write layer");

        let loader = SceneLoader::new(mod_dir).expect("create scene loader");
        let scene = loader
            .load_by_ref("packaged-intro")
            .expect("resolve packaged scene by id");
        assert_eq!(scene.id, "packaged-intro");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn prev_next_scene_id_wraps_in_discovery_order() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_dir.join("scenes/a.yml"),
            "id: scene-a\ntitle: A\nbg_colour: black\nlayers: []\n",
        )
        .expect("write scene a");
        fs::write(
            mod_dir.join("scenes/b.yml"),
            "id: scene-b\ntitle: B\nbg_colour: black\nlayers: []\n",
        )
        .expect("write scene b");
        fs::write(
            mod_dir.join("scenes/c.yml"),
            "id: scene-c\ntitle: C\nbg_colour: black\nlayers: []\n",
        )
        .expect("write scene c");

        let loader = SceneLoader::new(mod_dir).expect("create scene loader");
        assert_eq!(loader.next_scene_id("scene-a").as_deref(), Some("scene-b"));
        assert_eq!(loader.prev_scene_id("scene-a").as_deref(), Some("scene-c"));
    }
}
