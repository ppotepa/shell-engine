//! Shared layout primitives used by compositor containers.

use engine_core::scene::{HorizontalAlign, VerticalAlign};

/// A drawable area available to a sprite during render traversal.
#[derive(Clone, Copy)]
pub struct RenderArea {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u16,
    pub height: u16,
}

/// A resolved child rectangle inside a container layout.
#[derive(Clone, Copy)]
pub struct GridCellRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Resolves horizontal alignment plus authored offset into a local x origin.
pub fn resolve_x(
    offset_x: i32,
    align_x: &Option<HorizontalAlign>,
    area_w: u16,
    sprite_w: u16,
) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) | None => 0i32,
        Some(HorizontalAlign::Center) => (area_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => area_w.saturating_sub(sprite_w) as i32,
    };
    origin.saturating_add(offset_x)
}

/// Resolves vertical alignment plus authored offset into a local y origin.
pub fn resolve_y(
    offset_y: i32,
    align_y: &Option<VerticalAlign>,
    area_h: u16,
    sprite_h: u16,
) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) | None => 0i32,
        Some(VerticalAlign::Center) => (area_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => area_h.saturating_sub(sprite_h) as i32,
    };
    origin.saturating_add(offset_y)
}
