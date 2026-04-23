use crate::cli::SetupArgs;
use anyhow::Result;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &SetupArgs) -> Result<()> {
    if args.check {
        let _ = workspace_root;
        println!("Setup check:");
        println!("  ✓ Launcher has no required setup steps.");
        println!("  ✓ Runtime dependencies are resolved by Cargo/workspace configuration.");
        println!("  ✓ No extra backend-specific environment variables are required by launcher.");
        return Ok(());
    }

    let _ = workspace_root;
    println!("Setup:");
    println!("  ✓ Launcher has no required setup steps.");
    println!("  ✓ Runtime dependencies are resolved by Cargo/workspace configuration.");
    println!("  ✓ No extra backend-specific environment variables are required by launcher.");

    Ok(())
}
