/// Deterministic hash for 3-D integer lattice coordinates, mapped to [0,1).
#[inline(always)]
pub fn terrain_hash(xi: i32, yi: i32, zi: i32) -> f32 {
    let mut h = xi
        .wrapping_mul(1619)
        .wrapping_add(yi.wrapping_mul(31337))
        .wrapping_add(zi.wrapping_mul(6271));
    h ^= h >> 13;
    h = h.wrapping_mul(1_000_000_007);
    h ^= h >> 17;
    (h as u32 as f32) * (1.0 / u32::MAX as f32)
}

/// Smooth 3-D value noise (trilinear, smoothstep filtered). Output range: [0,1].
pub fn value_noise_3d(px: f32, py: f32, pz: f32) -> f32 {
    let xi = px.floor() as i32;
    let xf = px - xi as f32;
    let yi = py.floor() as i32;
    let yf = py - yi as f32;
    let zi = pz.floor() as i32;
    let zf = pz - zi as f32;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);
    let c000 = terrain_hash(xi, yi, zi);
    let c100 = terrain_hash(xi + 1, yi, zi);
    let c010 = terrain_hash(xi, yi + 1, zi);
    let c110 = terrain_hash(xi + 1, yi + 1, zi);
    let c001 = terrain_hash(xi, yi, zi + 1);
    let c101 = terrain_hash(xi + 1, yi, zi + 1);
    let c011 = terrain_hash(xi, yi + 1, zi + 1);
    let c111 = terrain_hash(xi + 1, yi + 1, zi + 1);
    let x0 = c000 + u * (c100 - c000);
    let x1 = c010 + u * (c110 - c010);
    let x2 = c001 + u * (c101 - c001);
    let x3 = c011 + u * (c111 - c011);
    let y0 = x0 + v * (x1 - x0);
    let y1 = x2 + v * (x3 - x2);
    y0 + w * (y1 - y0)
}

/// Variable-octave fBm with configurable lacunarity and persistence.
/// Output range ~= [0,1).
pub fn fbm_3d_full(x: f32, y: f32, z: f32, octaves: u8, lacunarity: f32, persistence: f32) -> f32 {
    let n = octaves.clamp(1, 8) as usize;
    let mut val = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..n {
        val += value_noise_3d(x * freq, y * freq, z * freq) * amp;
        max_val += amp;
        amp *= persistence;
        freq *= lacunarity;
    }
    val / max_val.max(1e-6)
}

/// Fixed-parameter fBm wrapper: lacunarity=2.0, persistence=0.5.
#[inline]
pub fn fbm_3d_octaves(x: f32, y: f32, z: f32, octaves: u8) -> f32 {
    fbm_3d_full(x, y, z, octaves, 2.0, 0.5)
}
