use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "se", about = "Shell Engine launcher", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Build and launch a mod
    Run(RunArgs),
    /// Run benchmark suite
    Bench(BenchArgs),
    /// Capture frames for visual regression testing
    Capture(CaptureArgs),
    /// Regenerate schema fragments
    Schemas(SchemasArgs),
    /// Platform setup helpers (backend-aware)
    Setup(SetupArgs),
    /// Verify toolchain & environment
    Doctor,
    /// Launch the TUI editor
    Editor(EditorArgs),
    /// Run devtool
    Devtool {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Compatibility-only legacy flag: request software backend (unavailable)
    #[arg(long = "software", hide = true, conflicts_with = "hardware")]
    pub software: bool,

    /// Compatibility-only legacy flag: force hardware backend
    #[arg(long = "hardware", hide = true, conflicts_with = "software")]
    pub hardware: bool,

    /// Compatibility-only legacy flag: render backend selector
    #[arg(
        long = "render-backend",
        hide = true,
        value_enum,
        default_value_t = RenderBackendArg::Hardware
    )]
    pub render_backend: RenderBackendArg,

    /// Mod name (resolves to mods/<name>/)
    #[arg(short = 'm', long = "mod", default_value = "asteroids")]
    pub mod_name: String,

    /// Explicit mod source path (overrides --mod)
    #[arg(long = "mod-source")]
    pub mod_source: Option<String>,

    /// Jump to specific scene
    #[arg(short = 's', long = "scene")]
    pub start_scene: Option<String>,

    /// Use release profile
    #[arg(short = 'r', long = "release")]
    pub release: bool,

    /// Cargo profile override (dev, fast-release, release)
    #[arg(long = "profile")]
    pub profile: Option<String>,

    /// Enable audio playback
    #[arg(long = "audio")]
    pub audio: bool,

    /// Enable dev helpers
    #[arg(long = "dev")]
    pub dev: bool,

    /// Disable dev helpers
    #[arg(long = "no-dev")]
    pub no_dev: bool,

    /// Skip engine splash
    #[arg(long = "skip-splash")]
    pub skip_splash: bool,

    /// Force-enable logging
    #[arg(long = "logs")]
    pub logs: bool,

    /// Force-disable logging
    #[arg(long = "no-logs")]
    pub no_logs: bool,

    /// Console log to stderr
    #[arg(long = "console-log")]
    pub console_log: bool,

    /// Override log directory
    #[arg(long = "log-root")]
    pub log_root: Option<String>,

    /// Override target FPS
    #[arg(long = "target-fps")]
    pub target_fps: Option<u16>,

    /// Validate scenes and exit
    #[arg(long = "check-scenes")]
    pub check_scenes: bool,

    /// Enable all optimizations
    #[arg(long = "opt")]
    pub opt: bool,

    /// Compositor optimizations
    #[arg(long = "opt-comp")]
    pub opt_comp: bool,

    /// Disable compositor optimizations
    #[arg(long = "no-opt-comp")]
    pub no_opt_comp: bool,

    /// Static frame skip
    #[arg(long = "opt-present")]
    pub opt_present: bool,

    /// Dirty-region diff scan
    #[arg(long = "opt-diff")]
    pub opt_diff: bool,

    /// Unified frame-skip coordination
    #[arg(long = "opt-skip")]
    pub opt_skip: bool,

    /// Row-level dirty skip
    #[arg(long = "opt-rowdiff")]
    pub opt_rowdiff: bool,

    /// Disable row-level dirty skip
    #[arg(long = "no-opt-rowdiff")]
    pub no_opt_rowdiff: bool,

    /// Async display sink
    #[arg(long = "opt-async")]
    pub opt_async: bool,

    /// Build cognitOS C# sidecar first
    #[arg(long = "with-sidecar")]
    pub with_sidecar: bool,

    /// Extra args passed to app binary
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

impl RunArgs {
    pub fn selected_render_backend(&self) -> RenderBackendArg {
        if self.hardware {
            RenderBackendArg::Hardware
        } else if self.software {
            RenderBackendArg::Software
        } else {
            self.render_backend
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum RenderBackendArg {
    Software,
    Hardware,
}

impl RenderBackendArg {
    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::Software => "software",
            Self::Hardware => "hardware",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, RenderBackendArg};
    use clap::Parser;

    #[test]
    fn run_defaults_to_hardware_backend() {
        let cli = Cli::parse_from(["se", "run"]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };
        assert_eq!(args.selected_render_backend(), RenderBackendArg::Hardware);
    }

    #[test]
    fn run_render_backend_flag_selects_hardware() {
        let cli = Cli::parse_from(["se", "run", "--render-backend", "hardware"]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };
        assert_eq!(args.selected_render_backend(), RenderBackendArg::Hardware);
    }

    #[test]
    fn run_hardware_shortcut_selects_hardware() {
        let cli = Cli::parse_from(["se", "run", "--hardware"]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };
        assert_eq!(args.selected_render_backend(), RenderBackendArg::Hardware);
    }

    #[test]
    fn run_software_shortcut_selects_software() {
        let cli = Cli::parse_from(["se", "run", "--software"]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };
        assert_eq!(args.selected_render_backend(), RenderBackendArg::Software);
    }
}

#[derive(Parser, Debug)]
pub struct BenchArgs {
    /// Scenario: quick, standard, extended
    pub scenario: Option<String>,

    /// Mod to benchmark
    #[arg(short = 'm', long = "mod", default_value = "asteroids")]
    pub mod_name: String,

    /// Single flag combo (e.g. "opt")
    #[arg(long = "combo")]
    pub combo: Option<String>,

    /// Duration per combo in seconds
    #[arg(long = "duration")]
    pub duration: Option<f32>,

    /// Output CSV path
    #[arg(long = "csv")]
    pub csv: Option<String>,
}

#[derive(Parser, Debug)]
pub struct CaptureArgs {
    /// Baseline capture directory
    #[arg(long = "baseline")]
    pub baseline: Option<String>,

    /// Optimized capture directory
    #[arg(long = "optimized")]
    pub optimized: Option<String>,

    /// Number of frames to capture
    #[arg(long = "frames", default_value_t = 5)]
    pub frames: u32,

    /// Use the default capture mod
    #[arg(long = "tests")]
    pub tests: bool,

    /// Mod to capture
    #[arg(short = 'm', long = "mod", default_value = "asteroids")]
    pub mod_name: String,
}

#[derive(Parser, Debug)]
pub struct SchemasArgs {
    /// Continuously refresh every 5s
    #[arg(long = "loop")]
    pub loop_mode: bool,

    /// Verify schemas without writing
    #[arg(long = "check")]
    pub check: bool,

    /// Single mod (default: all mods)
    #[arg(long = "mod")]
    pub mod_name: Option<String>,
}

#[derive(Parser, Debug)]
pub struct SetupArgs {
    /// Verify only, don't install
    #[arg(long = "check")]
    pub check: bool,
}

#[derive(Parser, Debug)]
pub struct EditorArgs {
    /// Mod to edit
    #[arg(short = 'm', long = "mod", default_value = "playground")]
    pub mod_name: String,

    /// Extra args passed to editor binary
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}
