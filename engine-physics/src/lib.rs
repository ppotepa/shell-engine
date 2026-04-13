//! 2D gameplay geometry queries.
//!
//! This crate is the engine boundary for collision-style math used by gameplay
//! systems and scripts. Rendering-specific helpers stay in `engine-vector`.

use geo::algorithm::contains::Contains;
use geo::algorithm::coords_iter::CoordsIter;
use geo::algorithm::intersects::Intersects;
use geo::BooleanOps;
use geo::{Coord, Line, LineString, MultiPolygon, Point, Polygon};

/// Returns `true` when `point` is inside or on the boundary of `polygon`
/// translated by `origin`.
pub fn point_in_polygon(point: [i32; 2], polygon: &[[i32; 2]], origin: [i32; 2]) -> bool {
    let Some(poly) = polygon_from_points(polygon, origin) else {
        return false;
    };
    let point = Point::new(point[0] as f64, point[1] as f64);
    poly.contains(&point) || poly.intersects(&point)
}

/// Returns `true` when two polygons intersect after applying their origins.
pub fn polygons_intersect(
    polygon_a: &[[i32; 2]],
    origin_a: [i32; 2],
    polygon_b: &[[i32; 2]],
    origin_b: [i32; 2],
) -> bool {
    let Some(poly_a) = polygon_from_points(polygon_a, origin_a) else {
        return false;
    };
    let Some(poly_b) = polygon_from_points(polygon_b, origin_b) else {
        return false;
    };
    poly_a.intersects(&poly_b)
}

/// Returns `true` when a segment intersects or enters a polygon translated by
/// `origin`.
pub fn segment_intersects_polygon(
    start: [i32; 2],
    end: [i32; 2],
    polygon: &[[i32; 2]],
    origin: [i32; 2],
) -> bool {
    let Some(poly) = polygon_from_points(polygon, origin) else {
        return false;
    };
    let segment = Line::new(
        Coord {
            x: start[0] as f64,
            y: start[1] as f64,
        },
        Coord {
            x: end[0] as f64,
            y: end[1] as f64,
        },
    );
    poly.intersects(&segment)
        || point_in_polygon(start, polygon, origin)
        || point_in_polygon(end, polygon, origin)
}

/// Subtract polygon `b` from polygon `a` and return the resulting polygons.
///
/// Uses boolean difference (CSG subtraction). Returns one polygon when the cut
/// creates a notch, multiple polygons when it splits the shape, or an empty vec
/// when `b` completely covers `a`.
pub fn subtract_polygons(polygon_a: &[[i32; 2]], polygon_b: &[[i32; 2]]) -> Vec<Vec<[i32; 2]>> {
    let Some(poly_a) = polygon_from_points(polygon_a, [0, 0]) else {
        return vec![];
    };
    let Some(poly_b) = polygon_from_points(polygon_b, [0, 0]) else {
        return vec![polygon_a.to_vec()];
    };
    let result: MultiPolygon<f64> = poly_a.difference(&poly_b);
    result
        .into_iter()
        .filter_map(|poly| {
            let coords: Vec<[i32; 2]> = poly
                .exterior()
                .coords()
                .take(poly.exterior().coords_count().saturating_sub(1))
                .map(|c| [c.x.round() as i32, c.y.round() as i32])
                .collect();
            if coords.len() >= 3 {
                Some(coords)
            } else {
                None
            }
        })
        .collect()
}

/// Compute the signed area of a polygon (Shoelace formula). Returns the
/// absolute area in square units (i32 precision).
pub fn polygon_area(polygon: &[[i32; 2]]) -> i64 {
    let n = polygon.len();
    if n < 3 {
        return 0;
    }
    let mut sum: i64 = 0;
    for i in 0..n {
        let j = (i + 1) % n;
        let xi = polygon[i][0] as i64;
        let yi = polygon[i][1] as i64;
        let xj = polygon[j][0] as i64;
        let yj = polygon[j][1] as i64;
        sum += xi * yj - xj * yi;
    }
    sum.abs() / 2
}

fn polygon_from_points(points: &[[i32; 2]], origin: [i32; 2]) -> Option<Polygon<f64>> {
    if points.len() < 3 {
        return None;
    }
    let mut coords = Vec::with_capacity(points.len() + 1);
    for point in points {
        coords.push(Coord {
            x: (point[0] + origin[0]) as f64,
            y: (point[1] + origin[1]) as f64,
        });
    }
    coords.push(coords[0]);
    Some(Polygon::new(LineString::new(coords), vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygons_intersect_detects_overlap() {
        let a = [[-4, -4], [4, -4], [4, 4], [-4, 4]];
        let b = [[-3, -3], [3, -3], [3, 3], [-3, 3]];
        assert!(polygons_intersect(&a, [0, 0], &b, [6, 0]));
        assert!(!polygons_intersect(&a, [0, 0], &b, [20, 0]));
    }

    #[test]
    fn point_in_polygon_treats_edges_as_hits() {
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
    fn subtract_polygons_creates_notch() {
        // Large square with a small square bite taken from its right edge
        let big = [[-10, -10], [10, -10], [10, 10], [-10, 10]];
        let bite = [[5, -3], [15, -3], [15, 3], [5, 3]];
        let result = subtract_polygons(&big, &bite);
        assert!(
            !result.is_empty(),
            "subtraction should produce at least one polygon"
        );
        // The result should have more vertices than the original (notch adds corners)
        assert!(result[0].len() > 4);
    }

    #[test]
    fn subtract_polygons_splits_shape() {
        // Thin horizontal bar split by a vertical cut through the middle
        let bar = [[-20, -3], [20, -3], [20, 3], [-20, 3]];
        let cut = [[-2, -10], [2, -10], [2, 10], [-2, 10]];
        let result = subtract_polygons(&bar, &cut);
        assert!(
            result.len() >= 2,
            "vertical cut through center should produce 2+ pieces, got {}",
            result.len()
        );
    }

    #[test]
    fn subtract_polygons_no_overlap_returns_original() {
        let poly = [[-5, -5], [5, -5], [5, 5], [-5, 5]];
        let far = [[50, 50], [60, 50], [60, 60], [50, 60]];
        let result = subtract_polygons(&poly, &far);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 4);
    }

    #[test]
    fn polygon_area_square() {
        // 10×10 square: area = 100
        let sq = [[0, 0], [10, 0], [10, 10], [0, 10]];
        assert_eq!(polygon_area(&sq), 100);
    }

    #[test]
    fn polygon_area_triangle() {
        // Right triangle with legs 6 and 4: area = 12
        let tri = [[0, 0], [6, 0], [0, 4]];
        assert_eq!(polygon_area(&tri), 12);
    }
}
