//! Text sprite rendering — writes terminal-cell or rasterized glyph text into the compositor buffer.

use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::markup::{parse_spans, strip_markup};
use engine_core::scene::{TextOverflowMode, TextTransform, TextWrapMode};
use std::cell::RefCell;
use std::path::Path;

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
    pub fn contains(self, cell_x: u16, cell_y: u16) -> bool {
        let x = i32::from(cell_x);
        let y = i32::from(cell_y);
        let x_end = self.x.saturating_add(i32::from(self.width));
        let y_end = self.y.saturating_add(i32::from(self.height));
        x >= self.x && y >= self.y && x < x_end && y < y_end
    }
}

#[derive(Debug, Clone)]
struct StyledRun {
    text: String,
    fg: Color,
}

#[derive(Debug, Clone)]
enum TextLineContent {
    Styled(Vec<StyledRun>),
    Plain(String),
}

#[derive(Debug, Clone)]
struct TextLineLayout {
    content: TextLineContent,
    raw_width: u16,
}

#[derive(Debug, Clone)]
struct TextLayout {
    lines: Vec<TextLineLayout>,
    width: u16,
    height: u16,
    line_height: u16,
    line_gap: u16,
}

#[derive(Debug, Clone)]
struct PlainTextLayout {
    lines: Vec<String>,
    raw_widths: Vec<u16>,
    width: u16,
    height: u16,
    line_height: u16,
    line_step: u16,
}

#[derive(Debug, Clone, Copy)]
struct ExtendedTextLayoutOptions {
    max_width: Option<u16>,
    overflow_mode: TextOverflowMode,
    wrap_mode: TextWrapMode,
    line_clamp: Option<u16>,
    reserve_width_ch: Option<u16>,
    line_height: u16,
}

#[inline]
fn scale_extent_allow_zero(value: u16, scale: f32) -> u16 {
    ((value as f32) * scale.max(0.01)).round().max(0.0) as u16
}

#[inline]
fn scale_extent(value: u16, scale: f32) -> u16 {
    scale_extent_allow_zero(value, scale).max(1)
}

#[inline]
fn transform_char(ch: char, transform: &TextTransform) -> char {
    match transform {
        TextTransform::Uppercase => ch.to_ascii_uppercase(),
        TextTransform::None => ch,
    }
}

fn transform_visible_text(content: &str, transform: &TextTransform) -> String {
    let visible = strip_markup(content);
    if matches!(transform, TextTransform::None) {
        visible
    } else {
        visible
            .chars()
            .map(|ch| transform_char(ch, transform))
            .collect()
    }
}

#[inline]
fn should_wrap(
    next_raw_width: u16,
    current_raw_width: u16,
    max_width: Option<u16>,
    scale_x: f32,
) -> bool {
    max_width.is_some_and(|limit| {
        current_raw_width > 0 && scale_extent_allow_zero(next_raw_width, scale_x) > limit.max(1)
    })
}

fn finalize_layout(
    lines: Vec<TextLineLayout>,
    line_height: u16,
    line_gap: u16,
    scale_x: f32,
    scale_y: f32,
) -> TextLayout {
    let lines = if lines.is_empty() {
        vec![TextLineLayout {
            content: TextLineContent::Plain(String::new()),
            raw_width: 0,
        }]
    } else {
        lines
    };
    let raw_width = lines
        .iter()
        .map(|line| line.raw_width)
        .max()
        .unwrap_or(0)
        .max(1);
    let line_count = lines.len() as u16;
    let raw_height = line_height
        .saturating_mul(line_count.max(1))
        .saturating_add(line_gap.saturating_mul(line_count.saturating_sub(1)))
        .max(1);

    TextLayout {
        lines,
        width: scale_extent(raw_width, scale_x),
        height: scale_extent(raw_height, scale_y),
        line_height,
        line_gap,
    }
}

