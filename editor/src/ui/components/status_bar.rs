//! Status bar component: renders the current mode, status text, and screen shortcuts.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders the status bar with the current mode, context, and key-hint sections.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    // Split into 3 sections: LEFT (mode), CENTER (context), RIGHT (hints)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // LEFT: mode
            Constraint::Min(1),     // CENTER: context
            Constraint::Length(68), // RIGHT: hints
        ])
        .split(area);

    // LEFT: Mode display
    let mode_para = Paragraph::new(Span::styled(
        format!(" {} ", app.current_mode_label()),
        theme::mode_badge(app.mode),
    ));
    frame.render_widget(mode_para, chunks[0]);

    // CENTER: Status text
    let context_text = if app.status.trim().is_empty() {
        app.current_screen_name()
    } else {
        app.status.clone()
    };
    let context_para = Paragraph::new(Span::styled(context_text, theme::fg_normal()));
    frame.render_widget(context_para, chunks[1]);

    // RIGHT: Hints
    let hints_para = Paragraph::new(Span::styled(app.current_shortcuts(), theme::fg_disabled()));
    frame.render_widget(hints_para, chunks[2]);
}
