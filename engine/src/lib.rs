//! Root crate for the Shell Quest engine — initialises a mod, runs startup checks, and drives the game loop.

pub mod bench;
pub mod debug_features;
pub mod debug_log;
mod error;
pub mod frame_capture;
pub mod frame_compare;
mod game_loop;
mod mod_loader;
pub use error::EngineError;

// Re-export core modules from engine-core for compatibility
pub use engine_core::{animations, buffer, effects, logging, markup, scene};
// Re-export audio subsystem
pub use engine_audio as audio;
// Re-export animation subsystem
pub use engine_animation as animation;
// Re-export 3D subsystem
pub use engine_3d as rendering_3d;
// Re-export terminal subsystem
pub use engine_terminal as terminal;
// Re-export game subsystem
pub use engine_game as game;
// Re-export pipeline subsystem
pub use engine_pipeline as pipeline;
// Re-export asset subsystem
pub use engine_asset as asset;

pub mod asset_cache;
pub mod asset_source;
pub mod assets;
pub mod behavior;
pub mod events;
pub mod game_object;
pub mod game_state;
pub mod image_loader;
pub mod mod_behaviors;
pub mod obj_prerender;
pub mod pipeline_flags;
pub mod pipelines;
pub mod rasterizer;
pub mod render_policy;
pub mod runtime_settings;
pub mod scene3d_atlas;
pub mod scene3d_format;
pub mod scene3d_resolve;
mod scene_loader;
pub mod scene_pipeline;
pub mod scene_runtime;
mod services;
mod splash;
pub mod strategy;
pub mod systems;
pub mod terminal_caps;
pub mod world;

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
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub renderer_mode: Option<String>,
    pub debug_feature: bool,
    pub audio: bool,
    /// Override the mod's entrypoint — jump straight to this scene path.
    pub start_scene: Option<String>,
    /// Skip the engine splash screen on startup.
    pub skip_splash: bool,
    /// Enable compositor optimizations (#4 layer-scratch, #5 dirty-halfblock).
    pub opt_comp: bool,
    /// Enable present optimizations (#13 hash-based frame skip).
    pub opt_present: bool,
    /// Enable dirty-region diff scan (experimental — off by default).
    pub opt_diff: bool,
    /// Enable unified frame-skip coordination (PostFX cache + Presenter sync).
    pub opt_skip: bool,
    /// Enable row-level dirty skip in diff scan (experimental — off by default).
    pub opt_rowdiff: bool,
    /// Enable async display sink: offload terminal I/O to background thread.
    /// Decouples main thread from write/flush latency (1-5ms/frame).
    pub opt_async_display: bool,
    /// Run benchmark mode for N seconds, then show results and exit.
    pub bench_secs: Option<f32>,
    /// Capture frames to this directory for visual regression testing.
    pub capture_frames_dir: Option<PathBuf>,
    /// Override the mod's target FPS (e.g. 240 for uncapped benchmarks).
    pub target_fps_override: Option<u16>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            renderer_mode: None,
            debug_feature: false,
            audio: false,
            start_scene: None,
            skip_splash: false,
            opt_comp: false,
            opt_present: false,
            opt_diff: false,
            opt_skip: false,
            opt_rowdiff: false,
            opt_async_display: false,
            bench_secs: None,
            capture_frames_dir: None,
            target_fps_override: None,
        }
    }
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
        use engine_animation::Animator;
        use engine_asset::SceneRepository;
        use engine_mod::startup::{
            StartupContext, StartupIssueLevel, StartupRunner, StartupSceneFile,
        };
        use events::EventQueue;
        use runtime_settings::RuntimeSettings;
        use scene_loader::SceneLoader;
        use scene_runtime::SceneRuntime;
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
                "starting engine run: mod_source={} entrypoint={} dev={} audio={}",
                self.mod_source.display(),
                entrypoint,
                self.config.debug_feature,
                self.config.audio
            ),
        );

        let scene_loader_fn =
            |mod_source: &std::path::Path| -> Result<Vec<StartupSceneFile>, EngineError> {
                let repo = engine_asset::create_scene_repository(mod_source)?;
                let paths = repo.discover_scene_paths()?;
                let mut scenes = Vec::with_capacity(paths.len());
                for path in paths {
                    let scene = repo.load_scene(&path)?;
                    scenes.push(StartupSceneFile { path, scene });
                }
                Ok(scenes)
            };
        let font_checker = |mod_src: Option<&std::path::Path>, font: &str| -> bool {
            rasterizer::has_font_assets(mod_src, font)
        };
        let glyph_checker =
            |mod_src: Option<&std::path::Path>, font: &str, text: &str| -> Option<Vec<char>> {
                rasterizer::missing_glyphs(mod_src, font, text)
            };
        let image_checker = |mod_src: &std::path::Path, source: &str| -> bool {
            image_loader::has_image_asset(mod_src, source)
        };
        let rhai_validator =
            |script: &str, src: Option<&str>, scene: &crate::scene::Scene| -> Result<(), String> {
                behavior::smoke_validate_rhai_script(script, src, scene)
            };
        let startup_ctx = StartupContext::new(
            &self.mod_source,
            &self.mod_manifest,
            entrypoint,
            &scene_loader_fn,
        )
        .with_font_asset_checker(&font_checker)
        .with_glyph_coverage_checker(&glyph_checker)
        .with_image_asset_checker(&image_checker)
        .with_rhai_script_validator(&rhai_validator);
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
        let target_fps = self
            .config
            .target_fps_override
            .unwrap_or_else(|| target_fps_from_manifest(&self.mod_manifest));
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
            self.config.audio,
            &self.mod_source.to_string_lossy(),
        ));
        world.register(runtime_settings);
        world.register(debug_features::DebugFeatures::from_enabled(
            self.config.debug_feature,
        ));
        world.register(debug_features::FpsCounter::default());
        world.register(debug_features::SystemTimings::default());
        world.register(debug_features::ProcessStats::default());
        world.register(debug_log::DebugLogBuffer::new(64));
        world.register(assets::AssetRoot::new(self.mod_source.clone()));
        // Load mod-defined behaviors from behaviors/*.yml
        let mod_behavior_registry = mod_behaviors::load_mod_behaviors(&self.mod_source);
        world.register(mod_behavior_registry);
        world.register(game_state::GameState::new());
        let mut pflags = pipeline_flags::PipelineFlags::default();
        pflags.opt_comp = self.config.opt_comp;
        pflags.opt_present = self.config.opt_present;
        pflags.opt_diff = self.config.opt_diff;
        pflags.opt_skip = self.config.opt_skip;
        pflags.opt_rowdiff = self.config.opt_rowdiff;
        pflags.opt_async_display = self.config.opt_async_display;
        world.register(pflags);
        world.register(strategy::PipelineStrategies::from_flags(
            self.config.opt_diff,
            self.config.opt_comp,
            self.config.opt_present,
            self.config.opt_rowdiff,
            self.config.opt_async_display,
            Box::new(strategy::AnsiBatchFlusher),
        ));
        if let Some(secs) = self.config.bench_secs {
            world.register(bench::BenchmarkState::new(
                secs,
                self.config.opt_comp,
                self.config.opt_present,
                self.config.opt_diff,
            ));
        }
        if runtime_settings.use_virtual_buffer {
            world.register(buffer::VirtualBuffer::new(virtual_w, virtual_h));
        }

        // Register frame-skip oracle (either AlwaysRender or CoordinatedSkip based on --opt-skip flag)
        if self.config.opt_skip {
            world.register(std::sync::Mutex::new(
                Box::new(strategy::CoordinatedSkip::default())
                    as Box<dyn strategy::FrameSkipOracle>,
            ));
        } else {
            world.register(std::sync::Mutex::new(
                Box::new(strategy::AlwaysRender) as Box<dyn strategy::FrameSkipOracle>
            ));
        }

        // Enter alt-screen, hard-reset console surface, then paint black before first frame.
        // This prevents the terminal's previous content from flashing on the first frame.
        let mut renderer = TerminalRenderer::new_with_async(self.config.opt_async_display)?;
        renderer.reset_console()?;
        renderer.clear_black()?;
        let splash_bg = scene
            .bg_colour
            .as_ref()
            .map(|tc| {
                let engine_color = engine_core::color::Color::from(tc);
                engine_render_terminal::color_convert::to_crossterm(engine_color)
            })
            .unwrap_or(crossterm::style::Color::Black);
        if !self.config.skip_splash {
            splash::show_splash(splash_bg);
        }
        world.register(renderer);

        world.register(SceneLoader::new(self.mod_source.clone())?);
        // Register the scene preparation pipeline as a world resource so that
        // scene_lifecycle can clone the Arc and run it without holding a borrow.
        world.register(std::sync::Arc::new(scene_pipeline::ScenePipeline::default()));
        // Prepare the entry scene (prerender, future steps) before activating it.
        if let Some(pipeline) = world
            .get::<std::sync::Arc<scene_pipeline::ScenePipeline>>()
            .cloned()
        {
            pipeline.prepare(&scene, &mut world);
        }
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator::new());

        // Initialize frame capture if requested
        let mut frame_capture = if let Some(ref dir) = self.config.capture_frames_dir {
            Some(frame_capture::FrameCapture::new(dir.clone())?)
        } else {
            None
        };

        let result = game_loop::game_loop(&mut world, target_fps, &mut frame_capture);

        // Write benchmark report if bench mode was active.
        if self.config.bench_secs.is_some() {
            if let Some(bs) = world.get::<bench::BenchmarkState>() {
                let results = bs.results();
                match bench::write_report(&results) {
                    Ok(path) => {
                        logging::info(
                            "engine.bench",
                            format!("benchmark report written to {}", path.display()),
                        );
                    }
                    Err(e) => {
                        logging::warn("engine.bench", format!("failed to write report: {e}"));
                    }
                }
            }
        }

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
    use crate::scene_loader;
    use crate::EngineError;
    use engine_asset::{create_scene_repository, SceneRepository};
    use engine_mod::startup::checks::{
        EffectRegistryCheck, FontGlyphCoverageCheck, FontManifestCheck, ImageAssetsCheck,
        SceneGraphCheck,
    };
    use engine_mod::startup::{StartupContext, StartupRunner, StartupSceneFile};
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

        let scene_loader_fn =
            |mod_source: &std::path::Path| -> Result<Vec<StartupSceneFile>, EngineError> {
                let repo = create_scene_repository(mod_source)?;
                let paths = repo.discover_scene_paths()?;
                let mut scenes = Vec::with_capacity(paths.len());
                for path in paths {
                    let scene = repo.load_scene(&path)?;
                    scenes.push(StartupSceneFile { path, scene });
                }
                Ok(scenes)
            };
        let font_checker = |mod_src: Option<&std::path::Path>, font: &str| -> bool {
            crate::rasterizer::has_font_assets(mod_src, font)
        };
        let glyph_checker =
            |mod_src: Option<&std::path::Path>, font: &str, text: &str| -> Option<Vec<char>> {
                crate::rasterizer::missing_glyphs(mod_src, font, text)
            };
        let image_checker = |mod_src: &std::path::Path, source: &str| -> bool {
            crate::image_loader::has_image_asset(mod_src, source)
        };
        let startup_ctx = StartupContext::new(
            &mod_dir,
            engine.mod_manifest(),
            entrypoint,
            &scene_loader_fn,
        )
        .with_font_asset_checker(&font_checker)
        .with_glyph_coverage_checker(&glyph_checker)
        .with_image_asset_checker(&image_checker);
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
