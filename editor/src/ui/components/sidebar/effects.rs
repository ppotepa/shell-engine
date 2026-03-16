use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::domain::effects_preview_scene::choose_preview_placement;
use crate::domain::effects_preview_scene::PreviewPlacement;
use crate::state::{focus::FocusPane, AppState};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let items: Vec<ListItem> = app
        .builtin_effects
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let is_active = idx == app.effect_cursor;
            let is_focused = app.focus == FocusPane::ProjectTree;

            let placement = choose_preview_placement(name);
            let is_transition = placement == PreviewPlacement::Scene;

            let (badge_char, badge_style) = if is_transition {
                ("T", theme::badge_transition())
            } else {
                ("E", theme::badge_effect())
            };

            let cursor = if is_active { ">" } else { " " };
            let name_style = if is_active && is_focused {
                theme::sidebar_active_entry()
            } else if is_active {
                theme::accent()
            } else {
                theme::fg_normal()
            };

            let line = Line::from(vec![
                Span::styled(format!("{cursor} "), name_style),
                Span::styled(badge_char, badge_style),
                Span::styled(format!(" {name}"), name_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let focused = app.focus == FocusPane::ProjectTree;
    let list = List::new(items)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title("Effects Browser")
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused)),
        );

    frame.render_widget(list, area);
}
