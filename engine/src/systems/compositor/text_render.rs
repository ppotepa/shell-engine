//! Text sprite rendering — writes terminal-cell or rasterized glyph text into the compositor buffer.

use crossterm::style::Color;
use std::path::Path;

use crate::buffer::Buffer;
use crate::markup::{parse_spans, strip_markup};
use crate::rasterizer;
use crate::rasterizer::generic;
use crate::scene::TextWrapMode;

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
    let mut offset_y = 0u16;
    for line in content.split('\n') {
        match font {
            None => {
                let spans = parse_spans(line);
                let mut col = 0u16;
                for span in &spans {
                    let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(fg);
                    for ch in span.text.chars() {
                        buf.set(x + col, y + offset_y, ch, span_fg, bg);
                        col += 1;
                    }
                }
            }
            Some(font_name) if font_name.starts_with("generic") => {
                let mode = generic::GenericMode::from_font_name(font_name);
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
                generic::rasterize_spans_mode(&colored_spans, mode, x, y + offset_y, buf);
            }
            Some(font_name) => {
                let stripped = strip_markup(line);
                if !stripped.is_empty() {
                    let text_buf =
                        rasterizer::rasterize_cached(mod_source, &stripped, font_name, fg, bg);
                    rasterizer::blit(&text_buf, buf, x, y + offset_y);
                }
            }
        }
        offset_y = offset_y.saturating_add(text_line_dimensions(mod_source, line, font, fg, bg).1);
    }
}

pub(super) fn text_sprite_dimensions(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
) -> (u16, u16) {
    let lines = content.split('\n').collect::<Vec<_>>();
    if lines.is_empty() {
        return (1, 1);
    }
    let mut width = 1u16;
    let mut height = 0u16;
    for line in lines {
        let (line_width, line_height) = text_line_dimensions(mod_source, line, font, fg, bg);
        width = width.max(line_width.max(1));
        height = height.saturating_add(line_height.max(1));
    }
    (width, height.max(1))
}

pub(super) fn wrap_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    wrap: Option<TextWrapMode>,
    max_width: Option<u16>,
) -> String {
    if !matches!(wrap, Some(TextWrapMode::Word)) {
        return content.to_string();
    }
    let Some(max_width) = max_width.filter(|width| *width > 0) else {
        return content.to_string();
    };
    if content != strip_markup(content) {
        return content.to_string();
    }

    let mut paragraphs = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.trim().is_empty() {
            paragraphs.push(String::new());
            continue;
        }

        let mut lines = Vec::new();
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if text_line_dimensions(mod_source, &candidate, font, fg, bg).0 <= max_width {
                current = candidate;
                continue;
            }
            if !current.is_empty() {
                lines.push(current);
            }

            if text_line_dimensions(mod_source, word, font, fg, bg).0 <= max_width {
                current = word.to_string();
                continue;
            }

            let mut chunk = String::new();
            for ch in word.chars() {
                let candidate = format!("{chunk}{ch}");
                if !chunk.is_empty()
                    && text_line_dimensions(mod_source, &candidate, font, fg, bg).0 > max_width
                {
                    lines.push(chunk);
                    chunk = ch.to_string();
                } else {
                    chunk = candidate;
                }
            }
            current = chunk;
        }
        if !current.is_empty() {
            lines.push(current);
        }
        paragraphs.push(lines.join("\n"));
    }

    paragraphs.join("\n")
}

fn text_line_dimensions(
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
            let text_buf = rasterizer::rasterize_cached(mod_source, &visible, font_name, fg, bg);
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

#[cfg(test)]
mod tests {
    use super::{text_sprite_dimensions, wrap_text_content};
    use crossterm::style::Color;

    #[test]
    fn wraps_plain_text_by_words_for_native_text() {
        let wrapped = wrap_text_content(
            None,
            "alpha beta gamma",
            None,
            Color::White,
            Color::Reset,
            Some(crate::scene::TextWrapMode::Word),
            Some(10),
        );
        assert_eq!(wrapped, "alpha beta\ngamma");
    }

    #[test]
    fn measures_multiline_native_text() {
        assert_eq!(
            text_sprite_dimensions(None, "abcd\nef", None, Color::White, Color::Reset),
            (4, 2)
        );
    }
}
