use crate::cargo::CargoCommand;
use anyhow::Result;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &[String]) -> Result<()> {
    let status = CargoCommand::new("devtool")
        .app_args(args.iter().cloned())
        .exec(workspace_root)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
