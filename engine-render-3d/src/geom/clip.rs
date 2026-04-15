#[derive(Clone, Copy)]
pub struct Viewport {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

/// Interpolate depths at clipped line endpoints using parametric projection.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn clipped_depths(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    cx0: i32,
    cy0: i32,
    cx1: i32,
    cy1: i32,
    z0: f32,
    z1: f32,
) -> (f32, f32) {
    let ldx = (x1 - x0) as f32;
    let ldy = (y1 - y0) as f32;
    let len_sq = ldx * ldx + ldy * ldy;
    if len_sq < 1.0 {
        return (z0, z1);
    }
    let t0 = ((cx0 - x0) as f32 * ldx + (cy0 - y0) as f32 * ldy) / len_sq;
    let t1 = ((cx1 - x0) as f32 * ldx + (cy1 - y0) as f32 * ldy) / len_sq;
    (
        z0 + (z1 - z0) * t0.clamp(0.0, 1.0),
        z0 + (z1 - z0) * t1.clamp(0.0, 1.0),
    )
}

pub fn clip_line_to_viewport(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    vp: Viewport,
) -> Option<(i32, i32, i32, i32)> {
    let mut out0 = out_code(x0, y0, vp);
    let mut out1 = out_code(x1, y1, vp);

    loop {
        if (out0 | out1) == 0 {
            return Some((x0, y0, x1, y1));
        }
        if (out0 & out1) != 0 {
            return None;
        }
        let out = if out0 != 0 { out0 } else { out1 };

        let (nx, ny) = if (out & OUT_TOP) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.min_y)?
        } else if (out & OUT_BOTTOM) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.max_y)?
        } else if (out & OUT_RIGHT) != 0 {
            intersect_vertical(x0, y0, x1, y1, vp.max_x)?
        } else {
            intersect_vertical(x0, y0, x1, y1, vp.min_x)?
        };

        if out == out0 {
            x0 = nx;
            y0 = ny;
            out0 = out_code(x0, y0, vp);
        } else {
            x1 = nx;
            y1 = ny;
            out1 = out_code(x1, y1, vp);
        }
    }
}

const OUT_LEFT: u8 = 1;
const OUT_RIGHT: u8 = 2;
const OUT_BOTTOM: u8 = 4;
const OUT_TOP: u8 = 8;

#[inline]
fn out_code(x: i32, y: i32, vp: Viewport) -> u8 {
    let mut code = 0u8;
    if x < vp.min_x {
        code |= OUT_LEFT;
    } else if x > vp.max_x {
        code |= OUT_RIGHT;
    }
    if y > vp.max_y {
        code |= OUT_BOTTOM;
    } else if y < vp.min_y {
        code |= OUT_TOP;
    }
    code
}

#[inline]
fn intersect_vertical(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> Option<(i32, i32)> {
    let dx = x1 - x0;
    if dx == 0 {
        return None;
    }
    let t = (x - x0) as f32 / dx as f32;
    let y = y0 as f32 + t * (y1 - y0) as f32;
    Some((x, y.round() as i32))
}

#[inline]
fn intersect_horizontal(x0: i32, y0: i32, x1: i32, y1: i32, y: i32) -> Option<(i32, i32)> {
    let dy = y1 - y0;
    if dy == 0 {
        return None;
    }
    let t = (y - y0) as f32 / dy as f32;
    let x = x0 as f32 + t * (x1 - x0) as f32;
    Some((x.round() as i32, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_line_inside_viewport_keeps_endpoints() {
        let vp = Viewport {
            min_x: 0,
            min_y: 0,
            max_x: 9,
            max_y: 9,
        };
        let clipped = clip_line_to_viewport(2, 2, 8, 8, vp);
        assert_eq!(clipped, Some((2, 2, 8, 8)));
    }

    #[test]
    fn clipped_depths_interpolate_endpoints() {
        let (z0, z1) = clipped_depths(0, 0, 10, 0, 2, 0, 8, 0, 1.0, 3.0);
        assert!((z0 - 1.4).abs() < 1e-4);
        assert!((z1 - 2.6).abs() < 1e-4);
    }
}
