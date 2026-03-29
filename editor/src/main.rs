mod app;
mod cli;
mod domain;
mod input;
mod io;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;
use engine_core::logging;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    init_logging(&cli);

    // Initialize theme before running app
    ui::theme::init_theme();
    logging::info(
        "editor.main",
        format!("editor startup: mod_source={}", cli.mod_source),
    );
    app::run(cli)
}

fn init_logging(cli: &cli::Cli) {
    let enabled = logging::resolve_enabled(cli.logs, cli.no_logs);
    match logging::init_run_logger(logging::RunLoggerConfig {
        app_name: String::from("editor"),
        enabled,
        root_dir: cli.log_root.as_ref().map(PathBuf::from),
        also_stderr: false,
    }) {
        Ok(Some(info)) => {
            println!("Logs: {}", info.file_path.display());
            logging::install_panic_hook("editor.panic");
            logging::info(
                "editor.main",
                format!(
                    "run logger initialized: run={} file={}",
                    info.run_number,
                    info.file_path.display()
                ),
            );
        }
        Ok(None) => {}
        Err(error) => eprintln!("Logging init failed: {error}"),
    }
}
