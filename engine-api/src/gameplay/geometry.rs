//! Pure geometry and trigonometric helpers for generic script math.

use rhai::Array as RhaiArray;

pub fn to_i32(value: rhai::INT) -> i32 {
    value.clamp(i32::MIN as rhai::INT, i32::MAX as rhai::INT) as i32
}

fn base_sin_i32(step: i32) -> i32 {
    match step {
        0 => 0,
        1 => 200,
        2 => 392,
        3 => 569,
        4 => 724,
        5 => 851,
        6 => 946,
        7 => 1004,
        _ => 1024,
    }
}

pub fn sin32_i32(idx: i32) -> i32 {
    let i = idx.rem_euclid(32);
    let q = i / 8;
    let o = i % 8;
    match q {
        0 => base_sin_i32(o),
        1 => base_sin_i32(8 - o),
        2 => -base_sin_i32(o),
        _ => -base_sin_i32(8 - o),
    }
}

pub fn rotate_points_i32(points: &[[i32; 2]], heading: i32) -> Vec<[i32; 2]> {
    let sin = i64::from(sin32_i32(heading));
    let cos = i64::from(sin32_i32(heading + 8));
    points
        .iter()
        .map(|p| {
            let x = i64::from(p[0]);
            let y = i64::from(p[1]);
            [
                ((x * cos) - (y * sin)).div_euclid(1024) as i32,
                ((x * sin) + (y * cos)).div_euclid(1024) as i32,
            ]
        })
        .collect()
}

pub fn regular_polygon_i32(sides: i32, radius: i32) -> Vec<[i32; 2]> {
    let sides = sides.clamp(3, 32);
    let radius = radius.max(1);
    (0..sides)
        .map(|idx| {
            let heading = (idx * 32).div_euclid(sides);
            let x = (i64::from(radius) * i64::from(sin32_i32(heading + 8))).div_euclid(1024);
            let y = (i64::from(radius) * i64::from(sin32_i32(heading))).div_euclid(1024);
            [x as i32, y as i32]
        })
        .collect()
}

pub fn jitter_points_i32(points: &[[i32; 2]], amount: i32, seed: i32) -> Vec<[i32; 2]> {
    let amount = amount.max(0);
    if amount == 0 {
        return points.to_vec();
    }

    points
        .iter()
        .enumerate()
        .map(|(idx, [x, y])| {
            let mix = seed
                .wrapping_mul(31)
                .wrapping_add(idx as i32 * 17)
                .wrapping_add(x.wrapping_mul(13))
                .wrapping_add(y.wrapping_mul(7));
            let dx = mix.rem_euclid(amount * 2 + 1) - amount;
            let dy = mix
                .wrapping_mul(19)
                .wrapping_add(23)
                .rem_euclid(amount * 2 + 1)
                - amount;
            [x + dx, y + dy]
        })
        .collect()
}

/// Push the closest vertex to `(impact_x, impact_y)` toward the polygon
/// centroid by `strength` percent (0–100). Returns the deformed polygon.
pub fn dent_polygon_i32(
    points: &[[i32; 2]],
    impact_x: i32,
    impact_y: i32,
    strength: i32,
) -> Vec<[i32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let strength = strength.clamp(0, 100);

    // Centroid
    let cx: i64 = points.iter().map(|p| p[0] as i64).sum::<i64>() / points.len() as i64;
    let cy: i64 = points.iter().map(|p| p[1] as i64).sum::<i64>() / points.len() as i64;

    // Find closest vertex to impact
    let ix = impact_x as i64;
    let iy = impact_y as i64;
    let mut best_idx = 0usize;
    let mut best_dist = i64::MAX;
    for (i, p) in points.iter().enumerate() {
        let dx = p[0] as i64 - ix;
        let dy = p[1] as i64 - iy;
        let d = dx * dx + dy * dy;
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }

    // Push closest vertex (and its neighbours for smoother denting) toward centroid
    let mut result = points.to_vec();
    let n = result.len();
    let dent = |px: i32, py: i32, frac: i64| -> [i32; 2] {
        let nx = px as i64 + (cx - px as i64) * frac / 100;
        let ny = py as i64 + (cy - py as i64) * frac / 100;
        [nx as i32, ny as i32]
    };

    let s = strength as i64;
    result[best_idx] = dent(result[best_idx][0], result[best_idx][1], s);
    // Half-strength on immediate neighbours for smooth falloff
    let prev = (best_idx + n - 1) % n;
    let next = (best_idx + 1) % n;
    result[prev] = dent(result[prev][0], result[prev][1], s / 2);
    result[next] = dent(result[next][0], result[next][1], s / 2);

    result
}

