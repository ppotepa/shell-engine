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
/// Called per sprite during layout — inline for alignment optimization.
#[inline]
pub fn resolve_x(
    offset_x: i32,
    align_x: &Option<HorizontalAlign>,
    area_w: u16,
    sprite_w: u16,
) -> i32 {
    let area_w = i32::from(area_w);
    let sprite_w = i32::from(sprite_w);
    let origin = match align_x {
        Some(HorizontalAlign::Left) | None => 0i32,
        Some(HorizontalAlign::Center) => (area_w - sprite_w) / 2,
        Some(HorizontalAlign::Right) => area_w - sprite_w,
    };
    origin.saturating_add(offset_x)
}

/// Resolves vertical alignment plus authored offset into a local y origin.
/// Called per sprite during layout — inline for alignment optimization.
#[inline]
pub fn resolve_y(
    offset_y: i32,
    align_y: &Option<VerticalAlign>,
    area_h: u16,
    sprite_h: u16,
) -> i32 {
    let area_h = i32::from(area_h);
    let sprite_h = i32::from(sprite_h);
    let origin = match align_y {
        Some(VerticalAlign::Top) | None => 0i32,
        Some(VerticalAlign::Center) => (area_h - sprite_h) / 2,
        Some(VerticalAlign::Bottom) => area_h - sprite_h,
    };
    origin.saturating_add(offset_y)
}

#[cfg(test)]
mod tests {
    use super::{resolve_x, resolve_y};
    use engine_core::scene::{HorizontalAlign, VerticalAlign};

    #[test]
    fn center_alignment_allows_negative_overscan_offsets() {
        assert_eq!(resolve_x(0, &Some(HorizontalAlign::Center), 640, 720), -40);
        assert_eq!(resolve_y(0, &Some(VerticalAlign::Center), 360, 405), -22);
    }

    #[test]
    fn right_and_bottom_alignment_allow_overscan() {
        assert_eq!(resolve_x(0, &Some(HorizontalAlign::Right), 640, 720), -80);
        assert_eq!(resolve_y(0, &Some(VerticalAlign::Bottom), 360, 405), -45);
    }
}
