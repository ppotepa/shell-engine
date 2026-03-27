use clap::Parser;
use engine::{logging, BackendKind, EngineConfig, ShellEngine};
use engine_mod::{load_mod_manifest, StartupOutputSetting};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "shell-quest", about = "Shell Quest terminal engine launcher")]
struct Cli {
    /// Mod to load by name (resolves to mods/<name>/). Default: shell-quest.
    #[arg(long = "mod", default_value = "shell-quest")]
    mod_name: String,
    /// Full mod source path (directory or .zip). Overrides --mod when set.
    #[arg(long = "mod-source", hide = true)]
    mod_source: Option<String>,
    /// Force renderer mode globally: cell | halfblock | quadblock | braille.
    #[arg(long = "renderer-mode")]
    renderer_mode: Option<String>,
    /// Force output backend: terminal, sdl2, or prompt. Overrides mod.yaml when set.
    #[arg(long = "output")]
    output: Option<String>,
    /// Shorthand for --output sdl2.
    #[arg(long = "sdl2")]
    sdl2: bool,
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
    /// Override run log root directory (default: ./logs).
    #[arg(long = "log-root")]
    log_root: Option<String>,
    /// Jump directly to a specific scene (overrides mod entrypoint).
    #[arg(long = "start-scene")]
    start_scene: Option<String>,
    /// Skip the engine splash screen on startup.
    #[arg(long = "skip-splash")]
    skip_splash: bool,
    /// Enable compositor optimizations (layer-scratch skip, dirty-halfblock narrowing).
    #[arg(long = "opt-comp")]
    opt_comp: bool,
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
    /// Enable row-level dirty skip in diff scan (experimental).
    /// Skips entire rows marked not dirty, ~10-20% faster on static regions.
    #[arg(long = "opt-rowdiff")]
    opt_rowdiff: bool,
    /// Enable async display sink: offload terminal I/O to background thread.
    /// Decouples main thread from terminal write/flush latency (1-5ms/frame).
    #[arg(long = "opt-async")]
    opt_async_display: bool,
    /// Enable ALL optimizations at once (equivalent to --opt-comp --opt-present --opt-diff --opt-skip --opt-rowdiff --opt-async).
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
}

