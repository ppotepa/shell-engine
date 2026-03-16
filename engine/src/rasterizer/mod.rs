mod font_loader;
pub mod generic;
mod types;

use crate::buffer::Buffer;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

/// Cache key for rasterized bitmap-font text buffers.
type RasterKey = (
    Option<String>, // mod_source path
    String,         // text content
    String,         // font name
    (u8, u8, u8),  // fg colour
    (u8, u8, u8),  // bg colour
);

thread_local! {
    static RASTER_CACHE: RefCell<HashMap<RasterKey, Buffer>> = RefCell::new(HashMap::new());
}

fn color_key(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Reset => (0, 0, 0),
        _ => (0, 0, 0),
    }
}

/// Cached variant of `rasterize` — returns a clone of a previously computed
/// buffer when the same (mod_source, text, font, fg, bg) tuple is requested again.
/// Uses a thread-local `HashMap` so there is no locking overhead.
pub fn rasterize_cached(
    mod_source: Option<&Path>,
    text: &str,
    font: &str,
    fg: Color,
    bg: Color,
) -> Buffer {
    // Skip cache for generic fonts — they are already cheap to compute.
    if font.starts_with("generic") {
        return rasterize(mod_source, text, font, fg, bg);
    }

    let key: RasterKey = (
        mod_source.map(|p| p.to_string_lossy().into_owned()),
        text.to_owned(),
        font.to_owned(),
        color_key(fg),
        color_key(bg),
    );

    RASTER_CACHE.with(|cache| {
        if let Some(buf) = cache.borrow().get(&key) {
            return buf.clone();
        }
        let buf = rasterize(mod_source, text, font, fg, bg);
        cache.borrow_mut().insert(key, buf.clone());
        buf
    })
}

/// Rasterize `text` using the named bitmap font into a new Buffer.
pub fn rasterize(
    mod_source: Option<&Path>,
    text: &str,
    font: &str,
    fg: Color,
    bg: Color,
) -> Buffer {
    // Handle built-in generic pixel font: "generic" or "generic:N"
    // preset 1 = 3×5 tiny, preset 2 = 5×7 (default), preset 3 = 5×7 ×2 scale
    if font.starts_with("generic") {
        let preset: u16 = font
            .strip_prefix("generic")
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

    let glyph_font = match mod_source.and_then(|source| font_loader::load_font_assets(source, font))
    {
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
            let y_base = max_height.saturating_sub(glyph.height.max(1));
            for (row, line) in glyph.lines.iter().enumerate() {
                let y = y_base.saturating_add(row as u16);
                for (col, gch) in line.chars().enumerate() {
                    if gch == ' ' {
                        continue;
                    }
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

pub fn has_font_assets(mod_source: Option<&Path>, font: &str) -> bool {
    if font.starts_with("generic") {
        return true;
    }
    mod_source
        .and_then(|source| font_loader::load_font_assets(source, font))
        .is_some()
}

pub fn missing_glyphs(mod_source: Option<&Path>, font: &str, text: &str) -> Option<Vec<char>> {
    if font.starts_with("generic") {
        return Some(Vec::new());
    }
    let loaded = font_loader::load_font_assets(mod_source?, font)?;
    let mut missing = Vec::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            continue;
        }
        if !loaded.glyphs.contains_key(&ch) && !missing.contains(&ch) {
            missing.push(ch);
        }
    }
    Some(missing)
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
