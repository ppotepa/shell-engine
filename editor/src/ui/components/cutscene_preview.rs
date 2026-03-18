//! Cutscene Maker center panel with source validation and quick export context.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::AppState;
use crate::ui::theme;

/// Renders Cutscene Maker details for stop-action frame source validation.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from("🎥 Cutscene Maker"),
        Line::from(""),
        Line::from(format!("Source folder: {}", app.cutscene_source_dir)),
        Line::from(format!("Output GIF: {}", app.cutscene_output_gif)),
        Line::from(format!(
            "Default frame duration: {} ms",
            app.cutscene_default_frame_ms
        )),
        Line::from(""),
    ];

    if let Some(err) = &app.cutscene_validation_error {
        lines.push(Line::from("Validation: INVALID"));
        lines.push(Line::from(format!("Reason: {err}")));
    } else {
        lines.push(Line::from("Validation: OK"));
        lines.push(Line::from(format!(
            "Detected chronological frames: {}",
            app.cutscene_frames.len()
        )));
    }

    if !app.cutscene_missing_frames.is_empty() {
        let missing = app
            .cutscene_missing_frames
            .iter()
            .take(14)
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if app.cutscene_missing_frames.len() > 14 {
            ", ..."
        } else {
            ""
        };
        lines.push(Line::from(format!("Missing: {missing}{suffix}")));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("First frames:"));
    if app.cutscene_frames.is_empty() {
        lines.push(Line::from("-"));
    } else {
        for name in app.cutscene_frames.iter().take(12) {
            lines.push(Line::from(format!("  - {name}")));
        }
        if app.cutscene_frames.len() > 12 {
            lines.push(Line::from("  - ..."));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Controls:"));
    lines.push(Line::from("  - Press F5 to rescan assets/raw"));
    lines.push(Line::from("  - Use panel 3 to preview scene integration"));

    let paragraph = Paragraph::new(lines)
        .style(theme::pane_background(false))
        .block(
            Block::default()
                .title("Cutscene Preview")
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL)
                .style(theme::pane_background(false)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
