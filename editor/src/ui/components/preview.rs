//! Content preview pane: renders a contextual view of the selected project tree item.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fs;
use std::path::Path;

use crate::state::{AppState, TreeItem};
use crate::ui::theme;

/// Renders the preview pane with contextual content for the selected project tree item.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let content = match app.selected_tree_item() {
        Some(TreeItem::ModYaml) => render_mod_yaml(app),
        Some(TreeItem::Scene(path)) => render_scene(app, path),
        Some(TreeItem::Image(path)) => render_image(app, path),
        Some(TreeItem::Font(path)) => render_font(app, path),
        Some(TreeItem::ScenesFolder) => vec![
            Line::from("📁 Scenes Folder"),
            Line::from(""),
            Line::from(format!(
                "{} scene(s) in project",
                app.index.scenes.scene_paths.len()
            )),
        ],
        Some(TreeItem::ImagesFolder) => vec![
            Line::from("📁 Images Folder"),
            Line::from(""),
            Line::from(format!("{} image(s) in project", app.index.images.len())),
        ],
        Some(TreeItem::FontsFolder) => vec![
            Line::from("📁 Fonts Folder"),
            Line::from(""),
            Line::from(format!("{} font(s) in project", app.index.fonts.len())),
        ],
        None => vec![
            Line::from("No item selected"),
            Line::from(""),
            Line::from("Use j/k to navigate the project tree"),
        ],
    };

    let paragraph = Paragraph::new(content)
        .style(theme::pane_background(false))
        .block(
            Block::default()
                .title("Content Preview")
                .title_style(theme::fg_active())
                .border_style(theme::fg_normal())
                .borders(Borders::ALL)
                .style(theme::pane_background(false)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_mod_yaml(app: &AppState) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("📄 mod.yaml"), Line::from("")];

    if let Some(manifest) = &app.index.manifest {
        lines.push(Line::from(format!(
            "Name: {}",
            manifest.name.as_deref().unwrap_or("N/A")
        )));
        lines.push(Line::from(format!(
            "Version: {}",
            manifest.version.as_deref().unwrap_or("N/A")
        )));
        lines.push(Line::from(format!(
            "Entrypoint: {}",
            manifest.entrypoint.as_deref().unwrap_or("N/A")
        )));
    } else {
        lines.push(Line::from("Failed to load manifest"));
    }

    lines
}

fn render_scene(app: &AppState, scene_path: &str) -> Vec<Line<'static>> {
    let full_path = Path::new(&app.mod_source).join(scene_path);
    let scene_display_name = app
        .index
        .scenes
        .scene_paths
        .iter()
        .position(|candidate| candidate == scene_path)
        .map(|idx| app.scene_display_name(idx))
        .unwrap_or_else(|| {
            Path::new(scene_path)
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or(scene_path)
                .to_string()
        });
    let mut lines = vec![
        Line::from(format!("🎬 {}", scene_display_name)),
        Line::from(""),
    ];

    match fs::read_to_string(&full_path) {
        Ok(content) => {
            lines.push(Line::from("--- Scene YAML ---"));
            lines.push(Line::from(""));
            for line in content.lines().take(30) {
                lines.push(Line::from(line.to_string()));
            }
            if content.lines().count() > 30 {
                lines.push(Line::from(""));
                lines.push(Line::from("... (truncated)"));
            }
        }
        Err(e) => {
            lines.push(Line::from(format!("Error reading file: {}", e)));
        }
    }

    lines
}

fn render_image(app: &AppState, image_path: &str) -> Vec<Line<'static>> {
    let full_path = Path::new(&app.mod_source).join(image_path);
    let mut lines = vec![
        Line::from(format!(
            "🖼️  {}",
            Path::new(image_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(image_path)
        )),
        Line::from(""),
    ];

    match fs::metadata(&full_path) {
        Ok(metadata) => {
            lines.push(Line::from(format!("Size: {} bytes", metadata.len())));
            lines.push(Line::from(format!("Path: {}", image_path)));
        }
        Err(e) => {
            lines.push(Line::from(format!("Error: {}", e)));
        }
    }

    lines
}

fn render_font(app: &AppState, font_path: &str) -> Vec<Line<'static>> {
    let full_path = Path::new(&app.mod_source).join(font_path);
    let mut lines = vec![
        Line::from(format!(
            "🔤 {}",
            Path::new(font_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(font_path)
        )),
        Line::from(""),
    ];

    match fs::metadata(&full_path) {
        Ok(metadata) => {
            lines.push(Line::from(format!("Size: {} bytes", metadata.len())));
            lines.push(Line::from(format!("Path: {}", font_path)));
        }
        Err(e) => {
            lines.push(Line::from(format!("Error: {}", e)));
        }
    }

    lines
}
