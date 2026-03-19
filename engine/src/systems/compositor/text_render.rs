//! Text sprite rendering — writes terminal-cell or rasterized glyph text into the compositor buffer.

use crossterm::style::Color;
use std::path::Path;

use crate::buffer::Buffer;
use crate::markup::{parse_spans, strip_markup};
use crate::rasterizer;
use crate::rasterizer::generic;

#[derive(Debug, Clone, Copy)]
pub(super) struct ClipRect {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl ClipRect {
    fn contains(self, cell_x: u16, cell_y: u16) -> bool {
        let x = i32::from(cell_x);
        let y = i32::from(cell_y);
        let x_end = self.x.saturating_add(i32::from(self.width));
        let y_end = self.y.saturating_add(i32::from(self.height));
        x >= self.x && y >= self.y && x < x_end && y < y_end
    }
}

pub(super) fn render_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    x: u16,
    y: u16,
    clip: Option<ClipRect>,
    buf: &mut Buffer,
) {
    match font {
        None => {
            let spans = parse_spans(content);
            let mut col = x;
            let mut row = y;
            for span in &spans {
                let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(fg);
                for ch in span.text.chars() {
                    if ch == '\n' {
                        col = x;
                        row = row.saturating_add(1);
                        continue;
                    }
                    if clip.is_none_or(|rect| rect.contains(col, row)) {
                        buf.set(col, row, ch, span_fg, bg);
                    }
                    col = col.saturating_add(1);
                }
            }
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let line_h = generic_mode_line_height(mode);
            for (line_idx, line) in content.split('\n').enumerate() {
                let spans = parse_spans(line);
                let colored_spans: Vec<(String, Color)> = spans
                    .iter()
                    .map(|s| {
                        (
                            s.text.clone(),
                            s.colour.as_ref().map(Color::from).unwrap_or(fg),
                        )
                    })
                    .collect();
                let line_y = y.saturating_add((line_idx as u16).saturating_mul(line_h));
                let line_width = colored_spans
                    .iter()
                    .map(|(text, _)| generic::generic_dimensions_mode(text, mode).0)
                    .fold(0u16, |acc, w| acc.saturating_add(w))
                    .max(1);
                let mut line_buf = Buffer::new(line_width, line_h.max(1));
                line_buf.fill(Color::Reset);
                generic::rasterize_spans_mode(&colored_spans, mode, 0, 0, &mut line_buf);
                blit_with_clip(&line_buf, buf, x, line_y, clip);
            }
        }
        Some(font_name) => {
            let stripped = strip_markup(content);
            let line_h = raster_line_height(mod_source, font_name, fg, bg);
            for (line_idx, line) in stripped.split('\n').enumerate() {
                let text_buf = rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg);
                let line_y = y.saturating_add((line_idx as u16).saturating_mul(line_h));
                blit_with_clip(&text_buf, buf, x, line_y, clip);
            }
        }
    }
}

fn blit_with_clip(src: &Buffer, dst: &mut Buffer, dx: u16, dy: u16, clip: Option<ClipRect>) {
    for sy in 0..src.height {
        for sx in 0..src.width {
            let tx = dx.saturating_add(sx);
            let ty = dy.saturating_add(sy);
            if clip.is_some_and(|rect| !rect.contains(tx, ty)) {
                continue;
            }
            if let Some(cell) = src.get(sx, sy) {
                if cell.symbol == ' ' && cell.bg == Color::Reset {
                    continue;
                }
                let bg = if cell.bg == Color::Reset {
                    dst.get(tx, ty)
                        .map(|under| under.bg)
                        .unwrap_or(Color::Reset)
                } else {
                    cell.bg
                };
                dst.set(tx, ty, cell.symbol, cell.fg, bg);
            }
        }
    }
}

pub(super) fn text_sprite_dimensions(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
) -> (u16, u16) {
    let visible = strip_markup(content);
    match font {
        None => {
            let lines = split_lines_preserve_empty(&visible);
            let width = lines
                .iter()
                .map(|line| line.chars().count() as u16)
                .max()
                .unwrap_or(0)
                .max(1);
            (width, lines.len() as u16)
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let lines = split_lines_preserve_empty(&visible);
            let width = lines
                .iter()
                .map(|line| generic::generic_dimensions_mode(line, mode).0)
                .max()
                .unwrap_or(0)
                .max(1);
            let height = generic_mode_line_height(mode).saturating_mul(lines.len() as u16);
            (width, height.max(1))
        }
        Some(font_name) => {
            let lines = split_lines_preserve_empty(&visible);
            let line_h = raster_line_height(mod_source, font_name, fg, bg);
            let width = lines
                .iter()
                .map(|line| {
                    rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg)
                        .width
                        .max(1)
                })
                .max()
                .unwrap_or(1);
            let height = line_h.saturating_mul(lines.len() as u16).max(1);
            (width, height)
        }
    }
}

fn split_lines_preserve_empty(content: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = content.split('\n').collect();
    if lines.is_empty() {
        lines.push("");
    }
    lines
}

fn generic_mode_line_height(mode: generic::GenericMode) -> u16 {
    match mode {
        generic::GenericMode::Tiny => 5,
        generic::GenericMode::Standard => 7,
        generic::GenericMode::Large => 14,
        generic::GenericMode::Half => 4,
        generic::GenericMode::Quad => 4,
        generic::GenericMode::Braille => 2,
    }
}

fn raster_line_height(mod_source: Option<&Path>, font_name: &str, fg: Color, bg: Color) -> u16 {
    rasterizer::rasterize_cached(mod_source, "A", font_name, fg, bg)
        .height
        .max(1)
}

pub(super) fn dim_colour(c: Color) -> Color {
    use crate::effects::utils::color::colour_to_rgb;
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * 0.25) as u8,
        g: (g as f32 * 0.25) as u8,
        b: (b as f32 * 0.25) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::blit_with_clip;
    use crate::buffer::Buffer;
    use crossterm::style::Color;

    #[test]
    fn blit_preserves_underlying_bg_for_reset_text_cells() {
        let mut dst = Buffer::new(3, 2);
        dst.fill(Color::DarkGrey);

        let mut src = Buffer::new(1, 1);
        src.fill(Color::Reset);
        src.set(0, 0, 'X', Color::White, Color::Reset);

        blit_with_clip(&src, &mut dst, 1, 1, None);
        let out = dst.get(1, 1).expect("blitted cell");
        assert_eq!(out.symbol, 'X');
        assert_eq!(out.fg, Color::White);
        assert_eq!(out.bg, Color::DarkGrey);
    }

    #[test]
    fn blit_skips_transparent_blank_cells() {
        let mut dst = Buffer::new(2, 1);
        dst.fill(Color::DarkGrey);
        dst.set(0, 0, 'P', Color::Yellow, Color::DarkGrey);

        let mut src = Buffer::new(1, 1);
        src.fill(Color::Reset);

        blit_with_clip(&src, &mut dst, 0, 0, None);
        let out = dst.get(0, 0).expect("destination cell");
        assert_eq!(out.symbol, 'P');
        assert_eq!(out.fg, Color::Yellow);
        assert_eq!(out.bg, Color::DarkGrey);
    }
}
