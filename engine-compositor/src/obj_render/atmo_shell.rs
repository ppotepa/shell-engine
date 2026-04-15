//! Atmosphere shell (corona) rendering.
//!
//! Renders a slightly-enlarged UV sphere around the planet to create a glowing halo effect.
//! The shell is rendered BEFORE the planet, so the planet overwrites interior pixels via depth testing.
//! Only the outer ring (where shell extends beyond planet silhouette) remains visible.

use engine_mesh::primitives::uv_sphere;

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
    // Pitch (X-axis rotation)
    let cp = pitch.cos();
    let sp = pitch.sin();
    let [x, y, z] = v;
    let [x, y, z] = [x, cp * y - sp * z, sp * y + cp * z];

    // Yaw (Y-axis rotation)
    let cy = yaw.cos();
    let sy = yaw.sin();
    let [x, y, z] = [cy * x + sy * z, y, -sy * x + cy * z];

    // Roll (Z-axis rotation)
    let cr = roll.cos();
    let sr = roll.sin();
    [cr * x - sr * y, sr * x + cr * y, z]
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn edge(px0: f32, py0: f32, px1: f32, py1: f32, px2: f32, py2: f32) -> f32 {
    (px1 - px0) * (py2 - py0) - (py1 - py0) * (px2 - px0)
}

struct ProjectedVertex {
    x: f32,
    y: f32,
    depth: f32,
    alpha: f32,
}

pub(crate) fn render_atmo_shell_pass(
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
    virtual_w: u16,
    virtual_h: u16,
    // Camera / projection
    pitch: f32,
    yaw: f32,
    roll: f32,
    inv_tan: f32,
    aspect: f32,
    near_clip: f32,
    camera_world: [f32; 3],
    view_right: [f32; 3],
    view_up: [f32; 3],
    view_forward: [f32; 3],
    shell_scale: f32,
    // Atmosphere
    atmo_color: [u8; 3],
    strength: f32,
    rim_power: f32,
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
) {
    if shell_scale < 1.001 || strength <= 0.0 {
        return;
    }

    let w = virtual_w as i32;
    let h = virtual_h as i32;
    if w < 2 || h < 2 {
        return;
    }

    // Generate UV sphere (unit radius, normalized normals)
    let mesh = uv_sphere(32, 64);

    // Project vertices
    let mut projected: Vec<ProjectedVertex> = Vec::new();
    projected.reserve(mesh.vertices.len());

    for (i, &v) in mesh.vertices.iter().enumerate() {
        // Scale vertex
        let scaled = [v[0] * shell_scale, v[1] * shell_scale, v[2] * shell_scale];

        // Rotate (same as planet)
        let rotated = rotate_xyz(scaled, pitch, yaw, roll);

        // View transform
        let rel = [
            rotated[0] - camera_world[0],
            rotated[1] - camera_world[1],
            rotated[2] - camera_world[2],
        ];
        let cam_x = dot3(rel, view_right);
        let cam_y = dot3(rel, view_up);
        let view_z = dot3(rel, view_forward);

        if view_z <= near_clip {
            projected.push(ProjectedVertex {
                x: f32::NAN,
                y: f32::NAN,
                depth: f32::INFINITY,
                alpha: 0.0,
            });
            continue;
        }

        // Perspective projection
        let ndc_x = (cam_x / aspect) * inv_tan / view_z;
        let ndc_y = cam_y * inv_tan / view_z;

        let screen_x = (ndc_x + 1.0) * 0.5 * (w as f32 - 1.0);
        let screen_y = (1.0 - (ndc_y + 1.0) * 0.5) * (h as f32 - 1.0);

        // Compute rim at this vertex
        let rotated_normal = rotate_xyz(mesh.normals[i], pitch, yaw, roll);
        let nd = dot3(rotated_normal, view_dir).abs().clamp(0.0, 1.0);
        let rim = (1.0 - nd).powf(rim_power.max(0.1));

        let day = smoothstep(-0.1, 0.3, dot3(rotated_normal, sun_dir));
        let alpha = rim * (0.55 + 0.90 * day) * strength.max(0.0);

        projected.push(ProjectedVertex {
            x: screen_x,
            y: screen_y,
            depth: view_z,
            alpha,
        });
    }

    // Sort faces back-to-front (simple z-sort by average depth)
    let mut face_indices: Vec<usize> = (0..mesh.faces.len()).collect();
    face_indices.sort_by(|&a, &b| {
        let avg_a = (projected[mesh.faces[a][0]].depth
            + projected[mesh.faces[a][1]].depth
            + projected[mesh.faces[a][2]].depth)
            / 3.0;
        let avg_b = (projected[mesh.faces[b][0]].depth
            + projected[mesh.faces[b][1]].depth
            + projected[mesh.faces[b][2]].depth)
            / 3.0;
        b.partial_cmp(&a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| avg_b.partial_cmp(&avg_a).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Rasterize each face
    for &fi in &face_indices {
        let face = &mesh.faces[fi];
        let [i0, i1, i2] = [face[0], face[1], face[2]];
        let v0 = &projected[i0];
        let v1 = &projected[i1];
        let v2 = &projected[i2];

        // Skip if any vertex is out of range (NAN check)
        if !v0.x.is_finite() || !v1.x.is_finite() || !v2.x.is_finite() {
            continue;
        }

        // Backface cull
        if edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
            continue;
        }

        // Rasterize triangle
        rasterize_shell_triangle(
            canvas, depth_buf, w, h, v0, v1, v2, atmo_color,
        );
    }
}

#[inline(always)]
fn rasterize_shell_triangle(
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
    w: i32,
    h: i32,
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    atmo_color: [u8; 3],
) {
    let min_x = (v0.x.min(v1.x).min(v2.x) - 0.5).ceil() as i32;
    let max_x = (v0.x.max(v1.x).max(v2.x) + 0.5).floor() as i32;
    let min_y = (v0.y.min(v1.y).min(v2.y) - 0.5).ceil() as i32;
    let max_y = (v0.y.max(v1.y).max(v2.y) + 0.5).floor() as i32;

    let x_start = min_x.max(0);
    let x_end = max_x.min(w - 1);
    let y_start = min_y.max(0);
    let y_end = max_y.min(h - 1);

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Edge function for barycentric
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, px, py);
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, px, py);
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, px, py);

            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }

            let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
            if area.abs() < 1e-6 {
                continue;
            }

            let w0 = w0 / area;
            let w1 = w1 / area;
            let w2 = w2 / area;

            // Interpolate depth and alpha
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let alpha = w0 * v0.alpha + w1 * v1.alpha + w2 * v2.alpha;

            // Skip pixels below threshold
            if alpha < 0.02 {
                continue;
            }

            let idx = (y * w + x) as usize;
            if idx >= canvas.len() || idx >= depth_buf.len() {
                continue;
            }

            // Depth test
            if z < depth_buf[idx] {
                // Blend: color × alpha
                let a = alpha.clamp(0.0, 1.0);
                let [r, g, b] = atmo_color;
                let fr = (r as f32 * a) as u8;
                let fg = (g as f32 * a) as u8;
                let fb = (b as f32 * a) as u8;

                canvas[idx] = Some([fr, fg, fb]);
                depth_buf[idx] = z;
            }
        }
    }
}
