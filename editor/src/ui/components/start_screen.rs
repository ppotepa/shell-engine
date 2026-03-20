//! Start screen component: welcome dialog with recent projects and action menu.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use serde::Deserialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::io::yaml::load_yaml;
use crate::state::{AppState, DirBrowserItem, StartDialog};
use crate::ui::theme;

/// Renders the start screen popup with the recent projects list and action menu.
pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Block::default()
            .title("Shell Engine Editor")
            .title_style(theme::fg_active())
            .border_style(theme::pane_border(app.mode, false))
            .borders(Borders::ALL),
        popup,
    );

    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner);

    match app.start_dialog {
        StartDialog::RecentMenu => render_recent_menu(frame, chunks[0], chunks[1], chunks[2], app),
        StartDialog::SchemaPicker => {
            render_schema_picker(frame, chunks[0], chunks[1], chunks[2], app)
        }
        StartDialog::DirectoryBrowser => {
            render_directory_browser(frame, chunks[0], chunks[1], chunks[2], app)
        }
    }
}

fn render_recent_menu(frame: &mut Frame, header: Rect, body: Rect, footer: Rect, app: &AppState) {
    use crate::state::StartFocus;

    frame.render_widget(
        Paragraph::new(Line::from("Shell Engine Editor - Start"))
            .style(theme::accent())
            .block(
                Block::default()
                    .border_style(theme::pane_border(app.mode, false))
                    .borders(Borders::ALL),
            ),
        header,
    );

    // Split body into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);

    // Left column: Recents
    let recents_focused = matches!(app.start.focus, StartFocus::Recents);
    let recents_items: Vec<ListItem> = if app.recent_projects.is_empty() {
        vec![ListItem::new("  No recent projects").style(theme::fg_disabled())]
    } else {
        app.recent_projects
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let (_, valid) = app.recent_project_status(idx);
                let selected = recents_focused && idx == app.start.recent_cursor;
                if selected && valid {
                    ListItem::new(format!("> {}", path)).style(theme::accent())
                } else if !valid {
                    ListItem::new(format!("  {}", path)).style(theme::fg_disabled())
                } else if valid {
                    ListItem::new(format!("  {}", path)).style(theme::fg_active())
                } else {
                    ListItem::new(format!("  {}", path)).style(theme::fg_normal())
                }
            })
            .collect()
    };

    frame.render_widget(
        List::new(recents_items)
            .style(theme::pane_background(recents_focused))
            .block(
                Block::default()
                    .title("Recent Projects")
                    .title_style(if recents_focused {
                        theme::accent()
                    } else {
                        theme::fg_normal()
                    })
                    .border_style(theme::pane_border(app.mode, recents_focused))
                    .borders(Borders::ALL)
                    .style(theme::pane_background(recents_focused)),
            ),
        columns[0],
    );

    // Right column: Actions
    let actions_focused = matches!(app.start.focus, StartFocus::Actions);
    let actions = [
        ("Open Project…", true),
        ("Find Schema YML…", true),
        ("New Project…", false),
        ("Quit", true),
    ];
    let action_items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(idx, (label, enabled))| {
            let selected = actions_focused && idx == app.start.action_cursor;
            if selected && *enabled {
                ListItem::new(format!("> {}", label)).style(theme::accent())
            } else if !enabled {
                ListItem::new(format!("  {}", label)).style(theme::fg_disabled())
            } else {
                ListItem::new(format!("  {}", label)).style(theme::fg_active())
            }
        })
        .collect();

    frame.render_widget(
        List::new(action_items)
            .style(theme::pane_background(actions_focused))
            .block(
                Block::default()
                    .title("Actions")
                    .title_style(if actions_focused {
                        theme::accent()
                    } else {
                        theme::fg_normal()
                    })
                    .border_style(theme::pane_border(app.mode, actions_focused))
                    .borders(Borders::ALL)
                    .style(theme::pane_background(actions_focused)),
            ),
        columns[1],
    );

    // Footer hints
    let hint =
        "Tab: switch panel | j/k: move | Enter: select | f: schema scan | x: prune stale | q: quit";
    frame.render_widget(
        Paragraph::new(hint).style(theme::fg_normal()).block(
            Block::default()
                .title("Hint")
                .title_style(theme::fg_normal())
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL),
        ),
        footer,
    );
}

