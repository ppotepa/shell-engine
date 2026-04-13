use engine_core::buffer::Buffer;
use engine_core::color::Color;

use super::obj_loader::ObjFace;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectedVertex {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) depth: f32,
    pub(crate) view: [f32; 3],
    /// Rotated smooth vertex normal in world space. Used for Gouraud per-vertex shading.
    pub(crate) normal: [f32; 3],
    /// Pre-rotation local-space position (centered + scaled, before any yaw/pitch/roll).
    pub(crate) local: [f32; 3],
    /// Pre-computed fBm terrain noise at this vertex's local position.
    /// Barycentrically interpolated per pixel — eliminates per-pixel noise evaluation.
    /// Only meaningful when `terrain_color` is set; otherwise 0.0.
    pub(crate) terrain_noise: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Viewport {
    pub(crate) min_x: i32,
    pub(crate) min_y: i32,
    pub(crate) max_x: i32,
    pub(crate) max_y: i32,
}

#[inline]
pub fn virtual_dimensions(target_w: u16, target_h: u16) -> (u16, u16) {
    (target_w, target_h)
}

/// Virtual-to-frame multiplier per axis.
#[inline]
pub fn virtual_dimensions_multiplier() -> (u16, u16) {
    (1, 1)
}

/// Interpolate depths at clipped line endpoints using parametric projection.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn clipped_depths(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    cx0: i32,
    cy0: i32,
    cx1: i32,
    cy1: i32,
    z0: f32,
    z1: f32,
) -> (f32, f32) {
    let ldx = (x1 - x0) as f32;
    let ldy = (y1 - y0) as f32;
    let len_sq = ldx * ldx + ldy * ldy;
    if len_sq < 1.0 {
        return (z0, z1);
    }
    let t0 = ((cx0 - x0) as f32 * ldx + (cy0 - y0) as f32 * ldy) / len_sq;
    let t1 = ((cx1 - x0) as f32 * ldx + (cy1 - y0) as f32 * ldy) / len_sq;
    (
        z0 + (z1 - z0) * t0.clamp(0.0, 1.0),
        z0 + (z1 - z0) * t1.clamp(0.0, 1.0),
    )
}

/// Simple Bresenham line — flat color, no depth test (fallback for face-less models).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line_flat(
    canvas: &mut [Option<[u8; 3]>],
    w: u16,
    h: u16,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: [u8; 3],
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u16) < w && (y0 as u16) < h {
            let idx = y0 as usize * w as usize + x0 as usize;
            if let Some(px) = canvas.get_mut(idx) {
                *px = Some(color);
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Bresenham line with z-buffer and depth-based brightness falloff.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line_depth(
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
    w: u16,
    h: u16,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    base_color: [u8; 3],
    z0: f32,
    z1: f32,
    depth_near: f32,
    depth_far: f32,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let total_steps = dx.max(-dy) as f32;
    let depth_range = depth_far - depth_near;
    let mut step = 0f32;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u16) < w && (y0 as u16) < h {
            let idx = y0 as usize * w as usize + x0 as usize;
            let t = if total_steps > 0.0 {
                step / total_steps
            } else {
                0.0
            };
            let z = z0 + (z1 - z0) * t;
            if z < depth_buf[idx] {
                depth_buf[idx] = z;
                let norm = if depth_range > f32::EPSILON {
                    ((z - depth_near) / depth_range).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                // Brightness: 1.0 at nearest, fades to 0.15 at farthest.
                let brightness = 1.0 - 0.85 * norm;
                let r = (base_color[0] as f32 * brightness) as u8;
                let g = (base_color[1] as f32 * brightness) as u8;
                let b = (base_color[2] as f32 * brightness) as u8;
                canvas[idx] = Some([r, g, b]);
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        step += 1.0;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rasterize_triangle(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    w: u16,
    h: u16,
    v0: ProjectedVertex,
    v1: ProjectedVertex,
    v2: ProjectedVertex,
    color: [u8; 3],
    clip_min_y: i32,
    clip_max_y: i32,
) {
    let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
    if area.abs() < 1e-5 {
        return;
    }
    let inv_area = 1.0 / area;

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;
    let min_y = min_y.max(clip_min_y);
    let max_y = max_y.min(clip_max_y);

    // Bounding box culling: skip if triangle is completely off-screen.
    if min_x > max_x || min_y > max_y {
        return;
    }
    for py in min_y..=max_y {
        let y = py as f32 + 0.5;
        let row_start = py as usize * w as usize;
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) * inv_area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) * inv_area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) * inv_area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = row_start + px as usize;
            if z < depth[idx] {
                depth[idx] = z;
                canvas[idx] = Some(color);
            }
        }
    }
}

/// Deterministic hash for 3-D integer lattice coordinates, mapped to [0.0, 1.0).
#[inline(always)]
fn terrain_hash(xi: i32, yi: i32, zi: i32) -> f32 {
    let mut h = xi
        .wrapping_mul(1619)
        .wrapping_add(yi.wrapping_mul(31337))
        .wrapping_add(zi.wrapping_mul(6271));
    h ^= h >> 13;
    h = h.wrapping_mul(1_000_000_007);
    h ^= h >> 17;
    (h as u32 as f32) * (1.0 / u32::MAX as f32)
}

/// Smooth 3-D value noise (trilinear, smoothstep filtered). Output range: [0.0, 1.0].
fn value_noise_3d(px: f32, py: f32, pz: f32) -> f32 {
    let xi = px.floor() as i32;
    let xf = px - xi as f32;
    let yi = py.floor() as i32;
    let yf = py - yi as f32;
    let zi = pz.floor() as i32;
    let zf = pz - zi as f32;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);
    let c000 = terrain_hash(xi, yi, zi);
    let c100 = terrain_hash(xi + 1, yi, zi);
    let c010 = terrain_hash(xi, yi + 1, zi);
    let c110 = terrain_hash(xi + 1, yi + 1, zi);
    let c001 = terrain_hash(xi, yi, zi + 1);
    let c101 = terrain_hash(xi + 1, yi, zi + 1);
    let c011 = terrain_hash(xi, yi + 1, zi + 1);
    let c111 = terrain_hash(xi + 1, yi + 1, zi + 1);
    let x0 = c000 + u * (c100 - c000);
    let x1 = c010 + u * (c110 - c010);
    let x2 = c001 + u * (c101 - c001);
    let x3 = c011 + u * (c111 - c011);
    let y0 = x0 + v * (x1 - x0);
    let y1 = x2 + v * (x3 - x2);
    y0 + w * (y1 - y0)
}

