use crate::cargo::CargoCommand;
use crate::cli::SchemasArgs;
use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run(workspace_root: &Path, args: &SchemasArgs) -> Result<()> {
    if args.loop_mode {
        println!("Schemas: loop mode (Ctrl+C to stop)");
        loop {
            run_once(workspace_root, args)?;
            thread::sleep(Duration::from_secs(5));
        }
    } else {
        run_once(workspace_root, args)?;
    }

    Ok(())
}

fn run_once(workspace_root: &Path, args: &SchemasArgs) -> Result<()> {
    let mut cmd = CargoCommand::new("schema-gen").app_arg("--all-mods");

    if args.check {
        cmd = cmd.app_arg("--check");
    }

    if let Some(ref mod_name) = args.mod_name {
        cmd = cmd.app_arg("--mod").app_arg(mod_name);
    }

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