fn render_schema_picker(frame: &mut Frame, header: Rect, body: Rect, footer: Rect, app: &AppState) {
    frame.render_widget(
        Paragraph::new(Line::from("Open from schema-tagged .yml"))
            .style(theme::accent())
            .block(
                Block::default()
                    .border_style(theme::pane_border(app.mode, false))
                    .borders(Borders::ALL),
            ),
        header,
    );

    let list_items: Vec<ListItem> = if app.schema_candidates.is_empty() {
        vec![ListItem::new("  (no matching files found)").style(theme::fg_disabled())]
    } else {
        app.schema_candidates
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                if idx == app.schema_cursor {
                    ListItem::new(format!("> {path}")).style(theme::accent())
                } else {
                    ListItem::new(format!("  {path}")).style(theme::fg_normal())
                }
            })
            .collect()
    };
    frame.render_widget(
        List::new(list_items).block(
            Block::default()
                .title("Schema YML")
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, true))
                .borders(Borders::ALL),
        ),
        body,
    );

    frame.render_widget(
        Paragraph::new("Enter open | Esc back | j/k move | q quit")
            .style(theme::fg_normal())
            .block(
                Block::default()
                    .title("Hint")
                    .title_style(theme::fg_normal())
                    .border_style(theme::pane_border(app.mode, false))
                    .borders(Borders::ALL),
            ),
        footer,
    );
}

