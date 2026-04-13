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
use crate::workspace;
use state::Selection;

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

    let mut cmd = CargoCommand::new("app");

    if flags.release {
        cmd = cmd.profile("release");
    }
    cmd = cmd.feature("engine/sdl2");

    cmd = cmd.app_arg("--mod-source").app_arg(&selection.mod_dir);

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

    // Auto-compute pixel scale from mod's render_size
    let pixel_scale = if let Some((w, h)) = workspace::parse_render_size(&selection.render_size) {
        workspace::auto_pixel_scale(w, h)
    } else {
        8
    };
    cmd = cmd.app_arg("--sdl-window-ratio").app_arg("16:9");
    cmd = cmd
        .app_arg("--sdl-pixel-scale")
        .app_arg(pixel_scale.to_string());

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        eprintln!(
            "\n\x1b[33m[se] process exited with code {}\x1b[0m",
            status.code().unwrap_or(1)
        );
    }

    Ok(())
}
