//! 2D gameplay geometry queries.
//!
//! This crate is the engine boundary for collision-style math used by gameplay
//! systems and scripts. Rendering-specific helpers stay in `engine-vector`.

use geo::algorithm::contains::Contains;
use geo::algorithm::intersects::Intersects;
use geo::{Coord, Line, LineString, Point, Polygon};

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
}
