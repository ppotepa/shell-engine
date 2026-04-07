//! Scene browser center panel: scene list, layer list, and live preview.

use std::path::PathBuf;

use engine::scene::{Scene, SceneRenderedMode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::domain::preview_renderer::{self, PreviewRenderRequest};
use crate::state::{focus::FocusPane, AppState};
use crate::ui::theme;

/// Renders the scenes browser view with scene list, layer list, and live preview.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let h_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(17), Constraint::Percentage(83)])
        .split(area);

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(h_split[0]);

    render_scenes_list(
        frame,
        left_split[0],
        app,
        app.focus == FocusPane::ProjectTree,
    );
    render_layers_list(frame, left_split[1], app, app.focus == FocusPane::Browser);
    render_live_preview(
        frame,
        h_split[1],
        app,
        app.focus == FocusPane::Inspector,
        false,
    );
}

/// Renders fullscreen live scene preview (used by F-hold / Ctrl+F modes).
pub fn render_fullscreen(frame: &mut Frame, area: Rect, app: &AppState) {
    render_live_preview(frame, area, app, true, true);
}

fn render_scenes_list(frame: &mut Frame, area: Rect, app: &AppState, focused: bool) {
    let items: Vec<ListItem> = if app.index.scenes.scene_paths.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No scenes discovered in this mod",
            theme::fg_disabled(),
        )))]
    } else {
        app.index
            .scenes
            .scene_paths
            .iter()
            .enumerate()
            .map(|(idx, _path)| {
                let label = app.scene_display_name(idx);
                let is_selected = idx == app.scenes.scene_cursor;
                let style = if is_selected && focused {
                    theme::sidebar_active_entry()
                } else if is_selected {
                    theme::accent()
                } else {
                    theme::fg_normal()
                };
                let prefix = if is_selected { ">" } else { " " };
                ListItem::new(Line::from(Span::styled(format!("{prefix} {label}"), style)))
            })
            .collect()
    };

    let list = List::new(items)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title("Scenes")
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused))
                .title_bottom(Span::styled(
                    " j/k move scenes | F5 soft-run | F6 run | Tab pane ",
                    theme::fg_disabled(),
                )),
        );
    frame.render_widget(list, area);
}

fn render_layers_list(frame: &mut Frame, area: Rect, app: &AppState, focused: bool) {
    let items: Vec<ListItem> = if app.scenes.scene_preview_layers.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No layers in selected scene",
            theme::fg_disabled(),
        )))]
    } else {
        app.scenes
            .scene_preview_layers
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let is_selected = idx == app.scenes.scene_layer_cursor;
                let is_enabled = app.scene_layer_enabled(idx);
                let style = if is_selected && focused {
                    theme::sidebar_active_entry()
                } else if is_selected {
                    theme::accent()
                } else {
                    theme::fg_normal()
                };
                let prefix = if is_selected { ">" } else { " " };
                ListItem::new(Line::from(Span::styled(
                    format!(
                        "{prefix} [{}] {idx:02}  {name}",
                        if is_enabled { "x" } else { " " }
                    ),
                    style,
                )))
            })
            .collect()
    };

    let list = List::new(items)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title("Layers")
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused))
                .title_bottom(Span::styled(
                    " j/k move | Space toggle | Enter solo ",
                    theme::fg_disabled(),
                )),
        );
    frame.render_widget(list, area);
}