#[inline]
fn generic_char_width(ch: char, mode: generic::GenericMode) -> u16 {
    let mut buf = [0u8; 4];
    generic::generic_dimensions_mode(ch.encode_utf8(&mut buf), mode)
        .0
        .max(1)
}

#[inline]
fn raster_char_width(
    mod_source: Option<&Path>,
    font_name: &str,
    fg: Color,
    bg: Color,
    ch: char,
) -> u16 {
    let mut buf = [0u8; 4];
    rasterizer::rasterize_cached(mod_source, ch.encode_utf8(&mut buf), font_name, fg, bg)
        .width
        .max(1)
}

fn push_styled_char(runs: &mut Vec<StyledRun>, fg: Color, ch: char) {
    if let Some(last) = runs.last_mut() {
        if last.fg == fg {
            last.text.push(ch);
            return;
        }
    }

    runs.push(StyledRun {
        text: ch.to_string(),
        fg,
    });
}

fn layout_styled_text(
    content: &str,
    fg: Color,
    mode: Option<generic::GenericMode>,
    transform: &TextTransform,
    scale_x: f32,
    scale_y: f32,
    max_width: Option<u16>,
) -> TextLayout {
    let spans = parse_spans(content);
    let mut lines = Vec::new();
    let mut current_runs = Vec::new();
    let mut current_raw_width = 0u16;
    let max_width = max_width.map(|limit| limit.max(1));
    let line_height = mode.map(generic_mode_line_height).unwrap_or(1);
    let line_gap = mode.map(generic_mode_line_gap).unwrap_or(0);

    let flush_line = |lines: &mut Vec<TextLineLayout>,
                      current_runs: &mut Vec<StyledRun>,
                      current_raw_width: &mut u16| {
        lines.push(TextLineLayout {
            content: TextLineContent::Styled(std::mem::take(current_runs)),
            raw_width: *current_raw_width,
        });
        *current_raw_width = 0;
    };

    for span in spans {
        let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(fg);
        for ch in span.text.chars() {
            let ch = transform_char(ch, transform);
            if ch == '\n' {
                flush_line(&mut lines, &mut current_runs, &mut current_raw_width);
                continue;
            }

            let ch_width = mode.map(|value| generic_char_width(ch, value)).unwrap_or(1);
            let next_raw_width = current_raw_width.saturating_add(ch_width);
            if should_wrap(next_raw_width, current_raw_width, max_width, scale_x) {
                flush_line(&mut lines, &mut current_runs, &mut current_raw_width);
            }

            push_styled_char(&mut current_runs, span_fg, ch);
            current_raw_width = current_raw_width.saturating_add(ch_width);
        }
    }

    flush_line(&mut lines, &mut current_runs, &mut current_raw_width);
    finalize_layout(lines, line_height, line_gap, scale_x, scale_y)
}

fn layout_raster_text(
    mod_source: Option<&Path>,
    content: &str,
    font_name: &str,
    fg: Color,
    bg: Color,
    transform: &TextTransform,
    scale_x: f32,
    scale_y: f32,
    max_width: Option<u16>,
) -> TextLayout {
    let visible = transform_visible_text(content, transform);
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_raw_width = 0u16;
    let max_width = max_width.map(|limit| limit.max(1));
    let line_height = raster_line_height(mod_source, font_name, fg, bg);

    let flush_line = |lines: &mut Vec<TextLineLayout>,
                      current_line: &mut String,
                      current_raw_width: &mut u16| {
        let line_text = std::mem::take(current_line);
        let measured_width = if line_text.is_empty() {
            0
        } else {
            rasterizer::rasterize_cached(mod_source, &line_text, font_name, fg, bg)
                .width
                .max(1)
        };
        lines.push(TextLineLayout {
            content: TextLineContent::Plain(line_text),
            raw_width: measured_width.max(*current_raw_width),
        });
        *current_raw_width = 0;
    };

    for ch in visible.chars() {
        if ch == '\n' {
            flush_line(&mut lines, &mut current_line, &mut current_raw_width);
            continue;
        }

        let ch_width = raster_char_width(mod_source, font_name, fg, bg, ch);
        let next_raw_width = current_raw_width.saturating_add(ch_width);
        if should_wrap(next_raw_width, current_raw_width, max_width, scale_x) {
            flush_line(&mut lines, &mut current_line, &mut current_raw_width);
        }

        current_line.push(ch);
        current_raw_width = current_raw_width.saturating_add(ch_width);
    }

    flush_line(&mut lines, &mut current_line, &mut current_raw_width);
    finalize_layout(lines, line_height, 0, scale_x, scale_y)
}

