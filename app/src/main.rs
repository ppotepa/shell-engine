use clap::Parser;
use engine::behavior::init_behavior_system;
use engine::{logging, EngineConfig, ShellEngine};
use engine_mod::startup::checks::{
    AudioSequencerCheck, CatalogsCheck, EffectRegistryCheck, FontGlyphCoverageCheck,
    FontManifestCheck, GuiWidgetBindingsCheck, ImageAssetsCheck, LevelConfigCheck,
    RhaiScriptsCheck, SceneGraphCheck,
};
use engine_mod::startup::{
    StartupContext, StartupIssueLevel, StartupReport, StartupRunner, StartupSceneFile,
};
use engine_mod::{load_mod_manifest, StartupOutputSetting};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "shell-engine", about = "Shell Engine SDL2 launcher")]
struct Cli {
    /// Mod to load by name (resolves to mods/<name>/). When omitted, the interactive launcher starts.
    #[arg(long = "mod")]
    mod_name: Option<String>,
    /// Full mod source path (directory or .zip). Overrides --mod when set.
    #[arg(long = "mod-source", hide = true)]
    mod_source: Option<String>,
    /// SDL window ratio for startup window sizing (e.g. 16:9, 4:3, free).
    #[arg(long = "sdl-window-ratio", default_value = "16:9")]
    sdl_window_ratio: String,
    /// SDL startup pixel scale multiplier for logical render surface.
    #[arg(long = "sdl-pixel-scale", default_value_t = 8)]
    sdl_pixel_scale: u32,
    /// Disable SDL VSync (can reduce latency, may increase tearing).
    #[arg(long = "no-sdl-vsync")]
    no_sdl_vsync: bool,
    /// Enable dev helpers (F1 overlay, F3/F4 scene navigation, debug controls).
    ///
    /// Defaults:
    /// - debug build: enabled automatically
    /// - release build: disabled unless `--dev` is passed
    #[arg(long = "dev")]
    dev: bool,
    /// Disable dev helpers even in debug builds.
    #[arg(long = "no-dev")]
    no_dev: bool,
    /// Backward-compatible alias for `--dev`.
    #[arg(long = "debug-feature", hide = true)]
    debug_feature: bool,
    /// Enable audio playback (uses system audio device via rodio).
    #[arg(long = "audio")]
    audio: bool,
    /// Force-enable run logging (also enabled by default in debug builds).
    #[arg(long = "logs")]
    logs: bool,
    /// Force-disable run logging.
    #[arg(long = "no-logs")]
    no_logs: bool,
    /// Also print log output to stderr in real time.
    /// Can also be enabled via SHELL_ENGINE_CONSOLE_LOG=1 env var.
    #[arg(long = "console-log")]
    console_log: bool,
    /// Override run log root directory (default: ./logs).
    #[arg(long = "log-root")]
    log_root: Option<String>,
    /// Jump directly to a specific scene (overrides mod entrypoint).
    #[arg(long = "start-scene")]
    start_scene: Option<String>,
    /// Skip the engine splash screen on startup.
    #[arg(long = "skip-splash")]
    skip_splash: bool,
    /// Enable compositor optimizations (layer-scratch skip, dirty-region narrowing).
    /// Enabled by default; use --no-opt-comp to disable.
    #[arg(long = "opt-comp")]
    opt_comp: bool,
    /// Disable compositor optimizations enabled by default.
    #[arg(long = "no-opt-comp")]
    no_opt_comp: bool,
    /// Enable present optimizations (hash-based frame skip for static scenes).
    #[arg(long = "opt-present")]
    opt_present: bool,
    /// Enable dirty-region diff scan (experimental — may cause artifacts).
    #[arg(long = "opt-diff")]
    opt_diff: bool,
    /// Enable unified frame-skip coordination (PostFX cache + Presenter hash sync).
    /// Prevents desynchronization-related flickering from independent skip mechanisms.
    #[arg(long = "opt-skip")]
    opt_skip: bool,
    /// Enable row-level dirty skip in diff scan.
    /// Enabled by default; use --no-opt-rowdiff to disable.
    #[arg(long = "opt-rowdiff")]
    opt_rowdiff: bool,
    /// Disable row-level dirty skip enabled by default.
    #[arg(long = "no-opt-rowdiff")]
    no_opt_rowdiff: bool,
    /// Enable async display-related optimizations.
    #[arg(long = "opt-async")]
    opt_async_display: bool,
    /// Enable ALL optional optimizations at once.
    /// Equivalent to --opt-present --opt-diff --opt-skip --opt-async,
    /// and also re-enables --opt-comp/--opt-rowdiff if they were disabled.
    #[arg(long = "opt")]
    opt_all: bool,
    /// Run benchmark: play demo for N seconds, display score, save report to reports/benchmark/.
    #[arg(long = "bench", value_name = "SECS", default_missing_value = "5")]
    bench: Option<f32>,
    /// Capture frames to a directory for visual regression testing (one .bin file per frame).
    #[arg(long = "capture-frames", value_name = "DIR")]
    capture_frames: Option<String>,
    /// Override target FPS (e.g. 240 for uncapped benchmarks). Default: from mod manifest (60).
    #[arg(long = "target-fps", value_name = "FPS")]
    target_fps: Option<u16>,
    /// Run startup scene checks for the selected mod and exit (no game loop).
    #[arg(long = "check-scenes")]
    check_scenes: bool,
}

