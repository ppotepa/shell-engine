use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::debug_features::{DebugFeatures, DebugOverlayMode};
use crate::debug_log::DebugLogBuffer;
use crate::runtime_settings::VirtualPolicy;
use crate::services::EngineWorldAccess;
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;
use crossterm::{cursor, execute, queue, style, terminal};
use engine_core::logging;
use std::cell::RefCell;
use std::io::{self, Write};

pub struct TerminalRenderer {
    stdout: io::Stdout,
}

thread_local! {
    static DIFF_SCRATCH: RefCell<Vec<(u16, u16, char, style::Color, style::Color)>> =
        RefCell::new(Vec::with_capacity(4096));
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self { stdout })
    }

    /// Paint the entire screen true-black before the first game frame.
    pub fn clear_black(&mut self) -> io::Result<()> {
        let (w, h) = terminal::size()?;
        let bg = style::Color::Rgb { r: 0, g: 0, b: 0 };
        let fg = style::Color::Rgb { r: 0, g: 0, b: 0 };
        queue!(
            self.stdout,
            style::SetForegroundColor(fg),
            style::SetBackgroundColor(bg)
        )?;
        for y in 0..h {
            queue!(self.stdout, cursor::MoveTo(0, y))?;
            for _ in 0..w {
                queue!(self.stdout, style::Print(' '))?;
            }
        }
        self.stdout.flush()
    }

    /// Hard refresh terminal surface before first frame.
    pub fn reset_console(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        self.stdout.flush()
    }

    pub fn shutdown(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()
    }
}

/// Flush only changed pixels to the terminal via crossterm.
pub fn renderer_system(world: &mut World) {
    if should_use_virtual_buffer(world) {
        present_virtual_to_output(world);
    }

    // Last-good-frame fallback: when script errors are present in debug mode,
    // restore the last flushed frame (front) into the back buffer so the
    // compositor-cleared blank is replaced with the last visible content.
    // The debug overlay will render on top immediately after.
    let has_script_errors = world
        .get::<DebugLogBuffer>()
        .map(|log| log.has_errors)
        .unwrap_or(false);
    let debug_enabled = world
        .get::<DebugFeatures>()
        .map(|d| d.enabled)
        .unwrap_or(false);
    if has_script_errors && debug_enabled {
        if let Some(buf) = world.buffer_mut() {
            buf.restore_front_to_back();
        }
    }

    apply_debug_overlay(world);

    // Fill the reusable scratch Vec with raw diff data (no per-frame allocation).
    DIFF_SCRATCH.with(|scratch| {
        let mut diffs = scratch.borrow_mut();
        diffs.clear();
        if let Some(buf) = world.buffer() {
            buf.diff_into(&mut diffs);
        }
    });

    let is_empty = DIFF_SCRATCH.with(|s| s.borrow().is_empty());
    if is_empty {
        if let Some(buf) = world.buffer_mut() {
            buf.swap();
        }
        return;
    }

    if let Some(renderer) = world.renderer_mut() {
        DIFF_SCRATCH.with(|scratch| {
            flush_batched(&mut renderer.stdout, &scratch.borrow());
        });
    }

    if let Some(buf) = world.buffer_mut() {
        buf.swap();
    }
}

