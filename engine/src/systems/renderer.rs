//! Backend-agnostic presentation helpers used by the engine runtime.

use crate::buffer::Buffer;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_animation::SceneStage;
use engine_core::color::Color;
use engine_core::logging;
use engine_core::scene::{
    Scene, Sprite, SpriteSizePreset, TextOverflowMode, TextTransform, TextWrapMode,
};
use engine_debug::DebugOverlayMode;
use engine_render::RenderBackendKind;
use engine_render::{FrameSubmission, OverlayData, OverlayLine, PreparedOverlay, PreparedUi, PreparedWorld};
use engine_render_2d::text_sprite_dimensions;
use engine_render_policy::{resolve_text_font_spec_with_capabilities, FontPolicyCapabilities};
use engine_runtime::{PresentationPolicy, RenderSize, RuntimeSettings};
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;

/// Present the current frame through the active output backend.
pub fn renderer_system(world: &mut World) {
    let has_script_errors = world
        .get::<crate::debug_log::DebugLogBuffer>()
        .map(|log| log.has_errors)
        .unwrap_or(false);
    let debug_enabled = world
        .get::<crate::debug_features::DebugFeatures>()
        .map(|debug| debug.enabled)
        .unwrap_or(false);
    if has_script_errors && debug_enabled {
        logging::warn(
            "renderer.flicker_diag",
            "restore_front_to_back TRIGGERED (has_script_errors=true + debug_enabled)".to_string(),
        );
        if let Some(buffer) = world.get_mut::<Buffer>() {
            buffer.restore_front_to_back();
        }
    }

    let overlay_data = collect_debug_overlay(world);
    apply_perf_hud(world);

    let buffer_ptr: *const Buffer = world
        .get::<Buffer>()
        .map(|buffer| buffer as *const Buffer)
        .unwrap_or(std::ptr::null());
    let vectors_ptr: *const engine_render::VectorOverlay = world
        .get::<engine_render::VectorOverlay>()
        .map(|overlay| overlay as *const _)
        .unwrap_or(std::ptr::null());

    if !buffer_ptr.is_null() {
        if let Some(renderer) = world.renderer_mut() {
            if let Some(ref overlay) = overlay_data {
                renderer.present_overlay(overlay);
            }
            if !vectors_ptr.is_null() {
                let vectors = unsafe { &*vectors_ptr };
                renderer.present_vectors(vectors);
            }
            let buffer = unsafe { &*buffer_ptr };
            present_active_frame(renderer, buffer);
        }
    }

    if let Some(buffer) = world.get_mut::<Buffer>() {
        buffer.swap();
    }
}

fn present_active_frame(renderer: &mut dyn engine_render::RendererBackend, buffer: &Buffer) {
    if renderer.backend_kind() != RenderBackendKind::Hardware {
        renderer.present_frame(buffer);
        return;
    }

    let submission = FrameSubmission {
        output_size: renderer.output_size(),
        present_mode: engine_render::PresentMode::VSync,
        world: PreparedWorld { ready: true },
        ui: PreparedUi { ready: true },
        overlay: PreparedOverlay {
            ready: true,
            line_count: 0,
            primitive_count: 0,
        },
    };

    if let Err(err) = renderer.submit_frame(&submission) {
        if renderer.backend_kind() == RenderBackendKind::Hardware {
            logging::warn(
                "renderer.hardware_fallback",
                format!("hardware submit failed ({err}); falling back to software frame"),
            );
        }
        renderer.present_frame(buffer);
    }
}

