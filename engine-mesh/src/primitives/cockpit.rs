//! Procedural cockpit mesh generator.
//!
//! Generates a stylised forward-facing cockpit shell intended for first-person
//! camera anchoring. Geometry stays in front of the local origin so the camera
//! can sit at `(0, 0, 0)` and look through the canopy opening.

use crate::mesh::{compute_smooth_normals, Mesh};

#[derive(Debug, Clone, PartialEq)]
pub struct CockpitParams {
    /// Overall horizontal scale multiplier.
    pub width_scale: f32,
    /// Overall forward depth multiplier.
    pub depth_scale: f32,
    /// Canopy / upper frame vertical scale multiplier.
    pub canopy_height: f32,
    /// Number of canopy ribs / dash details.
    pub detail: u8,
}

impl Default for CockpitParams {
    fn default() -> Self {
        Self {
            width_scale: 1.0,
            depth_scale: 1.0,
            canopy_height: 1.0,
            detail: 3,
        }
    }
}

/// Builds a stylised cockpit mesh plus a per-face color palette.
pub fn simulator_cockpit(params: CockpitParams) -> (Mesh, Vec<[u8; 3]>) {
    let w = params.width_scale.clamp(0.5, 2.5);
    let d = params.depth_scale.clamp(0.5, 2.5);
    let ch = params.canopy_height.clamp(0.5, 2.0);
    let detail = params.detail.clamp(0, 6);

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut colors = Vec::new();

    let hull = [52, 66, 82];
    let hull_mid = [70, 88, 108];
    let canopy = [96, 118, 144];
    let accent = [118, 170, 210];
    let warning = [214, 160, 92];

    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-1.78 * w, -1.18, 0.94 * d],
        [1.78 * w, -0.90, 2.72 * d],
        hull,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-1.62 * w, -0.92, 1.08 * d],
        [1.62 * w, -0.54, 2.54 * d],
        hull_mid,
    );

    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-1.66 * w, -0.96, 0.92 * d],
        [-1.04 * w, 0.20 * ch, 2.30 * d],
        hull,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [1.04 * w, -0.96, 0.92 * d],
        [1.66 * w, 0.20 * ch, 2.30 * d],
        hull,
    );

    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.28 * w, -1.02, 1.18 * d],
        [0.28 * w, -0.28, 1.98 * d],
        hull_mid,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.94 * w, -0.84, 1.14 * d],
        [-0.36 * w, -0.40, 1.88 * d],
        hull_mid,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [0.36 * w, -0.84, 1.14 * d],
        [0.94 * w, -0.40, 1.88 * d],
        hull_mid,
    );

    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.18 * w, -0.76, 1.14 * d],
        [0.18 * w, -0.48, 1.20 * d],
        accent,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.84 * w, -0.72, 1.10 * d],
        [-0.54 * w, -0.54, 1.16 * d],
        warning,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [0.54 * w, -0.72, 1.10 * d],
        [0.84 * w, -0.54, 1.16 * d],
        accent,
    );

    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-1.34 * w, -0.34, 0.90 * d],
        [-1.08 * w, 0.82 * ch, 1.48 * d],
        canopy,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [1.08 * w, -0.34, 0.90 * d],
        [1.34 * w, 0.82 * ch, 1.48 * d],
        canopy,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.98 * w, 0.58 * ch, 0.98 * d],
        [0.98 * w, 0.84 * ch, 1.46 * d],
        canopy,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.96 * w, -0.10, 0.98 * d],
        [-0.78 * w, 0.84 * ch, 1.36 * d],
        canopy,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [0.78 * w, -0.10, 0.98 * d],
        [0.96 * w, 0.84 * ch, 1.36 * d],
        canopy,
    );
    add_box(
        &mut vertices,
        &mut faces,
        &mut colors,
        [-0.88 * w, -0.38, 0.98 * d],
        [0.88 * w, -0.18, 1.30 * d],
        hull,
    );

    for idx in 0..detail {
        let t = (idx as f32 + 1.0) / (detail as f32 + 1.0);
        let z0 = (1.48 + t * 0.62) * d;
        let z1 = z0 + 0.08 * d;
        let rib_y_min = (0.18 + t * 0.08) * ch;
        let rib_y_max = 0.82 * ch;
        add_box(
            &mut vertices,
            &mut faces,
            &mut colors,
            [-1.22 * w, rib_y_min, z0],
            [-1.08 * w, rib_y_max, z1],
            canopy,
        );
        add_box(
            &mut vertices,
            &mut faces,
            &mut colors,
            [1.08 * w, rib_y_min, z0],
            [1.22 * w, rib_y_max, z1],
            canopy,
        );

        let panel_z0 = (1.28 + t * 0.82) * d;
        let panel_z1 = panel_z0 + 0.06 * d;
        add_box(
            &mut vertices,
            &mut faces,
            &mut colors,
            [(-0.60 + t * 0.18) * w, -0.82, panel_z0],
            [(-0.48 + t * 0.18) * w, -0.56, panel_z1],
            if idx % 2 == 0 { accent } else { warning },
        );
        add_box(
            &mut vertices,
            &mut faces,
            &mut colors,
            [(0.48 - t * 0.18) * w, -0.82, panel_z0],
            [(0.60 - t * 0.18) * w, -0.56, panel_z1],
            if idx % 2 == 0 { accent } else { warning },
        );
    }

    let normals = compute_smooth_normals(&vertices, &faces);
    (Mesh::new(vertices, normals, faces), colors)
}

