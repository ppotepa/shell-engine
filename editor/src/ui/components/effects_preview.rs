use engine::scene::Scene;
use engine_effects::{shared_dispatcher, ParamControl};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::domain::effect_params;
use crate::domain::effects_catalog;
use crate::domain::effects_preview_scene;
use crate::domain::preview_renderer::{self, PreviewRenderRequest};
use crate::state::{focus::FocusPane, AppState, EffectsCodeTab};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    if app.effects.effects_live_preview {
        // Horizontal split: left = code+params | right = live preview
        let h_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area);

        // Left half: code (top 50%) + params (bottom 50%)
        let left_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(h_split[0]);

        render_code(frame, left_split[0], app, app.focus == FocusPane::Browser);
        render_controls(frame, left_split[1], app, app.focus == FocusPane::Inspector);
        render_live(frame, h_split[1], app, false);
    } else {
        // No live preview: code (top 50%) + params (bottom 50%) across full area
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        render_code(frame, split[0], app, app.focus == FocusPane::Browser);
        render_controls(frame, split[1], app, app.focus == FocusPane::Inspector);
    }
}

/// Renders the code pane with tab bar (Info / Schema / YAML / Rust).
fn render_code(frame: &mut Frame, area: Rect, app: &AppState, focused: bool) {
    let effect_name = app.selected_builtin_effect();
    let tab = app.effects.effects_code_tab;

    // Build tab bar title: "[ Info ]  Schema   YAML   Rust"
    let tab_bar: String = EffectsCodeTab::ALL
        .iter()
        .map(|&t| {
            if t == tab {
                format!(" [{}] ", t.label())
            } else {
                format!("  {}  ", t.label())
            }
        })
        .collect::<Vec<_>>()
        .join("│");

    let title = format!(" {tab_bar} ");

    let lines: Vec<Line> = match effect_name {
        Some(name) => match tab {
            EffectsCodeTab::Info => tab_info(name),
            EffectsCodeTab::Schema => tab_schema(name),
            EffectsCodeTab::Yaml => tab_yaml(name),
            EffectsCodeTab::Rust => tab_rust(name),
        },
        None => vec![
            Line::from("No effect selected."),
            Line::from(""),
            Line::from("Use the sidebar to select a builtin effect."),
        ],
    };

    // Clamp scroll
    let visible = area.height.saturating_sub(2) as usize;
    let max_scroll = lines.len().saturating_sub(visible.max(1)) as u16;
    let scroll = app.effects.effects_code_scroll.min(max_scroll);

    let hint = if focused {
        "↑/↓ scroll  [/] tabs  Tab pane"
    } else {
        "[/] tabs  Tab to focus"
    };

    let widget = Paragraph::new(lines)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title(title)
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused))
                .title_bottom(Span::styled(format!(" {hint} "), theme::fg_disabled())),
        )
        .scroll((scroll, 0));

    frame.render_widget(widget, area);
}

// ─── Tab content builders ────────────────────────────────────────────────────

fn tab_info(name: &str) -> Vec<Line<'static>> {
    let doc = effects_catalog::effect_doc(name);
    let meta = shared_dispatcher().metadata(name);
    let t = theme::fg_disabled();
    let a = theme::fg_active();
    vec![
        Line::from(vec![
            Span::styled("name      ", t),
            Span::styled(name.to_string(), a),
        ]),
        Line::from(vec![
            Span::styled("category  ", t),
            Span::raw(doc.category.to_string()),
        ]),
        Line::from(vec![
            Span::styled("target    ", t),
            Span::raw(format!("{:?}", doc.target_kind)),
        ]),
        Line::from(""),
        Line::from(Span::styled(doc.summary, theme::fg_normal())),
        Line::from(""),
        Line::from(Span::styled("params", theme::fg_active())),
        Line::from(""),
    ]
    .into_iter()
    .chain(meta.params.iter().map(|p| {
        Line::from(vec![
            Span::styled(format!("  {:<16}", p.name), theme::fg_normal()),
            Span::styled(p.description.to_string(), theme::fg_disabled()),
        ])
    }))
    .collect()
}

fn tab_schema(name: &str) -> Vec<Line<'static>> {
    let meta = shared_dispatcher().metadata(name);
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!("# {} — parameter schema", meta.display_name),
            theme::fg_active(),
        )),
        Line::from(""),
    ];

    for p in meta.params {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<18}", p.name), theme::fg_active()),
            Span::styled(param_type_label(&p.control), theme::accent()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(p.description.to_string(), theme::fg_disabled()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(param_range_label(&p.control), theme::fg_normal()),
        ]));
        lines.push(Line::from(""));
    }
    lines
}

