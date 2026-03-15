mod mod_loader;
mod error;
mod game_loop;
pub use error::EngineError;
pub mod scene;
mod scene_loader;
pub mod renderer;
pub mod world;
pub mod events;
pub mod buffer;
pub mod components;
pub mod effects;
pub mod rasterizer;
pub mod systems;
pub mod terminal_caps;

use std::path::{Path, PathBuf};

use mod_loader::load_mod_manifest;
use serde_yaml::Value;

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

    pub fn run(&self) -> Result<(), EngineError> {
        use events::EventQueue;
        use scene_loader::SceneLoader;
        use systems::animator::Animator;
        use systems::renderer::TerminalRenderer;
        use terminal_caps::{TerminalCaps, TerminalRequirements};

        // Check terminal requirements declared by the mod before doing anything else.
        if let Some(req) = TerminalRequirements::from_manifest(&self.mod_manifest) {
            let caps = TerminalCaps::detect()?;
            let violations = caps.validate(&req);
            if !violations.is_empty() {
                let details = violations
                    .iter()
                    .map(|v| format!("{}: requires {}, detected {}", v.requirement, v.required, v.detected))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(EngineError::TerminalRequirementsNotMet(details));
            }
        }

        let entrypoint = self.mod_manifest["entrypoint"]
            .as_str()
            .expect("entrypoint already validated");

        let scene = scene_loader::load_scene(&self.mod_source, entrypoint)?;

        let (term_w, term_h) = crossterm::terminal::size()?;

        let mut world = world::World::new();
        world.register(EventQueue::new());
        world.register(buffer::Buffer::new(term_w, term_h));

        // Enter alt-screen and immediately paint black before the game loop starts.
        // This prevents the terminal's previous content from flashing on the first frame.
        let mut renderer = TerminalRenderer::new()?;
        renderer.clear_black()?;
        world.register(renderer);

        world.register(SceneLoader::new(self.mod_source.clone()));
        world.register_scoped(scene);
        world.register_scoped(Animator::new());

        let result = game_loop::game_loop(&mut world);

        if let Some(renderer) = world.get_mut::<TerminalRenderer>() {
            let _ = renderer.shutdown();
        }

        result
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
