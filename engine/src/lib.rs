mod error;
mod game_loop;
mod mod_loader;
pub use error::EngineError;
pub mod animations;
pub mod assets;
pub mod buffer;
pub mod components;
pub mod effects;
pub mod events;
pub mod image_loader;
pub mod markup;
pub mod pipelines;
pub mod rasterizer;
pub mod render_policy;
pub mod renderer;
pub mod runtime_settings;
pub mod scene;
mod scene_loader;
pub mod systems;
pub mod terminal_caps;
pub mod world;

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
        use pipelines::startup::{StartupContext, StartupIssueLevel, StartupRunner};
        use runtime_settings::RuntimeSettings;
        use scene_loader::SceneLoader;
        use systems::animator::Animator;
        use systems::renderer::TerminalRenderer;
        use terminal_caps::target_fps_from_manifest;

        let entrypoint = self.mod_manifest["entrypoint"]
            .as_str()
            .expect("entrypoint already validated");

        let startup_ctx = StartupContext::new(&self.mod_source, &self.mod_manifest, entrypoint);
        let startup_report = StartupRunner::default().run(&startup_ctx)?;
        for issue in startup_report.issues() {
            if matches!(issue.level, StartupIssueLevel::Warning) {
                eprintln!("[startup:{}] warning: {}", issue.check, issue.message);
            }
        }

        let scene = scene_loader::load_scene(&self.mod_source, entrypoint)?;
        let target_fps = target_fps_from_manifest(&self.mod_manifest);
        let runtime_settings = RuntimeSettings::from_manifest(&self.mod_manifest);

        let (term_w, term_h) = crossterm::terminal::size()?;

        let mut world = world::World::new();
        world.register(EventQueue::new());
        world.register(buffer::Buffer::new(term_w, term_h));
        world.register(runtime_settings);
        world.register(assets::AssetRoot::new(self.mod_source.clone()));
        if runtime_settings.use_virtual_buffer {
            world.register(buffer::VirtualBuffer::new(
                runtime_settings.virtual_width,
                runtime_settings.virtual_height,
            ));
        }

        // Enter alt-screen and immediately paint black before the game loop starts.
        // This prevents the terminal's previous content from flashing on the first frame.
        let mut renderer = TerminalRenderer::new()?;
        renderer.clear_black()?;
        world.register(renderer);

        world.register(SceneLoader::new(self.mod_source.clone()));
        world.register_scoped(scene);
        world.register_scoped(Animator::new());

        let result = game_loop::game_loop(&mut world, target_fps);

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
