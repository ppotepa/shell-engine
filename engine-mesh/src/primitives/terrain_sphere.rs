//! Procedural terrain sphere mesh generator.
//!
//! A cube-sphere where each vertex is radially displaced by a 3D noise function.
//! Using 3D Cartesian coordinates for noise input eliminates seams at the poles
//! and the antimeridian — the noise field is continuous across the entire sphere.
//!
//! # URI Integration
//!
//! `engine-compositor` exposes this as `terrain-sphere://N` (with optional query params):
//!
//! ```text
//! terrain-sphere://32                            → defaults (amp=1, freq=1, oct=3, rough=1)
//! terrain-sphere://32?amp=2.0&freq=0.5           → larger, lower-frequency mountains
//! terrain-sphere://32?ridge=1&lac=2.5            → sharp ridge terrain
//! terrain-sphere://32?plat=0.4&sea=0.2           → mesa plateaus with ocean floor
//! ```

use crate::mesh::{compute_smooth_normals, normalize, Mesh};
use crate::primitives::terrain_plane::TerrainParams;

/// Per-face RGB color computed from vertex altitude, for use with `earth_terrain_sphere`.
pub type FaceColors = Vec<[u8; 3]>;

/// The (normal, right, up) tangent basis for each cube face in the order [+X, -X, +Y, -Y, +Z, -Z].
const FACE_BASES: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
    ([1.0,  0.0,  0.0], [0.0,  0.0, -1.0], [0.0,  1.0,  0.0]),
    ([-1.0, 0.0,  0.0], [0.0,  0.0,  1.0], [0.0,  1.0,  0.0]),
    ([0.0,  1.0,  0.0], [1.0,  0.0,  0.0], [0.0,  0.0, -1.0]),
    ([0.0, -1.0,  0.0], [1.0,  0.0,  0.0], [0.0,  0.0,  1.0]),
    ([0.0,  0.0,  1.0], [1.0,  0.0,  0.0], [0.0,  1.0,  0.0]),
    ([0.0,  0.0, -1.0], [-1.0, 0.0,  0.0], [0.0,  1.0,  0.0]),
];

