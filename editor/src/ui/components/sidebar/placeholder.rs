//! Placeholder sidebar panel for panels not yet implemented.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders a placeholder panel displaying a title and informational text lines.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState, title: &str, lines: &[&str]) {
    let mut content = Vec::new();
    for line in lines {
        content.push(Line::from((*line).to_string()));
    }

    let panel = Paragraph::new(content)
        .style(theme::pane_background(false))
        .block(
            Block::default()
                .title(title)
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL)
                .style(theme::pane_background(false)),
        );

    frame.render_widget(panel, area);
}
