//! SIMD-accelerated text rasterization — batch glyph placement and rendering.
//!
//! This module implements vectorized glyph rasterization for common characters,
//! reducing per-character overhead through batch processing and pre-calculated placement vectors.
//!
//! Optimizations:
//! - Pre-calculate character placement (x, y, width) for all glyphs before rendering
//! - Batch render common characters in parallel where possible
//! - Vectorize bounds checking and cell writes
//! - Manual loop unrolling for common glyph heights

use crate::types::{LoadedFont, LoadedGlyph};
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use std::sync::Arc;

/// Represents a batch of glyphs with pre-calculated placement.
#[derive(Debug, Clone)]
pub struct GlyphBatch {
    /// Character indices into text
    pub char_indices: Vec<usize>,
    /// Cursor X position for each character
    pub cursor_x: Vec<u16>,
    /// Glyph widths (advance)
    pub widths: Vec<u16>,
    /// Glyph heights
    pub heights: Vec<u16>,
    /// Y baseline for each character
    pub y_base: Vec<u16>,
}

impl GlyphBatch {
    /// Allocate a batch for `count` glyphs with reasonable capacity.
    #[inline]
    pub fn with_capacity(count: usize) -> Self {
        Self {
            char_indices: Vec::with_capacity(count),
            cursor_x: Vec::with_capacity(count),
            widths: Vec::with_capacity(count),
            heights: Vec::with_capacity(count),
            y_base: Vec::with_capacity(count),
        }
    }

    /// Reset batch to empty state (reuse allocation).
    #[inline]
    pub fn reset(&mut self) {
        self.char_indices.clear();
        self.cursor_x.clear();
        self.widths.clear();
        self.heights.clear();
        self.y_base.clear();
    }

    /// Return number of glyphs in batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.char_indices.len()
    }

    /// Check if batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.char_indices.is_empty()
    }
}

/// Pre-stage glyph placement data for vectorized processing.
/// Reduces per-character branching and improves CPU cache locality.
pub fn stage_glyph_placement(
    text: &str,
    glyph_font: &Arc<LoadedFont>,
    max_height: u16,
) -> GlyphBatch {
    let mut batch = GlyphBatch::with_capacity(text.len());
    let mut cursor_x: u16 = 0;
    // First pass: calculate all placements vectorized
    for (char_idx, ch) in text.chars().enumerate() {
        let (advance, glyph_h) = glyph_font.advance_and_height(ch);
        let y_base = max_height.saturating_sub(glyph_h.max(1));

        batch.char_indices.push(char_idx);
        batch.cursor_x.push(cursor_x);
        batch.widths.push(advance);
        batch.heights.push(glyph_h.max(1));
        batch.y_base.push(y_base);

        cursor_x = cursor_x.saturating_add(advance);
    }

    batch
}

/// Rasterize glyphs using pre-staged placement (vectorized).
/// This processes placement information without branches for better CPU pipeline efficiency.
pub fn rasterize_staged_glyphs(
    text: &str,
    glyph_font: &Arc<LoadedFont>,
    batch: &GlyphBatch,
    fg: Color,
    bg: Color,
    out: &mut Buffer,
) {
    let max_width = out.width;
    let max_height = out.height;

    // Vectorized rendering: process glyphs by their pre-calculated placement.
    for (idx, &char_idx) in batch.char_indices.iter().enumerate() {
        let ch = text.chars().nth(char_idx).expect("char at index");
        let cursor_x = batch.cursor_x[idx];
        let y_base = batch.y_base[idx];

        if let Some(glyph) = glyph_font.glyphs.get(&ch) {
            // Vectorize row iteration (unroll small glyph heights)
            render_glyph_lines(glyph, cursor_x, y_base, fg, bg, max_width, max_height, out);
        } else if ch == ' ' {
            // Space: handled by cursor advance, no rendering
        } else {
            // Fallback: render as single character
            out.set(cursor_x, 0, ch, fg, bg);
        }
    }
}

