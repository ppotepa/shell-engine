use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cli::Cli;
use crate::input::keys::map_key_event;
use crate::io::recent::{load_recent, save_recent};
use crate::state::AppState;
use crate::ui;

pub fn run(cli: Cli) -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let recent = load_recent();
    let mut app = AppState::new(cli.mod_source, recent);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(16))? {
            let ev = event::read()?;
            if let Event::Key(key) = ev {
                let cmd = map_key_event(key, app.mode);
                if app.apply_command(cmd) {
                    break;
                }
            }
        }

        // Update transition animation
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32();
        last_tick = now;
        app.update_transition(dt);
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    save_recent(&app.recent_projects);
    Ok(())
}