fn apply_debug_overlay(world: &mut World) {
    let Some(debug) = world.get::<DebugFeatures>().copied() else {
        return;
    };
    if !debug.enabled || !debug.overlay_visible {
        return;
    }

    let stats_data = if matches!(debug.overlay_mode, DebugOverlayMode::Stats) {
        let scene_id = world
            .scene_runtime()
            .map(|runtime| runtime.scene().id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let stage_info = world
            .get::<Animator>()
            .map(|anim| {
                let stage = match anim.stage {
                    SceneStage::OnEnter => "on_enter",
                    SceneStage::OnIdle => "on_idle",
                    SceneStage::OnLeave => "on_leave",
                    SceneStage::Done => "done",
                };
                format!("stage: {} ({:.1}s)", stage, anim.elapsed_ms as f64 / 1000.0)
            })
            .unwrap_or_else(|| "stage: -".to_string());
        let virtual_info = format_virtual_info(world);
        let script_errors: Vec<String> = world
            .get::<DebugLogBuffer>()
            .map(|log| log.recent(usize::MAX).iter().map(|entry| entry.display_line()).collect())
            .unwrap_or_default();
        Some((scene_id, stage_info, virtual_info, script_errors))
    } else {
        None
    };

    let Some(buf) = world.buffer_mut() else {
        return;
    };

    match debug.overlay_mode {
        DebugOverlayMode::Stats => {
            let Some((scene_id, stage_info, virtual_info, mut script_errors)) = stats_data else {
                return;
            };
            let max_errors = buf.height.saturating_sub(4) as usize;
            if script_errors.len() > max_errors {
                script_errors = script_errors.split_off(script_errors.len() - max_errors);
            }
            apply_stats_overlay(buf, &scene_id, &stage_info, &virtual_info, &script_errors);
        }
        DebugOverlayMode::Logs => {
            apply_logs_overlay(buf);
        }
    }
}

fn apply_stats_overlay(
    buf: &mut Buffer,
    scene_id: &str,
    stage_info: &str,
    virtual_info: &str,
    script_errors: &[String],
) {
    let mut lines = vec![
        "DEBUG FEATURE MODE  [F1 overlay] [F3 prev] [F4 next]".to_string(),
        format!("scene: {scene_id}"),
        stage_info.to_string(),
        virtual_info.to_string(),
    ];
    lines.extend(script_errors.iter().cloned());

    let fg = style::Color::White;
    let bg = style::Color::DarkGrey;
    for (row, line) in lines.iter().enumerate() {
        let y = row as u16;
        if y >= buf.height {
            break;
        }
        // Error/warn lines get a distinct background.
        let line_bg = if line.starts_with("[ERR") {
            style::Color::DarkRed
        } else if line.starts_with("[WARN") {
            style::Color::Rgb { r: 100, g: 80, b: 0 }
        } else {
            bg
        };
        for x in 0..buf.width {
            buf.set(x, y, ' ', fg, line_bg);
        }
        for (x, ch) in line.chars().enumerate() {
            let x = x as u16;
            if x >= buf.width {
                break;
            }
            buf.set(x, y, ch, fg, line_bg);
        }
    }
}

fn format_virtual_info(world: &World) -> String {
    let Some(settings) = world.runtime_settings() else {
        return "virtual: unavailable".to_string();
    };
    if !settings.use_virtual_buffer {
        return "virtual: disabled".to_string();
    }

    let policy = match settings.virtual_policy {
        VirtualPolicy::Strict => "strict",
        VirtualPolicy::Fit => "fit",
    };

    if let Some(vbuf) = world.virtual_buffer() {
        return format!("virtual: {}x{} ({policy})", vbuf.0.width, vbuf.0.height);
    }

    if settings.virtual_size_max_available {
        "virtual: max-available".to_string()
    } else {
        format!(
            "virtual: {}x{} ({policy})",
            settings.virtual_width, settings.virtual_height
        )
    }
}

fn apply_logs_overlay(buf: &mut Buffer) {
    let header = "LOG OVERLAY  [~ close] [F1 stats]";
    let height = buf.height as usize;
    
    // Get recent log entries
    let logs = if height > 1 {
        logging::tail_recent(height.saturating_sub(1))
    } else {
        Vec::new()
    };

    // Render header
    let fg = style::Color::White;
    let bg = style::Color::DarkGrey;
    
    for x in 0..buf.width {
        buf.set(x, 0, ' ', fg, bg);
    }
    for (x, ch) in header.chars().enumerate() {
        let x = x as u16;
        if x >= buf.width {
            break;
        }
        buf.set(x, 0, ch, fg, bg);
    }

    // Render log lines
    if logs.is_empty() {
        let empty_msg = "  (no log entries)";
        let y = 1u16;
        if y < buf.height {
            for x in 0..buf.width {
                buf.set(x, y, ' ', fg, bg);
            }
            for (x, ch) in empty_msg.chars().enumerate() {
                let x = x as u16;
                if x >= buf.width {
                    break;
                }
                buf.set(x, y, ch, fg, bg);
            }
        }
    } else {
        for (row, log_line) in logs.iter().enumerate() {
            let y = (row + 1) as u16;
            if y >= buf.height {
                break;
            }

            // Determine color based on level
            let line_fg = match log_line.level {
                "WARN " => style::Color::Yellow,
                "ERROR" => style::Color::Red,
                _ => style::Color::White,
            };

            // Format the line
            let formatted = format!(
                "[{}] {} | {}",
                log_line.level, log_line.target, log_line.message
            );

            // Clear row
            for x in 0..buf.width {
                buf.set(x, y, ' ', line_fg, bg);
            }

            // Write the formatted line, clipping to width
            for (x, ch) in formatted.chars().enumerate() {
                let x = x as u16;
                if x >= buf.width {
                    break;
                }
                buf.set(x, y, ch, line_fg, bg);
            }
        }
    }
}

/// Resolve a color for output — `Color::Reset` is mapped to true black so that
/// terminal theme colours never bleed through transparent/unset pixels.
#[inline]
fn resolve_color(c: style::Color) -> style::Color {
    match c {
        style::Color::Reset => crate::buffer::TRUE_BLACK,
        other => other,
    }
}

#[inline]
#[allow(dead_code)]
fn to_ct(c: crossterm::style::Color) -> style::Color {
    c
}

fn should_use_virtual_buffer(world: &World) -> bool {
    world
        .runtime_settings()
        .map(|s| s.use_virtual_buffer)
        .unwrap_or(false)
        && world.virtual_buffer().is_some()
}

fn present_virtual_to_output(world: &mut World) {
    let settings = world.runtime_settings().cloned().unwrap_or_default();
    let virtual_snapshot = world.virtual_buffer().map(|v| v.0.clone());
    let Some(virtual_buf) = virtual_snapshot else {
        return;
    };
    let Some(output_buf) = world.buffer_mut() else {
        return;
    };

    output_buf.fill(TRUE_BLACK);
    if virtual_buf.width == 0 || virtual_buf.height == 0 {
        return;
    }
    if output_buf.width == 0 || output_buf.height == 0 {
        return;
    }

    let viewport = compute_viewport(
        output_buf.width,
        output_buf.height,
        virtual_buf.width,
        virtual_buf.height,
        settings.virtual_policy,
    );

    for oy in 0..viewport.height {
        for ox in 0..viewport.width {
            let (sx, sy) = match settings.virtual_policy {
                VirtualPolicy::Strict => (
                    viewport.src_offset_x.saturating_add(ox),
                    viewport.src_offset_y.saturating_add(oy),
                ),
                VirtualPolicy::Fit => sample_fit_source(
                    ox,
                    oy,
                    viewport.width,
                    viewport.height,
                    virtual_buf.width,
                    virtual_buf.height,
                ),
            };
            let Some(cell) = virtual_buf.get(sx, sy) else {
                continue;
            };
            let dx = viewport.dst_offset_x.saturating_add(ox);
            let dy = viewport.dst_offset_y.saturating_add(oy);
            copy_cell(output_buf, dx, dy, cell);
        }
    }
}

#[derive(Clone, Copy)]
struct Viewport {
    width: u16,
    height: u16,
    dst_offset_x: u16,
    dst_offset_y: u16,
    src_offset_x: u16,
    src_offset_y: u16,
}

fn compute_viewport(
    output_w: u16,
    output_h: u16,
    virtual_w: u16,
    virtual_h: u16,
    policy: VirtualPolicy,
) -> Viewport {
    match policy {
        VirtualPolicy::Strict => {
            let width = output_w.min(virtual_w);
            let height = output_h.min(virtual_h);
            Viewport {
                width,
                height,
                dst_offset_x: (output_w.saturating_sub(width)) / 2,
                dst_offset_y: (output_h.saturating_sub(height)) / 2,
                src_offset_x: (virtual_w.saturating_sub(width)) / 2,
                src_offset_y: (virtual_h.saturating_sub(height)) / 2,
            }
        }
        VirtualPolicy::Fit => {
            let sw = output_w as f32 / virtual_w.max(1) as f32;
            let sh = output_h as f32 / virtual_h.max(1) as f32;
            let mut scale = sw.min(sh);
            if scale >= 1.0 {
                scale = scale.floor().max(1.0);
            } else {
                scale = scale.max(0.01);
            }
            let width = ((virtual_w as f32 * scale).floor() as u16).clamp(1, output_w.max(1));
            let height = ((virtual_h as f32 * scale).floor() as u16).clamp(1, output_h.max(1));
            Viewport {
                width,
                height,
                dst_offset_x: (output_w.saturating_sub(width)) / 2,
                dst_offset_y: (output_h.saturating_sub(height)) / 2,
                src_offset_x: 0,
                src_offset_y: 0,
            }
        }
    }
}

fn sample_fit_source(
    ox: u16,
    oy: u16,
    viewport_w: u16,
    viewport_h: u16,
    virtual_w: u16,
    virtual_h: u16,
) -> (u16, u16) {
    let sx = ((ox as u32).saturating_mul(virtual_w as u32) / viewport_w.max(1) as u32)
        .min(virtual_w.saturating_sub(1) as u32) as u16;
    let sy = ((oy as u32).saturating_mul(virtual_h as u32) / viewport_h.max(1) as u32)
        .min(virtual_h.saturating_sub(1) as u32) as u16;
    (sx, sy)
}

/// Batch-flush diffs to the terminal.
///
/// Consecutive cells on the same row sharing the same fg+bg colour are merged
/// into a single `MoveTo + SetFg + SetBg + Print(run)` command, reducing the
/// number of terminal I/O operations from O(cells) toward O(colour-runs).
/// Diffs arrive in row-major order from `Buffer::diff_into`, so no sort is needed.
/// Raw (pre-resolve) colours are accepted; `Color::Reset` is mapped to true black here.
fn flush_batched(stdout: &mut io::Stdout, diffs: &[(u16, u16, char, style::Color, style::Color)]) {
    if diffs.is_empty() {
        return;
    }

    let mut run = String::new();
    let (mut rx, mut ry, _, raw_fg0, raw_bg0) = diffs[0];
    let (mut rfg, mut rbg) = (resolve_color(raw_fg0), resolve_color(raw_bg0));
    run.push(diffs[0].2);
    let mut run_len: u16 = 1;

    for &(x, y, ch, raw_fg, raw_bg) in &diffs[1..] {
        let fg = resolve_color(raw_fg);
        let bg = resolve_color(raw_bg);
        // O(1): compare cached length instead of scanning the string.
        let continues = y == ry && x == rx + run_len && fg == rfg && bg == rbg;
        if continues {
            run.push(ch);
            run_len += 1;
        } else {
            let _ = queue!(
                stdout,
                cursor::MoveTo(rx, ry),
                style::SetForegroundColor(rfg),
                style::SetBackgroundColor(rbg),
                style::Print(&run)
            );
            run.clear();
            run.push(ch);
            run_len = 1;
            rx = x;
            ry = y;
            rfg = fg;
            rbg = bg;
        }
    }

    let _ = queue!(
        stdout,
        cursor::MoveTo(rx, ry),
        style::SetForegroundColor(rfg),
        style::SetBackgroundColor(rbg),
        style::Print(&run)
    );
    let _ = stdout.flush();
}

fn copy_cell(dst: &mut Buffer, x: u16, y: u16, src: &Cell) {
    dst.set(x, y, src.symbol, src.fg, src.bg);
}
