use crossterm::style::Color;
use crate::animations::AnimationDispatcher;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::markup::{parse_spans, strip_markup};
use crate::rasterizer;
use crate::rasterizer::generic;
use crate::scene::{HorizontalAlign, Layer, Sprite, VerticalAlign};
use crate::systems::animator::SceneStage;
use super::effect_applicator::apply_sprite_effects;

/// Render all sprites in a layer onto `layer_buf`.
pub fn render_sprites(
    layer: &mut Layer,
    scene_w: u16,
    scene_h: u16,
    scene_elapsed_ms: u64,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    layer_buf: &mut Buffer,
) {
    layer.sprites.sort_by_key(|s| match s {
        Sprite::Text { z_index, .. } => *z_index,
    });

    for sprite in &layer.sprites {
        match sprite {
            Sprite::Text {
                content, x, y, font, align_x, align_y,
                fg_colour, bg_colour, appear_at_ms, disappear_at_ms,
                reveal_ms, hide_on_leave, stages, animations, glow, ..
            } => {
                if *hide_on_leave && matches!(current_stage, SceneStage::OnLeave) {
                    continue;
                }
                let appear_at = appear_at_ms.unwrap_or(0);
                if scene_elapsed_ms < appear_at { continue; }
                if let Some(disappear_at) = disappear_at_ms {
                    if scene_elapsed_ms >= *disappear_at { continue; }
                }

                let total_chars = content.chars().count();
                let rendered_content = match reveal_ms {
                    Some(reveal) if *reveal > 0 => {
                        let since = scene_elapsed_ms - appear_at;
                        let p = (since as f32 / *reveal as f32).clamp(0.0, 1.0);
                        let visible_chars = ((total_chars as f32) * p).ceil() as usize;
                        content.chars().take(visible_chars).collect::<String>()
                    }
                    _ => content.clone(),
                };
                if rendered_content.is_empty() { continue; }

                let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
                let sprite_bg = match bg_colour.as_ref() {
                    Some(c) => Color::from(c),
                    None => Color::Reset,
                };

                // Compute sprite dimensions for alignment
                let (sprite_width, sprite_height) =
                    sprite_dimensions(&rendered_content, font.as_deref(), fg, sprite_bg);

                let base_x = resolve_x(*x, align_x, scene_w, sprite_width);
                let base_y = resolve_y(*y, align_y, scene_h, sprite_height);
                let sprite_elapsed = scene_elapsed_ms.saturating_sub(appear_at);
                // TODO: move AnimationDispatcher to engine init instead of per-frame
                let anim_dispatcher = AnimationDispatcher::new();
                let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
                let draw_x = base_x
                    .saturating_add(transform.dx as i32)
                    .clamp(0, u16::MAX as i32) as u16;
                let draw_y = base_y
                    .saturating_add(transform.dy as i32)
                    .clamp(0, u16::MAX as i32) as u16;

                // Glow pass — render stripped content at each offset in glow colour
                if let Some(glow_opts) = glow.as_ref() {
                    let glow_col = glow_opts.colour.as_ref()
                        .map(|c| Color::from(c))
                        .unwrap_or_else(|| dim_colour(fg));
                    let radius = glow_opts.radius.max(1) as i32;
                    let glow_content = strip_markup(&rendered_content);
                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            if dx == 0 && dy == 0 { continue; }
                            let gx = (draw_x as i32 + dx).max(0) as u16;
                            let gy = (draw_y as i32 + dy).max(0) as u16;
                            render_text_content(&glow_content, font.as_deref(), glow_col, sprite_bg, gx, gy, layer_buf);
                        }
                    }
                }

                // Render actual content on top
                render_text_content(&rendered_content, font.as_deref(), fg, sprite_bg, draw_x, draw_y, layer_buf);

                let sprite_region = Region { x: draw_x, y: draw_y, width: sprite_width, height: sprite_height };
                apply_sprite_effects(stages, current_stage, step_idx, elapsed_ms, sprite_elapsed, sprite_region, layer_buf);
            }
        }
    }
}

/// Render text content at (x, y) on `buf`, respecting font and inline colour markup.
fn render_text_content(
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
            // Native text — parse markup spans and render each with its colour
            let spans = parse_spans(content);
            let mut col = 0u16;
            for span in &spans {
                let span_fg = span.colour.as_ref()
                    .map(|c| Color::from(c))
                    .unwrap_or(fg);
                for ch in span.text.chars() {
                    buf.set(x + col, y, ch, span_fg, bg);
                    col += 1;
                }
            }
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let spans = parse_spans(content);
            let colored_spans: Vec<(String, Color)> = spans.iter()
                .map(|s| {
                    let col = s.colour.as_ref().map(|c| Color::from(c)).unwrap_or(fg);
                    (s.text.clone(), col)
                })
                .collect();
            generic::rasterize_spans_mode(&colored_spans, mode, x, y, buf);
        }
        Some(font_name) => {
            // TODO: add per-span markup colour support for manifest fonts
            let stripped = strip_markup(content);
            let text_buf = rasterizer::rasterize(&stripped, font_name, fg, bg);
            rasterizer::blit(&text_buf, buf, x, y);
        }
    }
}

/// Compute the rendered dimensions (width, height) of content with the given font.
fn sprite_dimensions(content: &str, font: Option<&str>, fg: Color, bg: Color) -> (u16, u16) {
    let visible = strip_markup(content);
    match font {
        None => (visible.chars().count() as u16, 1),
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            generic::generic_dimensions_mode(&visible, mode)
        }
        Some(font_name) => {
            let text_buf = rasterizer::rasterize(&visible, font_name, fg, bg);
            (text_buf.width, text_buf.height)
        }
    }
}

/// Dim a colour to ~25% brightness for use as a default glow colour.
fn dim_colour(c: Color) -> Color {
    use crate::effects::utils::color::colour_to_rgb;
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * 0.25) as u8,
        g: (g as f32 * 0.25) as u8,
        b: (b as f32 * 0.25) as u8,
    }
}

fn resolve_x(offset_x: i32, align_x: &Option<HorizontalAlign>, scene_w: u16, sprite_w: u16) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) => 0i32,
        Some(HorizontalAlign::Center) => (scene_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => scene_w.saturating_sub(sprite_w) as i32,
        None => 0i32,
    };
    origin.saturating_add(offset_x)
}

fn resolve_y(offset_y: i32, align_y: &Option<VerticalAlign>, scene_h: u16, sprite_h: u16) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) => 0i32,
        Some(VerticalAlign::Center) => (scene_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => scene_h.saturating_sub(sprite_h) as i32,
        None => 0i32,
    };
    origin.saturating_add(offset_y)
}
