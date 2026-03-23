use clap::Parser;
use engine::{logging, EngineConfig, ShellEngine};
use std::path::PathBuf;

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
    /// Enable external sound server integration (audio commands over stdin/stdout JSONL).
    #[arg(long = "sound-server")]
    sound_server: bool,
    /// Override shell command used to spawn the sound server process.
    #[arg(long = "sound-server-cmd")]
    sound_server_cmd: Option<String>,
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
}

fn main() {
    let cli = Cli::parse();
    let debug_feature = resolve_dev_mode(&cli);
    let logs_enabled = resolve_logs_enabled(&cli);

    match logging::init_run_logger(logging::RunLoggerConfig {
        app_name: "app".to_string(),
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

    let config = EngineConfig {
        renderer_mode: cli.renderer_mode,
        debug_feature,
        sound_server: cli.sound_server,
        sound_server_cmd: cli.sound_server_cmd,
        start_scene: cli.start_scene,
        skip_splash: cli.skip_splash,
    };
    logging::debug(
        "app.main",
        format!(
            "engine config: dev={} sound_server={} sound_server_cmd={}",
            config.debug_feature,
            config.sound_server,
            config.sound_server_cmd.as_deref().unwrap_or("<default>")
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

#[cfg(test)]
mod tests {
    use super::{resolve_dev_mode, Cli};
    use clap::Parser;

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
}
