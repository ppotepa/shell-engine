//! Procedural terrain plane mesh generator.
//!
//! Generates a flat NxM grid with multi-octave sine-wave height variation,
//! simulating a stylised heightmap terrain. The mesh spans [-1, 1] in X and Z,
//! with Y heights in roughly [-0.22, 0.22] (at default params).
//!
//! # URI Integration
//!
//! `engine-compositor` exposes this as `terrain-plane://N` (with optional query params):
//!
//! ```text
//! terrain-plane://64                       → defaults (amp=1, freq=1, oct=3, rough=1)
//! terrain-plane://64?amp=2.0&freq=0.5      → tall, low-frequency hills
//! terrain-plane://64?oct=1&rough=0.0       → single-octave smooth terrain
//! terrain-plane://64?sx=3.2&sz=-1.7        → shifted seed region
//! ```

use crate::mesh::{compute_smooth_normals, Mesh};

/// Runtime-tweakable parameters for the terrain height generator.
///
/// All fields have safe defaults that reproduce the original hardcoded terrain.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainParams {
    /// Overall height scale multiplier. Default 1.0 (range 0.1 – 5.0).
    pub amplitude: f32,
    /// Frequency multiplier — higher = more features per unit. Default 1.0 (range 0.1 – 8.0).
    pub frequency: f32,
    /// Number of octave layers (1 = smooth, 3 = full detail). Default 3.
    pub octaves: u8,
    /// Weight of high-frequency octaves relative to octave 1. Default 1.0 (range 0.0 – 1.0).
    pub roughness: f32,
    /// X-axis seed offset — shifts the terrain region. Default 0.0.
    pub seed_x: f32,
    /// Z-axis seed offset — shifts the terrain region. Default 0.0.
    pub seed_z: f32,
    /// Frequency multiplier between successive octaves. Default 2.0 (range 1.0 – 4.0).
    /// Higher = octaves spread apart faster, more contrast between scales.
    pub lacunarity: f32,
    /// Ridge mode: uses abs() on each octave sample to produce sharp mountain ridges
    /// instead of smooth hills. Default false.
    pub ridge: bool,
    /// Plateau strength (0.0 = off, 1.0 = full mesa). Compresses peaks above a threshold,
    /// producing flat-topped mesas and stepped terrain. Default 0.0.
    pub plateau: f32,
    /// Sea level (0.0 = off, 1.0 = fully flooded). Clamps the terrain floor upward,
    /// creating flat ocean/lake areas. At 0.5 everything below mid-height is flat.
    /// Default 0.0.
    pub sea_level: f32,
    /// Anisotropic X stretch — stretches the noise field horizontally.
    /// Values > 1 produce elongated ridges along Z; values < 1 compress them. Default 1.0.
    pub scale_x: f32,
    /// Anisotropic Z stretch — stretches the noise field along the depth axis. Default 1.0.
    pub scale_z: f32,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            frequency: 1.0,
            octaves: 3,
            roughness: 1.0,
            seed_x: 0.0,
            seed_z: 0.0,
            lacunarity: 2.0,
            ridge: false,
            plateau: 0.0,
            sea_level: 0.0,
            scale_x: 1.0,
            scale_z: 1.0,
        }
    }
}