fn layout_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    transform: &TextTransform,
    scale_x: f32,
    scale_y: f32,
    max_width: Option<u16>,
) -> TextLayout {
    match font {
        None => layout_styled_text(content, fg, None, transform, scale_x, scale_y, max_width),
        Some(font_name) if font_name.starts_with("generic") => layout_styled_text(
            content,
            fg,
            Some(generic::GenericMode::from_font_name(font_name)),
            transform,
            scale_x,
            scale_y,
            max_width,
        ),
        Some(font_name) => layout_raster_text(
            mod_source, content, font_name, fg, bg, transform, scale_x, scale_y, max_width,
        ),
    }
}

#[inline]
fn uses_extended_text_layout(options: ExtendedTextLayoutOptions) -> bool {
    options.max_width.is_some()
        || options.overflow_mode != TextOverflowMode::Clip
        || options.wrap_mode != TextWrapMode::None
        || options.line_clamp.is_some()
        || options.reserve_width_ch.is_some()
        || options.line_height != 1
}

fn measure_plain_text_width(
    mod_source: Option<&Path>,
    text: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
) -> u16 {
    match font {
        None => text.chars().count() as u16,
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            text.chars()
                .map(|ch| generic_char_width(ch, mode))
                .fold(0u16, |acc, width| acc.saturating_add(width))
        }
        Some(font_name) => text
            .chars()
            .map(|ch| raster_char_width(mod_source, font_name, fg, bg, ch))
            .fold(0u16, |acc, width| acc.saturating_add(width)),
    }
}

fn measure_reserve_width(
    mod_source: Option<&Path>,
    reserve_width_ch: Option<u16>,
    font: Option<&str>,
    fg: Color,
    bg: Color,
) -> u16 {
    let Some(count) = reserve_width_ch else {
        return 0;
    };
    if count == 0 {
        return 0;
    }
    let sample = "0".repeat(count as usize);
    measure_plain_text_width(mod_source, &sample, font, fg, bg)
}

fn intrinsic_line_height(
    mod_source: Option<&Path>,
    font: Option<&str>,
    fg: Color,
    bg: Color,
) -> u16 {
    match font {
        None => 1,
        Some(font_name) if font_name.starts_with("generic") => {
            generic_mode_line_height(generic::GenericMode::from_font_name(font_name))
        }
        Some(font_name) => raster_line_height(mod_source, font_name, fg, bg),
    }
}

fn scaled_line_offset(line_idx: usize, line_step: u16, scale_y: f32) -> u16 {
    scale_extent_allow_zero((line_idx as u16).saturating_mul(line_step), scale_y)
}

