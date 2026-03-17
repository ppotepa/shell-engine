//! Text file editor pane component.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders the editor pane with line numbers and the current file content.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let title = app
        .editing_file
        .as_ref()
        .map(|f| format!("Editing: {}", f))
        .unwrap_or_else(|| "Editor".to_string());

    let lines: Vec<Line> = app
        .edit_content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            Line::from(vec![
                Span::styled(format!("{:>4} ", i + 1), theme::fg_disabled()),
                Span::styled(line.to_string(), theme::fg_normal()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(theme::pane_background(false))
        .block(
            Block::default()
                .title(title)
                .title_style(theme::accent())
                .border_style(theme::accent())
                .borders(Borders::ALL)
                .style(theme::pane_background(false)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
