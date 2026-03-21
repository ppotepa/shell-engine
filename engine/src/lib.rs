//! Root crate for the Shell Quest engine — initialises a mod, runs startup checks, and drives the game loop.

pub mod debug_features;
pub mod debug_log;
mod error;
mod game_loop;
mod mod_loader;
pub use error::EngineError;

// Re-export core modules from engine-core for compatibility
pub use engine_core::{animations, buffer, effects, logging, markup, scene};

pub mod asset_cache;
pub mod asset_source;
pub mod assets;
pub mod audio;
pub mod behavior;
pub mod events;
pub mod game_object;
pub mod game_state;
pub mod image_loader;
pub mod pipelines;
pub mod rasterizer;
pub mod render_policy;
pub mod repositories;
pub mod runtime_settings;
mod scene_compiler;
mod scene_loader;
pub mod scene_runtime;
mod services;
pub mod systems;
pub mod terminal_caps;
pub mod world;
mod splash;

/// Returns (behavior_name, fields) tuples for all built-in behaviors.
/// Re-exported from engine crate to make available in authoring catalog.
pub fn behavior_catalog() -> Vec<(
    &'static str,
    Vec<engine_core::authoring::metadata::FieldMetadata>,
)> {
    engine_core::authoring::catalog::behavior_catalog()
}

use std::path::{Path, PathBuf};

use mod_loader::load_mod_manifest;
use serde_yaml::Value;
use services::EngineWorldAccess;

/// Top-level engine handle that owns the mod source path and parsed manifest.
#[derive(Debug)]
pub struct ShellEngine {
    mod_source: PathBuf,
    mod_manifest: Value,
    config: EngineConfig,
}

/// Runtime launch options passed explicitly from the launcher.
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub renderer_mode: Option<String>,
    pub debug_feature: bool,
    pub sound_server: bool,
    pub sound_server_cmd: Option<String>,
    /// Override the mod's entrypoint — jump straight to this scene path.
    pub start_scene: Option<String>,
}

impl ShellEngine {
    /// Loads and validates a mod from `mod_source` (directory or `.zip`).
    pub fn new(mod_source: impl Into<PathBuf>) -> Result<Self, EngineError> {
        Self::new_with_config(mod_source, EngineConfig::default())
    }

    /// Loads and validates a mod from `mod_source` with explicit runtime config.
    pub fn new_with_config(
        mod_source: impl Into<PathBuf>,
        config: EngineConfig,
    ) -> Result<Self, EngineError> {
        let mod_source = mod_source.into();
        let mod_manifest = load_mod_manifest(&mod_source)?;

        Ok(Self {
            mod_source,
            mod_manifest,
            config,
        })
    }

    /// Runs startup checks, enters the alt-screen, and drives the game loop until the player quits.
    pub fn run(&self) -> Result<(), EngineError> {
        use events::EventQueue;
        use pipelines::startup::{StartupContext, StartupIssueLevel, StartupRunner};
        use runtime_settings::RuntimeSettings;
        use scene_loader::SceneLoader;
        use scene_runtime::SceneRuntime;
        use systems::animator::Animator;
        use systems::renderer::TerminalRenderer;
        use terminal_caps::target_fps_from_manifest;

        let manifest_entrypoint = self.mod_manifest["entrypoint"]
            .as_str()
            .expect("entrypoint already validated");
        let entrypoint = self
            .config
            .start_scene
            .as_deref()
            .unwrap_or(manifest_entrypoint);
        logging::info(
            "engine.run",
            format!(
                "starting engine run: mod_source={} entrypoint={} dev={} sound_server={}",
                self.mod_source.display(),
                entrypoint,
                self.config.debug_feature,
                self.config.sound_server
            ),
        );

        let startup_ctx = StartupContext::new(&self.mod_source, &self.mod_manifest, entrypoint);
        let startup_report = StartupRunner::default().run(&startup_ctx)?;
        for issue in startup_report.issues() {
            if matches!(issue.level, StartupIssueLevel::Warning) {
                logging::warn(
                    "engine.startup",
                    format!("check={} warning={}", issue.check, issue.message),
                );
            }
        }

        let scene = scene_loader::load_scene(&self.mod_source, entrypoint)?;
        logging::info(
            "engine.scene",
            format!("loaded entry scene: id={} title={}", scene.id, scene.title),
        );
        let target_fps = target_fps_from_manifest(&self.mod_manifest);
        let mut runtime_settings = RuntimeSettings::from_manifest(&self.mod_manifest);
        if let Some(mode) = self
            .config
            .renderer_mode
            .as_deref()
            .and_then(parse_renderer_mode)
        {
            runtime_settings.renderer_mode_override = Some(mode);
        }

        let (term_w, term_h) = crossterm::terminal::size()?;
        let (virtual_w, virtual_h) = runtime_settings.resolved_virtual_size(term_w, term_h);

        let mut world = world::World::new();
        world.register(EventQueue::new());
        world.register(buffer::Buffer::new(term_w, term_h));
        world.register(audio::AudioRuntime::from_options(
            self.config.sound_server,
            self.config.sound_server_cmd.clone(),
        ));
        world.register(runtime_settings);
        world.register(debug_features::DebugFeatures::from_enabled(
            self.config.debug_feature,
        ));
        world.register(debug_log::DebugLogBuffer::new(64));
        world.register(assets::AssetRoot::new(self.mod_source.clone()));
        world.register(game_state::GameState::new());
        if runtime_settings.use_virtual_buffer {
            world.register(buffer::VirtualBuffer::new(virtual_w, virtual_h));
        }

        // Enter alt-screen, hard-reset console surface, then paint black before first frame.
        // This prevents the terminal's previous content from flashing on the first frame.
        let mut renderer = TerminalRenderer::new()?;
        renderer.reset_console()?;
        renderer.clear_black()?;
        let splash_bg = scene
            .bg_colour
            .as_ref()
            .map(crossterm::style::Color::from)
            .unwrap_or(crossterm::style::Color::Black);
        splash::show_splash(splash_bg);
        world.register(renderer);

        world.register(SceneLoader::new(self.mod_source.clone())?);
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator::new());

