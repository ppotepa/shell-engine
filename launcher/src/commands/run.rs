use crate::cargo::CargoCommand;
use crate::cli::RunArgs;
use crate::workspace;
use anyhow::Result;
use std::path::Path;

/// Resolve effective pixel scale: 0 means auto-compute from mod's render_size.
fn effective_pixel_scale(user_scale: u32, render_size_str: &str) -> u32 {
    if user_scale > 0 {
        return user_scale;
    }
    if let Some((w, h)) = workspace::parse_render_size(render_size_str) {
        workspace::auto_pixel_scale(w, h)
    } else {
        8 // fallback for "match-output" or unparseable
    }
}

pub fn run(workspace_root: &Path, args: &RunArgs) -> Result<()> {
    let mod_source = if let Some(ref source) = args.mod_source {
        source.clone()
    } else {
        format!("mods/{}/", args.mod_name)
    };

    // Resolve mod name for manifest lookup
    let mod_dir_name = args.mod_source.as_deref().unwrap_or(&args.mod_name);
    let mod_name = std::path::Path::new(mod_dir_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&args.mod_name)
        .trim_end_matches('/');

    let render_size_str = workspace::read_mod_manifest(workspace_root, mod_name)
        .map(|m| m.display.render_size)
        .unwrap_or_default();

    let pixel_scale = effective_pixel_scale(args.sdl_pixel_scale, &render_size_str);

    let profile = args
        .profile
        .as_deref()
        .or(if args.release { Some("release") } else { None });

    let mut cmd = CargoCommand::new("app");

    if let Some(p) = profile {
        cmd = cmd.profile(p);
    }

    cmd = cmd.feature("engine/sdl2");

    cmd = cmd.app_arg("--mod-source").app_arg(&mod_source);

    if let Some(ref scene) = args.start_scene {
        cmd = cmd.app_arg("--start-scene").app_arg(scene);
    }

    if args.audio {
        cmd = cmd.app_arg("--audio");
    }
    if args.dev {
        cmd = cmd.app_arg("--dev");
    }
    if args.no_dev {
        cmd = cmd.app_arg("--no-dev");
    }
    if args.skip_splash {
        cmd = cmd.app_arg("--skip-splash");
    }
    if args.logs {
        cmd = cmd.app_arg("--logs");
    }
    if args.no_logs {
        cmd = cmd.app_arg("--no-logs");
    }
    if args.console_log {
        cmd = cmd.app_arg("--console-log");
    }

    if let Some(ref dir) = args.log_root {
        cmd = cmd.app_arg("--log-root").app_arg(dir);
    }

    if let Some(fps) = args.target_fps {
        cmd = cmd.app_arg("--target-fps").app_arg(fps.to_string());
    }

    if args.check_scenes {
        cmd = cmd.app_arg("--check-scenes");
    }

    if args.opt {
        cmd = cmd.app_arg("--opt");
    } else {
        if args.opt_comp {
            cmd = cmd.app_arg("--opt-comp");
        }
        if args.no_opt_comp {
            cmd = cmd.app_arg("--no-opt-comp");
        }
        if args.opt_present {
            cmd = cmd.app_arg("--opt-present");
        }
        if args.opt_diff {
            cmd = cmd.app_arg("--opt-diff");
        }
        if args.opt_skip {
            cmd = cmd.app_arg("--opt-skip");
        }
        if args.opt_rowdiff {
            cmd = cmd.app_arg("--opt-rowdiff");
        }
        if args.no_opt_rowdiff {
            cmd = cmd.app_arg("--no-opt-rowdiff");
        }
        if args.opt_async {
            cmd = cmd.app_arg("--opt-async");
        }
    }

    cmd = cmd
        .app_arg("--sdl-window-ratio")
        .app_arg(&args.sdl_window_ratio);
    cmd = cmd
        .app_arg("--sdl-pixel-scale")
        .app_arg(pixel_scale.to_string());

    if args.no_sdl_vsync {
        cmd = cmd.app_arg("--no-sdl-vsync");
    }

    cmd = cmd.app_args(args.extra_args.iter().cloned());

    if args.with_sidecar {
        println!("Building cognitOS sidecar...");
        build_sidecar(workspace_root)?;
    }

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn build_sidecar(workspace_root: &Path) -> Result<()> {
    let sidecar_dir = workspace_root.join("mods/shell-quest/os/cognitOS");

    let status = std::process::Command::new("dotnet")
        .arg("build")
        .arg("-c")
        .arg("Release")
        .current_dir(&sidecar_dir)
        .status()?;

    if !status.success() {
        anyhow::bail!("sidecar build failed");
    }

    Ok(())
}