fn main() {
    let cli = Cli::parse();
    let debug_feature = resolve_dev_mode(&cli);
    let logs_enabled = resolve_logs_enabled(&cli);

    match logging::init_run_logger(logging::RunLoggerConfig {
        app_name: String::from("app"),
        enabled: logs_enabled,
        root_dir: cli.log_root.clone().map(PathBuf::from),
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
        .unwrap_or_else(|| format!("mods/{}/", cli.mod_name));
    logging::info("app.main", format!("resolved mod_source={mod_source}"));

    let manifest = load_mod_manifest(Path::new(&mod_source)).unwrap_or_else(|error| {
        logging::error("app.main", format!("failed to read mod manifest: {error}"));
        eprintln!("Failed to read mod manifest: {error}");
        std::process::exit(1);
    });

    let manifest_output = StartupOutputSetting::from_manifest(&manifest).unwrap_or_else(|error| {
        logging::error("app.main", error.as_str());
        eprintln!("Failed to resolve startup output: {error}");
        std::process::exit(1);
    });
    let effective_output = if cli.sdl2 {
        Some("sdl2".to_string())
    } else {
        cli.output
    };
    let requested_output = resolve_startup_output(effective_output.as_deref(), manifest_output)
        .unwrap_or_else(|error| {
            logging::error("app.main", error.as_str());
            eprintln!("Failed to resolve startup output: {error}");
            std::process::exit(1);
        });
    let output_backend = resolve_backend_kind(requested_output).unwrap_or_else(|error| {
        logging::error("app.main", error.as_str());
        eprintln!("Failed to select output backend: {error}");
        std::process::exit(1);
    });

    let sdl_window_ratio = parse_ratio_arg(&cli.sdl_window_ratio).unwrap_or_else(|error| {
        logging::error("app.main", error.as_str());
        eprintln!("Invalid --sdl-window-ratio: {error}");
        std::process::exit(2);
    });

    let config = EngineConfig {
        output_backend,
        renderer_mode: cli.renderer_mode,
        debug_feature,
        audio: cli.audio,
        start_scene: cli.start_scene,
        skip_splash: cli.skip_splash || cli.bench.is_some(),
        opt_comp: cli.opt_comp || cli.opt_all,
        opt_present: cli.opt_present || cli.opt_all,
        opt_diff: cli.opt_diff || cli.opt_all,
        opt_skip: cli.opt_skip || cli.opt_all,
        opt_rowdiff: cli.opt_rowdiff || cli.opt_all,
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
            "engine config: dev={} audio={} output={:?}",
            config.debug_feature, config.audio, config.output_backend,
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
        "ShellEngine initialized with mod source: {}",
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
    env_flag_enabled("SHELL_QUEST_DEV") || env_flag_enabled("SHELL_QUEST_DEBUG_FEATURE")
}

fn resolve_logs_enabled(cli: &Cli) -> bool {
    logging::resolve_enabled(cli.logs, cli.no_logs)
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

fn resolve_startup_output(
    cli_output: Option<&str>,
    manifest_output: Option<StartupOutputSetting>,
) -> Result<StartupOutputSetting, String> {
    if let Some(cli_output) = cli_output {
        return StartupOutputSetting::parse(cli_output).ok_or_else(|| {
            format!("invalid --output `{cli_output}`; expected terminal, sdl2, or prompt")
        });
    }

    Ok(manifest_output.unwrap_or(StartupOutputSetting::Terminal))
}

fn resolve_backend_kind(setting: StartupOutputSetting) -> Result<BackendKind, String> {
    match setting {
        StartupOutputSetting::Terminal => Ok(BackendKind::Terminal),
        StartupOutputSetting::Sdl2 => Ok(BackendKind::Sdl2),
        StartupOutputSetting::Prompt => prompt_for_backend(),
    }
}

fn prompt_for_backend() -> Result<BackendKind, String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    prompt_for_backend_with_io(&mut reader, &mut writer)
}

fn prompt_for_backend_with_io<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<BackendKind, String> {
    writeln!(writer, "Choose output backend:").map_err(|error| error.to_string())?;
    writeln!(writer, "  1) Terminal").map_err(|error| error.to_string())?;
    writeln!(writer, "  2) SDL2 window").map_err(|error| error.to_string())?;
    writeln!(writer, "     (requires an SDL2-enabled build)").map_err(|error| error.to_string())?;

    loop {
        write!(writer, "> ").map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;

        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err(String::from("no output backend selected"));
        }

        match line.trim().to_ascii_lowercase().as_str() {
            "1" | "terminal" | "tty" => return Ok(BackendKind::Terminal),
            "2" | "sdl2" | "window" => return Ok(BackendKind::Sdl2),
            _ => {
                writeln!(writer, "Please enter 1/terminal or 2/sdl2.")
                    .map_err(|error| error.to_string())?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cli, parse_ratio_arg, prompt_for_backend_with_io, resolve_dev_mode, resolve_startup_output,
    };
    use clap::Parser;
    use engine::BackendKind;
    use engine_mod::StartupOutputSetting;

    #[test]
    fn dev_flag_enables_mode() {
        let cli = Cli::parse_from(["shell-quest", "--dev"]);
        assert!(resolve_dev_mode(&cli));
    }

    #[test]
    fn no_dev_flag_disables_mode() {
        let cli = Cli::parse_from(["shell-quest", "--dev", "--no-dev"]);
        assert!(!resolve_dev_mode(&cli));
    }

    #[test]
    fn debug_feature_flag_is_compat_alias() {
        let cli = Cli::parse_from(["shell-quest", "--debug-feature"]);
        assert!(resolve_dev_mode(&cli));
    }

    #[test]
    fn manifest_output_is_used_when_cli_output_missing() {
        assert_eq!(
            resolve_startup_output(None, Some(StartupOutputSetting::Prompt))
                .expect("startup output"),
            StartupOutputSetting::Prompt
        );
    }

    #[test]
    fn cli_output_overrides_manifest_output() {
        assert_eq!(
            resolve_startup_output(Some("terminal"), Some(StartupOutputSetting::Prompt))
                .expect("startup output"),
            StartupOutputSetting::Terminal
        );
    }

    #[test]
    fn invalid_cli_output_is_rejected() {
        let error =
            resolve_startup_output(Some("fancy"), Some(StartupOutputSetting::Prompt)).unwrap_err();
        assert!(error.contains("invalid --output"));
    }

    #[test]
    fn prompt_accepts_terminal_choice() {
        let mut input = std::io::Cursor::new(b"1\n".as_slice());
        let mut output = Vec::new();
        let selected =
            prompt_for_backend_with_io(&mut input, &mut output).expect("prompt selection");
        assert_eq!(selected, BackendKind::Terminal);
    }

    #[test]
    fn prompt_retries_until_valid_choice() {
        let mut input = std::io::Cursor::new(b"bad\nsdl2\n".as_slice());
        let mut output = Vec::new();
        let selected =
            prompt_for_backend_with_io(&mut input, &mut output).expect("prompt selection");
        assert_eq!(selected, BackendKind::Sdl2);
        let output = String::from_utf8(output).expect("utf8 output");
        assert!(output.contains("Please enter 1/terminal or 2/sdl2."));
    }

    #[test]
    fn parses_ratio_arg() {
        assert_eq!(parse_ratio_arg("16:9").expect("ratio"), Some((16, 9)));
        assert_eq!(parse_ratio_arg("free").expect("ratio"), None);
    }
}
