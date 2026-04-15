//! Atmosphere shell (corona) rendering.
//!
//! Inflates the planet's own displaced mesh outward by `atmo_margin` world units and renders
//! it as a transparent overlay AFTER the planet surface.  Because we reuse the planet's already-
//! projected vertices (which include terrain displacement), the shell follows the irregular terrain
//! shape exactly — mountains poke through the atmosphere boundary just as they should.
//!
//! At low density only the outer silhouette ring (the corona) is visible.
//! At 100% density the entire disc is covered, hiding the planet surface.

use crate::obj_loader::ObjFace;
use crate::obj_render_helpers::ProjectedVertex;

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
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn edge(px0: f32, py0: f32, px1: f32, py1: f32, px2: f32, py2: f32) -> f32 {
    (px1 - px0) * (py2 - py0) - (py1 - py0) * (px2 - px0)
}

struct ShellVertex {
    x: f32,
    y: f32,
    depth: f32,
    /// Opacity (0 = transparent, 1 = fully opaque atmosphere).
    alpha: f32,
    /// Sun-light brightness multiplier (dark side ≈0.1, bright sunlit limb ≈1.7+).
    brightness: f32,
}

/// Render the atmosphere shell overlay onto `canvas`.
///
/// The shell is derived by pushing every vertex of the already-displaced planet mesh
/// outward along its surface normal by `atmo_margin` world units, then re-projecting.
/// This ensures the shell exactly matches irregular terrain, not a perfect sphere.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_atmo_shell_pass(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    // Planet geometry — already displaced, in world space
    planet_projected: &[Option<ProjectedVertex>],
    planet_faces: &[ObjFace],
    object_translate: [f32; 3],  // planet's world-space origin
    atmo_margin: f32,             // world-space outward offset above the surface
    // Camera / projection
    inv_tan: f32,
    aspect: f32,
    near_clip: f32,
    camera_world: [f32; 3],
    view_right: [f32; 3],
    view_up: [f32; 3],
    view_forward: [f32; 3],
    // Atmosphere appearance
    atmo_color: [u8; 3],
    strength: f32,
    rim_power: f32,
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
) {
    if atmo_margin <= 0.0 || strength <= 0.0 {
        return;
    }

    let w = virtual_w as i32;
    let h = virtual_h as i32;
    if w < 2 || h < 2 {
        return;
    }

    // Build inflated shell vertices by pushing each planet vertex outward.
    let mut shell_verts: Vec<ShellVertex> = Vec::with_capacity(planet_projected.len());

    for pv_opt in planet_projected {
        let Some(pv) = pv_opt else {
            shell_verts.push(ShellVertex {
                x: f32::NAN,
                y: f32::NAN,
                depth: f32::INFINITY,
                alpha: 0.0,
                brightness: 0.0,
            });
            continue;
        };

        // Outward direction from planet centre = normalize(view - object_translate).
        // This works for any mesh shape because view is the displaced world position.
        let rel_center = [
            pv.view[0] - object_translate[0],
            pv.view[1] - object_translate[1],
            pv.view[2] - object_translate[2],
        ];
        let outward = normalize3(rel_center);

        // Push the vertex outward by atmo_margin in world space.
        let shell_world = [
            pv.view[0] + outward[0] * atmo_margin,
            pv.view[1] + outward[1] * atmo_margin,
            pv.view[2] + outward[2] * atmo_margin,
        ];

        // Camera-space transform
        let rel_cam = [
            shell_world[0] - camera_world[0],
            shell_world[1] - camera_world[1],
            shell_world[2] - camera_world[2],
        ];
        let cam_x = dot3(rel_cam, view_right);
        let cam_y = dot3(rel_cam, view_up);
        let view_z = dot3(rel_cam, view_forward);

        if view_z <= near_clip {
            shell_verts.push(ShellVertex {
                x: f32::NAN,
                y: f32::NAN,
                depth: f32::INFINITY,
                alpha: 0.0,
                brightness: 0.0,
            });
            continue;
        }

        // Perspective projection
        let ndc_x = (cam_x / aspect) * inv_tan / view_z;
        let ndc_y = cam_y * inv_tan / view_z;
        let screen_x = (ndc_x + 1.0) * 0.5 * (w as f32 - 1.0);
        let screen_y = (1.0 - (ndc_y + 1.0) * 0.5) * (h as f32 - 1.0);

        // --- Rim (how edge-on is this vertex to the camera) ---
        let nd = dot3(outward, view_dir).abs().clamp(0.0, 1.0);
        let rim = (1.0 - nd).powf(rim_power.max(0.1));

        // --- Day/night ---
        let day = smoothstep(-0.1, 0.3, dot3(outward, sun_dir));

        // --- Alpha ---
        // rim×day gives the classic corona at low density.
        // fill = strength² ramps full-disc coverage so at 100% the planet is hidden.
        let fill = strength * strength;
        let alpha = (rim * (0.55 + 0.90 * day) + fill).clamp(0.0, 1.0) * strength;

        // --- Brightness (sun lighting on the atmosphere color) ---
        // Dark side = dim (0.10 base), sunlit limb peaks > 1 (forward scatter glow).
        let sun_limb = rim * day;
        let brightness = (0.10 + 0.90 * day + 0.70 * sun_limb).max(0.0);

        shell_verts.push(ShellVertex {
            x: screen_x,
            y: screen_y,
            depth: view_z,
            alpha,
            brightness,
        });
    }

    // Sort faces back-to-front (painter's order) using average shell depth.
    let mut face_indices: Vec<usize> = (0..planet_faces.len()).collect();
    face_indices.sort_by(|&a, &b| {
        let fi_a = &planet_faces[a].indices;
        let fi_b = &planet_faces[b].indices;
        let avg_a = (shell_verts[fi_a[0]].depth
            + shell_verts[fi_a[1]].depth
            + shell_verts[fi_a[2]].depth)
            / 3.0;
        let avg_b = (shell_verts[fi_b[0]].depth
            + shell_verts[fi_b[1]].depth
            + shell_verts[fi_b[2]].depth)
            / 3.0;
        avg_b.partial_cmp(&avg_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Rasterize each face
    for &fi in &face_indices {
        let [i0, i1, i2] = planet_faces[fi].indices;
        let v0 = &shell_verts[i0];
        let v1 = &shell_verts[i1];
        let v2 = &shell_verts[i2];

        if !v0.x.is_finite() || !v1.x.is_finite() || !v2.x.is_finite() {
            continue;
        }

        // Backface cull
        if edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
            continue;
        }

        rasterize_shell_triangle(canvas, w, h, v0, v1, v2, atmo_color);
    }
}

/// Blend the lit atmosphere color over the existing canvas pixel using alpha compositing:
///   out = alpha × (atmo_color × brightness) + (1−alpha) × existing
#[inline(always)]
fn rasterize_shell_triangle(
    canvas: &mut [Option<[u8; 3]>],
    w: i32,
    h: i32,
    v0: &ShellVertex,
    v1: &ShellVertex,
    v2: &ShellVertex,
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

            let bw0 = w0 / area;
            let bw1 = w1 / area;
            let bw2 = w2 / area;

            let alpha = (bw0 * v0.alpha + bw1 * v1.alpha + bw2 * v2.alpha).clamp(0.0, 1.0);
            if alpha < 0.02 {
                continue;
            }

            let idx = (y * w + x) as usize;
            if idx >= canvas.len() {
                continue;
            }

            // Interpolate sun-lighting brightness and apply to the atmosphere color.
            let br = (bw0 * v0.brightness + bw1 * v1.brightness + bw2 * v2.brightness).max(0.0);
            let lit_r = (atmo_color[0] as f32 * br).clamp(0.0, 255.0);
            let lit_g = (atmo_color[1] as f32 * br).clamp(0.0, 255.0);
            let lit_b = (atmo_color[2] as f32 * br).clamp(0.0, 255.0);

            // Alpha-over compositing: lit atmosphere over existing pixel.
            let [er, eg, eb] = canvas[idx].unwrap_or([0, 0, 0]);
            let oma = 1.0 - alpha;
            canvas[idx] = Some([
                (lit_r * alpha + er as f32 * oma) as u8,
                (lit_g * alpha + eg as f32 * oma) as u8,
                (lit_b * alpha + eb as f32 * oma) as u8,
            ]);
        }
    }
}
