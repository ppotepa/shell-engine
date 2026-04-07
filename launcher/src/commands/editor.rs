use anyhow::Result;
use crate::cargo::CargoCommand;
use crate::cli::EditorArgs;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &EditorArgs) -> Result<()> {
    let mod_source = format!("mods/{}/", args.mod_name);
    
    let status = CargoCommand::new("editor")
        .app_arg("--mod-source")
        .app_arg(&mod_source)
        .app_args(args.extra_args.iter().cloned())
        .exec(workspace_root)?;
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Ok(())
}