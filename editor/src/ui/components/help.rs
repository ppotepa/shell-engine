//! Toggleable contextual help popup for the active screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders the current-screen help popup.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(72, 56, area);
    let lines: Vec<Line> = app.current_help().into_iter().map(Line::from).collect();

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .style(theme::pane_background(true))
            .block(
                Block::default()
                    .title(format!("{} Help", app.current_screen_name()))
                    .title_style(theme::fg_active())
                    .border_style(theme::pane_border(app.mode, true))
                    .borders(Borders::ALL)
                    .style(theme::pane_background(true))
                    .title_bottom(Span::styled(" F1 closes help ", theme::fg_disabled())),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
