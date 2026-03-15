use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};

use crate::scene::{Layer, Scene};

pub fn render_cutscene(scene: &Scene) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let bg = scene
        .bg_colour
        .as_ref()
        .map(Color::from)
        .unwrap_or(Color::Black);

    let result = run_scene_loop(&mut terminal, scene, bg);

    // always restore terminal regardless of errors
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_scene_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    scene: &Scene,
    bg: Color,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // Fill entire buffer with bg_colour — every cell gets the background colour.
            let backdrop = Block::default().style(Style::default().bg(bg));
            frame.render_widget(backdrop, area);

            // Render each layer onto the buffer.
            for layer in &scene.layers {
                render_layer(frame, layer, bg);
            }
        })?;

        if scene.skippable && key_pressed()? {
            break;
        }
    }
    Ok(())
}

fn render_layer(frame: &mut ratatui::Frame, layer: &Layer, scene_bg: Color) {
    match layer {
        Layer::Text {
            content,
            x,
            y,
            fg_colour,
            bg_colour,
        } => {
            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(scene_bg);
            let style = Style::default().fg(fg).bg(bg);

            let area = frame.area();
            let width = (content.len() as u16).min(area.width.saturating_sub(*x));
            let height = 1u16;

            if *x < area.width && *y < area.height && width > 0 {
                let rect = Rect::new(*x, *y, width, height);
                let span = Span::styled(content.as_str(), style);
                let paragraph = Paragraph::new(Line::from(span));
                frame.render_widget(paragraph, rect);
            }
        }
    }
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