fn add_box(
    vertices: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
    colors: &mut Vec<[u8; 3]>,
    min: [f32; 3],
    max: [f32; 3],
    color: [u8; 3],
) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;

    add_quad(
        vertices,
        faces,
        colors,
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0],
        color,
    );
    add_quad(
        vertices,
        faces,
        colors,
        [x1, y0, z1],
        [x0, y0, z1],
        [x0, y1, z1],
        [x1, y1, z1],
        color,
    );
    add_quad(
        vertices,
        faces,
        colors,
        [x0, y0, z1],
        [x0, y0, z0],
        [x0, y1, z0],
        [x0, y1, z1],
        color,
    );
    add_quad(
        vertices,
        faces,
        colors,
        [x1, y0, z0],
        [x1, y0, z1],
        [x1, y1, z1],
        [x1, y1, z0],
        color,
    );
    add_quad(
        vertices,
        faces,
        colors,
        [x0, y1, z0],
        [x1, y1, z0],
        [x1, y1, z1],
        [x0, y1, z1],
        color,
    );
    add_quad(
        vertices,
        faces,
        colors,
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y0, z0],
        [x0, y0, z0],
        color,
    );
}

fn add_quad(
    vertices: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
    colors: &mut Vec<[u8; 3]>,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
    color: [u8; 3],
) {
    let base = vertices.len();
    vertices.push(a);
    vertices.push(b);
    vertices.push(c);
    vertices.push(d);
    faces.push([base, base + 1, base + 2]);
    colors.push(color);
    faces.push([base, base + 2, base + 3]);
    colors.push(color);
}

#[cfg(test)]
mod tests {
    use super::{simulator_cockpit, CockpitParams};

    #[test]
    fn cockpit_mesh_builds_non_empty_forward_geometry() {
        let (mesh, colors) = simulator_cockpit(CockpitParams::default());
        assert!(!mesh.vertices.is_empty(), "cockpit should emit vertices");
        assert!(!mesh.faces.is_empty(), "cockpit should emit faces");
        assert_eq!(
            mesh.faces.len(),
            colors.len(),
            "every cockpit face should have an explicit color"
        );
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v[2])
            .fold(f32::INFINITY, f32::min);
        assert!(
            min_z > 0.5,
            "cockpit geometry should stay in front of the camera origin, got min_z={min_z}"
        );
    }

    #[test]
    fn cockpit_detail_increases_face_count() {
        let (coarse, _) = simulator_cockpit(CockpitParams {
            detail: 0,
            ..CockpitParams::default()
        });
        let (detailed, _) = simulator_cockpit(CockpitParams {
            detail: 5,
            ..CockpitParams::default()
        });
        assert!(
            detailed.faces.len() > coarse.faces.len(),
            "higher cockpit detail should add more geometry"
        );
    }
}