fn render_live_preview(
    frame: &mut Frame,
    area: Rect,
    app: &AppState,
    focused: bool,
    fullscreen: bool,
) {
    let scene_name = app
        .selected_scene_display_name()
        .unwrap_or_else(|| String::from("none"));
    let title = format!("Live Preview: {scene_name}");

    if area.width < 12 || area.height < 8 {
        let widget = if fullscreen {
            Paragraph::new("Panel too small.").style(theme::preview_background())
        } else {
            Paragraph::new("Panel too small.")
                .style(theme::preview_background())
                .block(
                    Block::default()
                        .title(title)
                        .title_style(theme::pane_title(focused))
                        .border_style(theme::pane_border(app.mode, focused))
                        .borders(Borders::ALL)
                        .style(theme::preview_background()),
                )
        };
        frame.render_widget(widget, area);
        return;
    }

    let content_w = if fullscreen {
        area.width
    } else {
        area.width.saturating_sub(2)
    };
    let content_h = if fullscreen {
        area.height
    } else {
        area.height.saturating_sub(2)
    };

    let progress = app.scene_preview_progress();
    let lines = match app.scenes.scene_preview_scene.as_ref() {
        Some(scene) => {
            let (inner_w, inner_h) =
                adjusted_preview_size(scene.rendered_mode, content_w, content_h);
            let mut filtered_scene = scene.clone();
            if !app.scenes.scene_layer_visibility.is_empty()
                && app.scenes.scene_layer_visibility.len() == filtered_scene.layers.len()
            {
                filtered_scene.layers = filtered_scene
                    .layers
                    .into_iter()
                    .enumerate()
                    .filter_map(|(idx, layer)| app.scene_layer_enabled(idx).then_some(layer))
                    .collect();
            }
            match render_scene_preview(&filtered_scene, inner_w, inner_h, &app.mod_source, progress)
            {
                Ok(buffer) => {
                    let mut lines = preview_renderer::buffer_to_lines(&buffer);
                    if !fullscreen {
                        let scene_path = app
                            .selected_scene_ref_path()
                            .unwrap_or_else(|| String::from("-"));
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            format!(
                                "mod: {} | scene: {} | path: {} | visible layers: {} | progress: {:.2}",
                                app.mod_source,
                                scene.id,
                                scene_path,
                                filtered_scene.layers.len(),
                                progress
                            ),
                            theme::fg_disabled(),
                        )));
                    }
                    lines
                }
                Err(err) => vec![
                    Line::from("Preview render failed:"),
                    Line::from(""),
                    Line::from(err),
                ],
            }
        }
        None => vec![
            Line::from("No scene selected."),
            Line::from(""),
            Line::from("Select a scene in the upper-left list."),
        ],
    };

    let widget = if fullscreen {
        Paragraph::new(lines).style(theme::preview_background())
    } else {
        Paragraph::new(lines)
            .style(theme::preview_background())
            .block(
                Block::default()
                    .title(title)
                    .title_style(theme::pane_title(focused))
                    .border_style(theme::pane_border(app.mode, focused))
                    .borders(Borders::ALL)
                    .style(theme::preview_background())
                    .title_bottom(Span::styled(
                        " F5 soft-run | F6 run | Tab pane | F hold fullscreen | Ctrl+F toggle ",
                        theme::fg_disabled(),
                    )),
            )
    };
    frame.render_widget(widget, area);
}

fn adjusted_preview_size(mode: SceneRenderedMode, target_w: u16, target_h: u16) -> (u16, u16) {
    let mut width = target_w.max(8);
    let mut height = target_h.max(6);
    match mode {
        SceneRenderedMode::Cell => {}
        SceneRenderedMode::HalfBlock => {
            if !height.is_multiple_of(2) {
                height = height.saturating_sub(1);
            }
        }
        SceneRenderedMode::QuadBlock => {
            if !width.is_multiple_of(2) {
                width = width.saturating_sub(1);
            }
            if !height.is_multiple_of(2) {
                height = height.saturating_sub(1);
            }
        }
        SceneRenderedMode::Braille => {
            if !width.is_multiple_of(2) {
                width = width.saturating_sub(1);
            }
            let rem = height % 4;
            if rem != 0 {
                height = height.saturating_sub(rem);
            }
        }
    }

    (width.max(2), height.max(2))
}

fn render_scene_preview(
    scene: &Scene,
    width: u16,
    height: u16,
    mod_source: &str,
    progress: f32,
) -> Result<engine::buffer::Buffer, String> {
    if mod_source.is_empty() {
        return Err(String::from("mod source is not set"));
    }
    let asset_root = PathBuf::from(mod_source);
    preview_renderer::render_scene_buffer(PreviewRenderRequest {
        scene,
        width,
        height,
        asset_root: asset_root.as_path(),
        progress,
        duration_ms: preview_renderer::DEFAULT_SCENE_PREVIEW_DURATION_MS,
    })
}
