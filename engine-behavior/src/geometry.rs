//! Pure geometry and trigonometric helpers for asteroids and shapes.

use rhai::Array as RhaiArray;

pub(crate) fn to_i32(value: rhai::INT) -> i32 {
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

pub(crate) fn sin32_i32(idx: i32) -> i32 {
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

pub(crate) fn ship_points_i32(heading: i32) -> Vec<[i32; 2]> {
    let fx = sin32_i32(heading);
    let fy = -sin32_i32(heading + 8);
    let rx = -fy;
    let ry = fx;
    vec![
        [(fx * 7) / 1024, (fy * 7) / 1024],
        [((-fx * 3) - (rx * 3)) / 1024, ((-fy * 3) - (ry * 3)) / 1024],
        [(-fx) / 1024, (-fy) / 1024],
        [((-fx * 3) + (rx * 3)) / 1024, ((-fy * 3) + (ry * 3)) / 1024],
    ]
}

fn asteroid_shape_i32(shape: i32) -> &'static [[i32; 2]] {
    match shape.rem_euclid(4) {
        0 => &[
            [0, -10],
            [8, -6],
            [10, 1],
            [4, 9],
            [-4, 9],
            [-10, 2],
            [-8, -7],
        ],
        1 => &[
            [-2, -10],
            [6, -10],
            [11, -5],
            [10, 1],
            [11, 8],
            [2, 10],
            [-7, 9],
            [-11, 2],
            [-8, -6],
        ],
        2 => &[
            [0, -11],
            [7, -8],
            [10, -1],
            [8, 7],
            [1, 11],
            [-6, 9],
            [-10, 3],
            [-9, -4],
            [-4, -10],
        ],
        _ => &[
            [1, -10],
            [8, -9],
            [11, -2],
            [9, 5],
            [4, 9],
            [-2, 10],
            [-9, 8],
            [-11, 1],
            [-10, -6],
            [-4, -10],
        ],
    }
}

fn asteroid_scale_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 3,
        1 => 5,
        2 => 8,
        _ => 12,
    }
}

pub(crate) fn asteroid_points_i32(shape: i32, size: i32) -> Vec<[i32; 2]> {
    let scale = asteroid_scale_i32(size);
    asteroid_shape_i32(shape)
        .iter()
        .map(|p| [(p[0] * scale) / 10, (p[1] * scale) / 10])
        .collect()
}

pub(crate) fn rotate_points_i32(points: &[[i32; 2]], heading: i32) -> Vec<[i32; 2]> {
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

pub(crate) fn asteroid_fragment_points_i32(shape: i32, size: i32, fragment: i32) -> Vec<[i32; 2]> {
    let points = asteroid_points_i32(shape, size);
    let count = points.len();
    if count < 3 {
        return points;
    }
    let fragment = fragment.rem_euclid(3) as usize;
    let cuts = [0, count / 3, (count * 2) / 3, count];
    let start = cuts[fragment];
    let end = cuts[fragment + 1];
    let mut out = Vec::with_capacity((end - start) + 3);
    out.push([0, 0]);
    for idx in start..=end {
        let wrapped = if idx == count { 0 } else { idx };
        out.push(points[wrapped]);
    }
    out.push([0, 0]);
    out
}

pub(crate) fn asteroid_radius_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 4,
        1 => 7,
        2 => 11,
        _ => 15,
    }
}

pub(crate) fn asteroid_score_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 35,
        1 => 25,
        2 => 15,
        _ => 10,
    }
}

pub(crate) fn points_to_rhai_array(points: Vec<[i32; 2]>) -> RhaiArray {
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

pub(crate) fn rhai_array_to_points(value: &RhaiArray) -> Vec<[i32; 2]> {
    let mut points = Vec::with_capacity(value.len());
    for item in value {
        let Some(pair) = item.clone().try_cast::<RhaiArray>() else {
            continue;
        };
        if pair.len() >= 2 {
            let x = pair.get(0).and_then(|v| v.clone().try_cast::<rhai::INT>());
            let y = pair.get(1).and_then(|v| v.clone().try_cast::<rhai::INT>());
            if let (Some(x), Some(y)) = (x, y) {
                points.push([x as i32, y as i32]);
            }
        }
    }
    points
}
