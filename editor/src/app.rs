//! Application entry point: terminal setup, main event loop, and teardown.

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, Event, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use engine_core::logging;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cli::Cli;
use crate::input::keys::map_key_event;
use crate::io::recent::{load_recent, save_recent};
use crate::state::{AppMode, AppState};
use crate::ui;

/// Initialises the terminal, runs the editor event loop, and restores the terminal on exit.
pub fn run(cli: Cli) -> Result<()> {
    logging::info(
        "editor.app",
        format!("starting editor loop: launch_mod_source={}", cli.mod_source),
    );
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let keyboard_flags = KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
    let keyboard_flags_pushed = stdout
        .execute(PushKeyboardEnhancementFlags(keyboard_flags))
        .is_ok();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let recent = load_recent();
    let mut app = AppState::new(cli.mod_source, recent);
    logging::debug(
        "editor.app",
        format!("recent projects loaded: {}", app.recent_projects.len()),
    );
    let mut last_tick = std::time::Instant::now();

    loop {
        if app.mode == AppMode::SceneRun {
            let size = terminal.size()?;
            app.ensure_scene_run_buffer_size(
                size.width.saturating_sub(2),
                size.height.saturating_sub(2),
            );
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

        let mut should_quit = false;
        if event::poll(Duration::from_millis(16))? {
            loop {
                let ev = event::read()?;
                match ev {
                    Event::Key(key) => {
                        if app.mode == AppMode::SceneRun {
                            if key.kind == KeyEventKind::Release {
                                // ignore release in run mode
                            } else if key.code == crossterm::event::KeyCode::Esc {
                                let _ = app.apply_command(crate::input::commands::Command::Back);
                            } else if matches!(key.code, crossterm::event::KeyCode::Char('q'))
                                && key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::CONTROL)
                            {
                                should_quit = true;
                            } else {
                                app.enqueue_scene_run_key(key);
                            }
                        } else {
                            let cmd = map_key_event(key, app.mode);
                            if app.apply_command(cmd) {
                                should_quit = true;
                            }
                        }
                    }
                    Event::Resize(w, h) => {
                        if app.mode == AppMode::SceneRun {
                            app.ensure_scene_run_buffer_size(
                                w.saturating_sub(2),
                                h.saturating_sub(2),
                            );
                            app.enqueue_scene_run_resize(w, h);
                        }
                    }
                    _ => {}
                }

                if should_quit || !event::poll(Duration::from_millis(0))? {
                    break;
                }
            }
        }
        if should_quit {
            logging::info("editor.app", "quit requested from event loop");
            break;
        }

        // Update transition animation
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32();
        last_tick = now;
        app.update_transition(dt);
    }

    disable_raw_mode()?;
    if keyboard_flags_pushed {
        let _ = terminal.backend_mut().execute(PopKeyboardEnhancementFlags);
    }
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    save_recent(&app.recent_projects);
    logging::info(
        "editor.app",
        format!(
            "editor loop stopped; recent projects saved: {}",
            app.recent_projects.len()
        ),
    );
    Ok(())
}
