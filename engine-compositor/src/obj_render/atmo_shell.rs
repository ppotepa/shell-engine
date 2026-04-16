//! Atmosphere shell rendering — physical model (Steps 1–3).
//!
//! ## How it works
//!
//! The shell is a smooth sphere placed at `planet_radius + atmo_margin` from the planet
//! centre.  Each planet vertex contributes one shell vertex, placed along the vertex's
//! **smooth surface normal** (rotated with the model matrix) at a fixed shell radius.
//! Using smooth normals instead of displaced terrain positions gives a clean spherical
//! envelope and prevents the "two separate rotating objects" visual artifact that
//! occurs when a heavily-displaced terrain-hugging shell is used.
//!
//! When `scale_height > 0`, the atmosphere density is computed physically:
//!
//! **Step 1 — Barometric formula:**
//!   `ρ = exp(-altitude / H)`
//!   where altitude = `atmo_margin` (fixed height of shell above reference sphere),
//!   and H = scale height.
//!
//! **Step 2 — Chapman column density:**
//!   `column = ρ / (cos_θ + cos_θ_min)`
//!   where `cos_θ_min = sqrt(H / 2πR)` is the critical grazing angle.
//!   At the limb (cos_θ → 0) the ray path through atmosphere diverges → bright corona.
//!   At zenith (cos_θ = 1) the path is short → dim centre.
//!
//! **Step 3 — Gravitational potential (irregular bodies):**
//!   `Φ(x) = Σ_i (-1 / |x − xᵢ|) / N_samples`
//!   A subsampled sum over ≤128 planet vertices replaces the spherical altitude estimate
//!   with a potential-derived altitude.  For spherical planets the result is identical to
//!   Step 1; for lumpy asteroids the atmosphere follows the gravity wells.
//!
//! When `scale_height == 0`, falls back to the empirical rim-power model.

use crate::obj_loader::ObjFace;
use crate::obj_render_helpers::ProjectedVertex;
use std::f32::consts::TAU;

/// Maximum planet vertices used for gravity potential (Step 3).
/// Keeps the O(N×M) inner product bounded at ~256k ops regardless of mesh resolution.
const MAX_GRAVITY_SAMPLES: usize = 128;

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
    alpha: f32,
    brightness: f32,
}