fn tab_yaml(name: &str) -> Vec<Line<'static>> {
    let doc = effects_catalog::effect_doc(name);
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("# Example YAML", theme::fg_disabled())),
        Line::from(""),
    ];
    for l in doc.sample.lines() {
        lines.push(Line::from(l.to_string()));
    }
    lines
}

fn tab_rust(name: &str) -> Vec<Line<'static>> {
    let Some(path) = rust_source_path(name) else {
        return vec![
            Line::from(Span::styled("Source file not found.", theme::fg_disabled())),
            Line::from(""),
            Line::from(format!(
                "Expected engine-core/src/effects/builtin/ for \"{}\"",
                name
            )),
        ];
    };

    match std::fs::read_to_string(&path) {
        Ok(src) => {
            let mut lines: Vec<Line> = vec![
                Line::from(Span::styled(format!("// {}", path), theme::fg_disabled())),
                Line::from(""),
            ];
            for l in src.lines() {
                lines.push(Line::from(l.to_string()));
            }
            lines
        }
        Err(e) => vec![
            Line::from(Span::styled("Failed to read source:", theme::fg_disabled())),
            Line::from(e.to_string()),
        ],
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn rust_source_path(effect_name: &str) -> Option<String> {
    // Walk up from CWD to find engine-core
    let cwd = std::env::current_dir().ok()?;
    let root = cwd
        .ancestors()
        .find(|p| p.join("engine-core").exists())?
        .to_path_buf();
    let builtin = root.join("engine-core/src/effects/builtin");

    let file = match effect_name {
        "fade-in" | "fade-out" => "fade.rs",
        "fade-to-black" => "fade_to_black.rs",
        "crt-on" => "crt_on.rs",
        "power-off" => "power_off.rs",
        "clear-to-colour" => "clear_to_colour.rs",
        "glitch-out" => "glitch.rs",
        "devour-out" => "devour.rs",
        "artifact-out" => "artifact.rs",
        "shatter-glitch" => "shatter.rs",
        "screen-shake" => "shake.rs",
        n if n.starts_with("lightning-") || n == "tesla-orb" => "lightning.rs",
        other => {
            // try exact file name: "shine" -> "shine.rs"
            let candidate = format!("{}.rs", other.replace('-', "_"));
            let p = builtin.join(&candidate);
            return if p.exists() {
                Some(p.to_string_lossy().into_owned())
            } else {
                None
            };
        }
    };

    let p = builtin.join(file);
    if p.exists() {
        Some(p.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn param_type_label(c: &ParamControl) -> &'static str {
    match c {
        ParamControl::Slider { .. } => "slider",
        ParamControl::Select { .. } => "select",
        ParamControl::Toggle { .. } => "toggle",
        ParamControl::Text { .. } => "text",
        ParamControl::Colour { .. } => "colour",
    }
}

fn param_range_label(c: &ParamControl) -> String {
    match c {
        ParamControl::Slider {
            min,
            max,
            step,
            unit,
        } => {
            if unit.is_empty() {
                format!("range {min}..{max}  step {step}")
            } else {
                format!("range {min}..{max}{unit}  step {step}{unit}")
            }
        }
        ParamControl::Select { options, default } => {
            format!("options: {}  default: {}", options.join(", "), default)
        }
        ParamControl::Toggle { default } => {
            format!("default: {}", if *default { "true" } else { "false" })
        }
        ParamControl::Text { default } => format!("default: {default}"),
        ParamControl::Colour { default } => format!("default: {default}"),
    }
}

fn render_live(frame: &mut Frame, area: Rect, app: &AppState, focused: bool) {
    let title = format!("Live: {}", app.selected_builtin_effect().unwrap_or("none"));

    if area.width < 12 || area.height < 8 {
        let widget = Paragraph::new("Panel too small.")
            .style(theme::preview_background())
            .block(
                Block::default()
                    .title(title)
                    .title_style(theme::pane_title(focused))
                    .border_style(theme::pane_border(app.mode, focused))
                    .borders(Borders::ALL)
                    .style(theme::preview_background()),
            );
        frame.render_widget(widget, area);
        return;
    }

    let inner_w = area.width.saturating_sub(2).max(8);
    let inner_h = area.height.saturating_sub(2).max(6);
    let progress = app.effect_preview_progress();
    let preview_yaml = build_responsive_preview_yaml(app, inner_w, inner_h);

    let lines = match render_preview_scene(&preview_yaml, inner_w, inner_h, progress) {
        Ok(buffer) => {
            let mut lines = preview_renderer::buffer_to_lines(&buffer);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("progress: {:.2}", progress),
                theme::fg_disabled(),
            )));
            lines
        }
        Err(err) => vec![
            Line::from("Preview render failed:"),
            Line::from(""),
            Line::from(err),
        ],
    };

    let widget = Paragraph::new(lines)
        .style(theme::preview_background())
        .block(
            Block::default()
                .title(title)
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::preview_background()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}

fn build_responsive_preview_yaml(app: &AppState, inner_w: u16, inner_h: u16) -> String {
    let Some(effect_name) = app.selected_builtin_effect() else {
        return app.effects.effects_preview_scene_yaml.clone();
    };

    let mut params = effect_params::default_effect_params(effect_name);
    effect_params::apply_overrides(
        effect_name,
        &app.effects.effect_param_overrides,
        &mut params,
    );
    effects_preview_scene::build_preview_scene_yaml(effect_name, &params, inner_w, inner_h)
}

fn render_controls(frame: &mut Frame, area: Rect, app: &AppState, focused: bool) {
    let effect_name = app.selected_builtin_effect().unwrap_or("shine");
    let specs = app.effect_param_specs();
    let mut params = effect_params::default_effect_params(effect_name);
    effect_params::apply_overrides(
        effect_name,
        &app.effects.effect_param_overrides,
        &mut params,
    );

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            if focused {
                "↑/↓ param  ←/→ adjust  Tab next pane"
            } else {
                "Tab to focus  Enter jump to controls"
            },
            theme::fg_disabled(),
        )),
        Line::from(""),
    ];

    if !specs.is_empty() {
        lines.push(Line::from(Span::styled(
            "Live controls",
            theme::fg_active(),
        )));
        for (idx, spec) in specs.iter().enumerate() {
            let value = app.effect_param_value(spec).as_float();
            let prefix = if idx == app.effects.effect_param_cursor {
                "▶"
            } else {
                " "
            };
            let rendered = format!("{prefix} {:<12} {}", spec.label, spec.render_value(value));
            let style = if idx == app.effects.effect_param_cursor && focused {
                theme::sidebar_active_entry()
            } else if idx == app.effects.effect_param_cursor {
                theme::accent()
            } else {
                theme::fg_normal()
            };
            lines.push(Line::from(Span::styled(rendered, style)));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No live controls.",
            theme::fg_disabled(),
        )));
    }

    // Extra non-slider params from docs
    let mut extra_params: Vec<(&str, String)> = Vec::new();
    for param in shared_dispatcher().metadata(effect_name).params {
        let name = param.name;
        if specs.iter().any(|spec| spec.name == name) {
            continue;
        }
        if let Some(value) = effect_params::param_text_value(&params, name) {
            extra_params.push((effect_params::param_label(name), value));
        }
    }

    if !extra_params.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Effect params",
            theme::fg_active(),
        )));
        for (label, value) in extra_params {
            lines.push(Line::from(format!("  {:<12} {}", label, value)));
        }
    }

    let widget = Paragraph::new(lines)
        .style(theme::pane_background(focused))
        .block(
            Block::default()
                .title("Parameters")
                .title_style(theme::pane_title(focused))
                .border_style(theme::pane_border(app.mode, focused))
                .borders(Borders::ALL)
                .style(theme::pane_background(focused)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}

fn render_preview_scene(
    yaml: &str,
    width: u16,
    height: u16,
    progress: f32,
) -> Result<engine::buffer::Buffer, String> {
    let scene: Scene =
        serde_yaml::from_str(yaml).map_err(|err| format!("YAML parse error: {err}"))?;
    let asset_root = effects_preview_scene::preview_asset_root()
        .ok_or_else(|| String::from("Preview asset root not found (expected mods/shell-quest)"))?;
    preview_renderer::render_scene_buffer(PreviewRenderRequest {
        scene: &scene,
        width,
        height,
        asset_root: asset_root.as_path(),
        progress,
        duration_ms: effects_preview_scene::PREVIEW_DURATION_MS,
    })
}
