//! Layout helpers and sprite measurement adapters for 2D rendering.

use std::cell::{Cell, RefCell};

use engine_core::assets::AssetRoot;
use engine_core::color::Color;
use engine_core::scene::{FlexDirection, Sprite, SpriteSizePreset};

pub use engine_layout::{
    compute_flex_cells, compute_grid_cells, parse_track_spec, resolve_x, resolve_y, GridCellRect,
    RenderArea, TrackSpec,
};

use crate::{image_sprite_dimensions, text_sprite_dimensions};

thread_local! {
    static PIXEL_BACKEND: Cell<bool> = const { Cell::new(false) };
    static DEFAULT_FONT_SPEC: RefCell<Option<String>> = const { RefCell::new(None) };
    static UI_FONT_SCALE: Cell<f32> = const { Cell::new(1.0) };
}

#[inline]
pub fn with_render_context<R>(
    is_pixel: bool,
    default_font: Option<&str>,
    ui_font_scale: f32,
    f: impl FnOnce() -> R,
) -> R {
    PIXEL_BACKEND.with(|c| c.set(is_pixel));
    DEFAULT_FONT_SPEC.with(|slot| {
        *slot.borrow_mut() = default_font.map(str::to_string);
    });
    UI_FONT_SCALE.with(|slot| slot.set(ui_font_scale.max(0.01)));
    let result = f();
    PIXEL_BACKEND.with(|c| c.set(false));
    DEFAULT_FONT_SPEC.with(|slot| {
        *slot.borrow_mut() = None;
    });
    UI_FONT_SCALE.with(|slot| slot.set(1.0));
    result
}

