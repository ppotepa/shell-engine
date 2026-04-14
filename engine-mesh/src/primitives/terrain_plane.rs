//! Procedural terrain plane mesh generator.
//!
//! Generates a flat NxM grid with multi-octave sine-wave height variation,
//! simulating a stylised heightmap terrain. The mesh spans [-1, 1] in X and Z,
//! with Y heights in roughly [-0.25, 0.25].
//!
//! # URI Integration
//!
//! `engine-compositor` exposes this as `terrain-plane://N` (square NxN grid).
//!
//! ```text
//! mesh-source: terrain-plane://64   → 64×64 grid, ~8k verts, ~8k quads
//! ```

use crate::mesh::{compute_smooth_normals, Mesh};

/// Generate a terrain plane with `cols` × `rows` quads.
///
/// The mesh spans X ∈ [-1, 1], Z ∈ [-1, 1].
/// Heights (Y) are derived from layered sine waves to produce gentle hills.
///
/// * `subdivisions = 32` →  1 089 verts, 2 048 tris  (fast preview)
/// * `subdivisions = 64` →  4 225 verts, 8 192 tris  (good quality)
pub fn terrain_plane(subdivisions: u32) -> Mesh {
    let cols = subdivisions.max(2) as usize;
    let rows = cols;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity((cols + 1) * (rows + 1));
    let mut faces: Vec<[usize; 3]> = Vec::with_capacity(2 * cols * rows);

    // Build vertex grid
    for row in 0..=rows {
        let t = row as f32 / rows as f32; // [0, 1]
        let z = t * 2.0 - 1.0;           // [-1, 1]
        for col in 0..=cols {
            let s = col as f32 / cols as f32; // [0, 1]
            let x = s * 2.0 - 1.0;           // [-1, 1]
            let y = height(x, z);
            vertices.push([x, y, z]);
        }
    }

    // Build quads as two CCW triangles
    let stride = cols + 1;
    for row in 0..rows {
        for col in 0..cols {
            let a = row * stride + col;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            // Upper-left triangle
            faces.push([a, c, b]);
            // Lower-right triangle
            faces.push([b, c, d]);
        }
    }

    let normals = compute_smooth_normals(&vertices, &faces);
    Mesh::new(vertices, normals, faces)
}

/// Multi-octave height function producing gently rolling hills.
///
/// Combines three octaves of sine waves at different frequencies and phases
/// to avoid obvious periodicity. Output range: approximately [-0.22, 0.22].
fn height(x: f32, z: f32) -> f32 {
    // Octave 1: broad sweeping hills
    let h1 = (x * 2.1 + 0.5).sin() * (z * 1.7 - 0.3).cos() * 0.12;
    // Octave 2: mid-frequency ridges
    let h2 = (x * 4.3 - z * 3.8 + 1.1).sin() * 0.07;
    // Octave 3: fine surface roughness
    let h3 = (x * 8.1 + z * 6.7 - 0.9).sin() * 0.03;
    h1 + h2 + h3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_and_face_counts() {
        for n in [4u32, 8, 16, 32] {
            let mesh = terrain_plane(n);
            let n = n as usize;
            assert_eq!(mesh.vertices.len(), (n + 1) * (n + 1), "n={n} verts");
            assert_eq!(mesh.faces.len(), 2 * n * n, "n={n} faces");
        }
    }

    #[test]
    fn all_face_indices_in_range() {
        let mesh = terrain_plane(8);
        let nv = mesh.vertices.len();
        for &[a, b, c] in &mesh.faces {
            assert!(a < nv && b < nv && c < nv, "index out of range");
        }
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = terrain_plane(8);
        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "normal not unit length: {len}");
        }
    }
}
