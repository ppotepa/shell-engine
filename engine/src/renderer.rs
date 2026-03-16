use std::io;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, ResetColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::scene::Scene;

pub fn render_cutscene(scene: &Scene) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let _bg = scene
        .bg_colour
        .as_ref()
        .map(Color::from)
        .unwrap_or(Color::Black);

    let result = run_scene_loop(scene);

    disable_raw_mode()?;
    execute!(stdout, ResetColor, cursor::Show, LeaveAlternateScreen)?;

    result
}

fn run_scene_loop(scene: &Scene) -> io::Result<()> {
    loop {
        if scene.cutscene && key_pressed()? {
            break;
        }
    }
    Ok(())
}

/// Returns `true` when any key is pressed (non-blocking check).
fn key_pressed() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            return Ok(matches!(
                key.code,
                KeyCode::Enter
                    | KeyCode::Esc
                    | KeyCode::Char(_)
                    | KeyCode::F(_)
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
            ));
        }
    }
    Ok(false)
}