fn collect_debug_overlay(world: &mut World) -> Option<OverlayData> {
    let debug = *world.get::<crate::debug_features::DebugFeatures>()?;
    if !debug.enabled || !debug.overlay_visible {
        return None;
    }

    let title_fg = Color::White;
    let title_bg = Color::Rgb {
        r: 40,
        g: 40,
        b: 120,
    };
    let sep_fg = Color::Rgb {
        r: 80,
        g: 80,
        b: 100,
    };
    let sep_bg = Color::Rgb {
        r: 15,
        g: 15,
        b: 30,
    };
    let label_fg = Color::Rgb {
        r: 140,
        g: 140,
        b: 160,
    };
    let console_bg = Color::Rgb {
        r: 12,
        g: 12,
        b: 25,
    };
    let console_alpha: u8 = 190;

    let mut lines = Vec::new();

    match debug.overlay_mode {
        DebugOverlayMode::Stats => {
            let scene_id = world
                .scene_runtime()
                .map(|runtime| runtime.scene().id.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let stage_info = world
                .animator()
                .map(|anim| {
                    let stage = match anim.stage {
                        SceneStage::OnEnter => "on_enter",
                        SceneStage::OnIdle => "on_idle",
                        SceneStage::OnLeave => "on_leave",
                        SceneStage::Done => "done",
                    };
                    format!("{} ({:.1}s)", stage, anim.elapsed_ms as f64 / 1000.0)
                })
                .unwrap_or_else(|| "-".to_string());
            let virtual_info = format_render_info(world.runtime_settings());
            let object_count = world.scene_runtime().map(|runtime| runtime.object_count());
            let timings_info = world
                .get::<crate::debug_features::SystemTimings>()
                .map(|timings| {
                    let work_us = timings.physics_us
                        + timings.behavior_us
                        + timings.compositor_us
                        + timings.postfx_us
                        + timings.renderer_us;
                    let headroom_pct = if timings.frame_us > 0.0 {
                        (timings.sleep_us / timings.frame_us * 100.0) as u32
                    } else {
                        0
                    };
                    format!(
                        "phys:{:.0}  beh:{:.0}  comp:{:.0}  pfx:{:.0}  rend:{:.0}  sleep:{:.0}µs  work:{:.0}µs  hdroom:{}%",
                        timings.physics_us,
                        timings.behavior_us,
                        timings.compositor_us,
                        timings.postfx_us,
                        timings.renderer_us,
                        timings.sleep_us,
                        work_us,
                        headroom_pct
                    )
                })
                .unwrap_or_default();
            let script_errors: Vec<String> = world
                .get::<crate::debug_log::DebugLogBuffer>()
                .map(|log| {
                    log.recent(usize::MAX)
                        .iter()
                        .map(|entry| entry.display_line())
                        .collect()
                })
                .unwrap_or_default();

            lines.push(OverlayLine::with_alpha(
                " ■ DEBUG CONSOLE          [~] toggle  [Tab] switch  [F3/F4] scene",
                title_fg,
                title_bg,
                220,
            ));
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]scene   │[/] [#66d9ef]{scene_id}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]stage   │[/] [#78dca0]{stage_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]render  │[/] [#ffd166]{virtual_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]timing  │[/] [#d39bff]{timings_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            let object_info = object_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "-".to_string());
            let gameplay_info = world
                .get::<engine_debug::GameplayDiagnostics>()
                .map(|diag| {
                    format!(
                        "ent:{} vis:{} — {}",
                        diag.entity_count, diag.visual_count, diag.summary
                    )
                })
                .unwrap_or_default();
            let objects_line = if gameplay_info.is_empty() {
                format!("  [#8c8ca0]objects │[/] [#f0b990]{object_info}[/]")
            } else {
                format!(
                    "  [#8c8ca0]objects │[/] [#f0b990]{object_info}[/]  [#78dca0]{gameplay_info}[/]"
                )
            };
            lines.push(OverlayLine::with_alpha(
                objects_line,
                label_fg,
                console_bg,
                console_alpha,
            ));

            if !script_errors.is_empty() {
                lines.push(OverlayLine::with_alpha(
                    "─── messages ─────────────────────────────────────────────────────",
                    sep_fg,
                    sep_bg,
                    console_alpha,
                ));
                for err in script_errors {
                    let (line_fg, line_bg) = if err.starts_with("[ERR") {
                        (
                            Color::Rgb {
                                r: 255,
                                g: 100,
                                b: 100,
                            },
                            Color::Rgb {
                                r: 60,
                                g: 10,
                                b: 10,
                            },
                        )
                    } else if err.starts_with("[WARN") {
                        (
                            Color::Rgb {
                                r: 255,
                                g: 200,
                                b: 80,
                            },
                            Color::Rgb { r: 50, g: 40, b: 5 },
                        )
                    } else {
                        (label_fg, console_bg)
                    };
                    lines.push(OverlayLine::with_alpha(
                        format!("  {err}"),
                        line_fg,
                        line_bg,
                        console_alpha,
                    ));
                }
            } else {
                lines.push(OverlayLine::with_alpha(
                    "  [green]all systems nominal[/]",
                    label_fg,
                    console_bg,
                    console_alpha,
                ));
            }
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
        }
        DebugOverlayMode::Logs => {
            lines.push(OverlayLine::with_alpha(
                " ■ LOG CONSOLE            [~] toggle  [Tab] switch",
                title_fg,
                title_bg,
                220,
            ));
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
            let logs = logging::tail_recent(40);
            if logs.is_empty() {
                lines.push(OverlayLine::with_alpha(
                    "  [green](no log entries)[/]",
                    label_fg,
                    console_bg,
                    console_alpha,
                ));
            } else {
                for log_line in &logs {
                    let (level_label, level_tag) = match log_line.level.trim() {
                        "TRACE" => ("TRC", "#7f7f7f"),
                        "DEBUG" => ("DBG", "#7c7c8f"),
                        "INFO" => ("INF", "#50c878"),
                        "WARN" => ("WRN", "#ffc850"),
                        "ERROR" => ("ERR", "#ff6464"),
                        _ => ("???", "white"),
                    };
                    let line_bg = match log_line.level.trim() {
                        "ERROR" => Color::Rgb { r: 40, g: 5, b: 5 },
                        "WARN" => Color::Rgb { r: 35, g: 30, b: 5 },
                        _ => console_bg,
                    };
                    let formatted = format!(
                        "  [{level_tag}]{level_label}[/] [#8c8ca0]{}[/] │ {}",
                        log_line.target, log_line.message
                    );
                    lines.push(OverlayLine::with_alpha(
                        formatted,
                        label_fg,
                        line_bg,
                        console_alpha,
                    ));
                }
            }
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
        }
        DebugOverlayMode::Layout => {
            let layout = collect_layout_overlay_snapshot(world);
            let layout_state = if layout.layout_stale {
                "[#ffc850]stale[/]"
            } else {
                "[#78dca0]clean[/]"
            };

            lines.push(OverlayLine::with_alpha(
                " ■ LAYOUT CONSOLE         [~] toggle  [Tab] switch",
                title_fg,
                title_bg,
                220,
            ));
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]scene   │[/] [#66d9ef]{}[/]", layout.scene_id),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!(
                    "  [#8c8ca0]text    │[/] [#ffd166]{} shown[/] [#8c8ca0](of {} runtime text objects)[/]",
                    layout.rows.len(),
                    layout.total_rows
                ),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                format!(
                    "  [#8c8ca0]layout  │[/] {layout_state} [#8c8ca0](fit/intr are measured text-layout sizes)[/]"
                ),
                label_fg,
                console_bg,
                console_alpha,
            ));
            lines.push(OverlayLine::with_alpha(
                "  [#8c8ca0]status  │[/] [#d39bff]ok wrap clip ellipsis clamp reserve[/] [#8c8ca0](n/a = no authored text layout)[/]",
                label_fg,
                console_bg,
                console_alpha,
            ));

            if !layout.rows.is_empty() {
                lines.push(OverlayLine::with_alpha(
                    "─── text objects (fit/intr, constraints, status) ─────────────────",
                    sep_fg,
                    sep_bg,
                    console_alpha,
                ));
                for row in layout.rows {
                    let visibility = if row.visible { "vis" } else { "hid" };
                    let row_fg = if row.visible {
                        label_fg
                    } else {
                        Color::Rgb {
                            r: 110,
                            g: 110,
                            b: 120,
                        }
                    };
                    lines.push(OverlayLine::with_alpha(
                        format!(
                            "  [#66d9ef]{}[/] [#78dca0]{}[/] [#8c8ca0]off[/] ({:+},{:+}) [#ffd166]fit[/] {} [#8c8ca0]intr[/] {} [#d39bff]{}[/] [#8c8ca0]{} {} fg:{} bg:{}[/] │ {}",
                            row.id,
                            visibility,
                            row.offset_x,
                            row.offset_y,
                            row.fit_size,
                            row.intrinsic_size,
                            row.status,
                            row.constraints,
                            row.font,
                            row.fg,
                            row.bg,
                            row.text
                        ),
                        row_fg,
                        console_bg,
                        console_alpha,
                    ));
                }
            } else {
                lines.push(OverlayLine::with_alpha(
                    "  [green](no runtime text objects captured)[/]",
                    label_fg,
                    console_bg,
                    console_alpha,
                ));
            }

            if !layout.log_lines.is_empty() {
                lines.push(OverlayLine::with_alpha(
                    "─── layout diagnostics ───────────────────────────────────────────",
                    sep_fg,
                    sep_bg,
                    console_alpha,
                ));
                for (severity, line) in layout.log_lines {
                    let (line_fg, line_bg) = match severity {
                        crate::debug_log::DebugSeverity::Error => (
                            Color::Rgb {
                                r: 255,
                                g: 100,
                                b: 100,
                            },
                            Color::Rgb {
                                r: 60,
                                g: 10,
                                b: 10,
                            },
                        ),
                        crate::debug_log::DebugSeverity::Warn => (
                            Color::Rgb {
                                r: 255,
                                g: 200,
                                b: 80,
                            },
                            Color::Rgb { r: 50, g: 40, b: 5 },
                        ),
                        crate::debug_log::DebugSeverity::Info => (label_fg, console_bg),
                    };
                    lines.push(OverlayLine::with_alpha(
                        format!("  {line}"),
                        line_fg,
                        line_bg,
                        console_alpha,
                    ));
                }
            } else {
                lines.push(OverlayLine::with_alpha(
                    "  [green](no layout diagnostics)[/]",
                    label_fg,
                    console_bg,
                    console_alpha,
                ));
            }

            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
        }
    }

    Some(OverlayData {
        lines,
        dim_scene: true,
    })
}

