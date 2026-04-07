use anyhow::Result;
use crate::{workspace, env};
use std::path::Path;

pub fn run(workspace_root: &Path) -> Result<()> {
    println!("Shell Engine Doctor\n");
    
    let platform = env::detect_platform_env();
    
    check_command("cargo", &["--version"]);
    check_command("rustc", &["--version"]);
    check_command("dotnet", &["--version"]);
    
    if platform.is_windows {
        println!("\nSDL2 (Windows):");
        if let Some(ref lib_dir) = platform.sdl2_lib_dir {
            if lib_dir.exists() {
                println!("  ✓ SDL2_LIB_DIR = {}", lib_dir.display());
            } else {
                println!("  ✗ SDL2_LIB_DIR set but path doesn't exist: {}", lib_dir.display());
            }
        } else {
            println!("  ✗ SDL2_LIB_DIR not set");
        }
        
        if let Some(ref flags) = platform.rustflags {
            println!("  ✓ RUSTFLAGS = {}", flags);
        } else {
            println!("  ✗ RUSTFLAGS not set (needed for SDL2 linking)");
        }
    } else {
        if which::which("sdl2-config").is_ok() {
            println!("\n  ✓ SDL2 available (sdl2-config found)");
        } else {
            println!("\n  ✗ SDL2 not found (sdl2-config missing)");
        }
    }
    
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