/// Voronoi (cellular) noise: returns distance to nearest cell center in [0, ~0.9].
/// Used for procedural craters on rocky surfaces.
fn voronoi_3d(px: f32, py: f32, pz: f32) -> f32 {
    let xi = px.floor() as i32;
    let yi = py.floor() as i32;
    let zi = pz.floor() as i32;
    let mut min_dist = f32::MAX;
    for dz in -1i32..=1 {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let cx = xi + dx;
                let cy = yi + dy;
                let cz = zi + dz;
                let fx = terrain_hash(cx.wrapping_mul(3), cy.wrapping_mul(5), cz.wrapping_mul(7));
                let fy = terrain_hash(cx.wrapping_mul(7), cy.wrapping_mul(3), cz.wrapping_mul(11));
                let fz = terrain_hash(cx.wrapping_mul(11), cy.wrapping_mul(7), cz.wrapping_mul(3));
                let cell_x = cx as f32 + fx;
                let cell_y = cy as f32 + fy;
                let cell_z = cz as f32 + fz;
                let ddx = px - cell_x;
                let ddy = py - cell_y;
                let ddz = pz - cell_z;
                let d = (ddx * ddx + ddy * ddy + ddz * ddz).sqrt();
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
    }
    min_dist.min(1.0)
}

/// Smoothstep cubic interpolation: maps [edge0, edge1] → [0.0, 1.0].
#[inline(always)]
fn ss(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Optional planet biome and atmosphere parameters for the Gouraud rasterizer.
/// All effects are opt-in: None fields or zero values disable the effect.
#[derive(Clone, Copy)]
pub(crate) struct PlanetBiomeParams {
    pub polar_ice_color: Option<[u8; 3]>,
    pub polar_ice_start: f32,
    pub polar_ice_end: f32,
    pub desert_color: Option<[u8; 3]>,
    pub desert_strength: f32,
    pub atmo_color: Option<[u8; 3]>,
    pub atmo_strength: f32,
    pub atmo_rim_power: f32,
    pub atmo_haze_strength: f32,
    pub atmo_haze_power: f32,
    pub night_light_color: Option<[u8; 3]>,
    pub night_light_threshold: f32,
    pub night_light_intensity: f32,
    /// Normalized sun direction in world space (from `light_dir_norm`).
    pub sun_dir: [f32; 3],
    /// Normalized camera direction in world space (= `-view_forward`, toward the camera).
    pub view_dir: [f32; 3],
}

/// Extra per-pixel terrain rendering parameters for the Gouraud rasterizer.
/// Groups new procedural features to avoid a very long function signature.
#[derive(Clone, Copy)]
pub(crate) struct PlanetTerrainParams {
    /// terrain_noise_scale - needed for gradient computation in normal perturbation.
    pub noise_scale: f32,
    /// Per-pixel normal perturbation strength. Fakes bumps responding to light.
    pub normal_perturb: f32,
    /// Ocean specular highlight (Phong) strength.
    pub ocean_specular: f32,
    /// Crater density scale (0 = disabled, >0 = Voronoi crater overlay).
    pub crater_density: f32,
    /// Crater rim brightness boost.
    pub crater_rim_height: f32,
    /// Snow line altitude (0–1 above terrain_threshold). 0 = disabled.
    pub snow_line: f32,
    /// Scale for ocean surface noise (higher = finer waves). Default 4.0.
    pub ocean_noise_scale: f32,
    /// Ocean base color override. When Some, replaces OBJ face color for below-threshold pixels.
    pub ocean_color_override: Option<[u8; 3]>,
}

/// Variable-octave fBm with configurable lacunarity and persistence.
/// Output range ≈ [0.0, 1.0).
pub(crate) fn fbm_3d_full(x: f32, y: f32, z: f32, octaves: u8, lacunarity: f32, persistence: f32) -> f32 {
    let n = octaves.clamp(1, 8) as usize;
    let mut val = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..n {
        val += value_noise_3d(x * freq, y * freq, z * freq) * amp;
        max_val += amp;
        amp *= persistence;
        freq *= lacunarity;
    }
    val / max_val.max(1e-6)
}

/// Legacy fixed-parameter wrapper: lacunarity=2.0, persistence=0.5.
pub(crate) fn fbm_3d_octaves(x: f32, y: f32, z: f32, octaves: u8) -> f32 {
    fbm_3d_full(x, y, z, octaves, 2.0, 0.5)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rasterize_triangle_gouraud(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    w: u16,
    h: u16,
    v0: ProjectedVertex,
    v1: ProjectedVertex,
    v2: ProjectedVertex,
    base_color: [u8; 3],
    shade0: f32,
    shade1: f32,
    shade2: f32,
    shadow_colour: Option<Color>,
    midtone_colour: Option<Color>,
    highlight_colour: Option<Color>,
    tone_mix: f32,
    cel_levels: u8,
    latitude_bands: u8,
    latitude_band_depth: f32,
    terrain_color: Option<[u8; 3]>,
    terrain_threshold: f32,
    marble_depth: f32,
    terrain_relief: f32,
    below_threshold_transparent: bool,
    biome: Option<PlanetBiomeParams>,
    terrain_extra: Option<PlanetTerrainParams>,
    clip_min_y: i32,
    clip_max_y: i32,
    // First global row at index 0 of `canvas`/`depth`. Set to strip's first row for parallel strip rendering.
    row_base: i32,
) {
    let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
    if area.abs() < 1e-5 {
        return;
    }
    let inv_area = 1.0 / area;

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;
    let min_y = min_y.max(clip_min_y);
    let max_y = max_y.min(clip_max_y);

    if min_x > max_x || min_y > max_y {
        return;
    }

    let use_bands = latitude_bands > 0 && latitude_band_depth > f32::EPSILON;

    for py in min_y..=max_y {
        let y = py as f32 + 0.5;
        let row_start = (py - row_base) as usize * w as usize;
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) * inv_area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) * inv_area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) * inv_area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = row_start + px as usize;
            if z < depth[idx] {
                depth[idx] = z;
                // Gouraud: barycentrically interpolate pre-computed per-vertex shade.
                let shade = (w0 * shade0 + w1 * shade1 + w2 * shade2).clamp(0.0, 1.0);
                // Latitude band modulation: sine wave along world-space Y.
                let shade = if use_bands {
                    let view_y = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                    let band = (view_y * latitude_bands as f32 * std::f32::consts::PI).sin();
                    (shade + band * latitude_band_depth * 0.5).clamp(0.0, 1.0)
                } else {
                    shade
                };

                let mut pixel = if let Some(tc) = terrain_color {
                    // Terrain noise was pre-computed per vertex and is barycentrically interpolated —
                    // no fbm call per pixel; just 3 multiplies + threshold compare.
                    let noise =
                        w0 * v0.terrain_noise + w1 * v1.terrain_noise + w2 * v2.terrain_noise;
                    if noise > terrain_threshold {
                        // ── LAND pixel ─────────────────────────────────────────────
                        // Elevation relief: brighten highlands, darken valleys.
                        // Normalise noise above the threshold to [0, 1] and shift shade.
                        let shade = if terrain_relief > 0.0 {
                            let elev = (noise - terrain_threshold)
                                / (1.0 - terrain_threshold).max(0.01);
                            (shade + (elev - 0.5) * terrain_relief).clamp(0.0, 1.0)
                        } else {
                            shade
                        };
                        // Per-pixel normal perturbation: finite-difference gradient of noise perturbs shade.
                        let shade = if let Some(te) = terrain_extra {
                            if te.normal_perturb > 0.0 && te.noise_scale > 0.0 {
                                let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                                let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                                let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                                let eps = 0.04 / te.noise_scale.max(0.1);
                                let s = te.noise_scale;
                                let n0 = value_noise_3d(lx * s, ly * s, lz * s);
                                let nx_ = value_noise_3d((lx + eps) * s, ly * s, lz * s) - n0;
                                let ny_ = value_noise_3d(lx * s, (ly + eps) * s, lz * s) - n0;
                                let nz_ = value_noise_3d(lx * s, ly * s, (lz + eps) * s) - n0;
                                // Project gradient tangent to sphere (remove radial component).
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                                let (rnx, rny, rnz) = (vx / vlen, vy / vlen, vz / vlen);
                                let rdot = nx_ * rnx + ny_ * rny + nz_ * rnz;
                                let (tx, ty, tz) = (nx_ - rdot * rnx, ny_ - rdot * rny, nz_ - rdot * rnz);
                                // Perturb shade using gradient dot sun.
                                let perturb = if let Some(b) = biome {
                                    let g_sun = tx * b.sun_dir[0] + ty * b.sun_dir[1] + tz * b.sun_dir[2];
                                    g_sun * te.normal_perturb * 1.5
                                } else {
                                    0.0
                                };
                                (shade + perturb).clamp(0.0, 1.0)
                            } else {
                                shade
                            }
                        } else {
                            shade
                        };
                        let mut land_color = tc;
                        // Snow line: high-altitude land turns snowy above snow_line_altitude.
                        if let Some(te) = terrain_extra {
                            if te.snow_line > 0.0 {
                                let elev = (noise - terrain_threshold)
                                    / (1.0 - terrain_threshold).max(0.01);
                                if elev > te.snow_line {
                                    let snow_mask =
                                        ss(te.snow_line, (te.snow_line + 0.2).min(1.0), elev);
                                    land_color = mix_rgb(land_color, [240, 248, 255], snow_mask);
                                }
                            }
                        }
                        if let Some(b) = biome {
                            // Surface normal: normalize interpolated world-space position.
                            // For a sphere, normalize(view) == surface normal in world space.
                            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                            let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                            let (nx, ny, nz) = (vx / vlen, vy / vlen, vz / vlen);
                            let lat_abs = ny.abs();

                            // Desert biome: equatorial dry zone
                            if let Some(dc) = b.desert_color {
                                if b.desert_strength > 0.0 {
                                    let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                                    let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                                    let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                                    // Weight peaks near ±18° lat (lat_abs ≈ 0.31), fades toward poles/equator.
                                    let eq_w =
                                        (1.0 - ((lat_abs - 0.28) * 3.5).abs()).clamp(0.0, 1.0);
                                    let d_noise = value_noise_3d(lx * 7.0, ly * 7.0, lz * 7.0);
                                    let d_mask = ss(0.62, 0.82, d_noise) * eq_w * b.desert_strength;
                                    if d_mask > 0.005 {
                                        land_color = mix_rgb(land_color, dc, d_mask);
                                    }
                                }
                            }
                            // Polar ice (overrides desert)
                            if let Some(ice_c) = b.polar_ice_color {
                                let elev_boost = (noise - terrain_threshold) * 0.15;
                                let ice_mask =
                                    ss(b.polar_ice_start, b.polar_ice_end, lat_abs + elev_boost);
                                if ice_mask > 0.005 {
                                    land_color = mix_rgb(land_color, ice_c, ice_mask);
                                }
                            }

                            let cel = quantize_shade(shade, cel_levels);
                            let mut px_color = apply_shading(land_color, cel);

                            // Night-side city lights (land, dark side only)
                            if let Some(city_c) = b.night_light_color {
                                if b.night_light_intensity > 0.0 {
                                    let sun_dot =
                                        nx * b.sun_dir[0] + ny * b.sun_dir[1] + nz * b.sun_dir[2];
                                    let night_f = ss(0.10, -0.08, sun_dot); // 1.0 on dark side
                                    if night_f > 0.01 {
                                        let lx =
                                            w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                                        let ly =
                                            w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                                        let lz =
                                            w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                                        let city_n =
                                            value_noise_3d(lx * 18.0, ly * 18.0, lz * 18.0);
                                        let city_m = ss(b.night_light_threshold, 1.0, city_n)
                                            * night_f
                                            * b.night_light_intensity;
                                        if city_m > 0.01 {
                                            px_color =
                                                mix_rgb(px_color, city_c, city_m.clamp(0.0, 0.95));
                                        }
                                    }
                                }
                            }
                            px_color
                        } else {
                            let cel = quantize_shade(shade, cel_levels);
                            apply_shading(land_color, cel)
                        }
                    } else {
                        // ── OCEAN / below-threshold pixel ───────────────────────────
                        if below_threshold_transparent {
                            continue;
                        }
                        // Polar ice on ocean (slightly tighter threshold than on land)
                        if let Some(b) = biome {
                            if let Some(ice_c) = b.polar_ice_color {
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                                let lat_abs = (vy / vlen).abs();
                                let ice_mask =
                                    ss(b.polar_ice_start + 0.05, b.polar_ice_end, lat_abs);
                                if ice_mask > 0.005 {
                                    let cel = quantize_shade(shade, cel_levels);
                                    let px_color = apply_shading(ice_c, cel);
                                    canvas[idx] = Some(px_color);
                                    continue;
                                }
                            }
                        }
                        // Ocean: cheap single-octave marble per pixel (8 hash calls).
                        let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                        let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                        let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                        let ocean_ns = terrain_extra.map(|te| te.ocean_noise_scale).unwrap_or(4.0);
                        let mn = value_noise_3d(lx * ocean_ns, ly * ocean_ns, lz * ocean_ns);
                        let ocean_base = terrain_extra.and_then(|te| te.ocean_color_override).unwrap_or(base_color);
                        let os = (shade + (mn - 0.5) * marble_depth).clamp(0.0, 1.0);
                        // Ocean specular highlight (sunglint).
                        let os = if let (Some(b), Some(te)) = (biome, terrain_extra) {
                            if te.ocean_specular > 0.0 {
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                                let (nx, ny, nz) = (vx / vlen, vy / vlen, vz / vlen);
                                let sun_dot = nx * b.sun_dir[0] + ny * b.sun_dir[1] + nz * b.sun_dir[2];
                                let rx = 2.0 * sun_dot * nx - b.sun_dir[0];
                                let ry = 2.0 * sun_dot * ny - b.sun_dir[1];
                                let rz = 2.0 * sun_dot * nz - b.sun_dir[2];
                                let spec_dot = (rx * b.view_dir[0] + ry * b.view_dir[1] + rz * b.view_dir[2]).max(0.0);
                                let spec = spec_dot.powf(32.0) * te.ocean_specular * sun_dot.max(0.0);
                                (os + spec).clamp(0.0, 1.0)
                            } else {
                                os
                            }
                        } else {
                            os
                        };
                        let cel = quantize_shade(os, cel_levels);
                        let sb = apply_shading(ocean_base, cel);
                        apply_tone_palette(
                            sb,
                            cel,
                            shadow_colour,
                            midtone_colour,
                            highlight_colour,
                            tone_mix,
                        )
                    }
                } else {
                    let cel_shade = quantize_shade(shade, cel_levels);
                    let shaded_base = apply_shading(base_color, cel_shade);
                    apply_tone_palette(
                        shaded_base,
                        cel_shade,
                        shadow_colour,
                        midtone_colour,
                        highlight_colour,
                        tone_mix,
                    )
                };

                // Crater overlay (Voronoi-based depressions for rocky/moon surfaces).
                if let Some(te) = terrain_extra {
                    if te.crater_density > 0.0 {
                        let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                        let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                        let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                        let cd = te.crater_density;
                        let v_dist = voronoi_3d(lx * cd, ly * cd, lz * cd);
                        let rim_w = ss(0.25, 0.42, v_dist) * (1.0 - ss(0.42, 0.58, v_dist));
                        let bowl_w = (1.0 - ss(0.0, 0.28, v_dist)) * 0.55;
                        if rim_w > 0.01 || bowl_w > 0.01 {
                            let (pr, pg, pb) = (pixel[0] as f32, pixel[1] as f32, pixel[2] as f32);
                            let rim_boost = rim_w * te.crater_rim_height * 80.0;
                            let bowl_dark = bowl_w * 60.0;
                            pixel = [
                                (pr + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
                                (pg + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
                                (pb + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
                            ];
                        }
                    }
                }

                // Atmosphere overlay: keep the existing thin rim, but add a broader low-opacity
                // haze term so planets read as volumetric instead of only edge-tinted.
                if let Some(b) = biome {
                    if let Some(ac) = b.atmo_color {
                        if b.atmo_strength > 0.0 || b.atmo_haze_strength > 0.0 {
                            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                            let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                            let (nx, ny, nz) = (vx / vlen, vy / vlen, vz / vlen);
                            let nd = (nx * b.view_dir[0] + ny * b.view_dir[1] + nz * b.view_dir[2])
                                .abs()
                                .clamp(0.0, 1.0);
                            let rim = (1.0 - nd).powf(b.atmo_rim_power);
                            let haze = (1.0 - nd).powf(b.atmo_haze_power.max(0.1));
                            if rim > 0.01 || haze > 0.01 {
                                let sun_dot =
                                    nx * b.sun_dir[0] + ny * b.sun_dir[1] + nz * b.sun_dir[2];
                                let day = ss(-0.05, 0.25, sun_dot);
                                let rim_alpha = rim * (0.20 + 0.80 * day) * b.atmo_strength;
                                let haze_alpha = haze * (0.05 + 0.30 * day) * b.atmo_haze_strength;
                                let ab = (rim_alpha + haze_alpha).clamp(0.0, 0.85);
                                pixel = mix_rgb(pixel, ac, ab);
                            }
                        }
                    }
                }

                canvas[idx] = Some(pixel);
            }
        }
    }
}

#[inline]
pub(crate) fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

#[inline(always)]
pub(crate) fn face_avg_depth(projected: &[Option<ProjectedVertex>], face: &ObjFace) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for &i in &face.indices {
        if let Some(Some(v)) = projected.get(i) {
            sum += v.depth;
            count += 1;
        }
    }
    if count == 0 {
        f32::INFINITY
    } else {
        sum / count as f32
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn face_shading_with_specular(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    ka: [f32; 3],
    ks: f32,
    ns: f32,
    light_dir: [f32; 3],
    light_2_dir: [f32; 3],
    half_dir_1: [f32; 3],
    half_dir_2: [f32; 3],
    light_2_intensity: f32,
    light_point: [f32; 3],
    light_point_intensity: f32,
    light_point_2: [f32; 3],
    light_point_2_intensity: f32,
    cel_levels: u8,
    tone_mix: f32,
    ambient: f32,
    view_dir: [f32; 3],
    point_falloff: f32,
    point_2_falloff: f32,
) -> (f32, f32, f32, f32) {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let normal = normalize3(cross3(e1, e2));
    // light_dir and light_2_dir arrive pre-normalized from the caller.
    let light_2_strength = light_2_intensity.clamp(0.0, 2.0);
    let point_strength = light_point_intensity.clamp(0.0, 4.0);
    let point_2_strength = light_point_2_intensity.clamp(0.0, 4.0);
    let centroid = [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ];
    let to_point = sub3(light_point, centroid);
    let point_dir = normalize3(to_point);
    let point_dist =
        (to_point[0] * to_point[0] + to_point[1] * to_point[1] + to_point[2] * to_point[2])
            .sqrt()
            .max(0.0001);
    let point_atten = 1.0 / (1.0 + point_falloff * point_dist * point_dist);
    let to_point_2 = sub3(light_point_2, centroid);
    let point_2_dir = normalize3(to_point_2);
    let point_2_dist = (to_point_2[0] * to_point_2[0]
        + to_point_2[1] * to_point_2[1]
        + to_point_2[2] * to_point_2[2])
        .sqrt()
        .max(0.0001);
    let point_2_atten = 1.0 / (1.0 + point_2_falloff * point_2_dist * point_2_dist);
    // One-sided Lambert: dark side stays dark (correct terminator line).
    let lambert_1 = dot3(normal, light_dir).max(0.0);
    let lambert_2 = dot3(normal, light_2_dir).max(0.0) * light_2_strength;
    let lambert_point = dot3(normal, point_dir).max(0.0) * point_strength * point_atten;
    let lambert_point_2 = dot3(normal, point_2_dir).max(0.0) * point_2_strength * point_2_atten;
    let lambert = (lambert_1 + lambert_2 + lambert_point + lambert_point_2).clamp(0.0, 1.0);
    // When tone_mix is high we intentionally reduce material influence so different OBJ
    // material packs still produce consistent silhouette lighting.
    let material_influence = (1.0 - tone_mix.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let ka_lum_material = (ka[0] * 0.299 + ka[1] * 0.587 + ka[2] * 0.114).clamp(0.03, 0.25);
    // Ambient provides a minimum diffuse floor so the dark side is never pitch-black.
    let ka_lum = (0.06 + (ka_lum_material - 0.06) * material_influence).max(ambient);
    // Point light half-vectors use the real per-frame view direction.
    let half_dir_point = normalize3([
        point_dir[0] + view_dir[0],
        point_dir[1] + view_dir[1],
        point_dir[2] + view_dir[2],
    ]);
    let half_dir_point_2 = normalize3([
        point_2_dir[0] + view_dir[0],
        point_2_dir[1] + view_dir[1],
        point_2_dir[2] + view_dir[2],
    ]);
    let shininess = 24.0 + (ns.clamp(2.0, 200.0) - 24.0) * material_influence;
    let spec_1 = dot3(normal, half_dir_1).abs().powf(shininess);
    let spec_2 = dot3(normal, half_dir_2).abs().powf(shininess) * light_2_strength * 0.7;
    let spec_point =
        dot3(normal, half_dir_point).abs().powf(shininess) * point_strength * point_atten * 0.9;
    let spec_point_2 = dot3(normal, half_dir_point_2).abs().powf(shininess)
        * point_2_strength
        * point_2_atten
        * 0.9;
    let ks_strength = 0.08 + (ks.clamp(0.0, 0.6) - 0.08) * material_influence;
    let spec = (spec_1 + spec_2 + spec_point + spec_point_2) * ks_strength;
    let diffuse = ka_lum + (1.0 - ka_lum) * lambert * 0.9;
    let cel_diffuse = quantize_shade(diffuse, cel_levels);
    (
        (cel_diffuse + spec).clamp(0.0, 1.0),
        cel_diffuse,
        lambert_point.clamp(0.0, 1.0),
        lambert_point_2.clamp(0.0, 1.0),
    )
}

pub(crate) fn quantize_shade(value: f32, levels: u8) -> f32 {
    if levels <= 1 {
        return value.clamp(0.0, 1.0);
    }
    let levels = levels.clamp(2, 8) as f32;
    let steps = levels - 1.0;
    let v = value.clamp(0.0, 1.0);
    (v * steps).round() / steps
}

#[inline(always)]
pub(crate) fn apply_shading(rgb: [u8; 3], shade: f32) -> [u8; 3] {
    // Apply shading in linear space then convert back.
    let lin = [
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    ];
    // Boost saturation slightly (1.25) before shading — compensates for terminal display.
    let sat_lin = saturate(lin, 1.25);
    [
        linear_to_srgb((sat_lin[0] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[1] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[2] * shade).clamp(0.0, 1.0)),
    ]
}

#[inline(always)]
pub(crate) fn apply_tone_palette(
    base_rgb: [u8; 3],
    tone: f32,
    shadow: Option<Color>,
    midtone: Option<Color>,
    highlight: Option<Color>,
    tone_mix: f32,
) -> [u8; 3] {
    let mix = tone_mix.clamp(0.0, 1.0);
    if mix <= 0.0 {
        return base_rgb;
    }
    let shadow_rgb = shadow.map(color_to_rgb).unwrap_or([0, 0, 0]);
    let midtone_rgb = midtone
        .map(color_to_rgb)
        .unwrap_or(mix_rgb(shadow_rgb, base_rgb, 0.45));
    let highlight_rgb = highlight.map(color_to_rgb).unwrap_or(base_rgb);
    let t = tone.clamp(0.0, 1.0);
    let toon_rgb = if t <= 0.5 {
        mix_rgb(shadow_rgb, midtone_rgb, t * 2.0)
    } else {
        mix_rgb(midtone_rgb, highlight_rgb, (t - 0.5) * 2.0)
    };
    mix_rgb(base_rgb, toon_rgb, mix)
}

#[inline(always)]
pub(crate) fn apply_point_light_tint(
    base_rgb: [u8; 3],
    light_1_colour: Option<Color>,
    light_1_strength: f32,
    light_2_colour: Option<Color>,
    light_2_strength: f32,
) -> [u8; 3] {
    let mut out = base_rgb;
    if let Some(colour) = light_1_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_1_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    if let Some(colour) = light_2_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_2_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    out
}

#[inline(always)]
pub(crate) fn flicker_multiplier(elapsed_s: f32, hz: f32, depth: f32, phase: f32) -> f32 {
    let d = depth.clamp(0.0, 1.0);
    if d <= f32::EPSILON {
        return 1.0;
    }
    let rate = hz.clamp(0.1, 40.0);
    let base = ((elapsed_s * std::f32::consts::TAU * rate + phase).sin() * 0.5 + 0.5).powf(1.5);
    let chatter = ((elapsed_s * std::f32::consts::TAU * (rate * 2.31) + phase * 1.7)
        .sin()
        .abs())
    .powf(2.3);
    let pulse = (base * 0.65 + chatter * 0.35).clamp(0.0, 1.0);
    ((1.0 - d) + d * pulse).clamp(0.0, 1.0)
}

#[inline(always)]
pub(crate) fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

/// Convert sRGB u8 → linear f32.
#[inline(always)]
pub(crate) fn srgb_to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear f32 → sRGB u8.
#[inline(always)]
pub(crate) fn linear_to_srgb(v: f32) -> u8 {
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Boost saturation of a linear-space RGB triplet by `factor`.
#[inline(always)]
pub(crate) fn saturate(lin: [f32; 3], factor: f32) -> [f32; 3] {
    let lum = lin[0] * 0.299 + lin[1] * 0.587 + lin[2] * 0.114;
    [
        (lum + (lin[0] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[1] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[2] - lum) * factor).clamp(0.0, 1.0),
    ]
}

#[inline(always)]
pub(crate) fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline(always)]
pub(crate) fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline(always)]
pub(crate) fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline(always)]
pub(crate) fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(clippy::too_many_arguments)]
pub fn blit_color_canvas(
    buf: &mut Buffer,
    canvas: &[Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    wireframe: bool,
    draw_char: char,
    _fg: Color,
    bg: Color,
    clip_row_min: usize,
    clip_row_max: usize,
) {
    let px = |vx: u16, vy: u16| -> Option<[u8; 3]> {
        if vx >= virtual_w || vy >= virtual_h {
            return None;
        }
        let vy_usize = vy as usize;
        if vy_usize < clip_row_min || vy_usize >= clip_row_max {
            return None;
        }
        canvas
            .get(vy_usize * virtual_w as usize + vx as usize)
            .copied()
            .unwrap_or(None)
    };

    // ── SDL2 pixel bypass: write virtual pixels directly ─────────────────
    if let Some(pc) = &mut buf.pixel_canvas {
        let pc_w = pc.width as usize;
        let virt_mult = virtual_dimensions_multiplier();
        let base_vx = x as usize * virt_mult.0 as usize;
        let base_vy = y as usize * virt_mult.1 as usize;
        for vy in 0..virtual_h {
            for vx in 0..virtual_w {
                let Some(rgb) = px(vx, vy) else { continue };
                let px_x = base_vx + vx as usize;
                let px_y = base_vy + vy as usize;
                if px_x < pc.width as usize && px_y < pc.height as usize {
                    let idx = (px_y * pc_w + px_x) * 4;
                    pc.data[idx] = rgb[0];
                    pc.data[idx + 1] = rgb[1];
                    pc.data[idx + 2] = rgb[2];
                    pc.data[idx + 3] = 255;
                    pc.dirty = true;
                }
            }
        }
        return;
    }

    let bg_rgb = color_to_rgb(bg);
    let bg_color = rgb_to_color(bg_rgb);

    for oy in 0..target_h {
        for ox in 0..target_w {
            let Some(rgb) = px(ox, oy) else {
                continue;
            };
            let symbol = if wireframe { draw_char } else { '█' };
            let fg_out = rgb_to_color(rgb);
            buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
        }
    }
}

// ── RGBA canvas compositing for planet cloud layers ──────────────────────────

/// Alpha-blend `src` RGBA canvas over `dst` RGBA canvas (premultiplied-style).
/// Both canvases must be the same size.  `None` entries in `src` are skipped.
pub fn composite_rgba_over(dst: &mut [Option<[u8; 4]>], src: &[Option<[u8; 4]>]) {
    debug_assert_eq!(dst.len(), src.len());
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        let Some(sp) = s else { continue };
        let sa = sp[3] as f32 / 255.0;
        if sa < 0.004 {
            continue;
        }
        if let Some(dp) = d {
            if sa >= 0.996 {
                *dp = *sp;
            } else {
                let inv = 1.0 - sa;
                dp[0] = (sp[0] as f32 * sa + dp[0] as f32 * inv).round() as u8;
                dp[1] = (sp[1] as f32 * sa + dp[1] as f32 * inv).round() as u8;
                dp[2] = (sp[2] as f32 * sa + dp[2] as f32 * inv).round() as u8;
                dp[3] = (sp[3] as f32 + dp[3] as f32 * inv).round().min(255.0) as u8;
            }
        } else {
            *d = Some(*sp);
        }
    }
}

