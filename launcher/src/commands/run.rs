use crate::cargo::CargoCommand;
use crate::cli::{RenderBackendArg, RunArgs};
use anyhow::Result;
use std::path::Path;

const SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE: &str =
    "[se] software backend is deprecated and unavailable because SDL2 runtime support was removed. Use hardware backend (default) or pass --hardware.";

pub fn run(workspace_root: &Path, args: &RunArgs) -> Result<()> {
    let selected_backend = args.selected_render_backend();
    ensure_backend_supported(selected_backend)?;

    let cmd = build_run_command(args);

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

fn build_run_command(args: &RunArgs) -> CargoCommand {
    let mod_source = if let Some(ref source) = args.mod_source {
        source.clone()
    } else {
        format!("mods/{}/", args.mod_name)
    };

    let profile = args
        .profile
        .as_deref()
        .or(if args.release { Some("release") } else { None });

    let mut cmd = CargoCommand::new("app");
    let selected_backend = args.selected_render_backend();
    debug_assert!(matches!(selected_backend, RenderBackendArg::Hardware));

    if let Some(p) = profile {
        cmd = cmd.profile(p);
    }

    cmd = cmd.no_default_features().feature("app/hardware-backend");

    cmd = cmd.app_arg("--mod-source").app_arg(&mod_source);
    cmd = cmd
        .app_arg("--render-backend")
        .app_arg(selected_backend.as_cli_value());

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

    if args.no_sdl_vsync {
        println!(
            "[se] note: --no-sdl-vsync only applies to software backend; ignoring for hardware."
        );
    }

    cmd.app_args(args.extra_args.iter().cloned())
}

fn ensure_backend_supported(selected_backend: RenderBackendArg) -> Result<()> {
    if matches!(selected_backend, RenderBackendArg::Software) {
        anyhow::bail!(SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE);
    }
    Ok(())
}

fn build_sidecar(workspace_root: &Path) -> Result<()> {
    let sidecar_dir = workspace_root.join("mods/asteroids/os/cognitOS");

    if !sidecar_dir.exists() {
        anyhow::bail!(
            "sidecar source not present in this workspace: {}",
            sidecar_dir.display()
        );
    }

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

#[cfg(test)]
mod tests {
    use super::{build_run_command, ensure_backend_supported, SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE};
    use crate::cli::{Cli, Command};
    use clap::Parser;

    #[test]
    fn hardware_run_omits_software_presenter_args() {
        let cli = Cli::parse_from(["se", "run", "--hardware", "--mod", "playground"]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };

        let built = build_run_command(&args).build_args();
        assert!(built
            .windows(2)
            .any(|pair| { pair[0] == "--features" && pair[1].contains("app/hardware-backend") }));
        assert!(!built
            .windows(2)
            .any(|pair| pair[0] == "--sdl-window-ratio" || pair[0] == "--sdl-pixel-scale"));
    }

    #[test]
    fn software_run_is_rejected() {
        let cli = Cli::parse_from([
            "se",
            "run",
            "--software",
            "--mod",
            "playground",
            "--sdl-pixel-scale",
            "5",
        ]);
        let Command::Run(args) = cli.command.expect("run command") else {
            panic!("expected run command");
        };

        let error = ensure_backend_supported(args.selected_render_backend()).expect_err("backend error");
        assert_eq!(error.to_string(), SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE);
    }
}
