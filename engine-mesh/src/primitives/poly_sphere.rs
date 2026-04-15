//! Polyhedron-based sphere generators (tetra/octa/icosa).
//!
//! These start from a regular polyhedron and repeatedly subdivide each
//! triangle, normalizing all vertices onto the unit sphere after each step.

use std::collections::HashMap;

use crate::mesh::{compute_smooth_normals, normalize, Mesh};

/// Generate a tetrahedron-based sphere with `levels` recursive subdivisions.
pub fn tetra_sphere(levels: u32) -> Mesh {
    // Regular tetrahedron (unit-ish, normalized below)
    let verts = vec![
        [1.0, 1.0, 1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
    ];
    let faces = vec![[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]];
    subdivide_poly_sphere(verts, faces, levels)
}

/// Generate an octahedron-based sphere with `levels` recursive subdivisions.
pub fn octa_sphere(levels: u32) -> Mesh {
    let verts = vec![
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];
    let faces = vec![
        [0, 4, 2],
        [2, 4, 1],
        [1, 4, 3],
        [3, 4, 0],
        [0, 2, 5],
        [2, 1, 5],
        [1, 3, 5],
        [3, 0, 5],
    ];
    subdivide_poly_sphere(verts, faces, levels)
}

/// Generate an icosahedron-based sphere with `levels` recursive subdivisions.
pub fn icosa_sphere(levels: u32) -> Mesh {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let verts = vec![
        [-1.0, phi, 0.0],
        [1.0, phi, 0.0],
        [-1.0, -phi, 0.0],
        [1.0, -phi, 0.0],
        [0.0, -1.0, phi],
        [0.0, 1.0, phi],
        [0.0, -1.0, -phi],
        [0.0, 1.0, -phi],
        [phi, 0.0, -1.0],
        [phi, 0.0, 1.0],
        [-phi, 0.0, -1.0],
        [-phi, 0.0, 1.0],
    ];
    let faces = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    subdivide_poly_sphere(verts, faces, levels)
}

fn subdivide_poly_sphere(
    mut vertices: Vec<[f32; 3]>,
    mut faces: Vec<[usize; 3]>,
    levels: u32,
) -> Mesh {
    for v in &mut vertices {
        *v = normalize(*v);
    }

    let iterations = levels.min(6);
    for _ in 0..iterations {
        let mut edge_midpoints: HashMap<(usize, usize), usize> = HashMap::new();
        let mut next_faces = Vec::with_capacity(faces.len() * 4);

        for [a, b, c] in faces {
            let ab = midpoint_index(a, b, &mut vertices, &mut edge_midpoints);
            let bc = midpoint_index(b, c, &mut vertices, &mut edge_midpoints);
            let ca = midpoint_index(c, a, &mut vertices, &mut edge_midpoints);

            next_faces.push([a, ab, ca]);
            next_faces.push([b, bc, ab]);
            next_faces.push([c, ca, bc]);
            next_faces.push([ab, bc, ca]);
        }

        faces = next_faces;
    }

    let normals = compute_smooth_normals(&vertices, &faces);
    Mesh::new(vertices, normals, faces)
}

fn midpoint_index(
    a: usize,
    b: usize,
    vertices: &mut Vec<[f32; 3]>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let va = vertices[a];
    let vb = vertices[b];
    let mid = normalize([
        (va[0] + vb[0]) * 0.5,
        (va[1] + vb[1]) * 0.5,
        (va[2] + vb[2]) * 0.5,
    ]);
    let idx = vertices.len();
    vertices.push(mid);
    cache.insert(key, idx);
    idx
}