/// Blit an RGBA canvas to a Buffer, using only the RGB channels (alpha already composited).
#[allow(clippy::too_many_arguments)]
pub fn blit_rgba_canvas(
    buf: &mut Buffer,
    canvas: &[Option<[u8; 4]>],
    virtual_w: u16,
    virtual_h: u16,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
) {
    let px = |vx: u16, vy: u16| -> Option<[u8; 3]> {
        if vx >= virtual_w || vy >= virtual_h {
            return None;
        }
        canvas
            .get(vy as usize * virtual_w as usize + vx as usize)
            .copied()
            .flatten()
            .map(|rgba| [rgba[0], rgba[1], rgba[2]])
    };

    // ── SDL2 pixel bypass: write virtual pixels directly ─────────────────
    if let Some(pc) = &mut buf.pixel_canvas {
        let pc_w = pc.width as usize;
        let virt_mult = virtual_dimensions_multiplier();
        let base_vx = x as usize * virt_mult.0 as usize;
        let base_vy = y as usize * virt_mult.1 as usize;
        for vy in 0..virtual_h {
            for vx in 0..virtual_w {
                let Some(rgb) = px(vx, vy) else { continue };
                let px_x = base_vx + vx as usize;
                let px_y = base_vy + vy as usize;
                if px_x < pc.width as usize && px_y < pc.height as usize {
                    let idx = (px_y * pc_w + px_x) * 4;
                    pc.data[idx] = rgb[0];
                    pc.data[idx + 1] = rgb[1];
                    pc.data[idx + 2] = rgb[2];
                    pc.data[idx + 3] = 255;
                    pc.dirty = true;
                }
            }
        }
        return;
    }

    let bg_color = Color::Reset;

    for oy in 0..target_h {
        for ox in 0..target_w {
            let Some(rgb) = px(ox, oy) else { continue };
            buf.set(x + ox, y + oy, '█', rgb_to_color(rgb), bg_color);
        }
    }
}