/// Render the atmosphere shell overlay onto `canvas`.
///
/// Pushes every displaced planet mesh vertex outward by `atmo_margin` and composites
/// the result over the already-rendered planet surface.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_atmo_shell_pass(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    planet_projected: &[Option<ProjectedVertex>],
    planet_faces: &[ObjFace],
    object_translate: [f32; 3],
    atmo_margin: f32,
    inv_tan: f32,
    aspect: f32,
    near_clip: f32,
    camera_world: [f32; 3],
    view_right: [f32; 3],
    view_up: [f32; 3],
    view_forward: [f32; 3],
    atmo_color: [u8; 3],
    strength: f32,
    rim_power: f32,
    planet_radius: f32,
    scale_height: f32,
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

    let use_physical = scale_height > 0.0 && planet_radius > 0.0;

    // ── Step 3 prep: build subsampled mass-point list ────────────────────────
    // Collect up to MAX_GRAVITY_SAMPLES planet vertex world-positions for Φ sum.
    // Only used when physical model is active.
    let gravity_mass_pts: Vec<[f32; 3]> = if use_physical {
        let stride = (planet_projected.len() / MAX_GRAVITY_SAMPLES).max(1);
        planet_projected
            .iter()
            .step_by(stride)
            .filter_map(|p| p.as_ref())
            .map(|pv| pv.view)
            .collect()
    } else {
        Vec::new()
    };

    // Precompute reference potential Φ_ref = Φ at a point on the reference sphere.
    // For a sphere: Φ_ref = −1/R.  We use the analytical value; for irregular
    // bodies this introduces a small bias that's acceptable at game quality.
    let phi_ref = if use_physical {
        -1.0 / planet_radius.max(1e-4)
    } else {
        0.0
    };

    // ── Pass 1: project inflated shell vertices & compute per-vertex attributes ──
    let mut shell_verts: Vec<ShellVertex> = Vec::with_capacity(planet_projected.len());

    for pv_opt in planet_projected {
        let Some(pv) = pv_opt else {
            shell_verts.push(ShellVertex {
                x: f32::NAN, y: f32::NAN, depth: f32::INFINITY,
                alpha: 0.0, brightness: 0.0,
            });
            continue;
        };

        // Use the smooth vertex normal as the outward direction.
        // This gives a clean spherical shell that follows the sphere topology rather
        // than the terrain displacement, preventing the "two objects" visual artifact
        // that occurs when a heavily-displaced terrain-hugging shell appears detached.
        // pv.normal is already normalized and rotated with the planet model matrix.
        let outward = pv.normal;

        // Place the shell at a fixed radius from the planet centre — just outside the
        // tallest terrain feature (planet_radius ≈ params.scale = the world-space max
        // vertex radius, so shell_radius sits just above the highest mountain).
        let shell_radius = planet_radius + atmo_margin;
        let shell_world = [
            object_translate[0] + outward[0] * shell_radius,
            object_translate[1] + outward[1] * shell_radius,
            object_translate[2] + outward[2] * shell_radius,
        ];

        // Camera-space transform.
        let rel_cam = [
            shell_world[0] - camera_world[0],
            shell_world[1] - camera_world[1],
            shell_world[2] - camera_world[2],
        ];
        let cam_x  = dot3(rel_cam, view_right);
        let cam_y  = dot3(rel_cam, view_up);
        let view_z = dot3(rel_cam, view_forward);

        if view_z <= near_clip {
            shell_verts.push(ShellVertex {
                x: f32::NAN, y: f32::NAN, depth: f32::INFINITY,
                alpha: 0.0, brightness: 0.0,
            });
            continue;
        }

        let ndc_x = (cam_x / aspect) * inv_tan / view_z;
        let ndc_y = cam_y * inv_tan / view_z;
        let screen_x = (ndc_x + 1.0) * 0.5 * (w as f32 - 1.0);
        let screen_y = (1.0 - (ndc_y + 1.0) * 0.5) * (h as f32 - 1.0);

        // cos(zenith angle) — 1 = normal faces camera (centre of disc), 0 = limb.
        let nd = dot3(outward, view_dir).abs().clamp(0.0, 1.0);
        // Day/night from sun direction.
        let day = smoothstep(-0.1, 0.3, dot3(outward, sun_dir));

        let (alpha, brightness) = if use_physical {
            // ── Step 1: barometric density ────────────────────────────────────
            // Shell is placed at a fixed radius (planet_radius + atmo_margin), so
            // the geometric altitude above the reference sphere is simply atmo_margin.
            let geom_altitude = atmo_margin;

            // ── Step 3: gravitational potential correction ────────────────────
            // Replace pure altitude with potential-derived altitude so atmosphere
            // follows gravity wells on irregular bodies.
            let altitude = if !gravity_mass_pts.is_empty() {
                let n = gravity_mass_pts.len() as f32;
                let phi = gravity_mass_pts.iter().fold(0.0f32, |acc, &mp| {
                    let dx = shell_world[0] - mp[0];
                    let dy = shell_world[1] - mp[1];
                    let dz = shell_world[2] - mp[2];
                    let dist = (dx*dx + dy*dy + dz*dz).sqrt().max(0.01);
                    acc - 1.0 / dist
                }) / n;
                // Convert potential difference to altitude units:
                // Δalt ≈ (Φ - Φ_ref) × R²  (first-order Taylor for a sphere)
                let delta = ((phi - phi_ref) * planet_radius * planet_radius).max(0.0);
                // Blend: for near-spherical planets delta ≈ geom_altitude.
                // For irregular bodies delta diverges from geom_altitude usefully.
                delta.max(geom_altitude * 0.5)
            } else {
                geom_altitude
            };

            // ── Step 1 (continued): barometric density ────────────────────────
            let density = (-altitude / scale_height).exp();

            // ── Step 2: Chapman column density ───────────────────────────────
            // cos_θ_min = sqrt(H / 2πR) — critical grazing angle.
            // At the limb (nd → 0) column → density/cos_θ_min → very large → alpha ≈ 1.
            // At zenith  (nd = 1)  column = density/(1+cos_θ_min) ≈ density → dim centre.
            let cos_theta_min = (scale_height / (TAU * planet_radius.max(1e-4))).sqrt();
            let column = density / (nd + cos_theta_min);

            let alpha = (column * (0.45 + 0.90 * day) * strength).clamp(0.0, 1.0);

            // Brightness: sunlit side bright, dark side very dim.
            // Extra peak at the sunlit limb — forward scatter / terminator glow.
            let rim = 1.0 - nd;
            let sun_limb = rim * day;
            let brightness = (0.10 + 0.90 * day + 0.70 * sun_limb).max(0.0);

            (alpha, brightness)
        } else {
            // ── Empirical fallback (rim_power model) ─────────────────────────
            let rim  = (1.0 - nd).powf(rim_power.max(0.1));
            let fill = strength * strength;
            let alpha = (rim * (0.55 + 0.90 * day) + fill).clamp(0.0, 1.0) * strength;
            let sun_limb = rim * day;
            let brightness = (0.10 + 0.90 * day + 0.70 * sun_limb).max(0.0);
            (alpha, brightness)
        };

        shell_verts.push(ShellVertex { x: screen_x, y: screen_y, depth: view_z, alpha, brightness });
    }

    // ── Sort faces back-to-front (painter's order) ──────────────────────────
    let mut face_indices: Vec<usize> = (0..planet_faces.len()).collect();
    face_indices.sort_by(|&a, &b| {
        let ia = &planet_faces[a].indices;
        let ib = &planet_faces[b].indices;
        let avg_a = (shell_verts[ia[0]].depth + shell_verts[ia[1]].depth + shell_verts[ia[2]].depth) / 3.0;
        let avg_b = (shell_verts[ib[0]].depth + shell_verts[ib[1]].depth + shell_verts[ib[2]].depth) / 3.0;
        avg_b.partial_cmp(&avg_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // ── Rasterize ───────────────────────────────────────────────────────────
    for &fi in &face_indices {
        let [i0, i1, i2] = planet_faces[fi].indices;
        let v0 = &shell_verts[i0];
        let v1 = &shell_verts[i1];
        let v2 = &shell_verts[i2];
        if !v0.x.is_finite() || !v1.x.is_finite() || !v2.x.is_finite() { continue; }
        if edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 { continue; }
        rasterize_shell_triangle(canvas, w, h, v0, v1, v2, atmo_color);
    }
}

/// Alpha-over composite: `out = α × (atmo_color × brightness) + (1−α) × existing`
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
    let x_end   = max_x.min(w - 1);
    let y_start = min_y.max(0);
    let y_end   = max_y.min(h - 1);

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = edge(v1.x, v1.y, v2.x, v2.y, px, py);
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, px, py);
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, px, py);
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 { continue; }

            let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
            if area.abs() < 1e-6 { continue; }

            let bw0 = w0 / area;
            let bw1 = w1 / area;
            let bw2 = w2 / area;

            let alpha = (bw0 * v0.alpha + bw1 * v1.alpha + bw2 * v2.alpha).clamp(0.0, 1.0);
            if alpha < 0.02 { continue; }

            let idx = (y * w + x) as usize;
            if idx >= canvas.len() { continue; }

            let br = (bw0 * v0.brightness + bw1 * v1.brightness + bw2 * v2.brightness).max(0.0);
            let lit_r = (atmo_color[0] as f32 * br).clamp(0.0, 255.0);
            let lit_g = (atmo_color[1] as f32 * br).clamp(0.0, 255.0);
            let lit_b = (atmo_color[2] as f32 * br).clamp(0.0, 255.0);

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