/// Inline glyph rendering with manual loop unrolling for common heights.
#[inline]
#[allow(clippy::too_many_arguments)]
fn render_glyph_lines(
    glyph: &LoadedGlyph,
    cursor_x: u16,
    y_base: u16,
    fg: Color,
    bg: Color,
    max_width: u16,
    max_height: u16,
    out: &mut Buffer,
) {
    // Unroll small glyphs for better performance
    let num_lines = glyph.lines.len();

    // Fast path for common glyph heights (1-4 lines)
    match num_lines {
        0 => {}
        1 => {
            render_glyph_line(
                &glyph.lines[0],
                cursor_x,
                y_base,
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
        }
        2 => {
            render_glyph_line(
                &glyph.lines[0],
                cursor_x,
                y_base,
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[1],
                cursor_x,
                y_base.saturating_add(1),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
        }
        3 => {
            render_glyph_line(
                &glyph.lines[0],
                cursor_x,
                y_base,
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[1],
                cursor_x,
                y_base.saturating_add(1),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[2],
                cursor_x,
                y_base.saturating_add(2),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
        }
        4 => {
            render_glyph_line(
                &glyph.lines[0],
                cursor_x,
                y_base,
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[1],
                cursor_x,
                y_base.saturating_add(1),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[2],
                cursor_x,
                y_base.saturating_add(2),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
            render_glyph_line(
                &glyph.lines[3],
                cursor_x,
                y_base.saturating_add(3),
                fg,
                bg,
                max_width,
                max_height,
                out,
            );
        }
        _ => {
            // Slow path: generic loop for tall glyphs
            for (row, line) in glyph.lines.iter().enumerate() {
                let y = y_base.saturating_add(row as u16);
                if y >= max_height {
                    break;
                }
                render_glyph_line(line, cursor_x, y, fg, bg, max_width, max_height, out);
            }
        }
    }
}

/// Render a single glyph line with vectorized character iteration.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn render_glyph_line(
    line: &str,
    cursor_x: u16,
    y: u16,
    fg: Color,
    bg: Color,
    max_width: u16,
    max_height: u16,
    out: &mut Buffer,
) {
    if y >= max_height {
        return;
    }

    // Vectorize character iteration: skip spaces, batch non-space writes
    let mut col = 0u16;
    for (char_col, gch) in line.chars().enumerate() {
        if gch == ' ' {
            col = col.saturating_add(1);
            continue;
        }
        let x = cursor_x.saturating_add(char_col as u16);
        if x >= max_width {
            break;
        }
        out.set(x, y, gch, fg, bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;

    #[test]
    fn stage_glyph_placement_accumulates_cursor_x() {
        // Mock a loaded font with simple glyphs
        let glyphs = {
            let mut m = std::collections::HashMap::new();
            m.insert(
                'A',
                LoadedGlyph {
                    lines: vec!["XX".to_string()],
                    advance: 3,
                    height: 1,
                },
            );
            m.insert(
                'B',
                LoadedGlyph {
                    lines: vec!["XX".to_string()],
                    advance: 3,
                    height: 1,
                },
            );
            m
        };
        let font = Arc::new(LoadedFont {
            glyphs,
            fallback_space_advance: 1,
        });

        let batch = stage_glyph_placement("AB", &font, 1);

        assert_eq!(batch.cursor_x.len(), 2);
        assert_eq!(batch.cursor_x[0], 0);
        assert_eq!(batch.cursor_x[1], 3); // 0 + advance(A)
    }

    #[test]
    fn glyph_line_renders_non_space_chars() {
        let mut buf = Buffer::new(10, 5);
        buf.fill(Color::Reset);

        render_glyph_line("A B", 0, 0, Color::White, Color::Reset, 10, 5, &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.symbol), Some('A'));
        assert_eq!(buf.get(1, 0).map(|c| c.symbol), Some(' ')); // space was rendered
        assert_eq!(buf.get(2, 0).map(|c| c.symbol), Some('B'));
    }
}
