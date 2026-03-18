//! Sidebar icon rail: the narrow column of numbered panel-switch glyphs.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{AppState, SidebarItem};
use crate::ui::theme;

/// Renders the sidebar icon rail with numbered glyphs for switching between panels.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let entries = [
        (SidebarItem::Explorer, "1", "\u{1f4c1}"), // 📁
        (SidebarItem::Search, "2", "\u{1f50d}"),   // 🔍
        (SidebarItem::Scenes, "3", "\u{1f3ac}"),   // 🎬
        (SidebarItem::Cutscene, "4", "\u{1f4f7}"), // 📷
    ];

    let mut lines = Vec::new();

    for (item, key, glyph) in entries.into_iter() {
        let is_active = item == app.sidebar_active;

        let style = if is_active {
            theme::sidebar_active_entry()
        } else {
            theme::fg_disabled()
        };

        lines.push(Line::from(Span::styled(
            format!(" {} {}", key, glyph),
            style,
        )));
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme::pane_background(false))
        .block(
            Block::default()
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::RIGHT)
                .style(theme::pane_background(false)),
        );

    frame.render_widget(paragraph, area);
}
