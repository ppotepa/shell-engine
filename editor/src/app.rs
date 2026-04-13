//! Editor entry point — SDL2 editor is not yet implemented.
//! The terminal TUI has been removed; an SDL2-based editor is planned.

use anyhow::Result;
use crate::cli::Cli;

pub fn run(_cli: Cli) -> Result<()> {
    eprintln!("The SDL2 editor is not yet implemented.");
    eprintln!("To author content, edit YAML files directly and use the engine's");
    eprintln!("  --check-scenes flag to validate your scenes.");
    Ok(())
}