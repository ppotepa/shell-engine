use clap::Parser;
use engine::ShellEngine;

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
}

fn main() {
    let cli = Cli::parse();
    let mod_source = cli
        .mod_source
        .unwrap_or_else(|| "mods/shell-quest/".to_string());

    if let Some(mode) = cli.renderer_mode {
        std::env::set_var("SHELL_QUEST_RENDERER_MODE", mode);
    }
    if cli.debug_feature {
        std::env::set_var("SHELL_QUEST_DEBUG_FEATURE", "1");
    }

    let engine = ShellEngine::new(&mod_source).unwrap_or_else(|error| {
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
