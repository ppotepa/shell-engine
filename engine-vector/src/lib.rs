//! 2D vector rasterization helpers for polyline/polygon sprites.

use engine_core::buffer::Buffer;
use engine_core::color::Color;

/// Axis-aligned bounds of a point list in local coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub width: u16,
    pub height: u16,
}

/// Computes local bounds for `points`. Returns `None` when empty.
pub fn bounds(points: &[[i32; 2]]) -> Option<VectorBounds> {
    let mut it = points.iter();
    let first = it.next()?;
    let mut min_x = first[0];
    let mut max_x = first[0];
    let mut min_y = first[1];
    let mut max_y = first[1];
    for p in it {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    Some(VectorBounds {
        min_x,
        min_y,
        width: (max_x - min_x + 1).max(1) as u16,
        height: (max_y - min_y + 1).max(1) as u16,
    })
}

/// Draws a vector polyline into `buffer` using Bresenham line rasterization.
pub fn draw_polyline(
    buffer: &mut Buffer,
    points: &[[i32; 2]],
    closed: bool,
    origin_x: i32,
    origin_y: i32,
    draw_char: char,
    fg: Color,
    bg: Color,
) {
    if points.is_empty() {
        return;
    }
    if points.len() == 1 {
        plot(
            buffer,
            origin_x.saturating_add(points[0][0]),
            origin_y.saturating_add(points[0][1]),
            draw_char,
            fg,
            bg,
        );
        return;
    }

    for idx in 0..(points.len() - 1) {
        draw_line(
            buffer,
            origin_x.saturating_add(points[idx][0]),
            origin_y.saturating_add(points[idx][1]),
            origin_x.saturating_add(points[idx + 1][0]),
            origin_y.saturating_add(points[idx + 1][1]),
            draw_char,
            fg,
            bg,
        );
    }
    if closed {
        let first = points[0];
        let last = points[points.len() - 1];
        draw_line(
            buffer,
            origin_x.saturating_add(last[0]),
            origin_y.saturating_add(last[1]),
            origin_x.saturating_add(first[0]),
            origin_y.saturating_add(first[1]),
            draw_char,
            fg,
            bg,
        );
    }
}

/// Fills the interior of a closed polygon into `buffer`.
/// No-op when fewer than 3 points are supplied.
pub fn fill_polygon(
    buffer: &mut Buffer,
    points: &[[i32; 2]],
    origin_x: i32,
    origin_y: i32,
    fill_char: char,
    fg: Color,
    bg: Color,
) {
    let Some(bounds) = bounds(points) else {
        return;
    };
    if points.len() < 3 {
        return;
    }

    let world_origin = [origin_x, origin_y];
    let x_start = origin_x.saturating_add(bounds.min_x);
    let x_end = x_start.saturating_add(i32::from(bounds.width).saturating_sub(1));
    let y_start = origin_y.saturating_add(bounds.min_y);
    let y_end = y_start.saturating_add(i32::from(bounds.height).saturating_sub(1));

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            if point_in_polygon([x, y], points, world_origin) {
                plot(buffer, x, y, fill_char, fg, bg);
            }
        }
    }
}

