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
            let mut col = 0u16;
            for span in &spans {
                let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(fg);
                for ch in span.text.chars() {
                    buf.set(x + col, y, ch, span_fg, bg);
                    col += 1;
                }
            }
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let spans = parse_spans(content);
            let colored_spans: Vec<(String, Color)> = spans
                .iter()
                .map(|s| {
                    (
                        s.text.clone(),
                        s.colour.as_ref().map(Color::from).unwrap_or(fg),
                    )
                })
                .collect();
            generic::rasterize_spans_mode(&colored_spans, mode, x, y, buf);
        }
        Some(font_name) => {
            let stripped = strip_markup(content);
            let text_buf = rasterizer::rasterize(mod_source, &stripped, font_name, fg, bg);
            rasterizer::blit(&text_buf, buf, x, y);
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
        None => (visible.chars().count() as u16, 1),
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            generic::generic_dimensions_mode(&visible, mode)
        }
        Some(font_name) => {
            let text_buf = rasterizer::rasterize(mod_source, &visible, font_name, fg, bg);
            (text_buf.width, text_buf.height)
        }
    }
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
