pub mod scanner;
pub mod state;
pub mod render;
pub mod input;

use anyhow::Result;
use std::path::Path;
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    cursor::{Hide, Show},
};
use std::io::{stdout, Write};

use crate::config::{self, LauncherConfig};
use scanner::scan_menu_entries;
use state::{MenuState, MenuAction};
use render::render_menu;
use input::wait_for_input;

pub fn run_menu(workspace_root: &Path) -> Result<()> {
    let mut config = config::load_config(workspace_root)?;

    let mods = scan_menu_entries(workspace_root)?;
    if mods.is_empty() {
        anyhow::bail!("no mods found in mods/ directory");
    }

    let mut state = MenuState::new(mods, config.flags.clone());

    terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Hide)?;

    // Clear screen once on entry
    print!("\x1b[2J\x1b[H");
    stdout().flush()?;

    let result = menu_loop(workspace_root, &mut state, &mut config);

    execute!(stdout(), LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn menu_loop(workspace_root: &Path, state: &mut MenuState, config: &mut LauncherConfig) -> Result<()> {
    // Initial render
    render_menu(state)?;

    loop {
        match wait_for_input(state)? {
            MenuAction::None => {
                // No redraw needed
            }
            MenuAction::Redraw => {
                render_menu(state)?;
            }
            MenuAction::Launch => {
                if let Some(selection) = state.get_selection() {
                    execute!(stdout(), LeaveAlternateScreen, Show)?;
                    terminal::disable_raw_mode()?;

                    launch_selection(workspace_root, &selection, &state.flags)?;

                    println!("\n\x1b[2m[se] press any key to return...\x1b[0m");
                    stdout().flush()?;

                    terminal::enable_raw_mode()?;
                    // Drain any buffered events from the just-exited game
                    loop {
                        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                            let _ = crossterm::event::read()?;
                        } else {
                            break;
                        }
                    }
                    // Wait for actual keypress
                    loop {
                        use crossterm::event::KeyEventKind;
                        if let crossterm::event::Event::Key(k) = crossterm::event::read()? {
                            if k.kind == KeyEventKind::Press {
                                break;
                            }
                        }
                    }
                    terminal::disable_raw_mode()?;

                    terminal::enable_raw_mode()?;
                    execute!(stdout(), EnterAlternateScreen, Hide)?;
                    print!("\x1b[2J\x1b[H");
                    stdout().flush()?;
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
use crate::workspace;
use state::Selection;
use crate::config::LaunchFlags;

fn launch_selection(workspace_root: &Path, selection: &Selection, flags: &LaunchFlags) -> Result<()> {
    println!("\x1b[1;36m[se]\x1b[0m launching \x1b[1m{}\x1b[0m", selection.mod_name);
    if let Some(ref scene) = selection.scene_path {
        println!("\x1b[2m     scene: {}\x1b[0m", scene);
    }
    println!();

    let mut cmd = CargoCommand::new("app");

    if flags.release {
        cmd = cmd.profile("release");
    }
    if flags.sdl2 {
        cmd = cmd.feature("sdl2");
        cmd = cmd.app_arg("--sdl2");
    }

    cmd = cmd.app_arg("--mod-source").app_arg(&selection.mod_dir);

    if let Some(ref scene) = selection.scene_path {
        cmd = cmd.app_arg("--start-scene").app_arg(scene);
    }

    if flags.audio { cmd = cmd.app_arg("--audio"); }
    if flags.dev { cmd = cmd.app_arg("--dev"); }
    if flags.skip_splash { cmd = cmd.app_arg("--skip-splash"); }
    if flags.check_scenes { cmd = cmd.app_arg("--check-scenes"); }
    if flags.all_opt { cmd = cmd.app_arg("--opt"); }

    // Auto-compute pixel scale from mod's render_size
    let pixel_scale = if let Some((w, h)) = workspace::parse_render_size(&selection.render_size) {
        workspace::auto_pixel_scale(w, h)
    } else {
        8
    };
    cmd = cmd.app_arg("--sdl-window-ratio").app_arg("16:9");
    cmd = cmd.app_arg("--sdl-pixel-scale").app_arg(&pixel_scale.to_string());

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        eprintln!("\n\x1b[33m[se] process exited with code {}\x1b[0m", status.code().unwrap_or(1));
    }

    Ok(())
}