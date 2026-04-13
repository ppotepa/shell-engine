use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "sq-editor",
    about = "Shell Quest scene editor/browser (MVP)"
)]
pub struct Cli {
    /// Path to mod root directory
    #[arg(long, default_value = "mods/shell-quest")]
    pub mod_source: String,
    /// Force-enable run logging (also enabled by default in debug builds).
    #[arg(long = "logs")]
    pub logs: bool,
    /// Force-disable run logging.
    #[arg(long = "no-logs")]
    pub no_logs: bool,
    /// Override run log root directory (default: ./logs).
    #[arg(long = "log-root")]
    pub log_root: Option<String>,
}
