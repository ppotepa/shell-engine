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
        .tree_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let (label, style) = match item {
                TreeItem::ModYaml => ("📄 mod.yaml".to_string(), theme::fg_active()),
                TreeItem::ScenesFolder => ("📁 scenes/".to_string(), theme::accent()),
                TreeItem::Scene(path) => {
                    let name = Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    (format!("  🎬 {}", name), theme::fg_normal())
                }
                TreeItem::ImagesFolder => ("📁 images/".to_string(), theme::accent()),
                TreeItem::Image(path) => {
                    let name = Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    (format!("  🖼️  {}", name), theme::fg_normal())
                }
                TreeItem::FontsFolder => ("📁 fonts/".to_string(), theme::accent()),
                TreeItem::Font(path) => {
                    let name = Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    (format!("  🔤 {}", name), theme::fg_normal())
                }
            };

            if idx == app.tree_cursor {
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
                .border_style(theme::pane_border(focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused)),
        );
    frame.render_widget(list, area);
}