/// Rasterize a Gouraud-shaded triangle into an RGBA canvas.
/// When `cloud_alpha_softness > 0`, pixels near the terrain threshold get soft alpha
/// edges instead of a binary cutoff.  Per-pixel noise is evaluated for cloud detail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rasterize_triangle_gouraud_rgba(
    canvas: &mut [Option<[u8; 4]>],
    depth: &mut [f32],
    w: u16,
    h: u16,
    v0: ProjectedVertex,
    v1: ProjectedVertex,
    v2: ProjectedVertex,
    base_color: [u8; 3],
    shade0: f32,
    shade1: f32,
    shade2: f32,
    cel_levels: u8,
    terrain_color: Option<[u8; 3]>,
    terrain_threshold: f32,
    terrain_noise_scale: f32,
    terrain_noise_octaves: u8,
    below_threshold_transparent: bool,
    cloud_alpha_softness: f32,
    biome: Option<PlanetBiomeParams>,
    clip_min_y: i32,
    clip_max_y: i32,
    row_base: i32,
    marble_depth: f32,
    shadow_colour: Option<Color>,
    midtone_colour: Option<Color>,
    highlight_colour: Option<Color>,
    tone_mix: f32,
    latitude_bands: u8,
    latitude_band_depth: f32,
) {
    let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
    if area.abs() < 1e-5 {
        return;
    }
    let inv_area = 1.0 / area;

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;
    let min_y = min_y.max(clip_min_y);
    let max_y = max_y.min(clip_max_y);
    if min_x > max_x || min_y > max_y {
        return;
    }

    let use_bands = latitude_bands > 0 && latitude_band_depth > f32::EPSILON;
    let per_pixel_noise = cloud_alpha_softness > 0.0 && terrain_color.is_some();
    let soft_edge = cloud_alpha_softness.max(0.0);

    for py in min_y..=max_y {
        let y = py as f32 + 0.5;
        let row_start = (py - row_base) as usize * w as usize;
        for px_coord in min_x..=max_x {
            let x = px_coord as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) * inv_area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) * inv_area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) * inv_area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = row_start + px_coord as usize;
            if z < depth[idx] {
                depth[idx] = z;
                let shade = (w0 * shade0 + w1 * shade1 + w2 * shade2).clamp(0.0, 1.0);
                let shade = if use_bands {
                    let view_y = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                    let band = (view_y * latitude_bands as f32 * std::f32::consts::PI).sin();
                    (shade + band * latitude_band_depth * 0.5).clamp(0.0, 1.0)
                } else {
                    shade
                };

                // Per-pixel noise for cloud detail (evaluated from local-space position).
                let noise = if per_pixel_noise {
                    let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                    let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                    let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                    fbm_3d_octaves(
                        lx * terrain_noise_scale,
                        ly * terrain_noise_scale,
                        lz * terrain_noise_scale,
                        terrain_noise_octaves,
                    )
                } else {
                    w0 * v0.terrain_noise + w1 * v1.terrain_noise + w2 * v2.terrain_noise
                };

                if let Some(tc) = terrain_color {
                    if noise > terrain_threshold {
                        let alpha = if soft_edge > 0.0 {
                            let edge_t = ((noise - terrain_threshold) / soft_edge).clamp(0.0, 1.0);
                            // Smooth ramp: 0 at threshold, 1 at threshold + softness.
                            let a = edge_t * edge_t * (3.0 - 2.0 * edge_t);
                            (a * 255.0).round() as u8
                        } else {
                            255
                        };
                        let cel = quantize_shade(shade, cel_levels);
                        let pixel = apply_shading(tc, cel);

                        // Atmosphere overlay for opaque surface pass.
                        let pixel = if let Some(b) = &biome {
                            apply_atmo_overlay(pixel, b, &v0, &v1, &v2, w0, w1, w2)
                        } else {
                            pixel
                        };

                        canvas[idx] = Some([pixel[0], pixel[1], pixel[2], alpha]);
                    } else if below_threshold_transparent {
                        continue;
                    } else {
                        // Ocean/surface below threshold — opaque.
                        let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                        let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                        let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                        let mn = value_noise_3d(lx * 4.0, ly * 4.0, lz * 4.0);
                        let os = (shade + (mn - 0.5) * marble_depth).clamp(0.0, 1.0);
                        let cel = quantize_shade(os, cel_levels);
                        let mut pixel = apply_shading(base_color, cel);
                        pixel = apply_tone_palette(
                            pixel,
                            cel,
                            shadow_colour,
                            midtone_colour,
                            highlight_colour,
                            tone_mix,
                        );
                        // Biome overlays on ocean.
                        if let Some(b) = &biome {
                            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                            let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
                            let lat_abs = (vy / vlen).abs();
                            if let Some(ice_c) = b.polar_ice_color {
                                let ice_mask =
                                    ss(b.polar_ice_start + 0.05, b.polar_ice_end, lat_abs);
                                if ice_mask > 0.005 {
                                    let cel2 = quantize_shade(shade, cel_levels);
                                    pixel = apply_shading(ice_c, cel2);
                                }
                            }
                            pixel = apply_atmo_overlay(pixel, b, &v0, &v1, &v2, w0, w1, w2);
                        }
                        canvas[idx] = Some([pixel[0], pixel[1], pixel[2], 255]);
                    }
                } else {
                    let cel = quantize_shade(shade, cel_levels);
                    let pixel = apply_shading(base_color, cel);
                    let pixel = apply_tone_palette(
                        pixel,
                        cel,
                        shadow_colour,
                        midtone_colour,
                        highlight_colour,
                        tone_mix,
                    );
                    canvas[idx] = Some([pixel[0], pixel[1], pixel[2], 255]);
                }
            }
        }
    }
}

