use crate::color_convert;
use crate::provider::RendererProvider;
use crate::strategy::{AnsiBatchFlusher, AsyncDisplaySink, TerminalFlusher};
use crossterm::{cursor, execute, queue, style, terminal};
use engine_animation::SceneStage;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::logging;
use engine_core::strategy::{DiffStrategy, FullScanDiff};
use engine_debug::DebugOverlayMode;
use engine_pipeline::{DisplayFrame, DisplaySink, PipelineStrategies};
use engine_render::{OutputBackend, OverlayData, RenderError};
use engine_runtime::{compute_presentation_layout, PresentationPolicy, RuntimeSettings};
use std::cell::RefCell;
use std::io::{self, Write};

pub struct TerminalRenderer {
    stdout: io::BufWriter<io::Stdout>,
    async_sink: Option<AsyncDisplaySink>,
    flusher: Box<dyn TerminalFlusher>,
    presentation_policy: PresentationPolicy,
    presented_output: Buffer,
    pending_overlay: Option<OverlayData>,
}

thread_local! {
    static DIFF_SCRATCH: RefCell<Vec<(u16, u16, char, Color, Color)>> =
        RefCell::new(Vec::with_capacity(4096));
    /// Reusable run buffer for RLE batching — avoids per-frame heap allocation.
    static RUN_BUF: RefCell<String> = RefCell::new(String::with_capacity(256));
    /// #3 opt-term-ansibuf: accumulate all ANSI into a contiguous buffer, single write_all per frame.
    static ANSI_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(65536));
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        Self::new_with_async(false, Box::new(AnsiBatchFlusher), PresentationPolicy::Fit)
    }

    pub fn new_with_async(
        async_display: bool,
        flusher: Box<dyn TerminalFlusher>,
        presentation_policy: PresentationPolicy,
    ) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::BufWriter::with_capacity(65536, io::stdout());
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        let (output_w, output_h) = terminal::size().unwrap_or((80, 24));

        let async_sink = if async_display {
            Some(AsyncDisplaySink::new())
        } else {
            None
        };

        Ok(Self {
            stdout,
            async_sink,
            flusher,
            presentation_policy,
            presented_output: Buffer::new(output_w.max(1), output_h.max(1)),
            pending_overlay: None,
        })
    }

    /// Paint the entire screen true-black before the first game frame.
    pub fn clear_black(&mut self) -> io::Result<()> {
        let (w, h) = terminal::size()?;
        let bg = Color::Rgb { r: 0, g: 0, b: 0 };
        let fg = Color::Rgb { r: 0, g: 0, b: 0 };
        queue!(
            self.stdout,
            style::SetForegroundColor(color_convert::to_crossterm(fg)),
            style::SetBackgroundColor(color_convert::to_crossterm(bg))
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
        if let Some(sink) = &mut self.async_sink {
            sink.drain();
        }
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()
    }
}

impl OutputBackend for TerminalRenderer {
    fn present_buffer(&mut self, buffer: &Buffer) {
        let output_size = terminal::size().unwrap_or((80, 24));
        self.ensure_output_buffer(output_size.0.max(1), output_size.1.max(1));
        project_buffer_to_output(buffer, &mut self.presented_output, self.presentation_policy);

        // When overlay is active, dim the output buffer cells so the scene
        // appears darker behind the console.
        let overlay = self.pending_overlay.take();
        if overlay.as_ref().map_or(false, |o| o.dim_scene) {
            dim_output_buffer(&mut self.presented_output);
        }

        let mut diffs = Vec::new();
        self.presented_output.diff_into(&mut diffs);

        // Flush game buffer diffs first (or skip if nothing changed and no overlay).
        if !diffs.is_empty() {
            if let Some(sink) = &mut self.async_sink {
                sink.submit(DisplayFrame { diffs, frame_id: 0 });
            } else {
                self.flusher.flush(&mut self.stdout, &diffs);
            }
        }
        self.presented_output.swap();

        // Render overlay AFTER the game buffer so it always appears on top.
        if let Some(ref overlay_data) = overlay {
            self.flush_overlay(overlay_data);
        }
    }

    fn present_overlay(&mut self, overlay: &engine_render::OverlayData) {
        self.pending_overlay = Some(overlay.clone());
    }

