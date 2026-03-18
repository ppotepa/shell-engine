//! Single-line header showing the current screen name and help state.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders the header row with the current mode and screen name.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(1),
            Constraint::Length(14),
        ])
        .split(area);

    let mode = format!(" {} ", app.current_mode_label());
    let screen = format!(" {}", app.current_screen_name());
    let help = if app.help_overlay_active {
        " F1 help ON "
    } else {
        " F1 help OFF"
    };

    frame.render_widget(
        Paragraph::new(Span::styled(mode, theme::mode_badge(app.mode))),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(screen, theme::fg_active())),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(help, theme::fg_disabled())),
        chunks[2],
    );
}
