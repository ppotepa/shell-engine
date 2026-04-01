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
    use super::{jitter_points_i32, regular_polygon_i32};

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
}
