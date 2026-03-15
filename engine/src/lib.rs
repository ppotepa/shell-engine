mod mod_loader;
pub mod scene;
mod scene_loader;
pub mod renderer;

use std::path::{Path, PathBuf};

use mod_loader::load_mod_manifest;
use serde_yaml::Value;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("mod source path does not exist: {0}")]
    SourceNotFound(PathBuf),
    #[error("unsupported mod source, expected directory or .zip file: {0}")]
    UnsupportedSource(PathBuf),
    #[error("failed to read mod manifest from {path}: {source}")]
    ManifestRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("zip archive error for {path}: {source}")]
    ZipArchive {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("missing required mod entrypoint file mod.yaml in source: {0}")]
    MissingModEntrypoint(PathBuf),
    #[error("missing required field `{field}` in mod.yaml for source: {path}")]
    MissingManifestField { path: PathBuf, field: String },
    #[error("invalid field `{field}` in mod.yaml for source {path}, expected {expected}")]
    InvalidManifestFieldType {
        path: PathBuf,
        field: String,
        expected: String,
    },
    #[error("entrypoint scene `{entrypoint}` not found in mod source: {mod_source}")]
    MissingSceneEntrypoint {
        mod_source: PathBuf,
        entrypoint: String,
    },
    #[error("invalid mod.yaml content in source {path}: {source}")]
    InvalidModYaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("render error: {0}")]
    Render(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ShellEngine {
    mod_source: PathBuf,
    mod_manifest: Value,
}

impl ShellEngine {
    pub fn new(mod_source: impl Into<PathBuf>) -> Result<Self, EngineError> {
        let mod_source = mod_source.into();
        let mod_manifest = load_mod_manifest(&mod_source)?;

        Ok(Self {
            mod_source,
            mod_manifest,
        })
    }

    /// Load and render the entrypoint scene declared in mod.yaml.
    pub fn run(&self) -> Result<(), EngineError> {
        let entrypoint = self.mod_manifest["entrypoint"]
            .as_str()
            .expect("entrypoint already validated in loader");

        let scene = scene_loader::load_scene(&self.mod_source, entrypoint)?;

        if scene.cutscene {
            renderer::render_cutscene(&scene)?;
        }

        Ok(())
    }

    pub fn mod_source(&self) -> &Path {
        &self.mod_source
    }

    pub fn mod_manifest(&self) -> &Value {
        &self.mod_manifest
    }
}

#[cfg(test)]
mod tests {
    use super::ShellEngine;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    fn write_valid_mod(mod_dir: &std::path::Path) {
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_dir.join("mod.yaml"),
            "name: Shell Quest\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
        )
        .expect("write manifest");
        fs::write(
            mod_dir.join("scenes/intro.yml"),
            "id: intro\ntitle: Intro\ncutscene: true\nskippable: true\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write scene");
    }

    #[test]
    fn loads_mod_from_directory_when_mod_yaml_and_scene_exist() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("shell-quest");
        write_valid_mod(&mod_dir);

        let engine = ShellEngine::new(mod_dir).expect("engine should initialize");

        assert_eq!(engine.mod_manifest()["name"], "Shell Quest");
    }

    #[test]
    fn fails_when_mod_yaml_is_missing() {
        let temp = tempdir().expect("temp dir");
        let mod_dir: PathBuf = temp.path().join("empty-mod");
        fs::create_dir_all(&mod_dir).expect("create mod dir");

        let result = ShellEngine::new(mod_dir);

        assert!(result.is_err());
    }

    #[test]
    fn fails_when_entrypoint_scene_is_missing() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("shell-quest");
        fs::create_dir_all(&mod_dir).expect("create mod dir");
        fs::write(
            mod_dir.join("mod.yaml"),
            "name: Shell Quest\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
        )
        .expect("write manifest");

        let result = ShellEngine::new(mod_dir);

        assert!(result.is_err());
    }
}
