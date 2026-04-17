//! Root crate for Shell Engine — initialises a mod, runs startup checks, and drives the game loop.

pub mod bench;
pub mod debug_features;
pub mod debug_log;
mod error;
pub mod frame_capture;
pub mod frame_compare;
mod game_loop;
mod mod_loader;
pub mod mod_manifest;
pub use error::EngineError;

// Re-export core modules from engine-core for compatibility
pub use engine_core::{animations, buffer, effects, logging, markup, scene};
// Re-export audio subsystem
pub use engine_audio as audio;
// Re-export animation subsystem
pub use engine_animation as animation;
// Re-export 3D subsystem
pub use engine_3d as rendering_3d;
// Re-export game subsystem
pub use engine_game as game;
// Re-export persistence subsystem
pub use engine_persistence as persistence;
// Re-export pipeline subsystem
pub use engine_pipeline as pipeline;
// Re-export asset subsystem
pub use engine_asset as asset;

pub mod asset_cache;
pub mod asset_source;
pub mod assets;
pub mod audio_sequencer;
pub mod behavior;
pub mod events;
pub mod game_object;
pub mod game_state;
pub mod level_state;
pub mod mod_behaviors;
#[cfg(feature = "render-3d")]
pub mod obj_prerender;
pub mod pipeline_flags;
pub mod pipelines;
pub mod rasterizer;
pub mod render_policy;
pub mod runtime_effects;
pub mod runtime_settings;
pub mod scene3d_atlas;
pub mod scene3d_format;
pub mod scene3d_resolve;
#[cfg(feature = "render-3d")]
pub mod scene3d_runtime_store;
mod scene_loader;
pub mod scene_pipeline;
pub mod scene_runtime;
mod services;
mod splash;
pub mod strategy;
pub mod systems;
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

use engine_render::RendererBackend;
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
    pub debug_feature: bool,
    pub audio: bool,
    /// Override the mod's entrypoint — jump straight to this scene path.
    pub start_scene: Option<String>,
    /// Skip the engine splash screen on startup.
    pub skip_splash: bool,
    /// Enable compositor optimizations (#4 layer-scratch, #5 dirty-region narrowing).
    /// Enabled by default in the launcher.
    pub opt_comp: bool,
    /// Enable present optimizations (#13 hash-based frame skip).
    pub opt_present: bool,
    /// Enable dirty-region diff scan (experimental — off by default).
    pub opt_diff: bool,
    /// Enable unified frame-skip coordination (PostFX cache + Presenter sync).
    pub opt_skip: bool,
    /// Enable row-level dirty skip in diff scan.
    /// Enabled by default in the launcher.
    pub opt_rowdiff: bool,
    /// Enable async display-related optimizations.
    pub opt_async_display: bool,
    /// Run benchmark mode for N seconds, then show results and exit.
    pub bench_secs: Option<f32>,
    /// Capture frames to this directory for visual regression testing.
    pub capture_frames_dir: Option<PathBuf>,
    /// Override the mod's target FPS (e.g. 240 for uncapped benchmarks).
    pub target_fps_override: Option<u16>,
    /// SDL startup window ratio constraint (None means free ratio).
    pub sdl_window_ratio: Option<(u32, u32)>,
    /// SDL startup pixel scale multiplier for logical surface.
    pub sdl_pixel_scale: u32,
    /// Enable SDL VSync at canvas creation time.
    pub sdl_vsync: bool,
}

const SDL_DEFAULT_OUTPUT_WIDTH: u16 = 120;
const SDL_DEFAULT_OUTPUT_HEIGHT: u16 = 40;