fn main() {
    let cli = Cli::parse();
    let debug_feature = resolve_dev_mode(&cli);
    let logs_enabled = resolve_logs_enabled(&cli);
    let opt_comp = resolve_opt_comp(&cli);
    let opt_rowdiff = resolve_opt_rowdiff(&cli);

    match logging::init_run_logger(logging::RunLoggerConfig {
        app_name: String::from("app"),
        enabled: logs_enabled,
        root_dir: cli.log_root.clone().map(PathBuf::from),
        also_stderr: cli.console_log,
    }) {
        Ok(Some(info)) => {
            println!("Logs: {}", info.file_path.display());
            logging::install_panic_hook("app.panic");
            logging::info(
                "app.main",
                format!(
                    "launcher init: run={} log_file={}",
                    info.run_number,
                    info.file_path.display()
                ),
            );
        }
        Ok(None) => {}
        Err(error) => eprintln!("Logging init failed: {error}"),
    }
    let mod_source = cli
        .mod_source
        .or_else(|| cli.mod_name.as_ref().map(|name| format!("mods/{}/", name)));

    let mod_source = match mod_source {
        Some(s) => s,
        None => {
            // No mod specified — launch the interactive TUI launcher.
            let launcher_exe = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("se")))
                .unwrap_or_else(|| PathBuf::from("se"));
            logging::info("app.main", "no --mod specified, launching interactive menu");
            let status = std::process::Command::new(&launcher_exe)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to start launcher ({launcher_exe:?}): {e}");
                    eprintln!("Hint: run `cargo build -p launcher` first, or specify --mod <name>");
                    std::process::exit(1);
                });
            std::process::exit(status.code().unwrap_or(0));
        }
    };
    logging::info("app.main", format!("resolved mod_source={mod_source}"));

    // Initialize the behavior system with the mod source so Rhai module resolution works
    init_behavior_system(&mod_source);

    let manifest = load_mod_manifest(Path::new(&mod_source)).unwrap_or_else(|error| {
        logging::error("app.main", format!("failed to read mod manifest: {error}"));
        eprintln!("Failed to read mod manifest: {error}");
        std::process::exit(1);
    });

    let requested_output = StartupOutputSetting::Sdl2;
    if cli.check_scenes {
        let entrypoint = cli
            .start_scene
            .as_deref()
            .or_else(|| manifest.get("entrypoint").and_then(|value| value.as_str()))
            .unwrap_or("");
        let check_output = output_for_scene_checks(requested_output);
        let report = run_scene_checks(Path::new(&mod_source), &manifest, entrypoint, check_output)
            .unwrap_or_else(|error| {
                logging::error("app.main", format!("scene checks failed: {error}"));
                eprintln!("Scene check error: {error}");
                std::process::exit(1);
            });
        print_scene_check_report(&report);
        logging::info("app.main", "scene checks finished");
        return;
    }

    let sdl_window_ratio = parse_ratio_arg(&cli.sdl_window_ratio).unwrap_or_else(|error| {
        logging::error("app.main", error.as_str());
        eprintln!("Invalid --sdl-window-ratio: {error}");
        std::process::exit(2);
    });
    let config = EngineConfig {
        debug_feature,
        audio: cli.audio,
        start_scene: cli.start_scene,
        skip_splash: cli.skip_splash || cli.bench.is_some(),
        opt_comp,
        opt_present: cli.opt_present || cli.opt_all,
        opt_diff: cli.opt_diff || cli.opt_all,
        opt_skip: cli.opt_skip || cli.opt_all,
        opt_rowdiff,
        opt_async_display: cli.opt_async_display || cli.opt_all,
        bench_secs: cli.bench,
        capture_frames_dir: cli.capture_frames.clone().map(std::path::PathBuf::from),
        target_fps_override: cli.target_fps,
        sdl_window_ratio,
        sdl_pixel_scale: cli.sdl_pixel_scale.max(1),
        sdl_vsync: !cli.no_sdl_vsync,
    };
    logging::debug(
        "app.main",
        format!(
            "engine config: dev={} audio={} output=sdl2",
            config.debug_feature, config.audio,
        ),
    );

    let engine = ShellEngine::new_with_config(&mod_source, config).unwrap_or_else(|error| {
        logging::error(
            "app.main",
            format!("failed to initialize ShellEngine: {error}"),
        );
        eprintln!("Failed to initialize ShellEngine: {error}");
        std::process::exit(1);
    });

    println!(
        "ShellEngine [{}] initialized with mod source: {}",
        env!("BUILD_GIT_HASH"),
        engine.mod_source().display()
    );
    logging::info(
        "app.main",
        format!(
            "engine initialized: mod_source={}",
            engine.mod_source().display()
        ),
    );

    if let Err(error) = engine.run() {
        logging::error("app.main", format!("engine run failed: {error}"));
        eprintln!("Engine error: {error}");
        std::process::exit(1);
    }
    logging::info("app.main", "engine run finished successfully");
}