fn fit_plain_line_to_width(
    mod_source: Option<&Path>,
    line: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    max_width: Option<u16>,
    overflow_mode: TextOverflowMode,
) -> String {
    let Some(limit) = max_width.map(|value| value.max(1)) else {
        return line.to_string();
    };

    if measure_plain_text_width(mod_source, line, font, fg, bg) <= limit {
        return line.to_string();
    }

    let mut out = String::new();
    let ellipsis = if overflow_mode == TextOverflowMode::Ellipsis {
        Some("...")
    } else {
        None
    };
    let ellipsis_width = ellipsis
        .map(|value| measure_plain_text_width(mod_source, value, font, fg, bg))
        .unwrap_or(0);

    for ch in line.chars() {
        let mut candidate = out.clone();
        candidate.push(ch);
        let width = measure_plain_text_width(mod_source, &candidate, font, fg, bg);
        let budget = if ellipsis.is_some() {
            limit.saturating_sub(ellipsis_width)
        } else {
            limit
        };
        if width > budget.max(1) {
            break;
        }
        out = candidate;
    }

    if let Some(ellipsis) = ellipsis {
        let mut candidate = out.clone();
        candidate.push_str(ellipsis);
        while !out.is_empty()
            && measure_plain_text_width(mod_source, &candidate, font, fg, bg) > limit
        {
            out.pop();
            candidate = out.clone();
            candidate.push_str(ellipsis);
        }
        if measure_plain_text_width(mod_source, ellipsis, font, fg, bg) <= limit
            && !candidate.is_empty()
        {
            return candidate;
        }
    }

    out
}

fn wrap_plain_line(
    mod_source: Option<&Path>,
    line: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    max_width: Option<u16>,
    wrap_mode: TextWrapMode,
) -> Vec<String> {
    let Some(limit) = max_width else {
        return vec![line.to_string()];
    };
    if limit == 0 {
        return vec![String::new()];
    }

    let wrap_char = |line: &str| {
        let mut lines = Vec::new();
        let mut current = String::new();
        for ch in line.chars() {
            let mut candidate = current.clone();
            candidate.push(ch);
            if !current.is_empty()
                && measure_plain_text_width(mod_source, &candidate, font, fg, bg) > limit
            {
                lines.push(current);
                current = ch.to_string();
            } else {
                current = candidate;
            }
        }
        lines.push(current);
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    };

    match wrap_mode {
        TextWrapMode::None => vec![line.to_string()],
        TextWrapMode::Char => wrap_char(line),
        TextWrapMode::Word => {
            let mut segments = Vec::new();
            let mut chars = line.chars().peekable();
            while chars.peek().is_some() {
                let mut segment = String::new();
                while let Some(&ch) = chars.peek() {
                    if !ch.is_whitespace() {
                        break;
                    }
                    segment.push(ch);
                    chars.next();
                }
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        break;
                    }
                    segment.push(ch);
                    chars.next();
                }
                while let Some(&ch) = chars.peek() {
                    if !ch.is_whitespace() {
                        break;
                    }
                    segment.push(ch);
                    chars.next();
                }
                if !segment.is_empty() {
                    segments.push(segment);
                }
            }
            if segments.is_empty() {
                segments.push(String::new());
            }

            let mut lines = Vec::new();
            let mut current = String::new();
            for segment in segments {
                let candidate = if current.is_empty() {
                    segment.clone()
                } else {
                    format!("{current}{segment}")
                };
                if current.is_empty()
                    || measure_plain_text_width(mod_source, &candidate, font, fg, bg) <= limit
                {
                    current = candidate;
                } else {
                    lines.push(current);
                    current = segment;
                }
                if measure_plain_text_width(mod_source, &current, font, fg, bg) > limit {
                    let wrapped = wrap_char(&current);
                    if let Some((last, rest)) = wrapped.split_last() {
                        lines.extend(rest.iter().cloned());
                        current = last.clone();
                    } else {
                        current.clear();
                    }
                    if measure_plain_text_width(mod_source, &current, font, fg, bg) > limit {
                        lines.extend(wrap_char(&current));
                        current = String::new();
                    }
                }
            }
            if lines.is_empty() && current.is_empty() && line.is_empty() {
                lines.push(String::new());
            } else if !current.is_empty() {
                lines.push(current);
            }
            lines
        }
    }
}

