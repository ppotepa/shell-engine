mod cargo;
mod cli;
mod commands;
mod config;
mod env;
mod menu;
mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    // Load platform env hints early (used by backend-specific launch paths).
    env::detect_platform_env();

    let cli = Cli::parse();

    let workspace_root = workspace::find_workspace_root()?;

    match cli.command {
        None => menu::run_menu(&workspace_root),
        Some(Command::Run(args)) => commands::run::run(&workspace_root, &args),
        Some(Command::Bench(args)) => commands::bench::run(&workspace_root, &args),
        Some(Command::Capture(args)) => commands::capture::run(&workspace_root, &args),
        Some(Command::Schemas(args)) => commands::schemas::run(&workspace_root, &args),
        Some(Command::Setup(args)) => commands::setup::run(&workspace_root, &args),
        Some(Command::Doctor) => commands::doctor::run(&workspace_root),
        Some(Command::Editor(args)) => commands::editor::run(&workspace_root, &args),
        Some(Command::Devtool { args }) => commands::devtool::run(&workspace_root, &args),
    }
}