fn parse_ratio_arg(raw: &str) -> Result<Option<(u32, u32)>, String> {
    let lowered = raw.trim().to_ascii_lowercase();
    if lowered == "free" || lowered == "none" || lowered == "off" {
        return Ok(None);
    }
    let (w, h) = lowered
        .split_once(':')
        .ok_or_else(|| String::from("expected WIDTH:HEIGHT or `free`"))?;
    let width = w
        .parse::<u32>()
        .map_err(|_| String::from("ratio width must be a positive integer"))?;
    let height = h
        .parse::<u32>()
        .map_err(|_| String::from("ratio height must be a positive integer"))?;
    if width == 0 || height == 0 {
        return Err(String::from("ratio width/height must be > 0"));
    }
    Ok(Some((width, height)))
}

fn resolve_dev_mode(cli: &Cli) -> bool {
    if cli.no_dev {
        return false;
    }
    if cli.dev || cli.debug_feature {
        return true;
    }

    // In debug builds, dev helpers are enabled by default.
    if cfg!(debug_assertions) {
        return true;
    }

    // In release builds, allow env opt-in without CLI flags.
    env_flag_enabled("SHELL_ENGINE_DEV") || env_flag_enabled("SHELL_ENGINE_DEBUG_FEATURE")
}

fn resolve_logs_enabled(cli: &Cli) -> bool {
    logging::resolve_enabled(cli.logs, cli.no_logs)
}

fn resolve_opt_comp(cli: &Cli) -> bool {
    !cli.no_opt_comp
}

fn resolve_opt_rowdiff(cli: &Cli) -> bool {
    !cli.no_opt_rowdiff
}

fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn output_for_scene_checks(setting: StartupOutputSetting) -> StartupOutputSetting {
    setting
}

fn run_scene_checks(
    mod_source: &Path,
    manifest: &serde_yaml::Value,
    entrypoint: &str,
    selected_output: StartupOutputSetting,
) -> Result<StartupReport, engine::EngineError> {
    use engine::asset::{create_scene_repository, SceneRepository};
    use engine::behavior::init_behavior_system;

    // Initialize behavior system with mod source for Rhai module resolution
    init_behavior_system(mod_source.to_str().expect("mod_source path is valid UTF-8"));

    let scene_loader_fn =
        |mod_root: &std::path::Path| -> Result<Vec<StartupSceneFile>, engine::EngineError> {
            let repo = create_scene_repository(mod_root)?;
            let paths = repo.discover_scene_paths()?;
            let mut scenes = Vec::with_capacity(paths.len());
            for path in paths {
                let scene = repo.load_scene(&path)?;
                scenes.push(StartupSceneFile { path, scene });
            }
            Ok(scenes)
        };
    let font_checker = |mod_src: Option<&std::path::Path>, font: &str| -> bool {
        engine::rasterizer::has_font_assets(mod_src, font)
    };
    let glyph_checker =
        |mod_src: Option<&std::path::Path>, font: &str, text: &str| -> Option<Vec<char>> {
            engine::rasterizer::missing_glyphs(mod_src, font, text)
        };
    let image_checker = |mod_src: &std::path::Path, source: &str| -> bool {
        engine::image_loader::has_image_asset(mod_src, source)
    };
    let rhai_validator =
        |script: &str, src: Option<&str>, scene: &engine::scene::Scene| -> Result<(), String> {
            engine::behavior::smoke_validate_rhai_script(script, src, scene)
        };
    let startup_ctx = StartupContext::new(mod_source, manifest, entrypoint, &scene_loader_fn)
        .with_selected_output(selected_output)
        .with_font_asset_checker(&font_checker)
        .with_glyph_coverage_checker(&glyph_checker)
        .with_image_asset_checker(&image_checker)
        .with_rhai_script_validator(&rhai_validator);

    // Scene diagnostics mode intentionally skips backend capability checks and
    // focuses on authored content consistency for all discovered scenes.
    StartupRunner::with_checks(vec![
        Box::new(SceneGraphCheck),
        Box::new(LevelConfigCheck),
        Box::new(CatalogsCheck),
        Box::new(AudioSequencerCheck),
        Box::new(RhaiScriptsCheck),
        Box::new(GuiWidgetBindingsCheck),
        Box::new(EffectRegistryCheck),
        Box::new(ImageAssetsCheck),
        Box::new(FontManifestCheck),
        Box::new(FontGlyphCoverageCheck),
    ])
    .run(&startup_ctx)
}