fn draw_line(
    buffer: &mut Buffer,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    draw_char: char,
    fg: Color,
    bg: Color,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        plot(buffer, x0, y0, draw_char, fg, bg);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn plot(buffer: &mut Buffer, x: i32, y: i32, draw_char: char, fg: Color, bg: Color) {
    if x < 0 || y < 0 {
        return;
    }
    buffer.set(x as u16, y as u16, draw_char, fg, bg);
}

/// Returns `true` when `point` is inside `polygon` translated by `origin`.
/// Points on the polygon edge are treated as inside.
pub fn point_in_polygon(point: [i32; 2], polygon: &[[i32; 2]], origin: [i32; 2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let p = (point[0] as i64, point[1] as i64);
    let mut inside = false;
    for idx in 0..polygon.len() {
        let a = to_world(polygon[idx], origin);
        let b = to_world(polygon[(idx + 1) % polygon.len()], origin);
        if point_on_segment(p, a, b) {
            return true;
        }
        let crosses_scanline = (a.1 > p.1) != (b.1 > p.1);
        if !crosses_scanline {
            continue;
        }
        let x_intersect = (b.0 - a.0) as f64 * (p.1 - a.1) as f64 / (b.1 - a.1) as f64 + a.0 as f64;
        if (p.0 as f64) < x_intersect {
            inside = !inside;
        }
    }
    inside
}

/// Returns `true` when two polygons intersect after applying their origins.
/// Works for convex and concave simple polygons.
pub fn polygons_intersect(
    polygon_a: &[[i32; 2]],
    origin_a: [i32; 2],
    polygon_b: &[[i32; 2]],
    origin_b: [i32; 2],
) -> bool {
    if polygon_a.is_empty() || polygon_b.is_empty() {
        return false;
    }

    if polygon_a.len() == 1 {
        let p = to_world(polygon_a[0], origin_a);
        return point_in_polygon([p.0 as i32, p.1 as i32], polygon_b, origin_b);
    }
    if polygon_b.len() == 1 {
        let p = to_world(polygon_b[0], origin_b);
        return point_in_polygon([p.0 as i32, p.1 as i32], polygon_a, origin_a);
    }

    for idx_a in 0..polygon_a.len() {
        let a0 = to_world(polygon_a[idx_a], origin_a);
        let a1 = to_world(polygon_a[(idx_a + 1) % polygon_a.len()], origin_a);
        for idx_b in 0..polygon_b.len() {
            let b0 = to_world(polygon_b[idx_b], origin_b);
            let b1 = to_world(polygon_b[(idx_b + 1) % polygon_b.len()], origin_b);
            if segments_intersect(a0, a1, b0, b1) {
                return true;
            }
        }
    }

    let a0 = to_world(polygon_a[0], origin_a);
    if point_in_polygon([a0.0 as i32, a0.1 as i32], polygon_b, origin_b) {
        return true;
    }
    let b0 = to_world(polygon_b[0], origin_b);
    point_in_polygon([b0.0 as i32, b0.1 as i32], polygon_a, origin_a)
}

/// Returns `true` when a segment intersects or enters a polygon translated by `origin`.
pub fn segment_intersects_polygon(
    start: [i32; 2],
    end: [i32; 2],
    polygon: &[[i32; 2]],
    origin: [i32; 2],
) -> bool {
    if polygon.is_empty() {
        return false;
    }
    let start = (start[0] as i64, start[1] as i64);
    let end = (end[0] as i64, end[1] as i64);

    if point_in_polygon([start.0 as i32, start.1 as i32], polygon, origin)
        || point_in_polygon([end.0 as i32, end.1 as i32], polygon, origin)
    {
        return true;
    }

    for idx in 0..polygon.len() {
        let a = to_world(polygon[idx], origin);
        let b = to_world(polygon[(idx + 1) % polygon.len()], origin);
        if segments_intersect(start, end, a, b) {
            return true;
        }
    }
    false
}

#[inline]
fn to_world(point: [i32; 2], origin: [i32; 2]) -> (i64, i64) {
    (
        point[0] as i64 + origin[0] as i64,
        point[1] as i64 + origin[1] as i64,
    )
}

#[inline]
fn orient(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> i128 {
    (b.0 - a.0) as i128 * (c.1 - a.1) as i128 - (b.1 - a.1) as i128 * (c.0 - a.0) as i128
}

#[inline]
fn point_on_segment(p: (i64, i64), a: (i64, i64), b: (i64, i64)) -> bool {
    orient(a, b, p) == 0
        && p.0 >= a.0.min(b.0)
        && p.0 <= a.0.max(b.0)
        && p.1 >= a.1.min(b.1)
        && p.1 <= a.1.max(b.1)
}

fn segments_intersect(a0: (i64, i64), a1: (i64, i64), b0: (i64, i64), b1: (i64, i64)) -> bool {
    let o1 = orient(a0, a1, b0);
    let o2 = orient(a0, a1, b1);
    let o3 = orient(b0, b1, a0);
    let o4 = orient(b0, b1, a1);

    if o1 == 0 && point_on_segment(b0, a0, a1) {
        return true;
    }
    if o2 == 0 && point_on_segment(b1, a0, a1) {
        return true;
    }
    if o3 == 0 && point_on_segment(a0, b0, b1) {
        return true;
    }
    if o4 == 0 && point_on_segment(a1, b0, b1) {
        return true;
    }

    (o1 > 0) != (o2 > 0) && (o3 > 0) != (o4 > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;

    #[test]
    fn bounds_computes_expected_rect() {
        let pts = [[-2, 1], [3, 5], [0, -1]];
        let b = bounds(&pts).expect("bounds");
        assert_eq!(b.min_x, -2);
        assert_eq!(b.min_y, -1);
        assert_eq!(b.width, 6);
        assert_eq!(b.height, 7);
    }

    #[test]
    fn polygons_intersect_detects_overlap() {
        let a = [[-4, -4], [4, -4], [4, 4], [-4, 4]];
        let b = [[-3, -3], [3, -3], [3, 3], [-3, 3]];
        assert!(polygons_intersect(&a, [0, 0], &b, [6, 0]));
        assert!(!polygons_intersect(&a, [0, 0], &b, [20, 0]));
    }

    #[test]
    fn point_in_polygon_treats_edges_as_inside() {
        let poly = [[0, 0], [8, 0], [8, 8], [0, 8]];
        assert!(point_in_polygon([4, 4], &poly, [0, 0]));
        assert!(point_in_polygon([0, 4], &poly, [0, 0]));
        assert!(!point_in_polygon([12, 4], &poly, [0, 0]));
    }

    #[test]
    fn segment_intersects_polygon_detects_crossing() {
        let poly = [[-4, -4], [4, -4], [4, 4], [-4, 4]];
        assert!(segment_intersects_polygon([-10, 0], [10, 0], &poly, [0, 0]));
        assert!(!segment_intersects_polygon(
            [-10, 10],
            [10, 10],
            &poly,
            [0, 0]
        ));
    }

    #[test]
    fn fill_polygon_writes_interior_cells() {
        let mut buffer = Buffer::new(12, 12);
        let poly = [[0, 0], [4, 0], [4, 4], [0, 4]];
        fill_polygon(&mut buffer, &poly, 2, 2, '█', Color::White, Color::Black);
        assert_eq!(buffer.get(4, 4).expect("filled").symbol, '█');
    }

    #[test]
    fn fill_polygon_aligns_with_outline_for_negative_coords() {
        // Polygon centered at origin with negative coords (like asteroid shapes)
        let poly = [[-4, -4], [4, -4], [4, 4], [-4, 4]];
        let origin_x = 20;
        let origin_y = 15;

        let mut fill_buf = Buffer::new(40, 30);
        fill_polygon(
            &mut fill_buf, &poly, origin_x, origin_y, '█',
            Color::White, Color::Black,
        );

        let mut outline_buf = Buffer::new(40, 30);
        draw_polyline(
            &mut outline_buf, &poly, true, origin_x, origin_y, '*',
            Color::White, Color::Black,
        );

        // Center of polygon in world coords should be filled
        assert!(
            fill_buf.get(origin_x as u16, origin_y as u16).is_some(),
            "center of polygon must be filled"
        );
        assert_eq!(
            fill_buf.get(origin_x as u16, origin_y as u16).unwrap().symbol, '█',
            "center must have fill char"
        );

        // Fill must not extend beyond the outline bounding box
        let outline_left = (origin_x - 4) as u16;
        let outline_right = (origin_x + 4) as u16;
        let outline_top = (origin_y - 4) as u16;
        let outline_bottom = (origin_y + 4) as u16;
        for y in 0..30u16 {
            for x in 0..40u16 {
                if let Some(cell) = fill_buf.get(x, y) {
                    if cell.symbol == '█' {
                        assert!(
                            x >= outline_left && x <= outline_right
                                && y >= outline_top && y <= outline_bottom,
                            "fill at ({x},{y}) is outside outline bounds ({outline_left}..{outline_right}, {outline_top}..{outline_bottom})"
                        );
                    }
                }
            }
        }
    }
}
