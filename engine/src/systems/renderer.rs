//! Backend-agnostic presentation helpers used by the engine runtime.

use crate::buffer::Buffer;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_animation::SceneStage;
use engine_core::color::Color;
use engine_core::logging;
use engine_debug::DebugOverlayMode;
use engine_render::{OverlayData, OverlayLine};
use engine_runtime::{PresentationPolicy, RenderSize, RuntimeSettings};
use std::cell::RefCell;

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
            renderer.present_frame(buffer);
        }
    }

    if let Some(buffer) = world.get_mut::<Buffer>() {
        buffer.swap();
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
    }

    Some(OverlayData {
        lines,
        dim_scene: true,
    })
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
                    .unwrap_or_else(|| settings.buffer_layout(output_dimensions.0, output_dimensions.1));
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
        rasterize_generic(hud_text, hud_scale, green, x, 0, buffer, &TextTransform::None);
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