/// Scale all points by `num/denom` (integer arithmetic, no float).
/// Used to resize a parent asteroid's shape for child fragments.
/// `denom` must not be zero.
pub fn scale_points_frac_i32(points: &[[i32; 2]], num: i32, denom: i32) -> Vec<[i32; 2]> {
    if denom == 0 {
        return points.to_vec();
    }
    let n = num as i64;
    let d = denom as i64;
    points
        .iter()
        .map(|p| {
            [
                (p[0] as i64 * n / d) as i32,
                (p[1] as i64 * n / d) as i32,
            ]
        })
        .collect()
}

/// Inserts a V-notch crack into the polygon at the vertex closest to the impact
/// point.  Replaces that one vertex with three vertices:
///   left_shoulder  → crack_tip (deepest) → right_shoulder
/// giving a visible crack line that moves and rotates with the polygon.
/// `depth` is 0–100: how far the crack tip is pushed toward the centroid.
/// Returns a polygon with vertex count = original + 2.
pub fn crack_polygon_i32(
    points: &[[i32; 2]],
    impact_x: i32,
    impact_y: i32,
    depth: i32,
) -> Vec<[i32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let depth = depth.clamp(0, 100) as i64;

    // Centroid
    let cx: i64 = points.iter().map(|p| p[0] as i64).sum::<i64>() / points.len() as i64;
    let cy: i64 = points.iter().map(|p| p[1] as i64).sum::<i64>() / points.len() as i64;

    // Closest vertex to impact
    let ix = impact_x as i64;
    let iy = impact_y as i64;
    let mut best_idx = 0usize;
    let mut best_dist = i64::MAX;
    for (i, p) in points.iter().enumerate() {
        let dx = p[0] as i64 - ix;
        let dy = p[1] as i64 - iy;
        let d = dx * dx + dy * dy;
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }

    let n = points.len();
    let prev_idx = (best_idx + n - 1) % n;
    let next_idx = (best_idx + 1) % n;

    // Lerp between two points at t/100
    let lerp = |a: [i32; 2], b: [i32; 2], t: i64| -> [i32; 2] {
        [
            (a[0] as i64 + (b[0] as i64 - a[0] as i64) * t / 100) as i32,
            (a[1] as i64 + (b[1] as i64 - a[1] as i64) * t / 100) as i32,
        ]
    };
    // Push a point toward centroid by frac/100
    let push = |p: [i32; 2], frac: i64| -> [i32; 2] {
        [
            (p[0] as i64 + (cx - p[0] as i64) * frac / 100) as i32,
            (p[1] as i64 + (cy - p[1] as i64) * frac / 100) as i32,
        ]
    };

    let impact_v = points[best_idx];
    let prev_v = points[prev_idx];
    let next_v = points[next_idx];

    // Shoulders sit 70% of the way from the neighbour to the impact vertex and
    // are pushed 35% as deep as the crack tip.
    let shoulder_depth = depth * 35 / 100;
    let left_shoulder = push(lerp(prev_v, impact_v, 70), shoulder_depth);
    let crack_tip = push(impact_v, depth);
    let right_shoulder = push(lerp(impact_v, next_v, 30), shoulder_depth);

    // Replace impact vertex with [left_shoulder, crack_tip, right_shoulder]
    let mut result = Vec::with_capacity(n + 2);
    for (i, &p) in points.iter().enumerate() {
        if i == best_idx {
            result.push(left_shoulder);
            result.push(crack_tip);
            result.push(right_shoulder);
        } else {
            result.push(p);
        }
    }
    result
}

/// Clips a convex or concave polygon to the half-plane where
/// `nx * x + ny * y >= 0` (Sutherland-Hodgman, single half-plane).
fn clip_polygon_halfplane(pts: &[[i32; 2]], nx: f64, ny: f64) -> Vec<[i32; 2]> {
    let n = pts.len();
    if n == 0 {
        return vec![];
    }
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let curr = pts[i];
        let next = pts[(i + 1) % n];
        let dc = nx * curr[0] as f64 + ny * curr[1] as f64;
        let dn = nx * next[0] as f64 + ny * next[1] as f64;
        if dc >= 0.0 {
            out.push(curr);
        }
        if (dc >= 0.0) != (dn >= 0.0) {
            let t = dc / (dc - dn);
            let ix = curr[0] as f64 + t * (next[0] - curr[0]) as f64;
            let iy = curr[1] as f64 + t * (next[1] - curr[1]) as f64;
            out.push([ix.round() as i32, iy.round() as i32]);
        }
    }
    out
}

