//! UV-sphere (lat-lon) primitive generator.
//!
//! Provides a reference implementation matching the original `sphere.obj` topology
//! (latitude bands × longitude meridians). Useful for comparison and fallback.

use crate::mesh::{normalize, Mesh};

/// Generate a UV sphere with `lat_bands` latitude rows and `lon_bands` longitude columns.
///
/// The existing `sphere.obj` used 40 × 80. For improved quality use 64 × 128 or higher.
pub fn uv_sphere(lat_bands: u32, lon_bands: u32) -> Mesh {
    let lats = lat_bands.max(2) as usize;
    let lons = lon_bands.max(3) as usize;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity((lats + 1) * (lons + 1));
    let mut faces: Vec<[usize; 3]> = Vec::new();

    use std::f32::consts::PI;

    for lat in 0..=lats {
        let theta = lat as f32 * PI / lats as f32; // 0..π
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        for lon in 0..=lons {
            let phi = lon as f32 * 2.0 * PI / lons as f32; // 0..2π
            vertices.push(normalize([sin_t * phi.cos(), cos_t, sin_t * phi.sin()]));
        }
    }

    for lat in 0..lats {
        for lon in 0..lons {
            let a = lat * (lons + 1) + lon;
            let b = a + 1;
            let c = a + (lons + 1);
            let d = c + 1;
            if lat != 0 {
                faces.push([a, b, c]);
            }
            if lat != lats - 1 {
                faces.push([b, d, c]);
            }
        }
    }

    // For a unit sphere: smooth normal = vertex position
    let normals = vertices.clone();
    Mesh::new(vertices, normals, faces)
}
