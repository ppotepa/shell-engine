use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "sq-editor",
    about = "Shell Quest terminal editor/browser (MVP)"
)]
pub struct Cli {
    /// Path to mod root directory
    #[arg(long, default_value = "mods/shell-quest")]
    pub mod_source: String,
}