fn render_directory_browser(
    frame: &mut Frame,
    header: Rect,
    body: Rect,
    footer: Rect,
    app: &AppState,
) {
    frame.render_widget(
        Paragraph::new(Line::from(format!(
            "Directory: {}  | depth: {}",
            app.dir_browser_path,
            path_depth(&app.dir_browser_path)
        )))
        .style(theme::accent())
        .block(
            Block::default()
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL),
        ),
        header,
    );

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(7)])
        .split(split[1]);

    let list_items: Vec<ListItem> = app
        .dir_browser_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let label = match item {
                DirBrowserItem::OpenHere => "Open this directory".to_string(),
                DirBrowserItem::Parent => "../".to_string(),
                DirBrowserItem::Directory { path, .. } => {
                    format!("{}{}", "\t", basename_only(path))
                }
            };
            let item_enabled = match item {
                DirBrowserItem::OpenHere => app.dir_can_open,
                DirBrowserItem::Directory { .. } => true,
                DirBrowserItem::Parent => true,
            };
            if idx == app.dir_cursor && item_enabled {
                ListItem::new(format!("> {label}")).style(theme::accent())
            } else if !item_enabled {
                ListItem::new(format!("  {label}")).style(theme::fg_disabled())
            } else if matches!(
                item,
                DirBrowserItem::Directory {
                    valid_project: true,
                    ..
                }
            ) {
                ListItem::new(format!("  {label}")).style(theme::fg_active())
            } else {
                ListItem::new(format!("  {label}")).style(theme::fg_normal())
            }
        })
        .collect();

    frame.render_widget(
        List::new(list_items).block(
            Block::default()
                .title("Navigator")
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, true))
                .borders(Borders::ALL),
        ),
        split[0],
    );

    let mut preview_items: Vec<ListItem> = Vec::new();
    if let Some(index) = &app.dir_preview_index {
        preview_items.push(
            ListItem::new(format!("{}/", basename_only(&app.dir_preview_path)))
                .style(theme::fg_active()),
        );
        preview_items.push(ListItem::new("├─ mod.yaml").style(theme::fg_normal()));
        preview_items.push(ListItem::new("├─ scenes/").style(theme::fg_normal()));
        for scene in sample_leaf_names(&index.scenes.scene_paths, 5) {
            preview_items.push(ListItem::new(format!("│  ├─ {scene}")).style(theme::fg_normal()));
        }
        if index.scenes.scene_paths.len() > 5 {
            preview_items.push(ListItem::new("│  └─ ...").style(theme::fg_disabled()));
        }
        preview_items.push(ListItem::new("└─ assets/").style(theme::fg_normal()));
        preview_items.push(
            ListItem::new(format!("   ├─ images/ ({})", index.images.len()))
                .style(theme::fg_normal()),
        );
        for img in sample_leaf_names(&index.images, 2) {
            preview_items
                .push(ListItem::new(format!("   │  ├─ {img}")).style(theme::fg_disabled()));
        }
        preview_items.push(
            ListItem::new(format!("   └─ fonts/ ({})", index.fonts.len()))
                .style(theme::fg_normal()),
        );
        for font in sample_leaf_names(&index.fonts, 2) {
            preview_items
                .push(ListItem::new(format!("      ├─ {font}")).style(theme::fg_disabled()));
        }
    } else {
        preview_items.push(ListItem::new("NO ENGINE DATA").style(theme::fg_disabled()));
        preview_items.push(
            ListItem::new("This folder is not a Shell Quest project").style(theme::fg_disabled()),
        );
        preview_items.push(ListItem::new("Select a valid mod root").style(theme::fg_disabled()));
    }

    frame.render_widget(
        List::new(preview_items).block(
            Block::default()
                .title("Project Preview")
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL),
        ),
        right[0],
    );

    let summary = if let Some(index) = &app.dir_preview_index {
        let (name, version, entrypoint) = if let Some(m) = &index.manifest {
            (
                m.name
                    .clone()
                    .unwrap_or_else(|| basename_only(&app.dir_preview_path)),
                m.version.clone().unwrap_or_else(|| "-".to_string()),
                m.entrypoint.clone().unwrap_or_else(|| "-".to_string()),
            )
        } else {
            ("-".to_string(), "-".to_string(), "-".to_string())
        };
        format!(
            "name: {name}\nversion: {version}\nentrypoint: {entrypoint}\nscenes: {}  images: {}  fonts: {}  yaml: {}",
            index.scenes.scene_paths.len(),
            index.images.len(),
            index.fonts.len(),
            index.project_yamls.len()
        )
    } else {
        "NO DATA\nThis folder is not a valid Shell Quest mod root".to_string()
    };

    frame.render_widget(
        Paragraph::new(summary).style(theme::fg_normal()).block(
            Block::default()
                .title("Summary")
                .title_style(theme::fg_normal())
                .border_style(theme::pane_border(app.mode, false))
                .borders(Borders::ALL),
        ),
        right[1],
    );

    frame.render_widget(
        Paragraph::new("Enter select | F5 preview | Esc back | j/k move | q quit")
            .style(theme::fg_normal())
            .block(
                Block::default()
                    .title("Hint")
                    .title_style(theme::fg_normal())
                    .border_style(theme::pane_border(app.mode, false))
                    .borders(Borders::ALL),
            ),
        footer,
    );

    if app.dir_preview_popup {
        render_live_preview_popup(frame, split[1], app);
    }
}

fn trim_path(path: &str) -> String {
    let p = path.replace('\\', "/");
    if let Some((_, tail)) = p.rsplit_once("/scenes/") {
        return format!("scenes/{tail}");
    }
    if p.len() > 48 {
        format!("...{}", &p[p.len() - 45..])
    } else {
        p
    }
}

fn basename_only(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn sample_leaf_names(paths: &[String], max: usize) -> Vec<String> {
    paths
        .iter()
        .take(max)
        .map(|p| {
            Path::new(p)
                .file_name()
                .and_then(|s| s.to_str())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| trim_path(p))
        })
        .collect()
}

fn render_live_preview_popup(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect_in(86, 72, area);
    frame.render_widget(Clear, popup);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let now_ms = millis;
    let elapsed_ms = now_ms.saturating_sub(app.dir_preview_started_at_ms);
    let simulated_ms = elapsed_ms.saturating_mul(u64::from(app.dir_preview_speed_mult));
    let text = render_engine_emulation_text(app, simulated_ms);

    frame.render_widget(
        Paragraph::new(text).style(theme::fg_normal()).block(
            Block::default()
                .title("Preview")
                .title_style(theme::fg_active())
                .border_style(theme::pane_border(app.mode, true))
                .borders(Borders::ALL),
        ),
        popup,
    );
}

