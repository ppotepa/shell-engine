mod app;
mod cli;
mod domain;
mod input;
mod io;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Initialize theme before running app
    ui::theme::init_theme();

    let cli = cli::Cli::parse();
    app::run(cli)
}
