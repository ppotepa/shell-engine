//! Fullscreen scene-run view used when launching a scene with F5 from Scene Browser.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::domain::preview_renderer;
use crate::state::{AppState, SceneRunKind};
use crate::ui::theme;

/// Renders the active scene-run buffer plus context metadata.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let lines = if let Some(buffer) = app.scene_run_buffer() {
        preview_renderer::buffer_to_lines(buffer)
    } else {
        vec![
            Line::from("Scene Run buffer unavailable."),
            Line::from(""),
            Line::from("Press Esc to return to editor."),
        ]
    };

    let title = match app.scene_run.kind {
        SceneRunKind::Soft => " SOFT RUN ",
        SceneRunKind::Hard => " RUN ",
    };

    let widget = Paragraph::new(lines)
        .style(theme::preview_background())
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(ratatui::style::Color::Green))
                .border_style(Style::default().fg(ratatui::style::Color::Green))
                .borders(Borders::ALL)
                .style(theme::preview_background()),
        );

    frame.render_widget(widget, area);
}
