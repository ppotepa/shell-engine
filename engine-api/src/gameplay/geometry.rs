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
    use super::{dent_polygon_i32, jitter_points_i32, regular_polygon_i32};

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
}
