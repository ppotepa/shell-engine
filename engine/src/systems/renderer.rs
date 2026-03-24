use crate::buffer::{Buffer, Cell, TRUE_BLACK, VirtualBuffer};
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
    stdout: io::BufWriter<io::Stdout>,
}

thread_local! {
    static DIFF_SCRATCH: RefCell<Vec<(u16, u16, char, style::Color, style::Color)>> =
        RefCell::new(Vec::with_capacity(4096));
    /// Reusable run buffer for RLE batching — avoids per-frame heap allocation.
    static RUN_BUF: RefCell<String> = RefCell::new(String::with_capacity(256));
    /// #3 opt-term-ansibuf: accumulate all ANSI into a contiguous buffer, single write_all per frame.
    static ANSI_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(65536));
    /// #13 opt-present-skipstatic: last seen VirtualBuffer write_count to skip redundant presents.
    static LAST_VBUF_WRITE_COUNT: RefCell<u64> = RefCell::new(u64::MAX);
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::BufWriter::with_capacity(65536, io::stdout());
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
    apply_perf_hud(world);

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
        let timings_info = world
            .get::<crate::debug_features::SystemTimings>()
            .map(|st| {
                format!(
                    "beh:{:.0} comp:{:.0} pfx:{:.0} rend:{:.0} us",
                    st.behavior_us, st.compositor_us, st.postfx_us, st.renderer_us
                )
            })
            .unwrap_or_default();
        let script_errors: Vec<String> = world
            .get::<DebugLogBuffer>()
            .map(|log| log.recent(usize::MAX).iter().map(|entry| entry.display_line()).collect())
            .unwrap_or_default();
        Some((scene_id, stage_info, virtual_info, timings_info, script_errors))
    } else {
        None
    };

    let Some(buf) = world.buffer_mut() else {
        return;
    };

    match debug.overlay_mode {
        DebugOverlayMode::Stats => {
            let Some((scene_id, stage_info, virtual_info, timings_info, mut script_errors)) = stats_data else {
                return;
            };
            let max_errors = buf.height.saturating_sub(5) as usize;
            if script_errors.len() > max_errors {
                script_errors = script_errors.split_off(script_errors.len() - max_errors);
            }
            apply_stats_overlay(buf, &scene_id, &stage_info, &virtual_info, &timings_info, &script_errors);
        }
        DebugOverlayMode::Logs => {
            apply_logs_overlay(buf);
        }
    }
}

/// Always-on performance HUD: FPS / CPU% / MEM in the top-right corner.
fn apply_perf_hud(world: &mut World) {
    use crate::rasterizer::generic::rasterize_generic_half;
    use engine_core::scene::sprite::TextTransform;

    let fps_val = world
        .get::<crate::debug_features::FpsCounter>()
        .map(|c| c.fps.round() as u32);
    let proc_stats = world
        .get::<crate::debug_features::ProcessStats>()
        .copied();

    let mut parts: Vec<String> = Vec::new();
    if let Some(fps) = fps_val {
        parts.push(format!("{fps} FPS"));
    }
    if let Some(ps) = &proc_stats {
        parts.push(format!("{:.0}% CPU", ps.cpu_percent));
        parts.push(format!("{:.1}MB", ps.rss_mb));
    }
    let hud_text = parts.join("  ");
    if hud_text.is_empty() {
        return;
    }

    let Some(buf) = world.buffer_mut() else {
        return;
    };
    // generic:half font advances 6 cols per char (5-wide glyph + 1 gap)
    let text_w = hud_text.len() as u16 * 6;
    let x = buf.width.saturating_sub(text_w);
    let green = style::Color::Rgb { r: 0, g: 255, b: 80 };
    rasterize_generic_half(&hud_text, green, x, 0, buf, &TextTransform::None);
}

