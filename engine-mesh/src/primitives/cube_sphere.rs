//! Cube-sphere primitive generator.
//!
//! A cube sphere is built by subdividing each face of a unit cube into an
//! N×N grid of quads, then normalising every vertex onto the unit sphere.
//! This avoids the pole singularity of a UV sphere: every triangle has a
//! similar area and the vertex distribution is nearly uniform.
//!
//! # Topology
//! - 6 faces × (N+1)² vertices → `6(N+1)²` total vertices (seam vertices duplicated)
//! - 6 faces × 2N² triangles  → `12N²` total triangles
//! - Smooth normals = vertex positions (unit sphere)
//!
//! For N = 64: 25 350 verts, 49 152 tris (vs 3 122 verts / 6 240 tris for the OBJ UV-sphere).

use crate::mesh::{compute_smooth_normals, normalize, Mesh};

/// The (right, up) tangent basis for each cube face in the order
/// [+X, -X, +Y, -Y, +Z, -Z].
///
/// Given face normal `n`, a grid point (s, t) in [-1, 1]² maps to:
///   cube_point = n + right * s + up * t
/// which is then normalised onto the sphere.
const FACE_BASES: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
    // normal        right           up
    ([1.0,  0.0,  0.0], [0.0,  0.0, -1.0], [0.0,  1.0,  0.0]), // +X
    ([-1.0, 0.0,  0.0], [0.0,  0.0,  1.0], [0.0,  1.0,  0.0]), // -X
    ([0.0,  1.0,  0.0], [1.0,  0.0,  0.0], [0.0,  0.0, -1.0]), // +Y
    ([0.0, -1.0,  0.0], [1.0,  0.0,  0.0], [0.0,  0.0,  1.0]), // -Y
    ([0.0,  0.0,  1.0], [1.0,  0.0,  0.0], [0.0,  1.0,  0.0]), // +Z
    ([0.0,  0.0, -1.0], [-1.0, 0.0,  0.0], [0.0,  1.0,  0.0]), // -Z
];

/// Generate a cube-sphere with `subdivisions` grid divisions per face edge.
///
/// * `subdivisions = 32`  → ~6 k verts, ~12 k tris  (fast, coarse)
/// * `subdivisions = 64`  → ~25 k verts, ~49 k tris  (good quality)
/// * `subdivisions = 128` → ~100 k verts, ~196 k tris (high-res)
pub fn cube_sphere(subdivisions: u32) -> Mesh {
    let n = subdivisions.max(1) as usize;
    let verts_per_face = (n + 1) * (n + 1);
    let tris_per_face = 2 * n * n;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(6 * verts_per_face);
    let mut faces: Vec<[usize; 3]> = Vec::with_capacity(6 * tris_per_face);

    for (face_idx, (fn_, right, up)) in FACE_BASES.iter().enumerate() {
        let base = face_idx * verts_per_face;

        // Vertices: (n+1) rows × (n+1) cols
        for row in 0..=n {
            for col in 0..=n {
                let s = (col as f32 / n as f32) * 2.0 - 1.0;
                let t = (row as f32 / n as f32) * 2.0 - 1.0;
                let x = fn_[0] + right[0] * s + up[0] * t;
                let y = fn_[1] + right[1] * s + up[1] * t;
                let z = fn_[2] + right[2] * s + up[2] * t;
                vertices.push(normalize([x, y, z]));
            }
        }

        // Quads split into 2 CCW triangles
        for row in 0..n {
            for col in 0..n {
                let a = base + row * (n + 1) + col;
                let b = a + 1;
                let c = a + (n + 1);
                let d = c + 1;
                faces.push([a, c, b]);
                faces.push([b, c, d]);
            }
        }
    }

    let normals = compute_smooth_normals(&vertices, &faces);
    Mesh::new(vertices, normals, faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_count_matches_formula() {
        for n in [1u32, 4, 8, 16, 32] {
            let mesh = cube_sphere(n);
            let expected_verts = 6 * (n as usize + 1) * (n as usize + 1);
            let expected_tris = 12 * n as usize * n as usize;
            assert_eq!(mesh.vertices.len(), expected_verts, "n={n} verts");
            assert_eq!(mesh.faces.len(), expected_tris, "n={n} tris");
        }
    }

    #[test]
    fn all_vertices_on_unit_sphere() {
        let mesh = cube_sphere(8);
        for v in &mesh.vertices {
            let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-5, "vertex not on unit sphere: r={r}");
        }
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = cube_sphere(8);
        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "normal not unit length: {len}");
        }
    }

    #[test]
    fn all_face_indices_in_range() {
        let mesh = cube_sphere(8);
        let nv = mesh.vertices.len();
        for &[a, b, c] in &mesh.faces {
            assert!(a < nv && b < nv && c < nv, "index out of range");
        }
    }
}
