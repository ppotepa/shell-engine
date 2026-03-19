//! Text sprite rendering — writes terminal-cell or rasterized glyph text into the compositor buffer.

use crossterm::style::Color;
use std::path::Path;

use crate::buffer::Buffer;
use crate::markup::{parse_spans, strip_markup};
use crate::rasterizer;
use crate::rasterizer::generic;

pub(super) fn render_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    x: u16,
    y: u16,
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
                    buf.set(col, row, ch, span_fg, bg);
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
                generic::rasterize_spans_mode(&colored_spans, mode, x, line_y, buf);
            }
        }
        Some(font_name) => {
            let stripped = strip_markup(content);
            let line_h = raster_line_height(mod_source, font_name, fg, bg);
            for (line_idx, line) in stripped.split('\n').enumerate() {
                let text_buf = rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg);
                let line_y = y.saturating_add((line_idx as u16).saturating_mul(line_h));
                rasterizer::blit(&text_buf, buf, x, line_y);
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
