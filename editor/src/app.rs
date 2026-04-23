//! Editor entry point — full editor UI is not yet implemented.
//! The terminal TUI has been removed; this binary currently runs as a stub.

use crate::cli::Cli;
use anyhow::Result;

pub fn run(_cli: Cli) -> Result<()> {
    eprintln!("The editor UI is not yet implemented.");
    eprintln!("To author content, edit YAML files directly and use the engine's");
    eprintln!("  --check-scenes flag to validate your scenes.");
    Ok(())
}
