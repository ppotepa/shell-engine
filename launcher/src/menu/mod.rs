pub mod input;
pub mod render;
pub mod scanner;
pub mod state;

use anyhow::Result;
use std::io::{stdout, Write};
use std::path::Path;

use crate::config::{self, LauncherConfig};
use input::wait_for_input;
use render::render_menu;
use scanner::scan_menu_entries;
use state::{MenuAction, MenuState};

pub fn run_menu(workspace_root: &Path) -> Result<()> {
    let mut config = config::load_config(workspace_root)?;

    let mods = scan_menu_entries(workspace_root)?;
    if mods.is_empty() {
        anyhow::bail!("no mods found in mods/ directory");
    }

    let mut state = MenuState::new(mods, config.flags.clone());

    let result = menu_loop(workspace_root, &mut state, &mut config);
    result
}

fn menu_loop(
    workspace_root: &Path,
    state: &mut MenuState,
    config: &mut LauncherConfig,
) -> Result<()> {
    // Initial render
    render_menu(state)?;

    loop {
        match wait_for_input(state)? {
            MenuAction::None => {}
            MenuAction::Redraw => {
                render_menu(state)?;
            }
            MenuAction::Launch => {
                if let Some(selection) = state.get_selection() {
                    launch_selection(workspace_root, &selection, &state.flags)?;

                    println!("\n\x1b[2m[se] press Enter to return to menu...\x1b[0m");
                    stdout().flush()?;
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf)?;

                    render_menu(state)?;
                }
            }
            MenuAction::Quit => break,
            MenuAction::FlagsChanged => {
                config.flags = state.flags.clone();
                config::save_config(workspace_root, config)?;
                render_menu(state)?;
            }
        }
    }

    Ok(())
}

use crate::cargo::CargoCommand;
use crate::config::LaunchFlags;
use crate::config::RenderBackendSetting;
use state::Selection;

const SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE: &str =
    "[se] software backend is deprecated and unavailable because SDL2 runtime support was removed. Switch backend to hardware.";

fn launch_selection(
    workspace_root: &Path,
    selection: &Selection,
    flags: &LaunchFlags,
) -> Result<()> {
    println!(
        "\x1b[1;36m[se]\x1b[0m launching \x1b[1m{}\x1b[0m",
        selection.mod_name
    );
    if let Some(ref scene) = selection.scene_path {
        println!("\x1b[2m     scene: {}\x1b[0m", scene);
    }
    println!();

    let cmd = build_launch_command(selection, flags)?;

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        eprintln!(
            "\n\x1b[33m[se] process exited with code {}\x1b[0m",
            status.code().unwrap_or(1)
        );
    }

    Ok(())
}

fn build_launch_command(selection: &Selection, flags: &LaunchFlags) -> Result<CargoCommand> {
    ensure_backend_supported(flags.render_backend)?;

    let mut cmd = CargoCommand::new("app");

    if flags.release {
        cmd = cmd.profile("release");
    }
    match flags.render_backend {
        crate::config::RenderBackendSetting::Software => {
            unreachable!("software backend must be rejected before launch command is built");
        }
        crate::config::RenderBackendSetting::Hardware => {
            cmd = cmd.no_default_features().feature("app/hardware-backend");
        }
    }

    cmd = cmd.app_arg("--mod-source").app_arg(&selection.mod_dir);
    cmd = cmd
        .app_arg("--render-backend")
        .app_arg(flags.render_backend.as_cli_value());

    if let Some(ref scene) = selection.scene_path {
        cmd = cmd.app_arg("--start-scene").app_arg(scene);
    }

    if flags.audio {
        cmd = cmd.app_arg("--audio");
    }
    if flags.dev {
        cmd = cmd.app_arg("--dev");
    }
    if flags.skip_splash {
        cmd = cmd.app_arg("--skip-splash");
    }
    if flags.check_scenes {
        cmd = cmd.app_arg("--check-scenes");
    }
    if flags.all_opt {
        cmd = cmd.app_arg("--opt");
    }

    Ok(cmd)
}

fn ensure_backend_supported(render_backend: RenderBackendSetting) -> Result<()> {
    if matches!(render_backend, RenderBackendSetting::Software) {
        anyhow::bail!(SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_launch_command, SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE, Selection};
    use crate::config::{LaunchFlags, RenderBackendSetting};

    fn selection_fixture() -> Selection {
        Selection {
            mod_name: "qwack-3d".to_string(),
            mod_dir: "mods/qwack-3d".to_string(),
            scene_path: Some("/scenes/main/scene.yml".to_string()),
        }
    }

    #[test]
    fn hardware_launch_uses_hardware_feature_and_disables_defaults() {
        let mut flags = LaunchFlags::default();
        flags.render_backend = RenderBackendSetting::Hardware;
        let args = build_launch_command(&selection_fixture(), &flags)
            .expect("hardware command")
            .build_args();

        assert!(args.iter().any(|arg| arg == "--no-default-features"));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--features" && pair[1].contains("app/hardware-backend") }));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--render-backend" && pair[1] == "hardware" }));
    }

    #[test]
    fn software_launch_is_rejected() {
        let mut flags = LaunchFlags::default();
        flags.render_backend = RenderBackendSetting::Software;
        match build_launch_command(&selection_fixture(), &flags) {
            Ok(_) => panic!("expected backend error"),
            Err(error) => assert_eq!(error.to_string(), SOFTWARE_BACKEND_UNAVAILABLE_MESSAGE),
        }
    }
}
