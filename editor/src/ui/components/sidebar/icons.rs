use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{AppState, SidebarItem};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let entries = [
        (SidebarItem::Explorer, "1", "\u{1f4c1}"),        // 📁
        (SidebarItem::Search, "2", "\u{1f50d}"),          // 🔍
        (SidebarItem::Git, "3", "\u{1f33f}"),             // 🌿
        (SidebarItem::Settings, "4", "\u{2699}\u{fe0f}"), // ⚙️
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
                .border_style(theme::fg_disabled())
                .borders(Borders::RIGHT)
                .style(theme::pane_background(false)),
        );

    frame.render_widget(paragraph, area);
}
