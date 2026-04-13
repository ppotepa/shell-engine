//! Text sprite rendering — writes terminal-cell or rasterized glyph text into the compositor buffer.

use engine_core::color::Color;
use engine_core::scene::sprite::TextTransform;
use std::cell::RefCell;
use std::path::Path;

use engine_core::buffer::Buffer;
use engine_core::markup::{parse_spans, strip_markup};
use engine_render::generic;
use engine_render::rasterizer;

thread_local! {
    static TEXT_LINE_BUF: RefCell<Buffer> = RefCell::new(Buffer::new(1, 1));
}

#[derive(Debug, Clone, Copy)]
pub struct ClipRect {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl ClipRect {
    #[inline]
    fn contains(self, cell_x: u16, cell_y: u16) -> bool {
        let x = i32::from(cell_x);
        let y = i32::from(cell_y);
        let x_end = self.x.saturating_add(i32::from(self.width));
        let y_end = self.y.saturating_add(i32::from(self.height));
        x >= self.x && y >= self.y && x < x_end && y < y_end
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    x: u16,
    y: u16,
    clip: Option<ClipRect>,
    buf: &mut Buffer,
    transform: &TextTransform,
    scale_x: f32,
    scale_y: f32,
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
            let line_gap = generic_mode_line_gap(mode);
            let line_step = line_h.saturating_add(line_gap);
            for (line_idx, line) in content.split('\n').enumerate() {
                let spans = parse_spans(line);
                let mut colored_spans: Vec<(&str, Color)> = Vec::with_capacity(spans.len());
                let mut line_width = 0u16;
                for span in &spans {
                    let text = span.text.as_str();
                    colored_spans.push((text, span.colour.as_ref().map(Color::from).unwrap_or(fg)));
                    line_width =
                        line_width.saturating_add(generic::generic_dimensions_mode(text, mode).0);
                }
                let line_y = y.saturating_add((line_idx as u16).saturating_mul(line_step));
                TEXT_LINE_BUF.with(|cell| {
                    let line_buf = &mut *cell.borrow_mut();
                    line_buf.resize(line_width.max(1), line_h.max(1));
                    line_buf.fill(Color::Reset);
                    generic::rasterize_spans_mode(&colored_spans, mode, 0, 0, line_buf, transform);
                    blit_scaled(line_buf, buf, x, line_y, clip, scale_x, scale_y);
                });
            }
        }
        Some(font_name) => {
            let stripped = strip_markup(content);
            let line_h = raster_line_height(mod_source, font_name, fg, bg);
            for (line_idx, line) in stripped.split('\n').enumerate() {
                let text_buf = rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg);
                let line_y = y.saturating_add((line_idx as u16).saturating_mul(line_h));
                blit_scaled(&text_buf, buf, x, line_y, clip, scale_x, scale_y);
            }
        }
    }
}

#[cfg(test)]
fn blit_with_clip(src: &Buffer, dst: &mut Buffer, dx: u16, dy: u16, clip: Option<ClipRect>) {
    blit_scaled(src, dst, dx, dy, clip, 1.0, 1.0);
}

#[inline(always)]
fn blit_scaled(
    src: &Buffer,
    dst: &mut Buffer,
    dx: u16,
    dy: u16,
    clip: Option<ClipRect>,
    scale_x: f32,
    scale_y: f32,
) {
    let scale_x = scale_x.max(0.01);
    let scale_y = scale_y.max(0.01);
    let dst_w = ((src.width as f32) * scale_x).round() as u16;
    let dst_h = ((src.height as f32) * scale_y).round() as u16;
    for ty in 0..dst_h {
        let sy = ((ty as f32) / scale_y) as u16;
        if sy >= src.height {
            continue;
        }
        for tx in 0..dst_w {
            let sx = ((tx as f32) / scale_x) as u16;
            if sx >= src.width {
                continue;
            }
            let out_x = dx.saturating_add(tx);
            let out_y = dy.saturating_add(ty);
            if clip.is_some_and(|rect| !rect.contains(out_x, out_y)) {
                continue;
            }
            if let Some(cell) = src.get(sx, sy) {
                if cell.symbol == ' ' && cell.bg == Color::Reset {
                    continue;
                }
                let bg = if cell.bg == Color::Reset {
                    dst.get(out_x, out_y)
                        .map(|under| under.bg)
                        .unwrap_or(Color::Reset)
                } else {
                    cell.bg
                };
                dst.set(out_x, out_y, cell.symbol, cell.fg, bg);
            }
        }
    }
}