/// Translates a polygon so its centroid sits at the origin.
pub fn center_points_i32(points: &[[i32; 2]]) -> Vec<[i32; 2]> {
    let n = points.len();
    if n == 0 {
        return vec![];
    }
    let cx = points.iter().map(|p| p[0] as i64).sum::<i64>() / n as i64;
    let cy = points.iter().map(|p| p[1] as i64).sum::<i64>() / n as i64;
    points
        .iter()
        .map(|p| [p[0] - cx as i32, p[1] - cy as i32])
        .collect()
}

/// Scales a polygon so its maximum vertex distance from the origin equals
/// `target_radius`.  The polygon must already be centred at the origin.
pub fn normalize_polygon_radius_i32(
    points: &[[i32; 2]],
    target_radius: i32,
) -> Vec<[i32; 2]> {
    if points.is_empty() {
        return vec![];
    }
    let max_dist_sq = points
        .iter()
        .map(|p| (p[0] as i64).pow(2) + (p[1] as i64).pow(2))
        .max()
        .unwrap_or(0);
    if max_dist_sq == 0 {
        return points.to_vec();
    }
    let max_dist = (max_dist_sq as f64).sqrt();
    let scale = target_radius as f64 / max_dist;
    points
        .iter()
        .map(|p| {
            [(p[0] as f64 * scale).round() as i32, (p[1] as f64 * scale).round() as i32]
        })
        .collect()
}

/// Returns one half of a polygon split along a line through its centroid.
///
/// `heading` is in the same 32-step space as `rotate_points`.
/// `side` 0 = the half on the positive side of the split normal,
///        1 = the other half.
///
/// The returned half is:
///   1. re-centred at its own centroid (so collider/visual align)
///   2. normalised so its max vertex distance equals `target_radius`
///
/// Falls back to the full polygon if clipping produces a degenerate result.
pub fn split_polygon_half_i32(
    points: &[[i32; 2]],
    heading: i32,
    side: i32,
    target_radius: i32,
) -> Vec<[i32; 2]> {
    let n = points.len();
    if n < 3 {
        return normalize_polygon_radius_i32(&center_points_i32(points), target_radius);
    }

    // Translate to centroid-relative space
    let cx = points.iter().map(|p| p[0] as i64).sum::<i64>() / n as i64;
    let cy = points.iter().map(|p| p[1] as i64).sum::<i64>() / n as i64;
    let rel: Vec<[i32; 2]> = points
        .iter()
        .map(|p| [p[0] - cx as i32, p[1] - cy as i32])
        .collect();

    // Normal perpendicular to heading
    let sin_h = sin32_i32(heading) as f64 / 1024.0;
    let cos_h = sin32_i32(heading + 8) as f64 / 1024.0;
    let (nx, ny) = if side == 0 {
        (-sin_h, cos_h)
    } else {
        (sin_h, -cos_h)
    };

    let clipped = clip_polygon_halfplane(&rel, nx, ny);
    let centred = if clipped.len() >= 3 {
        center_points_i32(&clipped)
    } else {
        center_points_i32(&rel)
    };
    normalize_polygon_radius_i32(&centred, target_radius)
}

pub fn split_polygon_i32(
    points: &[[i32; 2]],
    heading: i32,
) -> (Vec<[i32; 2]>, Vec<[i32; 2]>) {
    let n = points.len();
    if n < 3 {
        return (
            center_points_i32(points),
            center_points_i32(points),
        );
    }

    // Translate to centroid-relative space first
    let cx = points.iter().map(|p| p[0] as i64).sum::<i64>() / n as i64;
    let cy = points.iter().map(|p| p[1] as i64).sum::<i64>() / n as i64;
    let rel: Vec<[i32; 2]> = points
        .iter()
        .map(|p| [p[0] - cx as i32, p[1] - cy as i32])
        .collect();

    // Split-line normal: perpendicular to the heading direction.
    // heading 0 → right (cos=1, sin=0) → normal pointing up (-sin, cos) = (0, 1)
    let sin_h = sin32_i32(heading) as f64 / 1024.0;
    let cos_h = sin32_i32(heading + 8) as f64 / 1024.0;
    let nx = -sin_h;
    let ny = cos_h;

    let raw_a = clip_polygon_halfplane(&rel, nx, ny);
    let raw_b = clip_polygon_halfplane(&rel, -nx, -ny);

    // Re-centre each half around its own centroid so collider + visual align.
    let half_a = if raw_a.len() >= 3 {
        center_points_i32(&raw_a)
    } else {
        rel.clone()
    };
    let half_b = if raw_b.len() >= 3 {
        center_points_i32(&raw_b)
    } else {
        rel
    };
    (half_a, half_b)
}