fn print_scene_check_report(report: &StartupReport) {
    let mut warnings = 0usize;
    let mut infos = 0usize;

    for issue in report.issues() {
        match issue.level {
            StartupIssueLevel::Warning => {
                warnings += 1;
                println!("⚠️  [{}] {}", issue.check, issue.message);
            }
            StartupIssueLevel::Info => {
                infos += 1;
                println!("ℹ️  [{}] {}", issue.check, issue.message);
            }
        }
    }

    println!();
    println!(
        "Scene checks completed: {} warning(s), {} info item(s).",
        warnings, infos
    );
}

#[cfg(test)]
mod tests {
    use super::{
        parse_ratio_arg, resolve_dev_mode, resolve_opt_comp, resolve_opt_rowdiff,
        resolve_startup_output, Cli,
    };
    use clap::Parser;
    use engine_mod::StartupOutputSetting;

    #[test]
    fn dev_flag_enables_mode() {
        let cli = Cli::parse_from(["shell-engine", "--dev"]);
        assert!(resolve_dev_mode(&cli));
    }

    #[test]
    fn no_dev_flag_disables_mode() {
        let cli = Cli::parse_from(["shell-engine", "--dev", "--no-dev"]);
        assert!(!resolve_dev_mode(&cli));
    }

    #[test]
    fn debug_feature_flag_is_compat_alias() {
        let cli = Cli::parse_from(["shell-engine", "--debug-feature"]);
        assert!(resolve_dev_mode(&cli));
    }

    #[test]
    fn scene_checks_preserve_sdl2_output() {
        assert_eq!(
            output_for_scene_checks(StartupOutputSetting::Sdl2),
            StartupOutputSetting::Sdl2
        );
    }

    #[test]
    fn parses_ratio_arg() {
        assert_eq!(parse_ratio_arg("16:9").expect("ratio"), Some((16, 9)));
        assert_eq!(parse_ratio_arg("free").expect("ratio"), None);
    }

    #[test]
    fn opt_comp_is_enabled_by_default() {
        let cli = Cli::parse_from(["shell-engine"]);
        assert!(resolve_opt_comp(&cli));
    }

    #[test]
    fn no_opt_comp_disables_default() {
        let cli = Cli::parse_from(["shell-engine", "--no-opt-comp"]);
        assert!(!resolve_opt_comp(&cli));
    }

    #[test]
    fn no_opt_comp_overrides_opt() {
        let cli = Cli::parse_from(["shell-engine", "--opt", "--no-opt-comp"]);
        assert!(!resolve_opt_comp(&cli));
    }

    #[test]
    fn opt_rowdiff_is_enabled_by_default() {
        let cli = Cli::parse_from(["shell-engine"]);
        assert!(resolve_opt_rowdiff(&cli));
    }

    #[test]
    fn no_opt_rowdiff_disables_default() {
        let cli = Cli::parse_from(["shell-engine", "--no-opt-rowdiff"]);
        assert!(!resolve_opt_rowdiff(&cli));
    }

    #[test]
    fn no_opt_rowdiff_overrides_opt() {
        let cli = Cli::parse_from(["shell-engine", "--opt", "--no-opt-rowdiff"]);
        assert!(!resolve_opt_rowdiff(&cli));
    }

    #[test]
    fn parses_check_scenes_flag() {
        let cli = Cli::parse_from(["shell-engine", "--check-scenes"]);
        assert!(cli.check_scenes);
    }
}
