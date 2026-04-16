use engine_core::buffer::Buffer;
use engine_core::color::Color;

use crate::ClipRect;

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn render_panel_box(
    buffer: &mut Buffer,
    draw_x: i32,
    draw_y: i32,
    width: u16,
    height: u16,
    border_width: u16,
    corner_radius: u16,
    panel_bg: Color,
    border_color: Color,
    shadow_color: Color,
    shadow_x: i32,
    shadow_y: i32,
) {
    if width == 0 || height == 0 {
        return;
    }
    let rounded = corner_radius > 0 && width >= 4 && height >= 4;
    for py in 0..height {
        for px in 0..width {
            if !panel_cell_visible(px, py, width, height, rounded) {
                continue;
            }
            set_panel_cell(
                buffer,
                draw_x.saturating_add(px as i32).saturating_add(shadow_x),
                draw_y.saturating_add(py as i32).saturating_add(shadow_y),
                shadow_color,
            );
        }
    }

    for py in 0..height {
        for px in 0..width {
            if !panel_cell_visible(px, py, width, height, rounded) {
                continue;
            }
            let bw = border_width.min(width / 2).min(height / 2);
            let border = bw > 0
                && (px < bw
                    || py < bw
                    || px >= width.saturating_sub(bw)
                    || py >= height.saturating_sub(bw));
            let color = if border { border_color } else { panel_bg };
            set_panel_cell(
                buffer,
                draw_x.saturating_add(px as i32),
                draw_y.saturating_add(py as i32),
                color,
            );
        }
    }
}

#[inline]
pub fn intersect_clip_rect(a: Option<ClipRect>, b: Option<ClipRect>) -> Option<ClipRect> {
    match (a, b) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => {
            let left = a.x.max(b.x);
            let top = a.y.max(b.y);
            let right = (a.x + i32::from(a.width)).min(b.x + i32::from(b.width));
            let bottom = (a.y + i32::from(a.height)).min(b.y + i32::from(b.height));
            if right <= left || bottom <= top {
                return None;
            }
            Some(ClipRect {
                x: left,
                y: top,
                width: (right - left) as u16,
                height: (bottom - top) as u16,
            })
        }
    }
}

#[inline(always)]
#[allow(clippy::nonminimal_bool)]
fn panel_cell_visible(x: u16, y: u16, width: u16, height: u16, rounded: bool) -> bool {
    !rounded
        || !((x == 0 || x == width.saturating_sub(1)) && (y == 0 || y == height.saturating_sub(1)))
}

#[inline(always)]
fn set_panel_cell(buffer: &mut Buffer, x: i32, y: i32, bg: Color) {
    if x < 0 || y < 0 {
        return;
    }
    if matches!(bg, Color::Reset) {
        return;
    }
    buffer.set(x as u16, y as u16, ' ', Color::Reset, bg);
}
