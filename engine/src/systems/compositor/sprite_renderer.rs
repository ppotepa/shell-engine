use crossterm::style::Color;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::rasterizer;
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
                reveal_ms, stages, ..
            } => {
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
                let mut sprite_width = rendered_content.chars().count() as u16;
                let mut sprite_height = 1u16;

                let draw_x;
                let draw_y;

                match font {
                    None => {
                        draw_x = resolve_x(*x, align_x, scene_w, sprite_width);
                        draw_y = resolve_y(*y, align_y, scene_h, sprite_height);
                        for (i, ch) in rendered_content.chars().enumerate() {
                            layer_buf.set(draw_x + i as u16, draw_y, ch, fg, sprite_bg);
                        }
                    }
                    Some(font_name) => {
                        let text_buf = rasterizer::rasterize(&rendered_content, font_name, fg, sprite_bg);
                        sprite_width = text_buf.width;
                        sprite_height = text_buf.height;
                        draw_x = resolve_x(*x, align_x, scene_w, sprite_width);
                        draw_y = resolve_y(*y, align_y, scene_h, sprite_height);
                        rasterizer::blit(&text_buf, layer_buf, draw_x, draw_y);
                    }
                }

                let sprite_elapsed = scene_elapsed_ms.saturating_sub(appear_at);
                let sprite_region = Region { x: draw_x, y: draw_y, width: sprite_width, height: sprite_height };
                apply_sprite_effects(stages, current_stage, step_idx, elapsed_ms, sprite_elapsed, sprite_region, layer_buf);
            }
        }
    }
}

fn resolve_x(offset_x: u16, align_x: &Option<HorizontalAlign>, scene_w: u16, sprite_w: u16) -> u16 {
    let origin = match align_x {
        Some(HorizontalAlign::Left)   => 0,
        Some(HorizontalAlign::Center) => scene_w.saturating_sub(sprite_w) / 2,
        Some(HorizontalAlign::Right)  => scene_w.saturating_sub(sprite_w),
        None => 0,
    };
    origin.saturating_add(offset_x)
}

fn resolve_y(offset_y: u16, align_y: &Option<VerticalAlign>, scene_h: u16, sprite_h: u16) -> u16 {
    let origin = match align_y {
        Some(VerticalAlign::Top)    => 0,
        Some(VerticalAlign::Center) => scene_h.saturating_sub(sprite_h) / 2,
        Some(VerticalAlign::Bottom) => scene_h.saturating_sub(sprite_h),
        None => 0,
    };
    origin.saturating_add(offset_y)
}
