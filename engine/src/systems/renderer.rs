use crate::buffer::{Buffer, Cell, VirtualBuffer, TRUE_BLACK};
use crate::runtime_settings::{RuntimeSettings, VirtualPolicy};
use crate::world::World;
use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{self, Write};

pub struct TerminalRenderer {
    stdout: io::Stdout,
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

    let diffs: Vec<(u16, u16, char, style::Color, style::Color)> = {
        match world.get::<Buffer>() {
            Some(buf) => buf
                .diff()
                .into_iter()
                .map(|d| {
                    (
                        d.x,
                        d.y,
                        d.cell.symbol,
                        resolve_color(d.cell.fg),
                        resolve_color(d.cell.bg),
                    )
                })
                .collect(),
            None => return,
        }
    };

    if diffs.is_empty() {
        // Still need to swap so compositor can detect unchanged next frame.
        if let Some(buf) = world.get_mut::<Buffer>() {
            buf.swap();
        }
        return;
    }

    if let Some(renderer) = world.get_mut::<TerminalRenderer>() {
        let stdout = &mut renderer.stdout;
        for (x, y, symbol, fg, bg) in &diffs {
            let _ = queue!(
                stdout,
                cursor::MoveTo(*x, *y),
                style::SetForegroundColor(*fg),
                style::SetBackgroundColor(*bg),
                style::Print(symbol)
            );
        }
        let _ = stdout.flush();
    }

    if let Some(buf) = world.get_mut::<Buffer>() {
        buf.swap();
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
        .get::<RuntimeSettings>()
        .map(|s| s.use_virtual_buffer)
        .unwrap_or(false)
        && world.get::<VirtualBuffer>().is_some()
}

fn present_virtual_to_output(world: &mut World) {
    let settings = world.get::<RuntimeSettings>().cloned().unwrap_or_default();
    let virtual_snapshot = world.get::<VirtualBuffer>().map(|v| v.0.clone());
    let Some(virtual_buf) = virtual_snapshot else {
        return;
    };
    let Some(output_buf) = world.get_mut::<Buffer>() else {
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

fn copy_cell(dst: &mut Buffer, x: u16, y: u16, src: &Cell) {
    dst.set(x, y, src.symbol, src.fg, src.bg);
}