/// Generate a terrain sphere with `subdivisions` grid divisions per cube face edge.
///
/// Each vertex on the unit cube-sphere is radially displaced by a 3D noise
/// function sampled at the vertex's Cartesian position, producing seamless
/// planetary terrain without UV discontinuities.
///
/// * `subdivisions = 24` → ~3.5 k verts, ~6.9 k tris (fast, coarse terrain)
/// * `subdivisions = 32` → ~6 k verts, ~12 k tris  (good quality)
/// * `subdivisions = 48` → ~14 k verts, ~27 k tris  (high detail)
///
/// Pass `TerrainParams::default()` for a smooth, gently varied surface.
pub fn terrain_sphere(subdivisions: u32, params: TerrainParams) -> Mesh {
    let n = subdivisions.max(1) as usize;
    let verts_per_face = (n + 1) * (n + 1);
    let tris_per_face = 2 * n * n;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(6 * verts_per_face);
    let mut faces: Vec<[usize; 3]> = Vec::with_capacity(6 * tris_per_face);

    for (face_idx, (fn_, right, up)) in FACE_BASES.iter().enumerate() {
        let base = face_idx * verts_per_face;

        for row in 0..=n {
            for col in 0..=n {
                let s = (col as f32 / n as f32) * 2.0 - 1.0;
                let t = (row as f32 / n as f32) * 2.0 - 1.0;
                let x = fn_[0] + right[0] * s + up[0] * t;
                let y = fn_[1] + right[1] * s + up[1] * t;
                let z = fn_[2] + right[2] * s + up[2] * t;
                // Normalize onto unit sphere, then radially displace.
                let dir = normalize([x, y, z]);
                let h = sphere_height(dir, &params);
                // Displace outward along the surface normal (= unit direction on sphere).
                let r = 1.0 + h;
                vertices.push([dir[0] * r, dir[1] * r, dir[2] * r]);
            }
        }

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

    // Recompute normals from displaced geometry (not the original sphere normals).
    let normals = compute_smooth_normals(&vertices, &faces);
    Mesh::new(vertices, normals, faces)
}

/// 3D noise height for terrain sphere.
///
/// Uses the vertex's Cartesian direction as input — this is a continuous 3D
/// function so no seams appear where cube faces meet or at the poles.
///
/// The displacement range at `amplitude = 1.0` is approximately ±0.20.
fn sphere_height(dir: [f32; 3], p: &TerrainParams) -> f32 {
    let [dx, dy, dz] = dir;

    // Apply frequency and scale. scale_x stretches the XZ plane (longitude/latitude
    // bands); scale_z stretches the Y axis (polar vs equatorial features).
    let fx = dx * p.frequency * p.scale_x + p.seed_x;
    let fy = dy * p.frequency * p.scale_z + p.seed_z;
    let fz = dz * p.frequency * p.scale_x;
    let lac = p.lacunarity;

    // Each octave samples a different axis pair to break up regular patterns.
    let sample = |nx: f32, ny: f32, nz: f32| -> f32 {
        let raw = (nx * 2.1 + ny * 0.9 + 0.5).sin() * (nz * 1.7 + ny * 1.3 - 0.3).cos();
        if p.ridge { raw.abs() } else { raw }
    };

    let h1 = sample(fx, fy, fz) * 0.12 * p.amplitude;
    let h2 = if p.octaves >= 2 {
        sample(fx * lac, fy * lac, fz * lac) * 0.07 * p.amplitude * p.roughness
    } else {
        0.0
    };
    let h3 = if p.octaves >= 3 {
        sample(fx * lac * lac, fy * lac * lac, fz * lac * lac)
            * 0.03
            * p.amplitude
            * p.roughness
            * p.roughness
    } else {
        0.0
    };
    let mut h = h1 + h2 + h3;

    if p.plateau > 0.0 {
        let max_h = 0.22 * p.amplitude;
        let thresh = max_h * (1.0 - p.plateau * 0.8);
        if h > thresh {
            let compression = (1.0 - p.plateau).max(0.0);
            h = thresh + (h - thresh) * compression;
        }
    }

    if p.sea_level > 0.0 {
        let min_h = -0.22 * p.amplitude;
        let floor = min_h + (0.44 * p.amplitude * p.sea_level);
        if h < floor {
            h = floor;
        }
    }

    h
}

/// Generate a terrain sphere with per-face altitude-based Earth-like colors.
///
/// The mesh geometry is identical to [`terrain_sphere`]; each face receives an
/// RGB color sampled from the Earth palette based on the average radius of its
/// three vertices relative to the displacement range.
///
/// Radius meaning:
/// * `r < 1.0` — below sea level (ocean, blue gradient)
/// * `r ≈ 1.0` — sea level / beach
/// * `r > 1.0` — land (green → highland → rock → snow)
///
/// The returned `FaceColors` vector has one entry per face, in the same order as
/// `mesh.faces`.  Pass them to `colored_mesh_to_obj_mesh` in engine-compositor.
pub fn earth_terrain_sphere(subdivisions: u32, params: TerrainParams) -> (Mesh, FaceColors) {
    let mesh = terrain_sphere(subdivisions, params.clone());
    let amp = params.amplitude;
    let colors: Vec<[u8; 3]> = mesh
        .faces
        .iter()
        .map(|[a, b, c]| {
            let ra = vertex_radius(&mesh.vertices[*a]);
            let rb = vertex_radius(&mesh.vertices[*b]);
            let rc = vertex_radius(&mesh.vertices[*c]);
            let avg_r = (ra + rb + rc) / 3.0;
            earth_altitude_color(avg_r, amp)
        })
        .collect();
    (mesh, colors)
}

fn vertex_radius(v: &[f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Earth-like altitude colour gradient.
///
/// All thresholds are expressed as absolute sphere radius (sea level = 1.0).
/// The max displacement at `amplitude = 1.0` is ~±0.22, so a pixel can reach
/// radius 1.22 at its highest.  Thresholds scale linearly with `amplitude`.
fn earth_altitude_color(r: f32, amplitude: f32) -> [u8; 3] {
    let amp = amplitude.max(0.01);
    let max_d = 0.22 * amp; // positive half of displacement range

    if r < 1.0 {
        // Ocean: dark deep → lighter shallow as we approach surface.
        let depth = ((1.0 - r) / max_d).clamp(0.0, 1.0);
        // deep ocean #0a1f4a → shallow ocean #1a5fa0
        lerp_rgb([10, 31, 74], [26, 95, 160], 1.0 - depth)
    } else {
        let h = ((r - 1.0) / max_d).clamp(0.0, 1.0);
        if h < 0.06 {
            // Beach / sand
            [194, 178, 120]
        } else if h < 0.30 {
            // Lowland green
            let t = (h - 0.06) / 0.24;
            lerp_rgb([58, 150, 50], [40, 110, 35], t)
        } else if h < 0.55 {
            // Highland / shrubland
            let t = (h - 0.30) / 0.25;
            lerp_rgb([100, 85, 55], [120, 100, 70], t)
        } else if h < 0.80 {
            // Rock / mountain
            let t = (h - 0.55) / 0.25;
            lerp_rgb([130, 118, 105], [165, 155, 145], t)
        } else {
            // Snow cap
            let t = ((h - 0.80) / 0.20).min(1.0);
            lerp_rgb([200, 200, 210], [240, 242, 248], t)
        }
    }
}

fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_count_matches_formula() {
        for n in [1u32, 4, 8, 16, 24] {
            let mesh = terrain_sphere(n, TerrainParams::default());
            let expected_verts = 6 * (n as usize + 1) * (n as usize + 1);
            let expected_tris = 12 * n as usize * n as usize;
            assert_eq!(mesh.vertices.len(), expected_verts, "n={n} verts");
            assert_eq!(mesh.faces.len(), expected_tris, "n={n} tris");
        }
    }

    #[test]
    fn displaced_vertices_near_unit_sphere() {
        // Default params produce displacement of ~±0.22 at amplitude=1.
        let mesh = terrain_sphere(8, TerrainParams::default());
        for v in &mesh.vertices {
            let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!(r > 0.5 && r < 2.0, "vertex radius out of expected range: r={r}");
        }
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = terrain_sphere(8, TerrainParams::default());
        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-3, "normal not unit length: {len}");
        }
    }
}
