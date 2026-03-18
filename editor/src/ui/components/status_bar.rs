//! Status bar component: renders the one-line contextual hint at the bottom of the screen.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{focus::FocusPane, AppMode, AppState, SidebarItem};
use crate::ui::theme;

/// Renders the status bar with the current mode, context, and key-hint sections.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    // Split into 3 sections: LEFT (mode), CENTER (context), RIGHT (hints)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // LEFT: mode
            Constraint::Min(1),     // CENTER: context
            Constraint::Length(40), // RIGHT: hints
        ])
        .split(area);

    // LEFT: Mode display
    let mode_text = match app.mode {
        AppMode::Start => " START  ",
        AppMode::Browser => " NORMAL ",
        AppMode::EditMode => " EDIT   ",
    };
    let mode_para = Paragraph::new(Span::styled(mode_text, theme::accent()));
    frame.render_widget(mode_para, chunks[0]);

    // CENTER: Context info
    let context_text = if app.mode == AppMode::EditMode {
        format!(" editing: {}", app.editing_file.as_deref().unwrap_or(""))
    } else if app.sidebar_active == SidebarItem::Search {
        let selected_param = app
            .selected_effect_param_spec()
            .map(|spec| spec.label)
            .unwrap_or("none");
        format!(
            " effects: {} [{}] param: {}",
            app.selected_builtin_effect().unwrap_or("none"),
            if app.effects_live_preview {
                "live"
            } else {
                "static"
            },
            if app.focus == FocusPane::Inspector {
                selected_param
            } else {
                "-"
            },
        )
    } else if app.sidebar_active == SidebarItem::Scenes {
        let scene_name = app
            .selected_scene_display_name()
            .unwrap_or_else(|| "none".to_string());
        let visible_layers = app
            .scene_layer_visibility
            .iter()
            .filter(|enabled| **enabled)
            .count();
        format!(
            " scenes: {} | visible: {}/{} | layer: {}",
            scene_name,
            visible_layers,
            app.scene_preview_layers.len(),
            if app.focus == FocusPane::Browser {
                app.selected_scene_layer().unwrap_or("-")
            } else {
                "-"
            },
        )
    } else if !app.mod_source.is_empty() {
        format!(
            " {}",
            app.mod_source.split('/').last().unwrap_or(&app.mod_source)
        )
    } else {
        " No project".to_string()
    };
    let context_para = Paragraph::new(Span::styled(context_text, theme::fg_normal()));
    frame.render_widget(context_para, chunks[1]);

    // RIGHT: Hints
    let hints = match app.mode {
        AppMode::Start => "j/k: move | Enter: select | q: quit",
        AppMode::Browser if app.sidebar_active == SidebarItem::Search => {
            "Tab: focus | j/k: move | ←/→: adjust | F: live"
        }
        AppMode::Browser if app.sidebar_active == SidebarItem::Scenes => {
            "Tab: focus | j/k: scenes/layers | Space:toggle | Enter:solo | F/Ctrl+F fullscreen"
        }
        AppMode::Browser => "1-4: panels | T: sidebar | q: quit",
        AppMode::EditMode if app.sidebar_active == SidebarItem::Search => {
            "ESC: exit | F: live | T: sidebar | Ctrl+Q"
        }
        AppMode::EditMode => "ESC: exit | T: sidebar | Ctrl+Q: quit",
    };
    let hints_para = Paragraph::new(Span::styled(hints, theme::fg_disabled()));
    frame.render_widget(hints_para, chunks[2]);
}