/// Apply atmosphere rim + haze overlay.  Extracted so both RGB and RGBA rasterizers share it.
fn apply_atmo_overlay(
    mut pixel: [u8; 3],
    b: &PlanetBiomeParams,
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    w0: f32,
    w1: f32,
    w2: f32,
) -> [u8; 3] {
    if let Some(ac) = b.atmo_color {
        if b.atmo_strength > 0.0 || b.atmo_haze_strength > 0.0 {
            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
            let vlen = (vx * vx + vy * vy + vz * vz).sqrt().max(1e-6);
            let (nx, ny, nz) = (vx / vlen, vy / vlen, vz / vlen);
            let nd = (nx * b.view_dir[0] + ny * b.view_dir[1] + nz * b.view_dir[2])
                .abs()
                .clamp(0.0, 1.0);
            let rim = (1.0 - nd).powf(b.atmo_rim_power);
            let haze = (1.0 - nd).powf(b.atmo_haze_power.max(0.1));
            if rim > 0.01 || haze > 0.01 {
                let sun_dot = nx * b.sun_dir[0] + ny * b.sun_dir[1] + nz * b.sun_dir[2];
                let day = ss(-0.05, 0.25, sun_dot);
                let rim_alpha = rim * (0.20 + 0.80 * day) * b.atmo_strength;
                let haze_alpha = haze * (0.05 + 0.30 * day) * b.atmo_haze_strength;
                let ab = (rim_alpha + haze_alpha).clamp(0.0, 0.85);
                pixel = mix_rgb(pixel, ac, ab);
            }
        }
    }
    pixel
}

