//! Core mesh data type shared by all generators.

/// A triangle mesh in 3D space.
///
/// Vertices are in object space (unit sphere for sphere primitives).
/// Smooth normals are pre-computed per-vertex from face area-weighted averages.
/// For sphere primitives the smooth normal equals the normalised vertex position.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Triangle index triples into `vertices`.
    pub faces: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn new(vertices: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, faces: Vec<[usize; 3]>) -> Self {
        Self {
            vertices,
            normals,
            faces,
        }
    }
}

/// Compute area-weighted smooth normals from face geometry.
///
/// For a unit sphere the result equals the vertex positions, but this
/// general implementation works for any genus-0 mesh.
pub fn compute_smooth_normals(vertices: &[[f32; 3]], faces: &[[usize; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];

    for &[ia, ib, ic] in faces {
        let a = vertices[ia];
        let b = vertices[ib];
        let c = vertices[ic];

        // Edge vectors
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];

        // Cross product (area-weighted face normal)
        let n = cross(ab, ac);

        for &i in &[ia, ib, ic] {
            normals[i][0] += n[0];
            normals[i][1] += n[1];
            normals[i][2] += n[2];
        }
    }

    for n in &mut normals {
        *n = normalize(*n);
    }
    normals
}

#[inline]
pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
pub(crate) fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}