pub fn text_sprite_dimensions(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    scale_x: f32,
    scale_y: f32,
) -> (u16, u16) {
    let visible = strip_markup(content);
    let (w, h) = match font {
        None => {
            let mut width = 0u16;
            let mut line_count = 0u16;
            for line in visible.split('\n') {
                line_count = line_count.saturating_add(1);
                width = width.max(line.chars().count() as u16);
            }
            (width.max(1), line_count.max(1))
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let mut width = 0u16;
            let mut line_count = 0u16;
            for line in visible.split('\n') {
                line_count = line_count.saturating_add(1);
                width = width.max(generic::generic_dimensions_mode(line, mode).0);
            }
            let line_count = line_count.max(1);
            let line_h = generic_mode_line_height(mode);
            let line_gap = generic_mode_line_gap(mode);
            let height = line_h
                .saturating_mul(line_count)
                .saturating_add(line_gap.saturating_mul(line_count.saturating_sub(1)));
            (width.max(1), height.max(1))
        }
        Some(font_name) => {
            let line_h = raster_line_height(mod_source, font_name, fg, bg);
            let mut width = 1u16;
            let mut line_count = 0u16;
            for line in visible.split('\n') {
                line_count = line_count.saturating_add(1);
                width = width.max(
                    rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg)
                        .width
                        .max(1),
                );
            }
            let height = line_h.saturating_mul(line_count.max(1)).max(1);
            (width, height)
        }
    };
    let scaled_w = ((w as f32) * scale_x.max(0.01)).round() as u16;
    let scaled_h = ((h as f32) * scale_y.max(0.01)).round() as u16;
    (scaled_w.max(1), scaled_h.max(1))
}

#[inline]
fn generic_mode_line_height(mode: generic::GenericMode) -> u16 {
    match mode {
        generic::GenericMode::Tiny => 5,
        generic::GenericMode::Standard => 7,
        generic::GenericMode::Large => 14,
    }
}

#[inline]
fn generic_mode_line_gap(mode: generic::GenericMode) -> u16 {
    match mode {
        generic::GenericMode::Tiny
        | generic::GenericMode::Standard
        | generic::GenericMode::Large => 1,
    }
}

#[inline]
fn raster_line_height(mod_source: Option<&Path>, font_name: &str, fg: Color, bg: Color) -> u16 {
    rasterizer::rasterize_cached(mod_source, "A", font_name, fg, bg)
        .height
        .max(1)
}

pub fn dim_colour(c: Color) -> Color {
    use engine_effects::utils::color::colour_to_rgb;
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * 0.25) as u8,
        g: (g as f32 * 0.25) as u8,
        b: (b as f32 * 0.25) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::{blit_with_clip, render_text_content, text_sprite_dimensions};
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;
    use engine_core::scene::sprite::TextTransform;

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

    #[test]
    fn generic_multiline_dimensions_include_line_gap() {
        let (w, h) = text_sprite_dimensions(
            None,
            "A\nA",
            Some("generic:2"),
            Color::White,
            Color::Reset,
            1.0,
            1.0,
        );
        assert_eq!(w, 6);
        assert_eq!(h, 15);
    }

    #[test]
    fn generic_multiline_render_inserts_blank_separator_row() {
        let mut buf = Buffer::new(24, 20);
        buf.fill(Color::Reset);
        render_text_content(
            None,
            "A\nA",
            Some("generic:2"),
            Color::White,
            Color::Reset,
            0,
            0,
            None,
            &mut buf,
            &TextTransform::None,
            1.0,
            1.0,
        );

        let mut separator_row_has_pixels = false;
        let mut second_line_top_has_pixels = false;
        for x in 0..24u16 {
            if buf.get(x, 7).is_some_and(|c| c.symbol != ' ') {
                separator_row_has_pixels = true;
            }
            if buf.get(x, 8).is_some_and(|c| c.symbol != ' ') {
                second_line_top_has_pixels = true;
            }
        }

        assert!(
            !separator_row_has_pixels,
            "row 7 should remain empty as line separator"
        );
        assert!(
            second_line_top_has_pixels,
            "row 8 should contain second line glyph pixels"
        );
    }
}