fn centered_rect_in(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn path_depth(path: &str) -> usize {
    Path::new(path).components().count().saturating_sub(1)
}

#[derive(Debug, Clone, Deserialize)]
struct SceneEffect {
    name: String,
    duration: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneStep {
    duration: Option<u64>,
    effects: Option<Vec<SceneEffect>>,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneStage {
    steps: Option<Vec<SceneStep>>,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneStages {
    on_enter: Option<SceneStage>,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneDoc {
    id: Option<String>,
    title: Option<String>,
    stages: Option<SceneStages>,
}

fn render_engine_emulation_text(app: &AppState, simulated_ms: u64) -> String {
    let Some(index) = &app.dir_preview_index else {
        return "LIVE PREVIEW\n\nNO ENGINE DATA".to_string();
    };
    let entrypoint = index
        .manifest
        .as_ref()
        .and_then(|m| m.entrypoint.clone())
        .unwrap_or_default();
    if entrypoint.is_empty() {
        return "LIVE PREVIEW\n\nNO ENTRYPOINT".to_string();
    }
    let scene_path = Path::new(&app.dir_preview_path).join(entrypoint.trim_start_matches('/'));
    let Some(scene) = load_yaml::<SceneDoc>(&scene_path) else {
        return format!(
            "LIVE PREVIEW x{}\n\nproject: {}\nscene: {}\n\nCould not load entrypoint scene",
            app.dir_preview_speed_mult,
            basename_only(&app.dir_preview_path),
            entrypoint
        );
    };

    let scene_title = scene
        .title
        .clone()
        .unwrap_or_else(|| "<untitled>".to_string());
    let scene_id = scene.id.clone().unwrap_or_else(|| entrypoint.clone());
    let steps = scene
        .stages
        .as_ref()
        .and_then(|s| s.on_enter.as_ref())
        .and_then(|s| s.steps.as_ref())
        .cloned()
        .unwrap_or_default();
    if steps.is_empty() {
        return format!(
            "LIVE PREVIEW x{}\n\nscene: {}\n\nNo on_enter steps",
            app.dir_preview_speed_mult, scene_id
        );
    }

    let total_ms: u64 = steps
        .iter()
        .map(|s| s.duration.unwrap_or(0))
        .sum::<u64>()
        .max(1);
    let t = simulated_ms % total_ms;
    let mut acc = 0u64;
    let mut current = 0usize;
    for (i, step) in steps.iter().enumerate() {
        let d = step.duration.unwrap_or(0);
        if t < acc.saturating_add(d.max(1)) {
            current = i;
            break;
        }
        acc = acc.saturating_add(d);
    }
    let step = &steps[current];
    let local = t.saturating_sub(acc);
    let step_d = step.duration.unwrap_or(1).max(1);
    let bar_w = 20usize;
    let fill = ((local as f32 / step_d as f32) * bar_w as f32) as usize;
    let bar = format!(
        "[{}{}]",
        "#".repeat(fill.min(bar_w)),
        "-".repeat(bar_w.saturating_sub(fill.min(bar_w)))
    );
    let effects = step
        .effects
        .as_ref()
        .map(|fx| {
            if fx.is_empty() {
                "none".to_string()
            } else {
                fx.iter()
                    .map(|e| match e.duration {
                        Some(d) => format!("{}({}ms)", e.name, d),
                        None => e.name.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        })
        .unwrap_or_else(|| "none".to_string());

    format!(
        "LIVE PREVIEW x{}\nproject: {}\nscene: {} ({})\n\nengine_time: {}ms\nstep: {}/{}\nprogress: {}\nactive_effects: {}\n\n(ESC or F5 to close)",
        app.dir_preview_speed_mult,
        basename_only(&app.dir_preview_path),
        scene_title,
        scene_id,
        t,
        current + 1,
        steps.len(),
        bar,
        effects
    )
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