fn apply_stats_overlay(
    buf: &mut Buffer,
    scene_id: &str,
    stage_info: &str,
    virtual_info: &str,
    timings_info: &str,
    script_errors: &[String],
) {
    let mut lines = vec![
        "DEBUG FEATURE MODE  [F1 overlay] [F3 prev] [F4 next]".to_string(),
        format!("scene: {scene_id}"),
        stage_info.to_string(),
        virtual_info.to_string(),
        timings_info.to_string(),
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
    let Some(settings) = world.runtime_settings().cloned() else {
        return;
    };
    world.with_ref_and_mut::<VirtualBuffer, Buffer, _, _>(|vbuf, output_buf| {
        let virtual_buf = &vbuf.0;

        // #13 opt-present-skipstatic: skip when virtual buffer hasn't changed.
        let wc = virtual_buf.write_count;
        let skip = LAST_VBUF_WRITE_COUNT.with(|c| {
            let prev = *c.borrow();
            *c.borrow_mut() = wc;
            prev == wc
        });
        if skip {
            return;
        }

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

        // #14 opt-present-fitlut: use precomputed LUT for Fit mode.
        let use_fit_lut = matches!(settings.virtual_policy, VirtualPolicy::Fit);

        if use_fit_lut {
            FIT_LUT_CACHE.with(|cache| {
                let mut lut_opt = cache.borrow_mut();
                if lut_opt.as_ref().map_or(true, |l| !l.matches(
                    viewport.width, viewport.height, virtual_buf.width, virtual_buf.height
                )) {
                    *lut_opt = Some(FitLut::build(
                        viewport.width, viewport.height, virtual_buf.width, virtual_buf.height
                    ));
                }
                let lut = lut_opt.as_ref().unwrap();
                for oy in 0..viewport.height {
                    let sy = lut.y_map[oy as usize];
                    for ox in 0..viewport.width {
                        let sx = lut.x_map[ox as usize];
                        let Some(cell) = virtual_buf.get(sx, sy) else { continue; };
                        let dx = viewport.dst_offset_x.saturating_add(ox);
                        let dy = viewport.dst_offset_y.saturating_add(oy);
                        copy_cell(output_buf, dx, dy, cell);
                    }
                }
            });
        } else {
            for oy in 0..viewport.height {
                for ox in 0..viewport.width {
                    let (sx, sy) = (
                        viewport.src_offset_x.saturating_add(ox),
                        viewport.src_offset_y.saturating_add(oy),
                    );
                    let Some(cell) = virtual_buf.get(sx, sy) else { continue; };
                    let dx = viewport.dst_offset_x.saturating_add(ox);
                    let dy = viewport.dst_offset_y.saturating_add(oy);
                    copy_cell(output_buf, dx, dy, cell);
                }
            }
        }
    });
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

/// #14 opt-present-fitlut: precomputed coordinate LUT for Fit mode.
struct FitLut {
    x_map: Vec<u16>,
    y_map: Vec<u16>,
    viewport_w: u16,
    viewport_h: u16,
    virtual_w: u16,
    virtual_h: u16,
}

impl FitLut {
    fn build(viewport_w: u16, viewport_h: u16, virtual_w: u16, virtual_h: u16) -> Self {
        let vw = viewport_w.max(1) as u32;
        let vh = viewport_h.max(1) as u32;
        let vmax_x = virtual_w.saturating_sub(1) as u32;
        let vmax_y = virtual_h.saturating_sub(1) as u32;
        let x_map: Vec<u16> = (0..viewport_w)
            .map(|ox| ((ox as u32 * virtual_w as u32) / vw).min(vmax_x) as u16)
            .collect();
        let y_map: Vec<u16> = (0..viewport_h)
            .map(|oy| ((oy as u32 * virtual_h as u32) / vh).min(vmax_y) as u16)
            .collect();
        Self { x_map, y_map, viewport_w, viewport_h, virtual_w, virtual_h }
    }

    fn matches(&self, vw: u16, vh: u16, virt_w: u16, virt_h: u16) -> bool {
        self.viewport_w == vw && self.viewport_h == vh
            && self.virtual_w == virt_w && self.virtual_h == virt_h
    }
}

thread_local! {
    static FIT_LUT_CACHE: RefCell<Option<FitLut>> = RefCell::new(None);
}

/// Batch-flush diffs to the terminal.
///
/// Consecutive cells on the same row sharing the same fg+bg colour are merged
/// into a single `MoveTo + SetFg + SetBg + Print(run)` command, reducing the
/// number of terminal I/O operations from O(cells) toward O(colour-runs).
/// Diffs arrive in row-major order from `Buffer::diff_into`, so no sort is needed.
/// Raw (pre-resolve) colours are accepted; `Color::Reset` is mapped to true black here.
fn flush_batched(stdout: &mut io::BufWriter<io::Stdout>, diffs: &[(u16, u16, char, style::Color, style::Color)]) {
    if diffs.is_empty() {
        return;
    }

    // #3 opt-term-ansibuf: write all ANSI into Vec<u8>, then single write_all.
    // #2 opt-term-colorstate: track last-emitted fg/bg to skip redundant SetColor commands.
    ANSI_BUF.with(|ansi_cell| {
        RUN_BUF.with(|run_cell| {
            let mut ansi = ansi_cell.borrow_mut();
            let mut run = run_cell.borrow_mut();
            ansi.clear();
            run.clear();

            let (mut rx, mut ry, _, raw_fg0, raw_bg0) = diffs[0];
            let (mut rfg, mut rbg) = (resolve_color(raw_fg0), resolve_color(raw_bg0));
            run.push(diffs[0].2);
            let mut run_len: u16 = 1;
            let mut cursor_x = u16::MAX;
            let mut cursor_y = u16::MAX;
            let mut active_fg = style::Color::Reset;
            let mut active_bg = style::Color::Reset;

            // Inline helper: emit a queued run into the ANSI buffer.
            macro_rules! emit_run {
                () => {
                    if cursor_x != rx || cursor_y != ry {
                        let _ = queue!(&mut *ansi, cursor::MoveTo(rx, ry));
                    }
                    if rfg != active_fg {
                        let _ = queue!(&mut *ansi, style::SetForegroundColor(rfg));
                        active_fg = rfg;
                    }
                    if rbg != active_bg {
                        let _ = queue!(&mut *ansi, style::SetBackgroundColor(rbg));
                        active_bg = rbg;
                    }
                    let _ = queue!(&mut *ansi, style::Print(&*run));
                    cursor_x = rx + run_len;
                    cursor_y = ry;
                };
            }

            for &(x, y, ch, raw_fg, raw_bg) in &diffs[1..] {
                let fg = resolve_color(raw_fg);
                let bg = resolve_color(raw_bg);
                let continues = y == ry && x == rx + run_len && fg == rfg && bg == rbg;
                if continues {
                    run.push(ch);
                    run_len += 1;
                } else {
                    emit_run!();
                    run.clear();
                    run.push(ch);
                    run_len = 1;
                    rx = x;
                    ry = y;
                    rfg = fg;
                    rbg = bg;
                }
            }

            emit_run!();

            let _ = stdout.write_all(&ansi);
            let _ = stdout.flush();
        });
    });
}

fn copy_cell(dst: &mut Buffer, x: u16, y: u16, src: &Cell) {
    dst.set(x, y, src.symbol, src.fg, src.bg);
}

#[cfg(test)]
mod tests {
    use super::present_virtual_to_output;
    use crate::assets::AssetRoot;
    use crate::buffer::{Buffer, TRUE_BLACK, VirtualBuffer};
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::animator::{Animator, SceneStage};
    use crate::systems::compositor::compositor_system;
    use crate::world::World;
    use std::path::PathBuf;

    #[test]
    fn shell_quest_intro_logo_survives_virtual_presentation() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf();
        let mod_root = repo_root.join("mods/shell-quest");
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_ref("00.intro.logo")
            .expect("load shell-quest intro logo");

        let mut settings = RuntimeSettings::default();
        settings.use_virtual_buffer = true;
        settings.virtual_width = 120;
        settings.virtual_height = 40;

        let mut world = World::new();
        world.register(Buffer::new(120, 40));
        world.register(VirtualBuffer::new(120, 40));
        world.register(settings);
        world.register(AssetRoot::new(mod_root));
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 300,
            stage_elapsed_ms: 300,
            scene_elapsed_ms: 300,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        compositor_system(&mut world);
        present_virtual_to_output(&mut world);

        let buffer = world.get::<Buffer>().expect("output buffer");
        let has_visible_glyph = (0..buffer.height).any(|y| {
            (0..buffer.width).any(|x| {
                let cell = buffer.get(x, y).expect("cell in bounds");
                cell.symbol != ' ' && (cell.fg != TRUE_BLACK || cell.bg != TRUE_BLACK)
            })
        });
        assert!(
            has_visible_glyph,
            "virtual presentation should preserve intro logo glyphs"
        );
    }
}
