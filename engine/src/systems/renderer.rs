use std::io::{self, Write};
use crossterm::{execute, queue, terminal, cursor, style};
use crate::buffer::Buffer;
use crate::world::World;

pub struct TerminalRenderer {
    stdout: io::Stdout,
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide
        )?;
        Ok(Self { stdout })
    }

    /// Paint the entire screen true-black before the first game frame.
    pub fn clear_black(&mut self) -> io::Result<()> {
        let (w, h) = terminal::size()?;
        let bg = style::Color::Rgb { r: 0, g: 0, b: 0 };
        let fg = style::Color::Rgb { r: 0, g: 0, b: 0 };
        queue!(self.stdout, style::SetForegroundColor(fg), style::SetBackgroundColor(bg))?;
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
    let diffs: Vec<(u16, u16, char, style::Color, style::Color)> = {
        match world.get::<Buffer>() {
            Some(buf) => buf
                .diff()
                .into_iter()
                .map(|d| (d.x, d.y, d.cell.symbol, resolve_color(d.cell.fg), resolve_color(d.cell.bg)))
                .collect(),
            None => return,
        }
    };

    if diffs.is_empty() {
        // Still need to swap so compositor can detect unchanged next frame.
        if let Some(buf) = world.get_mut::<Buffer>() { buf.swap(); }
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