        let result = game_loop::game_loop(&mut world, target_fps);

        if let Some(renderer) = world.renderer_mut() {
            let _ = renderer.shutdown();
        }

        match &result {
            Ok(()) => logging::info("engine.run", "engine loop exited cleanly"),
            Err(error) => logging::error("engine.run", format!("engine loop failed: {error}")),
        }

        result
    }

    /// Returns the path to the mod source used to initialise this engine.
    pub fn mod_source(&self) -> &Path {
        &self.mod_source
    }

    /// Returns the parsed `mod.yaml` manifest value.
    pub fn mod_manifest(&self) -> &Value {
        &self.mod_manifest
    }
}

fn parse_renderer_mode(raw: &str) -> Option<scene::SceneRenderedMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "cell" => Some(scene::SceneRenderedMode::Cell),
        "halfblock" | "half-block" => Some(scene::SceneRenderedMode::HalfBlock),
        "quadblock" | "quad-block" => Some(scene::SceneRenderedMode::QuadBlock),
        "braille" => Some(scene::SceneRenderedMode::Braille),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ShellEngine;
    use crate::pipelines::startup::checks::{
        EffectRegistryCheck, FontGlyphCoverageCheck, FontManifestCheck, ImageAssetsCheck,
        SceneGraphCheck,
    };
    use crate::pipelines::startup::{StartupContext, StartupRunner};
    use crate::repositories::{create_scene_repository, SceneRepository};
    use crate::scene_loader;
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

    #[test]
    fn real_playground_mod_manifest_and_entrypoint_load() {
        assert_real_mod_starts("playground");
    }

    #[test]
    fn real_shell_quest_mod_manifest_and_entrypoint_load() {
        assert_real_mod_starts("shell-quest");
    }

    #[test]
    fn real_playground_scenes_all_load() {
        assert_real_mod_scenes_load("playground");
    }

    #[test]
    fn real_shell_quest_scenes_all_load() {
        assert_real_mod_scenes_load("shell-quest");
    }

    fn assert_real_mod_starts(mod_name: &str) {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .expect("repo root");
        let mod_dir = repo_root.join("mods").join(mod_name);

        let engine = ShellEngine::new(&mod_dir).expect("engine should initialize from real mod");
        let entrypoint = engine
            .mod_manifest()
            .get("entrypoint")
            .and_then(|value| value.as_str())
            .expect("entrypoint string");

        let startup_ctx = StartupContext::new(&mod_dir, engine.mod_manifest(), entrypoint);
        StartupRunner::with_checks(vec![
            Box::new(SceneGraphCheck),
            Box::new(EffectRegistryCheck),
            Box::new(ImageAssetsCheck),
            Box::new(FontManifestCheck),
            Box::new(FontGlyphCoverageCheck),
        ])
        .run(&startup_ctx)
        .expect("startup checks should pass");

        let scene = scene_loader::load_scene(&mod_dir, entrypoint)
            .expect("entrypoint scene should load from real mod");
        assert!(
            !scene.id.trim().is_empty(),
            "entrypoint scene id should not be empty for {mod_name}"
        );
        assert!(
            !scene.title.trim().is_empty(),
            "entrypoint scene title should not be empty for {mod_name}"
        );
    }

    fn assert_real_mod_scenes_load(mod_name: &str) {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .expect("repo root");
        let mod_dir = repo_root.join("mods").join(mod_name);
        let repo = create_scene_repository(&mod_dir).expect("scene repository");
        let scene_paths = repo.discover_scene_paths().expect("discover scene paths");

        assert!(
            !scene_paths.is_empty(),
            "expected at least one discoverable scene in {mod_name}"
        );

        for scene_path in scene_paths {
            let scene = repo
                .load_scene(&scene_path)
                .unwrap_or_else(|err| panic!("{mod_name} scene {scene_path} should load: {err}"));
            assert!(
                !scene.id.trim().is_empty(),
                "{mod_name} scene {scene_path} should have non-empty id"
            );
            assert!(
                !scene.title.trim().is_empty(),
                "{mod_name} scene {scene_path} should have non-empty title"
            );
        }
    }
}
