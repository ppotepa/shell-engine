//! Pure-Rust 3D value noise, fBm, ridged fBm.
//!
//! No external noise crate — single-file implementation using a fast hash.
//! All functions are deterministic and seed-driven.

/// Avalanche hash of three i32 coordinates + u64 seed → f64 in [0.0, 1.0).
#[inline(always)]
pub fn hash3(xi: i32, yi: i32, zi: i32, seed: u64) -> f64 {
    let mut h = seed
        .wrapping_add((xi as u64).wrapping_mul(2654435761))
        .wrapping_add((yi as u64).wrapping_mul(2246822519))
        .wrapping_add((zi as u64).wrapping_mul(3266489917));
    h ^= h >> 17;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 31;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 32;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Trilinear value noise sampled at (x, y, z) with the given seed.
/// Output is in [0.0, 1.0).
#[inline]
pub fn value_noise(x: f64, y: f64, z: f64, seed: u64) -> f64 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f64;
    let yf = y - yi as f64;
    let zf = z - zi as f64;
    // Smooth-step
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);

    let c000 = hash3(xi,     yi,     zi,     seed);
    let c100 = hash3(xi + 1, yi,     zi,     seed);
    let c010 = hash3(xi,     yi + 1, zi,     seed);
    let c110 = hash3(xi + 1, yi + 1, zi,     seed);
    let c001 = hash3(xi,     yi,     zi + 1, seed);
    let c101 = hash3(xi + 1, yi,     zi + 1, seed);
    let c011 = hash3(xi,     yi + 1, zi + 1, seed);
    let c111 = hash3(xi + 1, yi + 1, zi + 1, seed);

    let x0 = c000 + u * (c100 - c000);
    let x1 = c010 + u * (c110 - c010);
    let x2 = c001 + u * (c101 - c001);
    let x3 = c011 + u * (c111 - c011);
    let y0 = x0 + v * (x1 - x0);
    let y1 = x2 + v * (x3 - x2);
    y0 + w * (y1 - y0)
}

/// Fractional Brownian Motion — sums `octaves` noise octaves.
/// Lacunarity = 2.0, persistence = 0.5. Output in [0.0, 1.0).
pub fn fbm(x: f64, y: f64, z: f64, octaves: u8, seed: u64) -> f64 {
    let n = octaves.clamp(1, 8) as usize;
    let mut val = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut max_val = 0.0;
    for _ in 0..n {
        val += value_noise(x * freq, y * freq, z * freq, seed) * amp;
        max_val += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    val / max_val
}

/// Ridged fBm — inverted absolute value gives sharp ridges.
/// Output in [0.0, 1.0): 1.0 at ridge peaks, 0.0 at valleys.
pub fn ridged_fbm(x: f64, y: f64, z: f64, octaves: u8, seed: u64) -> f64 {
    let n = octaves.clamp(1, 8) as usize;
    let mut val = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut max_val = 0.0;
    for _ in 0..n {
        let n_raw = value_noise(x * freq, y * freq, z * freq, seed);
        // Map [0,1] → ridge shape: peak at 0.5, valleys at 0 and 1
        let ridged = 1.0 - (n_raw * 2.0 - 1.0).abs();
        val += ridged * amp;
        max_val += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    val / max_val
}

/// Two-level domain-warped fBm for organic continent shapes.
///
/// Works in 3D sphere space: input (cx, cy, cz) should be on the unit sphere.
/// `scale` controls continent size (smaller = bigger continents).
/// `warp` controls coastline chaos (0 = smooth, 1.5 = very chaotic).
pub fn continent_noise(cx: f64, cy: f64, cz: f64, scale: f64, warp: f64, octaves: u8, seed: u64) -> f64 {
    let s = scale;
    // First warp layer
    let wx1 = (fbm(cx * s,         cy * s,         cz * s,         octaves, seed + 100) - 0.5) * warp;
    let wy1 = (fbm(cx * s + 3.2,   cy * s + 1.8,   cz * s + 0.7,   octaves, seed + 200) - 0.5) * warp;
    let wz1 = (fbm(cx * s + 1.5,   cy * s + 4.1,   cz * s + 2.3,   octaves, seed + 300) - 0.5) * warp;

    // Second warp layer (warp-of-warp) — half strength
    let w2 = warp * 0.45;
    let wx2 = (fbm((cx + wx1) * s,         (cy + wy1) * s,         (cz + wz1) * s,         3, seed + 400) - 0.5) * w2;
    let wy2 = (fbm((cx + wx1) * s + 2.0,   (cy + wy1) * s,         (cz + wz1) * s,         3, seed + 500) - 0.5) * w2;
    let wz2 = (fbm((cx + wx1) * s,         (cy + wy1) * s + 2.0,   (cz + wz1) * s,         3, seed + 600) - 0.5) * w2;

    fbm(
        (cx + wx1 + wx2) * s,
        (cy + wy1 + wy2) * s,
        (cz + wz1 + wz2) * s,
        octaves,
        seed,
    )
}