fn sdl_startup_output_size(manifest: &Value) -> (u16, u16) {
    use engine_runtime::RuntimeSettings;
    let settings = RuntimeSettings::from_manifest(manifest);
    // For a fixed render size, the output IS the render buffer — use it directly.
    if let Some((w, h)) = settings.render_size.fixed() {
        return (w, h);
    }
    (SDL_DEFAULT_OUTPUT_WIDTH, SDL_DEFAULT_OUTPUT_HEIGHT)
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            debug_feature: false,
            audio: false,
            start_scene: None,
            skip_splash: false,
            opt_comp: true,
            opt_present: false,
            opt_diff: false,
            opt_skip: false,
            opt_rowdiff: true,
            opt_async_display: false,
            bench_secs: None,
            capture_frames_dir: None,
            target_fps_override: None,
            sdl_window_ratio: Some((16, 9)),
            sdl_pixel_scale: 8,
            sdl_vsync: true,
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
        use engine_behavior::init_behavior_system;
        use engine_events::InputBackend;
        use engine_mod::display_config::target_fps_from_manifest;
        use engine_mod::startup::{
            StartupContext, StartupIssueLevel, StartupRunner, StartupSceneFile,
        };
        use engine_mod::StartupOutputSetting;
        use events::EventQueue;
        use runtime_settings::RuntimeSettings;
        use scene_loader::SceneLoader;
        use scene_runtime::SceneRuntime;

        // Initialize behavior system with mod source for Rhai module resolution
        init_behavior_system(
            self.mod_source
                .to_str()
                .expect("mod_source path is valid UTF-8"),
        );

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
            engine_asset::has_image_asset(mod_src, source)
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
        .with_selected_output(StartupOutputSetting::Sdl2)
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
        let window_title = self
            .mod_manifest
            .get("name")
            .and_then(serde_yaml::Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .map(|name| format!("{name} - Shell Engine"))
            .unwrap_or_else(|| "Shell Engine".to_string());
        let mut runtime_settings = RuntimeSettings::from_manifest(&self.mod_manifest);
        runtime_settings.is_pixel_backend = true;

        let (output_w, output_h) = sdl_startup_output_size(&self.mod_manifest);
        let layout = runtime_settings::buffer_layout_for_scene(
            &runtime_settings,
            &scene,
            output_w,
            output_h,
        );
        logging::info(
            "engine.runtime",
            format!(
                "output={} output_size={}x{} render_size={}x{} policy={:?} scene_override={}",
                "sdl2",
                output_w,
                output_h,
                layout.render_width,
                layout.render_height,
                runtime_settings.presentation_policy,
                scene.virtual_size_override.as_deref().unwrap_or("none"),
            ),
        );

        let mut world = world::World::new();
        let presentation_policy = runtime_settings.presentation_policy;
        world.register(EventQueue::new());
        world.register(buffer::Buffer::new(
            layout.render_width,
            layout.render_height,
        ));
        let synth_cues =
            audio_sequencer::AudioSequencerState::synthesize_note_sheets_if_any(&self.mod_source)
                .unwrap_or_default();
        let mut audio_runtime = audio::AudioRuntime::from_options(
            self.config.audio,
            &self.mod_source.to_string_lossy(),
        );
        for (cue, (sr, samples)) in synth_cues {
            audio_runtime.register_memory_cue(&cue, sr, samples);
        }
        world.register(audio_runtime);
        world.register(audio_sequencer::AudioSequencerState::from_mod_source(
            &self.mod_source,
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
        world.register(level_state::load_level_state(
            &self.mod_source,
            &self.mod_manifest,
        ));
        world.register(game::GameplayWorld::new());
        world.register(game::GameplayStrategies::default());
        world.register(game::CollisionStrategies::default());
        world.register(engine_behavior::EmitterState::default());
        // Load mod catalogs (prefabs, weapons, emitters, etc.) into the world
        // so behaviors can access them at runtime.
        {
            let catalogs_dir = self.mod_source.join("catalogs");
            if catalogs_dir.is_dir() {
                if let Ok(cats) =
                    engine_behavior::catalog::ModCatalogs::load_from_directory(&catalogs_dir)
                {
                    world.register(cats);
                }
            }
        }
        // Load mod palettes from palettes/ and register default_palette from mod.yaml.
        {
            let palettes_dir = self.mod_source.join("palettes");
            match engine_behavior::palette::PaletteStore::load_from_directory(&palettes_dir) {
                Ok(store) => {
                    world.register(store);
                }
                Err(e) => {
                    eprintln!("[palette] load failed: {}", e);
                }
            }
            world.register(mod_manifest::ModManifestData::from_manifest(
                &self.mod_manifest,
            ));
        }
        let persistence_namespace = self
            .mod_manifest
            .get("name")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("shell-engine");
        world.register(engine_persistence::PersistenceStore::new(
            persistence_namespace,
        ));
        let pflags = pipeline_flags::PipelineFlags {
            opt_comp: self.config.opt_comp,
            opt_present: self.config.opt_present,
            opt_diff: self.config.opt_diff,
            opt_skip: self.config.opt_skip,
            opt_rowdiff: self.config.opt_rowdiff,
            opt_async_display: self.config.opt_async_display,
            ..pipeline_flags::PipelineFlags::default()
        };
        world.register(pflags);
        world.register(strategy::PipelineStrategies::from_flags(
            self.config.opt_diff,
            self.config.opt_comp,
            self.config.opt_present,
            self.config.opt_rowdiff,
            self.config.opt_async_display,
        ));
        if let Some(secs) = self.config.bench_secs {
            world.register(bench::BenchmarkState::new(
                secs,
                self.config.opt_comp,
                self.config.opt_present,
                self.config.opt_diff,
            ));
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

        let splash_bg = scene
            .bg_colour
            .as_ref()
            .map(engine_core::color::Color::from)
            .unwrap_or(engine_core::color::Color::Black);
        let splash_config = splash::config_from_manifest(&self.mod_source, &self.mod_manifest);
        let splash_enabled = !self.config.skip_splash && splash_config.enabled;
        let splash_scene_path = splash_config.scene_path.as_deref();

        let mut input_backend: Box<dyn InputBackend> = {
            #[cfg(feature = "sdl2")]
            {
                let (mut renderer, input) = engine_render_sdl2::renderer::Sdl2Backend::new_pair(
                    layout.render_width,
                    layout.render_height,
                    presentation_policy,
                    self.config.sdl_window_ratio,
                    self.config.sdl_pixel_scale,
                    self.config.sdl_vsync,
                    window_title,
                )
                .map_err(|error| EngineError::Render(std::io::Error::other(error)))?;
                renderer.clear().map_err(|error| {
                    EngineError::Render(std::io::Error::other(error.to_string()))
                })?;
                if splash_enabled {
                    renderer.set_splash_mode(true).map_err(|error| {
                        EngineError::Render(std::io::Error::other(error.to_string()))
                    })?;
                    splash::show_splash_on_output(
                        &mut renderer,
                        splash_bg,
                        (output_w, output_h),
                        splash_scene_path,
                    );
                    renderer.set_splash_mode(false).map_err(|error| {
                        EngineError::Render(std::io::Error::other(error.to_string()))
                    })?;
                }
                world.register(Box::new(renderer) as Box<dyn RendererBackend>);
                Box::new(input)
            }
            #[cfg(not(feature = "sdl2"))]
            {
                return Err(EngineError::Render(std::io::Error::other(
                    "SDL2 backend requested but engine was built without the `sdl2` feature",
                )));
            }
        };

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

        let result = game_loop::game_loop(
            &mut world,
            target_fps,
            input_backend.as_mut(),
            &mut frame_capture,
        );

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

#[cfg(test)]
mod tests {
    use super::sdl_startup_output_size;
    use super::ShellEngine;
    use crate::scene_loader;
    use crate::EngineError;
    use engine_asset::{create_scene_repository, SceneRepository};
    use engine_mod::startup::checks::{
        EffectRegistryCheck, FontGlyphCoverageCheck, FontManifestCheck, ImageAssetsCheck,
        SceneGraphCheck,
    };
    use engine_mod::startup::{StartupContext, StartupRunner, StartupSceneFile};
    use serde_yaml::Value;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    fn write_valid_mod(mod_dir: &std::path::Path) {
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_dir.join("mod.yaml"),
            "name: Shell Engine\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
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
        let mod_dir = temp.path().join("shell-engine");
        write_valid_mod(&mod_dir);

        let engine = ShellEngine::new(mod_dir).expect("engine should initialize");

        assert_eq!(engine.mod_manifest()["name"], "Shell Engine");
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
        let mod_dir = temp.path().join("shell-engine");
        fs::create_dir_all(&mod_dir).expect("create mod dir");
        fs::write(
            mod_dir.join("mod.yaml"),
            "name: Shell Engine\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
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
    fn real_asteroids_mod_manifest_and_entrypoint_load() {
        assert_real_mod_starts("asteroids");
    }

    #[test]
    fn real_playground_scenes_all_load() {
        assert_real_mod_scenes_load("playground");
    }

    #[test]
    fn real_asteroids_scenes_all_load() {
        assert_real_mod_scenes_load("asteroids");
    }

    #[test]
    fn sdl_startup_output_uses_default_render_size_when_no_display_block() {
        let manifest = serde_yaml::from_str::<Value>("name: demo\n").expect("manifest");

        // No display block → RuntimeSettings uses default render size (320x240)
        assert_eq!(sdl_startup_output_size(&manifest), (320, 240));
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
            engine_asset::has_image_asset(mod_src, source)
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
