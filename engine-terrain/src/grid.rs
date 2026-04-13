//! Lat/lon grid helpers.

/// Convert grid cell (x, y) to a 3D unit vector on the sphere.
/// x ∈ [0, width), y ∈ [0, height)
/// lon = 2π * (x+0.5)/width,   lat = π * (y+0.5)/height  (0=north pole)
#[inline]
pub fn cell_to_xyz(x: usize, y: usize, width: usize, height: usize) -> (f64, f64, f64) {
    let lon = std::f64::consts::TAU * (x as f64 + 0.5) / width as f64;
    let lat = std::f64::consts::PI * (y as f64 + 0.5) / height as f64;
    let sin_lat = lat.sin();
    (sin_lat * lon.cos(), lat.cos(), sin_lat * lon.sin())
}

/// Spherical dot product (cosine similarity) of two unit vectors.
#[inline]
pub fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    (a.0 * b.0 + a.1 * b.1 + a.2 * b.2).clamp(-1.0, 1.0)
}
