use clap::Parser;
use engine::{EngineConfig, ShellEngine};

#[derive(Parser, Debug)]
#[command(name = "shell-quest", about = "Shell Quest terminal engine launcher")]
struct Cli {
    /// Mod source path (directory or .zip).
    #[arg(long)]
    mod_source: Option<String>,
    /// Force renderer mode globally: cell | halfblock | quadblock | braille.
    #[arg(long = "renderer-mode")]
    renderer_mode: Option<String>,
    /// Enable generic debug helpers (F1 overlay, F3/F4 scene navigation).
    #[arg(long = "debug-feature")]
    debug_feature: bool,
    /// Enable external sound server integration (audio commands over stdin/stdout JSONL).
    #[arg(long = "sound-server")]
    sound_server: bool,
    /// Override shell command used to spawn the sound server process.
    #[arg(long = "sound-server-cmd")]
    sound_server_cmd: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let mod_source = cli
        .mod_source
        .unwrap_or_else(|| "mods/shell-quest/".to_string());

    let config = EngineConfig {
        renderer_mode: cli.renderer_mode,
        debug_feature: cli.debug_feature,
        sound_server: cli.sound_server,
        sound_server_cmd: cli.sound_server_cmd,
    };

    let engine = ShellEngine::new_with_config(&mod_source, config).unwrap_or_else(|error| {
        eprintln!("Failed to initialize ShellEngine: {error}");
        std::process::exit(1);
    });

    println!(
        "ShellEngine initialized with mod source: {}",
        engine.mod_source().display()
    );

    if let Err(error) = engine.run() {
        eprintln!("Engine error: {error}");
        std::process::exit(1);
    }
}