    fn output_size(&self) -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        self.presented_output.invalidate();
        self.reset_console()
            .map_err(|e| RenderError::InitFailed(e.to_string()))?;
        self.clear_black()
            .map_err(|e| RenderError::InitFailed(e.to_string()))
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        TerminalRenderer::shutdown(self).map_err(|e| RenderError::ShutdownFailed(e.to_string()))
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

impl TerminalRenderer {
    fn ensure_output_buffer(&mut self, width: u16, height: u16) {
        if self.presented_output.width != width || self.presented_output.height != height {
            self.presented_output.resize(width, height);
            self.presented_output.invalidate();
        }
    }

    /// Render overlay lines directly to the terminal via ANSI cursor commands.
    /// Called AFTER game buffer has been flushed so overlay appears on top.
    fn flush_overlay(&mut self, overlay: &OverlayData) {
        use engine_core::markup::parse_spans;

        if overlay.is_empty() {
            return;
        }
        let (term_w, _term_h) = terminal::size().unwrap_or((80, 24));
        for (row, line) in overlay.lines.iter().enumerate() {
            let ct_bg = color_convert::to_crossterm(line.bg);
            let _ = queue!(
                self.stdout,
                cursor::MoveTo(0, row as u16),
                style::SetBackgroundColor(ct_bg)
            );

            let mut printed_cols: u16 = 0;
            let spans = parse_spans(&line.text);
            for span in spans {
                if printed_cols >= term_w {
                    break;
                }
                let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(line.fg);
                let mut chunk = String::new();
                for ch in span.text.chars() {
                    if printed_cols >= term_w {
                        break;
                    }
                    chunk.push(ch);
                    printed_cols = printed_cols.saturating_add(1);
                }
                if !chunk.is_empty() {
                    let _ = queue!(
                        self.stdout,
                        style::SetForegroundColor(color_convert::to_crossterm(span_fg)),
                        style::Print(chunk)
                    );
                }
            }

            if printed_cols < term_w {
                let pad_len = (term_w - printed_cols) as usize;
                if pad_len > 0 {
                    let _ = queue!(
                        self.stdout,
                        style::SetForegroundColor(color_convert::to_crossterm(line.fg)),
                        style::Print(" ".repeat(pad_len))
                    );
                }
            }
        }
        let _ = queue!(self.stdout, style::ResetColor);
        let _ = self.stdout.flush();
    }
}

/// Darken all cells in the output buffer by reducing RGB values.
/// Used when the debug overlay is visible to visually separate the console
/// from the game scene.
fn dim_output_buffer(buf: &mut Buffer) {
    const DIM_FACTOR: f32 = 0.35;
    let w = buf.width;
    let h = buf.height;
    for y in 0..h {
        for x in 0..w {
            if let Some(cell) = buf.get(x, y) {
                let symbol = cell.symbol;
                let fg = dim_color(cell.fg, DIM_FACTOR);
                let bg = dim_color(cell.bg, DIM_FACTOR);
                buf.set(x, y, symbol, fg, bg);
            }
        }
    }
}

fn dim_color(c: Color, factor: f32) -> Color {
    let (r, g, b) = c.to_rgb();
    Color::Rgb {
        r: (r as f32 * factor) as u8,
        g: (g as f32 * factor) as u8,
        b: (b as f32 * factor) as u8,
    }
}

/// Flush only changed pixels to the terminal via crossterm.
pub fn renderer_system<T: RendererProvider>(world: &mut T) {
    // Last-good-frame fallback: when script errors are present in debug mode,
    // restore the last flushed frame (front) into the back buffer so the
    // compositor-cleared blank is replaced with the last visible content.
    // The debug overlay will render on top immediately after.
    let has_script_errors = world.debug_log().map(|log| log.has_errors).unwrap_or(false);
    let debug_enabled = world.debug_features().map(|d| d.enabled).unwrap_or(false);
    if has_script_errors && debug_enabled {
        world.restore_front_to_back();
    }

    // Collect overlay data (rendered after present, directly to output surface).
    let overlay_data = collect_debug_overlay(world);

    apply_perf_hud(world);

    // Extract raw pointer to avoid a long-lived PipelineStrategies borrow conflicting
    // with buffer borrows taken below. Pattern mirrors layers_ptr in compositor_system.
    // SAFETY: PipelineStrategies is registered at startup and never mutated or dropped
    // during frame processing. Pointer valid for duration of render_system.
    let strats_ptr: *const PipelineStrategies = world.pipeline_strategies_ptr();
    static FALLBACK_DIFF: FullScanDiff = FullScanDiff;
    let diff_strategy: &dyn DiffStrategy = if strats_ptr.is_null() {
        &FALLBACK_DIFF
    } else {
        unsafe { (*strats_ptr).diff.as_ref() }
    };

    // Fill the reusable scratch Vec with raw diff data (no per-frame allocation).
    DIFF_SCRATCH.with(|scratch| {
        let mut diffs = scratch.borrow_mut();
        diffs.clear();
        if let Some(buf) = world.buffer() {
            diff_strategy.diff_into(buf, &mut diffs);
        }
    });

    // Store diff count on buffer for benchmark instrumentation.
    let diff_len = DIFF_SCRATCH.with(|s| s.borrow().len()) as u32;
    if let Some(buf) = world.buffer_mut() {
        buf.last_diff_count = diff_len;
    }

    let buffer_ptr: *const Buffer = world
        .buffer()
        .map(|buffer| buffer as *const Buffer)
        .unwrap_or(std::ptr::null());
    // SAFETY: VectorOverlay is a separate World resource slot, never mutated during present.
    let vectors_ptr: *const engine_render::VectorOverlay = world
        .vector_overlay()
        .map(|v| v as *const _)
        .unwrap_or(std::ptr::null());
    if !buffer_ptr.is_null() {
        if let Some(renderer) = world.renderer_mut() {
            // Set overlay data first — backend uses it during present.
            if let Some(ref overlay) = overlay_data {
                renderer.present_overlay(overlay);
            }
            // Stage vector primitives for native rendering on pixel backends.
            if !vectors_ptr.is_null() {
                let vectors = unsafe { &*vectors_ptr };
                renderer.present_vectors(vectors);
            }
            // SAFETY: buffer_ptr points at the world Buffer resource, which remains valid
            // while the renderer backend is borrowed mutably from a different resource slot.
            let buffer = unsafe { &*buffer_ptr };
            renderer.present_buffer(buffer);
        }
    }

    world.swap_buffers();
}

fn collect_debug_overlay<T: RendererProvider>(world: &mut T) -> Option<OverlayData> {
    let debug = world.debug_features().copied()?;
    if !debug.enabled || !debug.overlay_visible {
        return None;
    }

    use engine_render::OverlayLine;

    // Color palette for the overlay console.
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
            let scene_id = world.current_scene_id();
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
            let virtual_info = {
                let settings = world.runtime_settings();
                format_render_info(settings)
            };
            let timings_info = world
                .system_timings()
                .map(|st| {
                    format!(
                        "beh:{:.0}  comp:{:.0}  pfx:{:.0}  rend:{:.0} µs",
                        st.behavior_us, st.compositor_us, st.postfx_us, st.renderer_us
                    )
                })
                .unwrap_or_default();
            let script_errors: Vec<String> = world
                .debug_log()
                .map(|log| {
                    log.recent(usize::MAX)
                        .iter()
                        .map(|entry| entry.display_line())
                        .collect()
                })
                .unwrap_or_default();

            // Title bar
            lines.push(OverlayLine::with_alpha(
                " ■ DEBUG CONSOLE          [~] toggle  [Tab] switch  [F3/F4] scene",
                title_fg,
                title_bg,
                220,
            ));
            // Separator
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
            // Scene info
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]scene   │[/] [#66d9ef]{scene_id}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            // Stage info
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]stage   │[/] [#78dca0]{stage_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            // Render info
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]render  │[/] [#ffd166]{virtual_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));
            // Timings
            lines.push(OverlayLine::with_alpha(
                format!("  [#8c8ca0]timing  │[/] [#d39bff]{timings_info}[/]"),
                label_fg,
                console_bg,
                console_alpha,
            ));

            if !script_errors.is_empty() {
                // Error separator
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
            // Bottom border
            lines.push(OverlayLine::with_alpha(
                "─────────────────────────────────────────────────────────────────────",
                sep_fg,
                sep_bg,
                console_alpha,
            ));
        }
        DebugOverlayMode::Logs => {
            // Title bar
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
            // Bottom border
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

/// Always-on performance HUD: FPS / CPU% / MEM in the top-right corner.
fn apply_perf_hud<T: RendererProvider>(world: &mut T) {
    use crate::rasterizer::generic::rasterize_generic_half;
    use engine_core::scene::sprite::TextTransform;
    use std::fmt::Write;

    let fps_val = world.fps_counter().map(|c| c.fps.round() as u32);
    let proc_stats = world.process_stats().copied();

    thread_local! {
        static HUD_STR: RefCell<String> = RefCell::new(String::with_capacity(64));
    }

    HUD_STR.with(|cell| {
        let hud_text = &mut *cell.borrow_mut();
        hud_text.clear();
        if let Some(fps) = fps_val {
            let _ = write!(hud_text, "{fps} FPS");
        }
        if let Some(ps) = &proc_stats {
            if !hud_text.is_empty() {
                hud_text.push_str("  ");
            }
            let _ = write!(hud_text, "{:.0}% CPU  {:.1}MB", ps.cpu_percent, ps.rss_mb);
        }
        if hud_text.is_empty() {
            return;
        }

        let Some(buf) = world.buffer_mut() else {
            return;
        };
        let text_w = hud_text.len() as u16 * 6;
        let x = buf.width.saturating_sub(text_w);
        let green = Color::Rgb {
            r: 0,
            g: 255,
            b: 80,
        };
        rasterize_generic_half(hud_text, green, x, 0, buf, &TextTransform::None);
    });
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

    if settings.render_size_matches_output() {
        format!("render: match-output ({policy})")
    } else {
        let (width, height) = settings.fixed_render_size().unwrap_or((0, 0));
        format!("render: {}x{} ({policy})", width, height)
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
    render_w: u16,
    render_h: u16,
    policy: PresentationPolicy,
) -> Viewport {
    let layout = compute_presentation_layout(
        output_w as u32,
        output_h as u32,
        render_w as u32,
        render_h as u32,
        policy,
    );
    Viewport {
        width: layout.dst_width as u16,
        height: layout.dst_height as u16,
        dst_offset_x: layout.dst_x as u16,
        dst_offset_y: layout.dst_y as u16,
        src_offset_x: layout.src_x as u16,
        src_offset_y: layout.src_y as u16,
    }
}

fn project_buffer_to_output(source: &Buffer, output: &mut Buffer, policy: PresentationPolicy) {
    output.fill(Color::BLACK);
    if source.width == 0 || source.height == 0 || output.width == 0 || output.height == 0 {
        return;
    }

    let viewport = compute_viewport(
        output.width,
        output.height,
        source.width,
        source.height,
        policy,
    );

    match policy {
        PresentationPolicy::Strict => {
            for oy in 0..viewport.height {
                for ox in 0..viewport.width {
                    let sx = viewport.src_offset_x.saturating_add(ox);
                    let sy = viewport.src_offset_y.saturating_add(oy);
                    let dx = viewport.dst_offset_x.saturating_add(ox);
                    let dy = viewport.dst_offset_y.saturating_add(oy);
                    let Some(cell) = source.get(sx, sy) else {
                        continue;
                    };
                    output.set(dx, dy, cell.symbol, cell.fg, cell.bg);
                }
            }
        }
        PresentationPolicy::Fit | PresentationPolicy::Stretch => {
            for oy in 0..viewport.height {
                let sy = ((oy as u32).saturating_mul(source.height as u32)
                    / viewport.height.max(1) as u32)
                    .min(source.height.saturating_sub(1) as u32) as u16;
                for ox in 0..viewport.width {
                    let sx = ((ox as u32).saturating_mul(source.width as u32)
                        / viewport.width.max(1) as u32)
                        .min(source.width.saturating_sub(1) as u32)
                        as u16;
                    let dx = viewport.dst_offset_x.saturating_add(ox);
                    let dy = viewport.dst_offset_y.saturating_add(oy);
                    let Some(cell) = source.get(sx, sy) else {
                        continue;
                    };
                    output.set(dx, dy, cell.symbol, cell.fg, cell.bg);
                }
            }
        }
    }
}

/// Resolve a color for output — `Color::Reset` is mapped to true black so that
/// Resolve Color::Reset to TRUE_BLACK so
/// terminal theme colours never bleed through transparent/unset pixels.
#[inline]
pub fn resolve_color(c: style::Color) -> style::Color {
    match c {
        style::Color::Reset => style::Color::Rgb { r: 0, g: 0, b: 0 },
        other => other,
    }
}

#[inline]
#[allow(dead_code)]
fn to_ct(c: crossterm::style::Color) -> style::Color {
    c
}

/// Batch-flush diffs to the terminal.
///
/// Consecutive cells on the same row sharing the same fg+bg colour are merged
/// into a single `MoveTo + SetFg + SetBg + Print(run)` command, reducing the
/// number of terminal I/O operations from O(cells) toward O(colour-runs).
/// Diffs arrive in row-major order from `Buffer::diff_into`, so no sort is needed.
/// Raw (pre-resolve) colours are accepted; `Color::Reset` is mapped to true black here.
pub fn flush_batched(
    stdout: &mut io::BufWriter<io::Stdout>,
    diffs: &[(u16, u16, char, Color, Color)],
) {
    if diffs.is_empty() {
        return;
    }

    // Convert engine colors to crossterm colors at the boundary
    let crossterm_diffs: Vec<(u16, u16, char, style::Color, style::Color)> = diffs
        .iter()
        .map(|(x, y, ch, fg, bg)| {
            (
                *x,
                *y,
                *ch,
                color_convert::to_crossterm(*fg),
                color_convert::to_crossterm(*bg),
            )
        })
        .collect();

    // #3 opt-term-ansibuf: write all ANSI into Vec<u8>, then single write_all.
    // #2 opt-term-colorstate: track last-emitted fg/bg to skip redundant SetColor commands.
    // #1 opt-term-cursor: skip redundant MoveTo (cursor auto-advances after Print).
    //    Use CursorRight(n) for small gaps (3 bytes vs 6 for MoveTo).
    ANSI_BUF.with(|ansi_cell| {
        RUN_BUF.with(|run_cell| {
            let mut ansi = ansi_cell.borrow_mut();
            let mut run = run_cell.borrow_mut();
            ansi.clear();
            run.clear();

            let (mut rx, mut ry, _, raw_fg0, raw_bg0) = crossterm_diffs[0];
            let (mut rfg, mut rbg) = (resolve_color(raw_fg0), resolve_color(raw_bg0));
            run.push(crossterm_diffs[0].2);
            let mut run_len: u16 = 1;
            let mut cursor_x = u16::MAX;
            let mut cursor_y = u16::MAX;
            let mut active_fg = style::Color::Reset;
            let mut active_bg = style::Color::Reset;

            // Inline helper: emit a queued run into the ANSI buffer.
            // Optimizations:
            //   - Skip MoveTo if already at correct position (cursor auto-advances)
            //   - Use CursorRight(n) for small horizontal gaps (cheaper than MoveTo)
            //   - Only emit SetFg/Bg if color changed (already tracked)
            macro_rules! emit_run {
                () => {
                    // Cursor movement: skip redundant MoveTo, use CursorRight for small gaps
                    if cursor_y != ry {
                        let _ = queue!(&mut *ansi, cursor::MoveTo(rx, ry));
                    } else if cursor_x != rx {
                        let gap = rx.saturating_sub(cursor_x) as usize;
                        if gap > 0 && gap <= 3 {
                            // CursorRight is cheaper for gaps 1-3 cells
                            for _ in 0..gap {
                                let _ = queue!(&mut *ansi, cursor::MoveRight(1));
                            }
                        } else if gap > 3 {
                            let _ = queue!(&mut *ansi, cursor::MoveTo(rx, ry));
                        }
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
                    // Suppress "value never read" on final invocation.
                    let _ = (cursor_x, cursor_y, active_fg, active_bg);
                };
            }

            for &(x, y, ch, raw_fg, raw_bg) in &crossterm_diffs[1..] {
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

#[cfg(test)]
mod tests {
    use super::compute_viewport;
    use engine_runtime::PresentationPolicy;

    #[test]
    fn fit_viewport_uses_full_width_when_upscaling() {
        let viewport = compute_viewport(210, 109, 180, 30, PresentationPolicy::Fit);
        assert_eq!(viewport.width, 210);
        assert_eq!(viewport.height, 35);
        assert_eq!(viewport.dst_offset_x, 0);
        assert_eq!(viewport.dst_offset_y, 37);
    }

    #[test]
    fn fit_viewport_downscales_to_visible_full_frame() {
        let viewport = compute_viewport(80, 24, 180, 30, PresentationPolicy::Fit);
        assert_eq!(viewport.width, 80);
        assert_eq!(viewport.height, 13);
        assert_eq!(viewport.dst_offset_x, 0);
        assert_eq!(viewport.dst_offset_y, 5);
    }
}