/// Generate a terrain plane with `cols` × `rows` quads.
///
/// The mesh spans X ∈ [-1, 1], Z ∈ [-1, 1].
/// Heights (Y) are derived from layered sine waves to produce gentle hills.
///
/// * `subdivisions = 32` →  1 089 verts, 2 048 tris  (fast preview)
/// * `subdivisions = 64` →  4 225 verts, 8 192 tris  (good quality)
///
/// Pass `TerrainParams::default()` for the original hardcoded appearance.
pub fn terrain_plane(subdivisions: u32, params: TerrainParams) -> Mesh {
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
            let y = height(x, z, &params);
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
/// With default params the output range is approximately [-0.22, 0.22].
/// `amplitude` scales all outputs; `frequency` zooms the noise field;
/// `roughness` attenuates the higher octaves (0.0 = smooth, 1.0 = full detail);
/// `lacunarity` controls frequency ratio between octaves;
/// `ridge` enables sharp mountain-ridge mode via abs() on each octave;
/// `plateau` flattens peaks above a threshold for mesa terrain;
/// `sea_level` clamps the floor upward to create flat ocean areas;
/// `scale_x`/`scale_z` apply anisotropic stretch to the noise field.
fn height(x: f32, z: f32, p: &TerrainParams) -> f32 {
    // Apply anisotropic scale, frequency, and seed offset.
    let fx = x * p.frequency * p.scale_x + p.seed_x;
    let fz = z * p.frequency * p.scale_z + p.seed_z;
    let lac = p.lacunarity;

    // Helper: sample the base wave pattern at (nx, nz).
    // In ridge mode the absolute value produces V-shaped ridges instead of sinusoidal hills.
    let sample = |nx: f32, nz: f32| -> f32 {
        let raw = (nx * 2.1 + 0.5).sin() * (nz * 1.7 - 0.3).cos();
        if p.ridge { raw.abs() } else { raw }
    };

    // Octave 1: broad sweeping hills (always present).
    let h1 = sample(fx, fz) * 0.12 * p.amplitude;
    // Octave 2: mid-frequency detail, attenuated by roughness.
    let h2 = if p.octaves >= 2 {
        sample(fx * lac, fz * lac) * 0.07 * p.amplitude * p.roughness
    } else {
        0.0
    };
    // Octave 3: fine surface roughness, attenuated by roughness².
    let h3 = if p.octaves >= 3 {
        sample(fx * lac * lac, fz * lac * lac) * 0.03 * p.amplitude * p.roughness * p.roughness
    } else {
        0.0
    };
    let mut h = h1 + h2 + h3;

    // Plateau: compress heights above a threshold to create flat-topped mesas.
    if p.plateau > 0.0 {
        let max_h = 0.22 * p.amplitude;
        let thresh = max_h * (1.0 - p.plateau * 0.8);
        if h > thresh {
            let compression = (1.0 - p.plateau).max(0.0);
            h = thresh + (h - thresh) * compression;
        }
    }

    // Sea level: clamp the floor upward to create flat ocean/lake areas.
    if p.sea_level > 0.0 {
        let max_h = 0.22 * p.amplitude;
        // sea_level=0 → floor at -max_h, sea_level=1 → floor at +max_h (fully flooded).
        let floor = -max_h + p.sea_level * 2.0 * max_h;
        h = h.max(floor);
    }

    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_and_face_counts() {
        for n in [4u32, 8, 16, 32] {
            let mesh = terrain_plane(n, TerrainParams::default());
            let n = n as usize;
            assert_eq!(mesh.vertices.len(), (n + 1) * (n + 1), "n={n} verts");
            assert_eq!(mesh.faces.len(), 2 * n * n, "n={n} faces");
        }
    }

    #[test]
    fn all_face_indices_in_range() {
        let mesh = terrain_plane(8, TerrainParams::default());
        let nv = mesh.vertices.len();
        for &[a, b, c] in &mesh.faces {
            assert!(a < nv && b < nv && c < nv, "index out of range");
        }
    }

    #[test]
    fn normals_are_unit_length() {
        let mesh = terrain_plane(8, TerrainParams::default());
        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "normal not unit length: {len}");
        }
    }

    #[test]
    fn params_affect_height() {
        let base = terrain_plane(8, TerrainParams::default());
        let tall = terrain_plane(8, TerrainParams { amplitude: 3.0, ..Default::default() });
        let flat = terrain_plane(8, TerrainParams { amplitude: 0.1, ..Default::default() });
        // Tall mesh should have larger Y range than default, flat should have smaller.
        let base_range: f32 = base.vertices.iter().map(|v| v[1].abs()).fold(0.0_f32, f32::max);
        let tall_range: f32 = tall.vertices.iter().map(|v| v[1].abs()).fold(0.0_f32, f32::max);
        let flat_range: f32 = flat.vertices.iter().map(|v| v[1].abs()).fold(0.0_f32, f32::max);
        assert!(tall_range > base_range, "amplitude=3.0 should exceed default");
        assert!(flat_range < base_range, "amplitude=0.1 should be below default");
    }
}