#[inline(always)]
pub(crate) fn color_to_rgb(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb { r, g, b } => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::DarkGrey => [80, 80, 80],
        Color::Grey => [160, 160, 160],
        Color::White => [255, 255, 255],
        Color::Red | Color::DarkRed => [220, 64, 64],
        Color::Green | Color::DarkGreen => [64, 220, 64],
        Color::Blue | Color::DarkBlue => [64, 64, 220],
        Color::Yellow | Color::DarkYellow => [220, 220, 64],
        Color::Magenta | Color::DarkMagenta => [220, 64, 220],
        Color::Cyan | Color::DarkCyan => [64, 220, 220],
        _ => [255, 255, 255],
    }
}

#[inline]
pub(crate) fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

pub(crate) fn clip_line_to_viewport(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    vp: Viewport,
) -> Option<(i32, i32, i32, i32)> {
    let mut out0 = out_code(x0, y0, vp);
    let mut out1 = out_code(x1, y1, vp);

    loop {
        if (out0 | out1) == 0 {
            return Some((x0, y0, x1, y1));
        }
        if (out0 & out1) != 0 {
            return None;
        }
        let out = if out0 != 0 { out0 } else { out1 };

        let (nx, ny) = if (out & OUT_TOP) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.min_y)?
        } else if (out & OUT_BOTTOM) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.max_y)?
        } else if (out & OUT_RIGHT) != 0 {
            intersect_vertical(x0, y0, x1, y1, vp.max_x)?
        } else {
            intersect_vertical(x0, y0, x1, y1, vp.min_x)?
        };

        if out == out0 {
            x0 = nx;
            y0 = ny;
            out0 = out_code(x0, y0, vp);
        } else {
            x1 = nx;
            y1 = ny;
            out1 = out_code(x1, y1, vp);
        }
    }
}

