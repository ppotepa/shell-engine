#[inline(always)]
pub fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline(always)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline(always)]
pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline(always)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[inline(always)]
pub fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
    let (sp, cp) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();
    let (sr, cr) = roll.sin_cos();

    let x1 = v[0];
    let y1 = v[1] * cp - v[2] * sp;
    let z1 = v[1] * sp + v[2] * cp;

    let x2 = x1 * cy + z1 * sy;
    let y2 = y1;
    let z2 = -x1 * sy + z1 * cy;

    let x3 = x2 * cr - y2 * sr;
    let y3 = x2 * sr + y2 * cr;
    [x3, y3, z2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_zero_fallback() {
        assert_eq!(normalize3([0.0, 0.0, 0.0]), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn rotate_identity_when_zero_angles() {
        let v = [1.2, -3.4, 5.6];
        assert_eq!(rotate_xyz(v, 0.0, 0.0, 0.0), v);
    }
}