fn layout_plain_text_content(
    mod_source: Option<&Path>,
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    transform: &TextTransform,
    scale_x: f32,
    scale_y: f32,
    options: ExtendedTextLayoutOptions,
) -> PlainTextLayout {
    let visible = transform_visible_text(content, transform);
    let intrinsic_height = intrinsic_line_height(mod_source, font, fg, bg).max(1);
    let line_multiplier = options.line_height.max(1);
    let line_step = intrinsic_height.saturating_mul(line_multiplier);
    let mut lines = Vec::new();

    for raw_line in visible.split('\n') {
        let wrapped = wrap_plain_line(
            mod_source,
            raw_line,
            font,
            fg,
            bg,
            options.max_width,
            options.wrap_mode,
        );
        for line in wrapped {
            let fitted = if matches!(options.wrap_mode, TextWrapMode::None) {
                fit_plain_line_to_width(
                    mod_source,
                    &line,
                    font,
                    fg,
                    bg,
                    options.max_width,
                    options.overflow_mode,
                )
            } else {
                line
            };
            lines.push(fitted);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    if let Some(clamp) = options.line_clamp {
        let clamp = clamp.max(1) as usize;
        if lines.len() > clamp {
            lines.truncate(clamp);
            if let Some(last) = lines.last_mut() {
                if options.overflow_mode == TextOverflowMode::Ellipsis {
                    *last = if options.max_width.is_some() {
                        fit_plain_line_to_width(
                            mod_source,
                            &(last.clone() + "..."),
                            font,
                            fg,
                            bg,
                            options.max_width,
                            TextOverflowMode::Ellipsis,
                        )
                    } else {
                        format!("{last}...")
                    };
                }
            }
        }
    }

    let raw_widths: Vec<u16> = lines
        .iter()
        .map(|line| measure_plain_text_width(mod_source, line, font, fg, bg))
        .collect();
    let reserve_width = measure_reserve_width(mod_source, options.reserve_width_ch, font, fg, bg);
    let raw_width = raw_widths
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
        .max(reserve_width)
        .max(1);
    let raw_height = intrinsic_height
        .saturating_add(line_step.saturating_mul(lines.len().saturating_sub(1) as u16))
        .max(1);

    PlainTextLayout {
        lines,
        raw_widths,
        width: scale_extent(raw_width, scale_x),
        height: scale_extent(raw_height, scale_y),
        line_height: intrinsic_height,
        line_step,
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
    max_width: Option<u16>,
    overflow_mode: TextOverflowMode,
    wrap_mode: TextWrapMode,
    line_clamp: Option<u16>,
    reserve_width_ch: Option<u16>,
    line_height: u16,
    scale_x: f32,
    scale_y: f32,
) {
    let options = ExtendedTextLayoutOptions {
        max_width,
        overflow_mode,
        wrap_mode,
        line_clamp,
        reserve_width_ch,
        line_height,
    };
    if uses_extended_text_layout(options) {
        let layout = layout_plain_text_content(
            mod_source, content, font, fg, bg, transform, scale_x, scale_y, options,
        );
        match font {
            None => {
                for (line_idx, line) in layout.lines.iter().enumerate() {
                    let line_y =
                        y.saturating_add(scaled_line_offset(line_idx, layout.line_step, scale_y));
                    TEXT_LINE_BUF.with(|cell| {
                        let line_buf = &mut *cell.borrow_mut();
                        line_buf.resize(
                            layout.raw_widths[line_idx].max(1),
                            layout.line_height.max(1),
                        );
                        line_buf.fill(Color::Reset);
                        let mut col = 0u16;
                        for ch in line.chars() {
                            line_buf.set(col, 0, ch, fg, bg);
                            col = col.saturating_add(1);
                        }
                        blit_scaled(line_buf, buf, x, line_y, clip, scale_x, scale_y);
                    });
                }
            }
            Some(font_name) if font_name.starts_with("generic") => {
                let mode = generic::GenericMode::from_font_name(font_name);
                for (line_idx, line) in layout.lines.iter().enumerate() {
                    let line_y =
                        y.saturating_add(scaled_line_offset(line_idx, layout.line_step, scale_y));
                    TEXT_LINE_BUF.with(|cell| {
                        let line_buf = &mut *cell.borrow_mut();
                        line_buf.resize(
                            layout.raw_widths[line_idx].max(1),
                            layout.line_height.max(1),
                        );
                        line_buf.fill(Color::Reset);
                        generic::rasterize_spans_mode(
                            &[(line.as_str(), fg)],
                            mode,
                            0,
                            0,
                            line_buf,
                            &TextTransform::None,
                        );
                        blit_scaled(line_buf, buf, x, line_y, clip, scale_x, scale_y);
                    });
                }
            }
            Some(font_name) => {
                for (line_idx, line) in layout.lines.iter().enumerate() {
                    if line.is_empty() {
                        continue;
                    }
                    let text_buf =
                        rasterizer::rasterize_cached(mod_source, line, font_name, fg, bg);
                    let line_y =
                        y.saturating_add(scaled_line_offset(line_idx, layout.line_step, scale_y));
                    blit_scaled(&text_buf, buf, x, line_y, clip, scale_x, scale_y);
                }
            }
        }
        return;
    }

    let layout = layout_text_content(
        mod_source, content, font, fg, bg, transform, scale_x, scale_y, max_width,
    );
    let line_step = layout.line_height.saturating_add(layout.line_gap);

    match font {
        None => {
            for (line_idx, line) in layout.lines.iter().enumerate() {
                let TextLineContent::Styled(runs) = &line.content else {
                    continue;
                };
                let line_y = y.saturating_add(scaled_line_offset(line_idx, line_step, scale_y));
                TEXT_LINE_BUF.with(|cell| {
                    let line_buf = &mut *cell.borrow_mut();
                    line_buf.resize(line.raw_width.max(1), 1);
                    line_buf.fill(Color::Reset);

                    let mut col = 0u16;
                    for run in runs {
                        for ch in run.text.chars() {
                            line_buf.set(col, 0, ch, run.fg, bg);
                            col = col.saturating_add(1);
                        }
                    }

                    blit_scaled(line_buf, buf, x, line_y, clip, scale_x, scale_y);
                });
            }
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            for (line_idx, line) in layout.lines.iter().enumerate() {
                let TextLineContent::Styled(runs) = &line.content else {
                    continue;
                };
                let line_y = y.saturating_add(scaled_line_offset(line_idx, line_step, scale_y));
                TEXT_LINE_BUF.with(|cell| {
                    let line_buf = &mut *cell.borrow_mut();
                    line_buf.resize(line.raw_width.max(1), layout.line_height.max(1));
                    line_buf.fill(Color::Reset);
                    let colored_spans: Vec<(&str, Color)> =
                        runs.iter().map(|run| (run.text.as_str(), run.fg)).collect();
                    generic::rasterize_spans_mode(
                        &colored_spans,
                        mode,
                        0,
                        0,
                        line_buf,
                        &TextTransform::None,
                    );
                    blit_scaled(line_buf, buf, x, line_y, clip, scale_x, scale_y);
                });
            }
        }
        Some(font_name) => {
            for (line_idx, line) in layout.lines.iter().enumerate() {
                let TextLineContent::Plain(text) = &line.content else {
                    continue;
                };
                if text.is_empty() {
                    continue;
                }
                let text_buf = rasterizer::rasterize_cached(mod_source, text, font_name, fg, bg);
                let line_y = y.saturating_add(scaled_line_offset(line_idx, line_step, scale_y));
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
    transform: &TextTransform,
    max_width: Option<u16>,
    overflow_mode: TextOverflowMode,
    wrap_mode: TextWrapMode,
    line_clamp: Option<u16>,
    reserve_width_ch: Option<u16>,
    line_height: u16,
    scale_x: f32,
    scale_y: f32,
) -> (u16, u16) {
    let options = ExtendedTextLayoutOptions {
        max_width,
        overflow_mode,
        wrap_mode,
        line_clamp,
        reserve_width_ch,
        line_height,
    };
    if uses_extended_text_layout(options) {
        let layout = layout_plain_text_content(
            mod_source, content, font, fg, bg, transform, scale_x, scale_y, options,
        );
        (layout.width, layout.height)
    } else {
        let layout = layout_text_content(
            mod_source, content, font, fg, bg, transform, scale_x, scale_y, max_width,
        );
        (layout.width, layout.height)
    }
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
    use engine_core::scene::{TextOverflowMode, TextTransform, TextWrapMode};

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
            &TextTransform::None,
            None,
            TextOverflowMode::Clip,
            TextWrapMode::None,
            None,
            None,
            1,
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
            None,
            TextOverflowMode::Clip,
            TextWrapMode::None,
            None,
            None,
            1,
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

    #[test]
    fn plain_text_wrap_dimensions_match_rendering() {
        let (w, h) = text_sprite_dimensions(
            None,
            "ABCD",
            None,
            Color::White,
            Color::Reset,
            &TextTransform::None,
            Some(2),
            TextOverflowMode::Clip,
            TextWrapMode::Char,
            None,
            None,
            1,
            1.0,
            1.0,
        );
        assert_eq!((w, h), (2, 2));

        let mut buf = Buffer::new(4, 3);
        buf.fill(Color::Reset);
        render_text_content(
            None,
            "ABCD",
            None,
            Color::White,
            Color::Reset,
            0,
            0,
            None,
            &mut buf,
            &TextTransform::None,
            Some(2),
            TextOverflowMode::Clip,
            TextWrapMode::Char,
            None,
            None,
            1,
            1.0,
            1.0,
        );

        assert_eq!(buf.get(0, 0).expect("a").symbol, 'A');
        assert_eq!(buf.get(1, 0).expect("b").symbol, 'B');
        assert_eq!(buf.get(0, 1).expect("c").symbol, 'C');
        assert_eq!(buf.get(1, 1).expect("d").symbol, 'D');
    }

    #[test]
    fn ellipsis_dimensions_respect_max_width() {
        let (w, h) = text_sprite_dimensions(
            None,
            "ABCDEFG",
            None,
            Color::White,
            Color::Reset,
            &TextTransform::None,
            Some(5),
            TextOverflowMode::Ellipsis,
            TextWrapMode::None,
            None,
            None,
            1,
            1.0,
            1.0,
        );
        assert_eq!((w, h), (5, 1));
    }

    #[test]
    fn uppercase_transform_affects_terminal_rendering() {
        let mut buf = Buffer::new(4, 1);
        buf.fill(Color::Reset);
        render_text_content(
            None,
            "ab",
            None,
            Color::White,
            Color::Reset,
            0,
            0,
            None,
            &mut buf,
            &TextTransform::Uppercase,
            None,
            TextOverflowMode::Clip,
            TextWrapMode::None,
            None,
            None,
            1,
            1.0,
            1.0,
        );

        assert_eq!(buf.get(0, 0).expect("first").symbol, 'A');
        assert_eq!(buf.get(1, 0).expect("second").symbol, 'B');
    }

    #[test]
    fn word_wrap_preserves_visible_spaces() {
        let mut buf = Buffer::new(4, 2);
        buf.fill(Color::Reset);
        render_text_content(
            None,
            "A B",
            None,
            Color::White,
            Color::Reset,
            0,
            0,
            None,
            &mut buf,
            &TextTransform::None,
            Some(2),
            TextOverflowMode::Clip,
            TextWrapMode::Word,
            None,
            None,
            1,
            1.0,
            1.0,
        );

        assert_eq!(buf.get(0, 0).expect("first line a").symbol, 'A');
        assert_eq!(buf.get(1, 0).expect("first line space").symbol, ' ');
        assert_eq!(buf.get(0, 1).expect("second line b").symbol, 'B');
    }
}
