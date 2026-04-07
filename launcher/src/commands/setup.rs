use anyhow::Result;
use crate::cli::SetupArgs;
use crate::env;
use crate::cargo::ensure_sdl2_dll;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &SetupArgs) -> Result<()> {
    let platform = env::detect_platform_env();

    if !platform.is_windows {
        println!("Setup: nothing to do on Linux/macOS (install SDL2 via package manager)");
        return Ok(());
    }

    if args.check {
        println!("Setup check (Windows):");
        match (&platform.sdl2_lib_dir, &platform.rustflags) {
            (Some(dir), Some(_)) => {
                let dll = dir.join("SDL2.dll");
                if dll.exists() {
                    println!("  ✓ SDL2_LIB_DIR = {}", dir.display());
                    println!("  ✓ SDL2.dll found at {}", dll.display());

                    // Check if DLL is already in target dirs
                    for profile in &["debug", "release"] {
                        let dst = workspace_root.join("target").join(profile).join("SDL2.dll");
                        if dst.exists() {
                            println!("  ✓ SDL2.dll already in target/{}", profile);
                        } else {
                            println!("  ○ SDL2.dll not yet in target/{} (will be copied on next run)", profile);
                        }
                    }
                } else {
                    println!("  ✗ SDL2_LIB_DIR set but SDL2.dll not found at {}", dll.display());
                }
            }
            _ => {
                println!("  ✗ SDL2 not configured");
                print_sdl2_instructions();
            }
        }
        return Ok(());
    }

    // Proactively copy SDL2.dll into both target dirs
    match &platform.sdl2_lib_dir {
        Some(dir) => {
            let dll_src = dir.join("SDL2.dll");
            if dll_src.exists() {
                for profile in &["debug", "release"] {
                    let target_dir = workspace_root.join("target").join(profile);
                    if target_dir.exists() {
                        let dst = target_dir.join("SDL2.dll");
                        if !dst.exists() {
                            match std::fs::copy(&dll_src, &dst) {
                                Ok(_) => println!("  ✓ Copied SDL2.dll → target/{}/", profile),
                                Err(e) => println!("  ✗ Could not copy to target/{}: {}", profile, e),
                            }
                        } else {
                            println!("  ✓ SDL2.dll already present in target/{}/", profile);
                        }
                    }
                }

                // Also ensure it runs for current profile
                ensure_sdl2_dll(workspace_root, None);
                println!("\n  SDL2 is ready. Run with:  se run <mod> --sdl2");
            } else {
                println!("  ✗ SDL2_LIB_DIR is set but SDL2.dll not found.");
                print_sdl2_instructions();
            }
        }
        None => {
            println!("  SDL2_LIB_DIR is not set.");
            print_sdl2_instructions();
        }
    }

    Ok(())
}

fn print_sdl2_instructions() {
    println!();
    println!("  To install SDL2 for Windows (MSVC):");
    println!("    1. Download SDL2-devel-*-VC.zip from https://github.com/libsdl-org/SDL/releases/tag/release-2.30.12");
    println!("    2. Extract to %USERPROFILE%\\SDL2\\SDL2-2.30.12\\");
    println!("    3. Set environment variables:");
    println!("         $env:SDL2_LIB_DIR = \"$env:USERPROFILE\\SDL2\\SDL2-2.30.12\\lib\\x64\"");
    println!("         $env:RUSTFLAGS   = \"-L native=$env:SDL2_LIB_DIR\"");
    println!("    4. Re-run: se setup");
}