pub fn measure_sprite_for_layout(sprite: &Sprite, asset_root: Option<&AssetRoot>) -> (u16, u16) {
    match sprite {
        Sprite::Text {
            content,
            size,
            font,
            force_font_mode,
            fg_colour,
            bg_colour,
            scale_x,
            scale_y,
            ..
        } => {
            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let default_font = DEFAULT_FONT_SPEC.with(|slot| slot.borrow().clone());
            let resolved_font = engine_render_policy::resolve_text_font_spec(
                font.as_deref(),
                force_font_mode.as_deref(),
                *size,
                PIXEL_BACKEND.with(|c| c.get()),
                default_font.as_deref(),
            );
            text_sprite_dimensions(
                asset_root.map(|root| root.mod_source()),
                content,
                resolved_font.as_deref(),
                fg,
                bg,
                *scale_x * UI_FONT_SCALE.with(|slot| slot.get()),
                *scale_y * UI_FONT_SCALE.with(|slot| slot.get()),
            )
        }
        Sprite::Image {
            source,
            size,
            width,
            height,
            spritesheet_columns,
            spritesheet_rows,
            frame_index,
            ..
        } => image_sprite_dimensions(
            source,
            *width,
            *height,
            *size,
            *spritesheet_columns,
            *spritesheet_rows,
            *frame_index,
            asset_root,
        ),
        Sprite::Planet {
            width,
            height,
            size,
            ..
        }
        | Sprite::Obj {
            width,
            height,
            size,
            ..
        } => mesh_sprite_dimensions(*width, *height, *size),
        Sprite::Vector { points, .. } => engine_vector::bounds(points)
            .map(|b| (b.width.max(1), b.height.max(1)))
            .unwrap_or((1, 1)),
        Sprite::Grid {
            width,
            height,
            columns,
            rows,
            gap_x,
            gap_y,
            children,
            ..
        } => {
            if let (Some(w), Some(h)) = (width.as_ref().copied(), height.as_ref().copied()) {
                return (w.max(1), h.max(1));
            }

            let col_specs: Vec<TrackSpec> = if columns.is_empty() {
                vec![TrackSpec::Fr(1)]
            } else {
                columns.iter().map(|c| parse_track_spec(c)).collect()
            };
            let row_specs: Vec<TrackSpec> = if rows.is_empty() {
                vec![TrackSpec::Fr(1)]
            } else {
                rows.iter().map(|r| parse_track_spec(r)).collect()
            };

            let mut col_auto_reqs = vec![1u16; col_specs.len().max(1)];
            let mut row_auto_reqs = vec![1u16; row_specs.len().max(1)];

            for child in children {
                let (pref_w, pref_h) = measure_sprite_for_layout(child, asset_root);
                let (row, col, row_span, col_span) = child.grid_position();

                let col_idx = (col as usize)
                    .saturating_sub(1)
                    .min(col_auto_reqs.len().saturating_sub(1));
                let row_idx = (row as usize)
                    .saturating_sub(1)
                    .min(row_auto_reqs.len().saturating_sub(1));
                let col_span_clamped = (col_span as usize)
                    .max(1)
                    .min(col_auto_reqs.len().saturating_sub(col_idx));
                let row_span_clamped = (row_span as usize)
                    .max(1)
                    .min(row_auto_reqs.len().saturating_sub(row_idx));

                let col_gaps = gap_x.saturating_mul((col_span_clamped.saturating_sub(1)) as u16);
                let row_gaps = gap_y.saturating_mul((row_span_clamped.saturating_sub(1)) as u16);
                let col_share = pref_w
                    .saturating_sub(col_gaps)
                    .saturating_div(col_span_clamped as u16)
                    .max(1);
                let row_share = pref_h
                    .saturating_sub(row_gaps)
                    .saturating_div(row_span_clamped as u16)
                    .max(1);

                for req in &mut col_auto_reqs[col_idx..(col_idx + col_span_clamped)] {
                    *req = (*req).max(col_share);
                }
                for req in &mut row_auto_reqs[row_idx..(row_idx + row_span_clamped)] {
                    *req = (*req).max(row_share);
                }
            }

            let measured_w = width.unwrap_or_else(|| {
                let tracks_sum = col_specs.iter().enumerate().fold(0u16, |acc, (idx, spec)| {
                    let size = match spec {
                        TrackSpec::Fixed(px) => *px,
                        TrackSpec::Auto | TrackSpec::Fr(_) => col_auto_reqs[idx].max(1),
                    };
                    acc.saturating_add(size)
                });
                let gaps = gap_x.saturating_mul(col_specs.len().saturating_sub(1) as u16);
                tracks_sum.saturating_add(gaps).max(1)
            });

            let measured_h = height.unwrap_or_else(|| {
                let tracks_sum = row_specs.iter().enumerate().fold(0u16, |acc, (idx, spec)| {
                    let size = match spec {
                        TrackSpec::Fixed(px) => *px,
                        TrackSpec::Auto | TrackSpec::Fr(_) => row_auto_reqs[idx].max(1),
                    };
                    acc.saturating_add(size)
                });
                let gaps = gap_y.saturating_mul(row_specs.len().saturating_sub(1) as u16);
                tracks_sum.saturating_add(gaps).max(1)
            });

            (measured_w.max(1), measured_h.max(1))
        }
        Sprite::Panel {
            width,
            width_percent: _,
            height,
            padding,
            border_width,
            children,
            ..
        } => {
            if let (Some(w), Some(h)) = (width.as_ref().copied(), height.as_ref().copied()) {
                return (w.max(1), h.max(1));
            }
            let inset = border_width.saturating_add(*padding).max(1);
            let max_w = children
                .iter()
                .map(|c| measure_sprite_for_layout(c, asset_root).0)
                .max()
                .unwrap_or(1);
            let sum_h: u16 = children
                .iter()
                .map(|c| measure_sprite_for_layout(c, asset_root).1)
                .fold(0u16, |acc, h| acc.saturating_add(h))
                .max(1);
            let measured_w = if let Some(explicit) = *width {
                explicit
            } else {
                max_w.saturating_add(inset.saturating_mul(2))
            };
            let measured_h = height.unwrap_or(sum_h.saturating_add(inset.saturating_mul(2)));
            (measured_w.max(1), measured_h.max(1))
        }
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
                        .map(|c| measure_sprite_for_layout(c, asset_root).0)
                        .max()
                        .unwrap_or(1);
                    let sum_h: u16 = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, asset_root).1)
                        .fold(0u16, |acc, h| acc.saturating_add(h));
                    let gaps = gap.saturating_mul(n.saturating_sub(1) as u16);
                    (
                        width.unwrap_or(max_w),
                        height.unwrap_or(sum_h.saturating_add(gaps)),
                    )
                }
                FlexDirection::Row => {
                    let sum_w: u16 = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, asset_root).0)
                        .fold(0u16, |acc, w| acc.saturating_add(w));
                    let max_h = children
                        .iter()
                        .map(|c| measure_sprite_for_layout(c, asset_root).1)
                        .max()
                        .unwrap_or(1);
                    let gaps = gap.saturating_mul(n.saturating_sub(1) as u16);
                    (
                        width.unwrap_or(sum_w.saturating_add(gaps)),
                        height.unwrap_or(max_h),
                    )
                }
            };
            (total_w.max(1), total_h.max(1))
        }
        Sprite::Scene3D { .. } => (1, 1),
    }
}

fn mesh_sprite_dimensions(
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
) -> (u16, u16) {
    match (width, height) {
        (Some(w), Some(h)) => (w.max(1), h.max(1)),
        (Some(w), None) => (w.max(1), 24),
        (None, Some(h)) => (64, h.max(1)),
        (None, None) => size.unwrap_or(SpriteSizePreset::Medium).obj_dimensions(),
    }
}