pub fn points_to_rhai_array(points: Vec<[i32; 2]>) -> RhaiArray {
    points
        .into_iter()
        .map(|[x, y]| {
            let mut pair = RhaiArray::with_capacity(2);
            pair.push((x as rhai::INT).into());
            pair.push((y as rhai::INT).into());
            pair.into()
        })
        .collect()
}

pub fn rhai_array_to_points(value: &RhaiArray) -> Vec<[i32; 2]> {
    let mut points = Vec::with_capacity(value.len());
    for item in value {
        let Some(pair) = item.clone().try_cast::<RhaiArray>() else {
            continue;
        };
        if pair.len() >= 2 {
            let x = pair.first().and_then(|v| v.clone().try_cast::<rhai::INT>());
            let y = pair.get(1).and_then(|v| v.clone().try_cast::<rhai::INT>());
            if let (Some(x), Some(y)) = (x, y) {
                points.push([x as i32, y as i32]);
            }
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::{crack_polygon_i32, dent_polygon_i32, jitter_points_i32, regular_polygon_i32};

    #[test]
    fn regular_polygon_generates_requested_vertex_count() {
        let points = regular_polygon_i32(7, 12);
        assert_eq!(points.len(), 7);
    }

    #[test]
    fn jitter_points_is_deterministic_for_same_seed() {
        let base = vec![[10, 0], [0, 10], [-10, 0], [0, -10]];
        let a = jitter_points_i32(&base, 3, 42);
        let b = jitter_points_i32(&base, 3, 42);
        let c = jitter_points_i32(&base, 3, 43);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn dent_polygon_pushes_vertex_toward_centroid() {
        let square = vec![[10, 0], [0, 10], [-10, 0], [0, -10]];
        let dented = dent_polygon_i32(&square, 10, 0, 50);
        // The vertex at [10,0] should have moved toward centroid [0,0]
        assert!(dented[0][0] < 10, "x should decrease toward centroid");
        // Neighbours get half-strength dent
        assert!(dented[1] != square[1] || dented[3] != square[3]);
    }

    #[test]
    fn dent_polygon_strength_zero_is_identity() {
        let tri = vec![[0, 10], [-8, -5], [8, -5]];
        let dented = dent_polygon_i32(&tri, 0, 10, 0);
        assert_eq!(dented, tri);
    }

    #[test]
    fn dent_polygon_strength_100_collapses_to_centroid() {
        let square = vec![[10, 10], [-10, 10], [-10, -10], [10, -10]];
        let dented = dent_polygon_i32(&square, 10, 10, 100);
        // Centroid is (0,0); vertex at [10,10] should now be [0,0]
        assert_eq!(dented[0], [0, 0]);
    }

    #[test]
    fn crack_polygon_inserts_two_extra_vertices() {
        let square = vec![[10, 0], [0, 10], [-10, 0], [0, -10]];
        let cracked = crack_polygon_i32(&square, 10, 0, 60);
        assert_eq!(cracked.len(), 6, "one vertex replaced by three → +2 vertices");
    }

    #[test]
    fn crack_polygon_tip_is_deeper_than_original_vertex() {
        let square = vec![[10, 0], [0, 10], [-10, 0], [0, -10]];
        let cracked = crack_polygon_i32(&square, 10, 0, 60);
        // Crack tip is index 1 (left_shoulder=0, tip=1, right_shoulder=2)
        let tip = cracked[1];
        let dist_tip = tip[0] * tip[0] + tip[1] * tip[1];
        let dist_orig = 10 * 10; // original vertex was [10,0], centroid is [0,0]
        assert!(dist_tip < dist_orig, "crack tip must be pushed toward centroid");
    }
}
