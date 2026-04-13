//! Font rendering — rasterizes text into buffers using TTF and bitmap fonts.

use crate::font_loader;
use crate::generic;
pub use crate::types::{GlyphManifest, LoadedFont, LoadedGlyph};
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::sprite::TextTransform;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

fn color_key(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Reset => (0, 0, 0),
        _ => (0, 0, 0),
    }
}

// Hash-based cache key — eliminates 3× String allocations per lookup.
fn raster_cache_hash(
    mod_source: Option<&Path>,
    text: &str,
    font: &str,
    fg: Color,
    bg: Color,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    mod_source.map(|p| p.to_string_lossy()).hash(&mut h);
    text.hash(&mut h);
    font.hash(&mut h);
    color_key(fg).hash(&mut h);
    color_key(bg).hash(&mut h);
    h.finish()
}

thread_local! {
    static RASTER_CACHE: RefCell<HashMap<u64, Arc<Buffer>>> = RefCell::new(HashMap::new());
}

/// Cached variant of `rasterize` — returns an Arc-wrapped buffer.
/// Cache hit = Arc::clone (cheap refcount bump), zero buffer cloning.
pub fn rasterize_cached(
    mod_source: Option<&Path>,
    text: &str,
    font: &str,
    fg: Color,
    bg: Color,
) -> Arc<Buffer> {
    let key = raster_cache_hash(mod_source, text, font, fg, bg);

    RASTER_CACHE.with(|cache| {
        if let Some(arc_buf) = cache.borrow().get(&key) {
            return Arc::clone(arc_buf);
        }
        let buf = Arc::new(rasterize(mod_source, text, font, fg, bg));
        let mut guard = cache.borrow_mut();
        if guard.len() >= 512 {
            guard.clear();
        }
        guard.insert(key, Arc::clone(&buf));
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
    // preset 1 = compact 4×5 tiny, preset 2 = 5×7 (default), preset 3 = 5×7 ×2 scale
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
                generic::rasterize_generic_tiny(text, fg, 0, 0, &mut out, &TextTransform::None);
                return out;
            }
            3 => {
                let (width, height) = generic::generic_dimensions(text, 2);
                let mut out = Buffer::new(width.max(1), height.max(1));
                out.fill(Color::Reset);
                generic::rasterize_generic(text, 2, fg, 0, 0, &mut out, &TextTransform::None);
                return out;
            }
            _ => {
                let (width, height) = generic::generic_dimensions(text, 1);
                let mut out = Buffer::new(width.max(1), height.max(1));
                out.fill(Color::Reset);
                generic::rasterize_generic(text, 1, fg, 0, 0, &mut out, &TextTransform::None);
                return out;
            }
        }
    }

    let glyph_font = match mod_source.and_then(|source| font_loader::load_font_assets(source, font))
    {
        Some(f) => f,
        None => return rasterize_engine_fallback(text, fg),
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

/// Engine-level fallback used when named font assets are missing.
///
/// Uses the engine-wide generic fallback spec so the active renderer keeps
/// readable glyph shapes instead of raw cell symbols.
fn rasterize_engine_fallback(text: &str, fg: Color) -> Buffer {
    let mode = generic::GenericMode::from_font_name(generic::ENGINE_FALLBACK_FONT_SPEC);
    let (width, height) = generic::generic_dimensions_mode(text, mode);
    let mut out = Buffer::new(width.max(1), height.max(1));
    out.fill(Color::Reset);
    match mode {
        generic::GenericMode::Tiny => {
            generic::rasterize_generic_tiny(text, fg, 0, 0, &mut out, &TextTransform::None);
        }
        generic::GenericMode::Standard => {
            generic::rasterize_generic(text, 1, fg, 0, 0, &mut out, &TextTransform::None);
        }
        generic::GenericMode::Large => {
            generic::rasterize_generic(text, 2, fg, 0, 0, &mut out, &TextTransform::None);
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::rasterize;
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;

    fn symbol_fingerprint(buf: &Buffer) -> String {
        let mut out = String::new();
        for y in 0..buf.height {
            for x in 0..buf.width {
                let ch = buf.get(x, y).map(|c| c.symbol).unwrap_or(' ');
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn generic_font_preserves_case_in_rasterizer_path() {
        let lower = rasterize(None, "g", "generic:2", Color::White, Color::Reset);
        let upper = rasterize(None, "G", "generic:2", Color::White, Color::Reset);
        assert_ne!(symbol_fingerprint(&lower), symbol_fingerprint(&upper));
    }
}
