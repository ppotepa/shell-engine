use crate::cargo::CargoCommand;
use crate::cli::CaptureArgs;
use anyhow::Result;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &CaptureArgs) -> Result<()> {
    let mod_name = if args.tests {
        "asteroids"
    } else {
        &args.mod_name
    };

    let mod_source = format!("mods/{}/", mod_name);

    let baseline_dir = args.baseline.as_deref().unwrap_or("frames/baseline");
    let optimized_dir = args.optimized.as_deref().unwrap_or("frames/optimized");

    println!("Frame capture:");
    println!("  Mod: {}", mod_name);
    println!("  Frames: {}", args.frames);
    println!("  Baseline: {}", baseline_dir);
    println!("  Optimized: {}", optimized_dir);
    println!();

    println!("Capturing baseline frames...");
    capture_frames(
        workspace_root,
        &mod_source,
        baseline_dir,
        args.frames,
        false,
    )?;

    println!("Capturing optimized frames...");
    capture_frames(
        workspace_root,
        &mod_source,
        optimized_dir,
        args.frames,
        true,
    )?;

    println!("\nFrame capture complete!");
    println!("Run frame regression tests with:");
    println!("  cargo test -p engine --test frame_regression");

    Ok(())
}

fn capture_frames(
    workspace_root: &Path,
    mod_source: &str,
    output_dir: &str,
    count: u32,
    optimized: bool,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    let mut cmd = CargoCommand::new("app")
        .feature("app/software-backend")
        .app_arg("--mod-source")
        .app_arg(mod_source)
        .app_arg("--capture-frames")
        .app_arg(output_dir)
        .app_arg("--skip-splash");

    if optimized {
        cmd = cmd.app_arg("--opt");
    }

    std::env::set_var("SHELL_ENGINE_CAPTURE_FRAMES", count.to_string());

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        anyhow::bail!("frame capture failed");
    }

    Ok(())
}
