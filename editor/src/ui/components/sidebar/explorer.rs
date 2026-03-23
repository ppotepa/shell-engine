//! File explorer sidebar panel.

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use std::path::Path;

use crate::state::focus::FocusPane;
use crate::state::{AppState, TreeItem};
use crate::ui::theme;

/// Renders the project tree as a navigable list with folder and file icons.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let items: Vec<ListItem> = app
        .explorer
        .items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let (label, style) = match item {
                TreeItem::ModYaml => (String::from("📄 mod.yaml"), theme::fg_active()),
                TreeItem::ScenesFolder => (String::from("📁 scenes/"), theme::accent()),
                TreeItem::Scene(path) => {
                    let idx = app
                        .index
                        .scenes
                        .scene_paths
                        .iter()
                        .position(|candidate| candidate == path)
                        .unwrap_or(0);
                    (
                        format!("  🎬 {}", app.scene_display_name(idx)),
                        theme::fg_normal(),
                    )
                }
                TreeItem::ImagesFolder => (String::from("📁 images/"), theme::accent()),
                TreeItem::Image(path) => {
                    let name = Path::new(path)
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    (format!("  🖼️  {}", name), theme::fg_normal())
                }
                TreeItem::FontsFolder => (String::from("📁 fonts/"), theme::accent()),
                TreeItem::Font(path) => {
                    let name = Path::new(path)
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    (format!("  🔤 {}", name), theme::fg_normal())
                }
            };

            if idx == app.explorer.cursor {
                ListItem::new(format!("> {}", label)).style(theme::accent())
            } else {
                ListItem::new(format!("  {}", label)).style(style)
            }
        })
        .collect();

    let focused = app.focus == FocusPane::ProjectTree;
    let list = List::new(items)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title("Project Tree")
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused)),
        );
    frame.render_widget(list, area);
}
