use crate::workspace;
use anyhow::Result;
use std::path::Path;

pub fn run(workspace_root: &Path) -> Result<()> {
    println!("Shell Engine Doctor\n");
    println!("Launcher diagnostics:");
    println!("  - launch path: cargo workspace packages");
    println!("  - environment assumptions: backend-neutral\n");

    check_command("cargo", &["--version"]);
    check_command("rustc", &["--version"]);
    check_command("dotnet", &["--version"]);

    println!("\nMods:");
    match workspace::scan_mods(workspace_root) {
        Ok(mods) => {
            println!("  ✓ {} mod(s) found", mods.len());
            for m in &mods {
                let scene_count = m.scenes.len();
                println!("    - {} ({} scenes)", m.manifest.name, scene_count);
            }
        }
        Err(e) => {
            println!("  ✗ Failed to scan mods: {}", e);
        }
    }

    println!("\nWorkspace build:");
    check_command("cargo", &["check", "--workspace", "--quiet"]);

    Ok(())
}

fn check_command(cmd: &str, args: &[&str]) {
    match which::which(cmd) {
        Ok(path) => {
            print!("  ✓ {} ", cmd);

            if let Ok(output) = std::process::Command::new(&path).args(args).output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    let first_line = version.lines().next().unwrap_or("");
                    println!("{}", first_line);
                } else {
                    println!("(found but command failed)");
                }
            } else {
                println!("(found at {})", path.display());
            }
        }
        Err(_) => {
            println!("  ✗ {} (not found)", cmd);
        }
    }
}