#[derive(Debug)]
struct LayoutOverlayRow {
    id: String,
    visible: bool,
    offset_x: i32,
    offset_y: i32,
    fit_size: String,
    intrinsic_size: String,
    constraints: String,
    status: String,
    text: String,
    font: String,
    fg: String,
    bg: String,
}

#[derive(Debug, Default)]
struct LayoutOverlaySnapshot {
    scene_id: String,
    layout_stale: bool,
    total_rows: usize,
    rows: Vec<LayoutOverlayRow>,
    log_lines: Vec<(crate::debug_log::DebugSeverity, String)>,
}

fn collect_layout_overlay_snapshot(world: &mut World) -> LayoutOverlaySnapshot {
    const MAX_LAYOUT_ROWS: usize = 10;
    const MAX_LAYOUT_LOGS: usize = 6;

    let mut snapshot = LayoutOverlaySnapshot::default();
    snapshot.scene_id = "unknown".to_string();
    let mod_source = world
        .asset_root()
        .map(|asset_root| asset_root.mod_source().to_path_buf());
    let (default_font, font_policy_caps) = world
        .runtime_settings()
        .map(|settings| (settings.default_font.clone(), runtime_font_policy_capabilities(settings)))
        .unwrap_or((None, FontPolicyCapabilities::default()));

    if let Some(runtime) = world.scene_runtime_mut() {
        snapshot.scene_id = runtime.scene().id.clone();
        snapshot.layout_stale = runtime.layout_regions_stale();
        let authored_layout = collect_authored_text_layout_configs(runtime.scene());
        let kinds = runtime.object_kind_snapshot();
        let texts = runtime.object_text_snapshot();
        let props = runtime.object_props_snapshot();
        let states = runtime.effective_object_states_snapshot();

        let mut text_ids: Vec<String> = kinds
            .iter()
            .filter_map(|(id, kind)| {
                if kind == "text" || texts.contains_key(id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        text_ids.sort();
        snapshot.total_rows = text_ids.len();

        for id in text_ids.into_iter().take(MAX_LAYOUT_ROWS) {
            let state = states.get(&id).cloned().unwrap_or_default();
            let props = props.get(&id);
            let authored_id = runtime
                .object(&id)
                .and_then(|object| object.aliases.first())
                .map(String::as_str)
                .unwrap_or(id.as_str());
            let authored_layout = authored_layout.get(authored_id);
            let content = texts.get(&id).map(String::as_str).unwrap_or("(empty)");
            let font_override = json_prop_str(props, "text", "font");
            let fg = json_prop_str(props, "style", "fg").unwrap_or("-");
            let bg = json_prop_str(props, "style", "bg").unwrap_or("-");
            let measurement = authored_layout.map(|cfg| {
                measure_text_layout(
                    mod_source.as_deref(),
                    default_font.as_deref(),
                    font_policy_caps,
                    content,
                    font_override,
                    cfg.clone(),
                )
            });
            snapshot.rows.push(LayoutOverlayRow {
                id: overlay_safe_text(&id, 18),
                visible: state.visible,
                offset_x: state.offset_x,
                offset_y: state.offset_y,
                fit_size: measurement
                    .as_ref()
                    .map(|value| format!("{}x{}", value.fit_w, value.fit_h))
                    .unwrap_or_else(|| "--".to_string()),
                intrinsic_size: measurement
                    .as_ref()
                    .map(|value| format!("{}x{}", value.intrinsic_w, value.intrinsic_h))
                    .unwrap_or_else(|| "--".to_string()),
                constraints: measurement
                    .as_ref()
                    .map(|value| value.constraints.clone())
                    .unwrap_or_else(|| "-".to_string()),
                status: measurement
                    .as_ref()
                    .map(|value| value.status.clone())
                    .unwrap_or_else(|| "n/a".to_string()),
                text: overlay_safe_text(content, 28),
                font: overlay_safe_text(
                    measurement
                        .as_ref()
                        .map(|value| value.font.as_str())
                        .or(font_override)
                        .unwrap_or("-"),
                    12,
                ),
                fg: overlay_safe_text(fg, 10),
                bg: overlay_safe_text(bg, 10),
            });
        }
    }

    if let Some(log) = world.get::<crate::debug_log::DebugLogBuffer>() {
        let mut lines: Vec<_> = log
            .recent(usize::MAX)
            .iter()
            .filter(|entry| entry.message.starts_with("[layout:"))
            .rev()
            .take(MAX_LAYOUT_LOGS)
            .map(|entry| (entry.severity, overlay_safe_text(&entry.display_line(), 72)))
            .collect();
        lines.reverse();
        snapshot.log_lines = lines;
    }

    snapshot
}

fn json_prop_str<'a>(props: Option<&'a JsonValue>, group: &str, name: &str) -> Option<&'a str> {
    props
        .and_then(|props| props.get(group))
        .and_then(|group| group.get(name))
        .and_then(JsonValue::as_str)
}

fn overlay_safe_text(text: &str, limit: usize) -> String {
    let sanitized = text.replace(['[', ']'], "");
    let mut out = String::with_capacity(sanitized.len().min(limit + 1));
    for (idx, ch) in sanitized.chars().enumerate() {
        if idx >= limit {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

#[derive(Debug, Clone)]
struct AuthoredTextLayoutConfig {
    font: Option<String>,
    force_font_mode: Option<String>,
    size: Option<SpriteSizePreset>,
    transform: TextTransform,
    max_width: Option<u16>,
    overflow_mode: TextOverflowMode,
    wrap_mode: TextWrapMode,
    line_clamp: Option<u16>,
    reserve_width_ch: Option<u16>,
    line_height: u16,
    scale_x: f32,
    scale_y: f32,
}

#[derive(Debug, Clone)]
struct LayoutTextMeasurement {
    font: String,
    fit_w: u16,
    fit_h: u16,
    intrinsic_w: u16,
    intrinsic_h: u16,
    constraints: String,
    status: String,
}

fn collect_authored_text_layout_configs(
    scene: &Scene,
) -> HashMap<String, AuthoredTextLayoutConfig> {
    let mut out = HashMap::new();
    for layer in &scene.layers {
        for sprite in &layer.sprites {
            sprite.walk_recursive(&mut |node| {
                let Sprite::Text {
                    id: Some(id),
                    font,
                    force_font_mode,
                    size,
                    text_transform,
                    max_width,
                    overflow_mode,
                    wrap_mode,
                    line_clamp,
                    reserve_width_ch,
                    line_height,
                    scale_x,
                    scale_y,
                    ..
                } = node
                else {
                    return;
                };
                out.insert(
                    id.clone(),
                    AuthoredTextLayoutConfig {
                        font: font.clone(),
                        force_font_mode: force_font_mode.clone(),
                        size: *size,
                        transform: text_transform.clone(),
                        max_width: *max_width,
                        overflow_mode: *overflow_mode,
                        wrap_mode: *wrap_mode,
                        line_clamp: *line_clamp,
                        reserve_width_ch: *reserve_width_ch,
                        line_height: *line_height,
                        scale_x: *scale_x,
                        scale_y: *scale_y,
                    },
                );
            });
        }
    }
    out
}

fn measure_text_layout(
    mod_source: Option<&std::path::Path>,
    default_font: Option<&str>,
    font_policy_caps: FontPolicyCapabilities,
    content: &str,
    runtime_font: Option<&str>,
    cfg: AuthoredTextLayoutConfig,
) -> LayoutTextMeasurement {
    let resolved_font = resolve_text_font_spec_with_capabilities(
        runtime_font.or(cfg.font.as_deref()),
        cfg.force_font_mode.as_deref(),
        cfg.size,
        font_policy_caps,
        default_font,
    )
    .unwrap_or_else(|| "-".to_string());
    let layout_font = if resolved_font == "-" {
        None
    } else {
        Some(resolved_font.as_str())
    };

    let fit = text_sprite_dimensions(
        mod_source,
        content,
        layout_font,
        Color::White,
        Color::Black,
        &cfg.transform,
        cfg.max_width,
        cfg.overflow_mode,
        cfg.wrap_mode,
        cfg.line_clamp,
        cfg.reserve_width_ch,
        cfg.line_height,
        cfg.scale_x,
        cfg.scale_y,
    );
    let intrinsic = text_sprite_dimensions(
        mod_source,
        content,
        layout_font,
        Color::White,
        Color::Black,
        &cfg.transform,
        None,
        TextOverflowMode::Clip,
        TextWrapMode::None,
        None,
        None,
        cfg.line_height,
        cfg.scale_x,
        cfg.scale_y,
    );
    let unclamped = text_sprite_dimensions(
        mod_source,
        content,
        layout_font,
        Color::White,
        Color::Black,
        &cfg.transform,
        cfg.max_width,
        cfg.overflow_mode,
        cfg.wrap_mode,
        None,
        cfg.reserve_width_ch,
        cfg.line_height,
        cfg.scale_x,
        cfg.scale_y,
    );

    let mut states = Vec::new();
    if matches!(cfg.wrap_mode, TextWrapMode::Word | TextWrapMode::Char) && fit.1 > intrinsic.1 {
        states.push("wrap");
    }
    if cfg.max_width.is_some() && matches!(cfg.wrap_mode, TextWrapMode::None) && intrinsic.0 > fit.0
    {
        states.push(match cfg.overflow_mode {
            TextOverflowMode::Clip => "clip",
            TextOverflowMode::Ellipsis => "ellipsis",
        });
    }
    if cfg.line_clamp.is_some() && fit.1 < unclamped.1 {
        states.push("clamp");
    }
    if cfg.reserve_width_ch.is_some() && fit.0 > intrinsic.0 {
        states.push("reserve");
    }

    let mut constraints = Vec::new();
    if let Some(max_width) = cfg.max_width {
        constraints.push(format!("mw:{max_width}"));
    }
    if let Some(line_clamp) = cfg.line_clamp {
        constraints.push(format!("lc:{line_clamp}"));
    }
    if let Some(reserve_width_ch) = cfg.reserve_width_ch {
        constraints.push(format!("rw:{reserve_width_ch}"));
    }
    if !matches!(cfg.wrap_mode, TextWrapMode::None) {
        constraints.push(format!(
            "wrap:{}",
            match cfg.wrap_mode {
                TextWrapMode::None => "none",
                TextWrapMode::Word => "word",
                TextWrapMode::Char => "char",
            }
        ));
    }

    LayoutTextMeasurement {
        font: resolved_font,
        fit_w: fit.0,
        fit_h: fit.1,
        intrinsic_w: intrinsic.0,
        intrinsic_h: intrinsic.1,
        constraints: if constraints.is_empty() {
            "-".to_string()
        } else {
            constraints.join(" ")
        },
        status: if states.is_empty() {
            "ok".to_string()
        } else {
            states.join("+")
        },
    }
}

fn runtime_font_policy_capabilities(settings: &RuntimeSettings) -> FontPolicyCapabilities {
    // RuntimeSettings::prefers_raster_fonts() is capability-first and retains
    // legacy compatibility fallback internally.
    FontPolicyCapabilities::new(settings.prefers_raster_fonts())
}

fn apply_perf_hud(world: &mut World) {
    use crate::rasterizer::generic::{generic_dimensions, rasterize_generic};
    use engine_core::scene::sprite::TextTransform;
    use std::fmt::Write;

    let fps_val = world
        .get::<crate::debug_features::FpsCounter>()
        .map(|counter| counter.fps.round() as u32);
    let proc_stats = world.get::<crate::debug_features::ProcessStats>().copied();

    thread_local! {
        static HUD_STR: RefCell<String> = RefCell::new(String::with_capacity(64));
    }

    HUD_STR.with(|cell| {
        let hud_text = &mut *cell.borrow_mut();
        hud_text.clear();
        if let Some(fps) = fps_val {
            let _ = write!(hud_text, "{fps} FPS");
        }
        if let Some(stats) = &proc_stats {
            if !hud_text.is_empty() {
                hud_text.push_str("  ");
            }
            let _ = write!(
                hud_text,
                "{:.0}% CPU  {:.1}MB",
                stats.cpu_percent, stats.rss_mb
            );
        }
        if hud_text.is_empty() {
            return;
        }

        let hud_scale_factor = world
            .runtime_settings()
            .and_then(|settings| {
                let output_dimensions = world.output_dimensions().unwrap_or((80, 24));
                let layout = world
                    .scene_runtime()
                    .map(|runtime| {
                        crate::runtime_settings::buffer_layout_for_scene(
                            settings,
                            runtime.scene(),
                            output_dimensions.0,
                            output_dimensions.1,
                        )
                    })
                    .unwrap_or_else(|| {
                        settings.buffer_layout(output_dimensions.0, output_dimensions.1)
                    });
                Some(
                    ((layout.ui_width.max(1) as f32 / layout.world_width.max(1) as f32).round()
                        as u16)
                        .max(1),
                )
            })
            .unwrap_or(1);
        let Some(buffer) = world.get_mut::<Buffer>() else {
            return;
        };
        let hud_scale = 2u16.saturating_mul(hud_scale_factor).min(8);
        let (text_w, _) = generic_dimensions(hud_text, hud_scale);
        let x = buffer.width.saturating_sub(text_w);
        let green = Color::Rgb {
            r: 0,
            g: 255,
            b: 80,
        };
        rasterize_generic(
            hud_text,
            hud_scale,
            green,
            x,
            0,
            buffer,
            &TextTransform::None,
        );
    });
}

fn render_size_label(size: RenderSize) -> String {
    match size {
        RenderSize::Fixed { width, height } => format!("{width}x{height}"),
        RenderSize::MatchOutput => "match-output".to_string(),
        RenderSize::FitWidth { width } => format!("{width}x~"),
    }
}

fn format_render_info(settings: Option<&RuntimeSettings>) -> String {
    let Some(settings) = settings else {
        return "render: unavailable".to_string();
    };

    let policy = match settings.presentation_policy {
        PresentationPolicy::Fit => "fit",
        PresentationPolicy::Stretch => "stretch",
        PresentationPolicy::Strict => "strict",
    };

    let world = render_size_label(settings.world_render_size);
    let ui = settings
        .ui_render_size
        .map(render_size_label)
        .unwrap_or_else(|| "world".to_string());
    let ui_layout = settings
        .ui_layout_size
        .map(render_size_label)
        .unwrap_or_else(|| "ui".to_string());

    format!("world:{world} ui:{ui} layout:{ui_layout} ({policy})")
}

#[cfg(test)]
mod tests {
    use super::{present_active_frame, runtime_font_policy_capabilities};
    use crate::buffer::Buffer;
    use engine_core::color::Color;
    use engine_runtime::{RuntimeRenderCapabilities, RuntimeSettings, TextPresentationKind};
    use engine_render::{
        FrameSubmission, OverlayData, RenderBackendKind, RenderError, RendererBackend,
        VectorOverlay,
    };

    #[derive(Default)]
    struct RendererProbe {
        backend_kind: RenderBackendKind,
        software_calls: usize,
        submit_calls: usize,
        submit_ok: bool,
    }

    impl RendererBackend for RendererProbe {
        fn present_frame(&mut self, _buffer: &Buffer) {
            self.software_calls += 1;
        }

        fn backend_kind(&self) -> RenderBackendKind {
            self.backend_kind
        }

        fn submit_frame(&mut self, _submission: &FrameSubmission) -> Result<(), RenderError> {
            self.submit_calls += 1;
            if self.submit_ok {
                Ok(())
            } else {
                Err(RenderError::PresentFailed("probe".to_string()))
            }
        }

        fn present_overlay(&mut self, _overlay: &OverlayData) {}

        fn present_vectors(&mut self, _vectors: &VectorOverlay) {}

        fn output_size(&self) -> (u16, u16) {
            (160, 90)
        }

        fn clear(&mut self) -> Result<(), RenderError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), RenderError> {
            Ok(())
        }
    }

    #[test]
    fn software_backend_uses_software_present_path() {
        let mut renderer = RendererProbe::default();
        let mut buffer = Buffer::new(2, 2);
        buffer.fill(Color::Black);

        present_active_frame(&mut renderer, &buffer);

        assert_eq!(renderer.software_calls, 1);
        assert_eq!(renderer.submit_calls, 0);
    }

    #[test]
    fn hardware_present_failure_falls_back_to_software_frame() {
        let mut renderer = RendererProbe {
            backend_kind: RenderBackendKind::Hardware,
            ..RendererProbe::default()
        };
        let mut buffer = Buffer::new(2, 2);
        buffer.fill(Color::Black);

        present_active_frame(&mut renderer, &buffer);

        assert_eq!(renderer.submit_calls, 1);
        assert_eq!(renderer.software_calls, 1);
    }

    #[test]
    fn hardware_present_success_does_not_fallback_to_software_frame() {
        let mut renderer = RendererProbe {
            backend_kind: RenderBackendKind::Hardware,
            submit_ok: true,
            ..RendererProbe::default()
        };
        let mut buffer = Buffer::new(2, 2);
        buffer.fill(Color::Black);

        present_active_frame(&mut renderer, &buffer);

        assert_eq!(renderer.submit_calls, 1);
        assert_eq!(renderer.software_calls, 0);
    }

    #[test]
    fn font_policy_uses_capability_descriptor_when_available() {
        let mut settings = RuntimeSettings::default();
        settings.set_render_capabilities(RuntimeRenderCapabilities::hardware_presenter());

        let caps = runtime_font_policy_capabilities(&settings);
        assert!(caps.prefer_raster_named_fonts);
    }

    #[test]
    fn font_policy_respects_capability_text_presentation() {
        let mut settings = RuntimeSettings::default();
        let mut capabilities = RuntimeRenderCapabilities::software_presenter();
        capabilities.text_presentation = TextPresentationKind::CellGlyphs;
        settings.set_render_capabilities(capabilities);

        let caps = runtime_font_policy_capabilities(&settings);
        assert!(!caps.prefer_raster_named_fonts);
    }
}
