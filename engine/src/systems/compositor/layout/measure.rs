//! Measurement helpers that estimate sprite bounds before rasterization.

use crossterm::style::Color;

use crate::assets::AssetRoot;
use crate::render_policy;
use crate::scene::{FlexDirection, SceneRenderedMode, Sprite};
use crate::systems::compositor::image_render::image_sprite_dimensions;
use crate::systems::compositor::obj_render::obj_sprite_dimensions;
use crate::systems::compositor::text_render::text_sprite_dimensions;

/// Measures the approximate render size of a sprite for layout purposes.
pub(crate) fn measure_sprite_for_layout(
    sprite: &Sprite,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> (u16, u16) {
    match sprite {
        Sprite::Text {
            content,
            size,
            font,
            force_renderer_mode,
            force_font_mode,
            fg_colour,
            bg_colour,
            ..
        } => {
            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let resolved_font = render_policy::resolve_text_font_spec(
                font.as_deref(),
                force_font_mode.as_deref(),
                *size,
                inherited_mode,
                *force_renderer_mode,
            );
            text_sprite_dimensions(
                asset_root.map(|root| root.mod_source()),
                content,
                resolved_font.as_deref(),
                fg,
                bg,
            )
        }
        Sprite::Image {
            source,
            size,
            width,
            height,
            force_renderer_mode,
            ..
        } => {
            let mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            image_sprite_dimensions(source, *width, *height, *size, mode, asset_root)
        }
        Sprite::Grid { width, height, .. } => (width.unwrap_or(1).max(1), height.unwrap_or(1).max(1)),
        Sprite::Obj { width, height, size, .. } => obj_sprite_dimensions(*width, *height, *size),
        Sprite::Flex {
            width,
            height,
            direction,
            gap,
            children,
            ..
        } => {
            let n = children.len();
            if n == 0 {
                return (1, 1);
            }
            let (total_w, total_h) = match direction {
                FlexDirection::Column => {
                    let max_w = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, inherited_mode, asset_root).0)
                        .max()
                        .unwrap_or(1);
                    let sum_h: u16 = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, inherited_mode, asset_root).1)
                        .fold(0u16, |acc, h| acc.saturating_add(h));
                    let gaps = gap.saturating_mul(n.saturating_sub(1) as u16);
                    (width.unwrap_or(max_w), height.unwrap_or(sum_h.saturating_add(gaps)))
                }
                FlexDirection::Row => {
                    let sum_w: u16 = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, inherited_mode, asset_root).0)
                        .fold(0u16, |acc, w| acc.saturating_add(w));
                    let max_h = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, inherited_mode, asset_root).1)
                        .max()
                        .unwrap_or(1);
                    let gaps = gap.saturating_mul(n.saturating_sub(1) as u16);
                    (width.unwrap_or(sum_w.saturating_add(gaps)), height.unwrap_or(max_h))
                }
            };
            (total_w.max(1), total_h.max(1))
        }
    }
}