const OUT_LEFT: u8 = 1;
const OUT_RIGHT: u8 = 2;
const OUT_BOTTOM: u8 = 4;
const OUT_TOP: u8 = 8;

#[inline]
pub(crate) fn out_code(x: i32, y: i32, vp: Viewport) -> u8 {
    let mut code = 0u8;
    if x < vp.min_x {
        code |= OUT_LEFT;
    } else if x > vp.max_x {
        code |= OUT_RIGHT;
    }
    if y > vp.max_y {
        code |= OUT_BOTTOM;
    } else if y < vp.min_y {
        code |= OUT_TOP;
    }
    code
}

#[inline]
pub(crate) fn intersect_vertical(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> Option<(i32, i32)> {
    let dx = x1 - x0;
    if dx == 0 {
        return None;
    }
    let t = (x - x0) as f32 / dx as f32;
    let y = y0 as f32 + t * (y1 - y0) as f32;
    Some((x, y.round() as i32))
}

#[inline]
pub(crate) fn intersect_horizontal(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    y: i32,
) -> Option<(i32, i32)> {
    let dy = y1 - y0;
    if dy == 0 {
        return None;
    }
    let t = (y - y0) as f32 / dy as f32;
    let x = x0 as f32 + t * (x1 - x0) as f32;
    Some((x.round() as i32, y))
}

#[inline(always)]
pub(crate) fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
    let (sp, cp) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();
    let (sr, cr) = roll.sin_cos();

    let x1 = v[0];
    let y1 = v[1] * cp - v[2] * sp;
    let z1 = v[1] * sp + v[2] * cp;

    let x2 = x1 * cy + z1 * sy;
    let y2 = y1;
    let z2 = -x1 * sy + z1 * cy;

    let x3 = x2 * cr - y2 * sr;
    let y3 = x2 * sr + y2 * cr;
    [x3, y3, z2]
}

#[cfg(test)]
mod tests {
    use super::obj_sprite_dimensions;
    use engine_core::scene::SpriteSizePreset;

    #[test]
    fn obj_size_preset_uses_type_defaults() {
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Small)),
            (32, 12)
        );
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Medium)),
            (64, 24)
        );
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Large)),
            (96, 36)
        );
    }
}
