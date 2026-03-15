mod types;
mod font_loader;
pub mod generic;

use crate::buffer::Buffer;
use crossterm::style::Color;

/// Rasterize `text` using the named bitmap font into a new Buffer.
pub fn rasterize(text: &str, font: &str, fg: Color, bg: Color) -> Buffer {
    // Handle built-in generic pixel font: "generic" or "generic:N"
    // preset 1 = 3×5 tiny, preset 2 = 5×7 (default), preset 3 = 5×7 ×2 scale
    if font.starts_with("generic") {
        let preset: u16 = font.strip_prefix("generic")
            .and_then(|s| s.strip_prefix(':'))
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        match preset {
            1 => {
                let (width, height) = generic::generic_dimensions_tiny(text);
                let mut out = Buffer::new(width.max(1), height.max(1));
                out.fill(Color::Reset);
                generic::rasterize_generic_tiny(text, fg, 0, 0, &mut out);
                return out;
            }
            3 => {
                let (width, height) = generic::generic_dimensions(text, 2);
                let mut out = Buffer::new(width.max(1), height.max(1));
                out.fill(Color::Reset);
                generic::rasterize_generic(text, 2, fg, 0, 0, &mut out);
                return out;
            }
            _ => {
                let (width, height) = generic::generic_dimensions(text, 1);
                let mut out = Buffer::new(width.max(1), height.max(1));
                out.fill(Color::Reset);
                generic::rasterize_generic(text, 1, fg, 0, 0, &mut out);
                return out;
            }
        }
    }

    let glyph_font = match font_loader::load_font_assets(font) {
        Some(f) => f,
        None => return rasterize_native(text, fg, bg),
    };

    let mut width: u16 = 0;
    let mut max_height: u16 = 1;
    for ch in text.chars() {
        let (advance, glyph_h) = glyph_font.advance_and_height(ch);
        width = width.saturating_add(advance);
        max_height = max_height.max(glyph_h.max(1));
    }
    width = width.max(1);

    let mut out = Buffer::new(width, max_height);
    out.fill(Color::Reset);
    let mut cursor_x: u16 = 0;
    for ch in text.chars() {
        if let Some(glyph) = glyph_font.glyphs.get(&ch) {
            for (row, line) in glyph.lines.iter().enumerate() {
                let y = row as u16;
                for (col, gch) in line.chars().enumerate() {
                    if gch == ' ' { continue; }
                    let x = cursor_x.saturating_add(col as u16);
                    if x < out.width && y < out.height {
                        out.set(x, y, gch, fg, bg);
                    }
                }
            }
            cursor_x = cursor_x.saturating_add(glyph.advance);
        } else {
            if ch == ' ' {
                cursor_x = cursor_x.saturating_add(1);
                continue;
            }
            out.set(cursor_x, 0, ch, fg, bg);
            cursor_x = cursor_x.saturating_add(1);
        }
    }

    out
}

fn rasterize_native(text: &str, fg: Color, bg: Color) -> Buffer {
    let width = text.chars().count() as u16;
    let mut buf = Buffer::new(width.max(1), 1);
    buf.fill(Color::Reset);
    for (i, ch) in text.chars().enumerate() {
        buf.set(i as u16, 0, ch, fg, bg);
    }
    buf
}

/// Blit `src` buffer onto `dst` buffer at position (dx, dy).
/// Pixels outside `dst` bounds are silently clipped.
pub fn blit(src: &Buffer, dst: &mut Buffer, dx: u16, dy: u16) {
    for y in 0..src.height {
        for x in 0..src.width {
            if let Some(cell) = src.get(x, y) {
                dst.set(dx + x, dy + y, cell.symbol, cell.fg, cell.bg);
            }
        }
    }
}
