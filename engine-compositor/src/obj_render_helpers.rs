use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_render_3d::effects::atmosphere::apply_atmosphere_overlay_barycentric;
use engine_render_3d::effects::biome::{land_biome_signals, polar_ice_mask_ocean_from_view};
pub(crate) use engine_render_3d::effects::noise::fbm_3d_octaves;
pub(crate) use engine_render_3d::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use engine_render_3d::effects::terrain::{
    apply_crater_overlay_rgb, land_elevation_relief, normal_perturb_shade, ocean_shade_from_local,
    ocean_specular_add, snow_line_mask, CraterParams,
};
pub(crate) use engine_render_3d::geom::clip::{clip_line_to_viewport, clipped_depths, Viewport};
pub(crate) use engine_render_3d::geom::math::{dot3, normalize3, rotate_xyz};
pub(crate) use engine_render_3d::geom::raster::edge;
pub(crate) use engine_render_3d::geom::types::ProjectedVertex;
pub(crate) use engine_render_3d::shading::{
    apply_point_light_tint, apply_shading, apply_tone_palette, color_to_rgb,
    face_shading_with_specular, flicker_multiplier, mix_rgb, quantize_shade,
};

use engine_asset::ObjFace;

#[inline]
pub fn virtual_dimensions(target_w: u16, target_h: u16) -> (u16, u16) {
    (target_w, target_h)
}

/// Virtual-to-frame multiplier per axis.
#[inline]
pub fn virtual_dimensions_multiplier() -> (u16, u16) {
    (1, 1)
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
                        let shade =
                            land_elevation_relief(shade, noise, terrain_threshold, terrain_relief);
                        // Per-pixel normal perturbation: finite-difference gradient of noise perturbs shade.
                        let shade = if let Some(te) = terrain_extra {
                            if te.normal_perturb > 0.0 && te.noise_scale > 0.0 {
                                let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                                let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                                let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                if let Some(b) = biome {
                                    normal_perturb_shade(
                                        shade,
                                        [lx, ly, lz],
                                        [vx, vy, vz],
                                        b.sun_dir,
                                        te.noise_scale,
                                        te.normal_perturb,
                                    )
                                } else {
                                    shade
                                }
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
                                    let snow_mask = snow_line_mask(te.snow_line, elev);
                                    land_color = mix_rgb(land_color, [240, 248, 255], snow_mask);
                                }
                            }
                        }
                        if let Some(b) = biome {
                            // Surface sample position for biome masks.
                            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                            let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                            let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                            let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                            let sig = land_biome_signals(
                                [lx, ly, lz],
                                [vx, vy, vz],
                                noise,
                                terrain_threshold,
                                b.desert_strength,
                                b.polar_ice_start,
                                b.polar_ice_end,
                                b.night_light_threshold,
                                b.night_light_intensity,
                                b.sun_dir,
                            );

                            // Desert biome: equatorial dry zone
                            if let Some(dc) = b.desert_color {
                                if sig.desert_mask > 0.005 {
                                    land_color = mix_rgb(land_color, dc, sig.desert_mask);
                                }
                            }
                            // Polar ice (overrides desert)
                            if let Some(ice_c) = b.polar_ice_color {
                                if sig.ice_mask > 0.005 {
                                    land_color = mix_rgb(land_color, ice_c, sig.ice_mask);
                                }
                            }

                            let cel = quantize_shade(shade, cel_levels);
                            let mut px_color = apply_shading(land_color, cel);

                            // Night-side city lights (land, dark side only)
                            if let Some(city_c) = b.night_light_color {
                                if b.night_light_intensity > 0.0 {
                                    if sig.city_mask > 0.01 {
                                        px_color = mix_rgb(
                                            px_color,
                                            city_c,
                                            sig.city_mask.clamp(0.0, 0.95),
                                        );
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
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                let ice_mask = polar_ice_mask_ocean_from_view(
                                    [vx, vy, vz],
                                    b.polar_ice_start,
                                    b.polar_ice_end,
                                );
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
                        let ocean_base = terrain_extra
                            .and_then(|te| te.ocean_color_override)
                            .unwrap_or(base_color);
                        let os =
                            ocean_shade_from_local(shade, [lx, ly, lz], ocean_ns, marble_depth);
                        // Ocean specular highlight (sunglint).
                        let os = if let (Some(b), Some(te)) = (biome, terrain_extra) {
                            if te.ocean_specular > 0.0 {
                                let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                                let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                                let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                                let spec = ocean_specular_add(
                                    [vx, vy, vz],
                                    b.sun_dir,
                                    b.view_dir,
                                    te.ocean_specular,
                                    32.0,
                                );
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
                        pixel = apply_crater_overlay_rgb(
                            pixel,
                            [lx, ly, lz],
                            CraterParams {
                                density: te.crater_density,
                                rim_height: te.crater_rim_height,
                            },
                        );
                    }
                }

                // Atmosphere overlay.
                if let Some(b) = biome {
                    pixel =
                        apply_atmosphere_overlay_barycentric(pixel, &b, &v0, &v1, &v2, w0, w1, w2);
                }

                canvas[idx] = Some(pixel);
            }
        }
    }
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
                            apply_atmosphere_overlay_barycentric(
                                pixel, b, &v0, &v1, &v2, w0, w1, w2,
                            )
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
                        let os = ocean_shade_from_local(shade, [lx, ly, lz], 4.0, marble_depth);
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
                            let vx = w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0];
                            let vy = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                            let vz = w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2];
                            if let Some(ice_c) = b.polar_ice_color {
                                let ice_mask = polar_ice_mask_ocean_from_view(
                                    [vx, vy, vz],
                                    b.polar_ice_start,
                                    b.polar_ice_end,
                                );
                                if ice_mask > 0.005 {
                                    let cel2 = quantize_shade(shade, cel_levels);
                                    pixel = apply_shading(ice_c, cel2);
                                }
                            }
                            pixel = apply_atmosphere_overlay_barycentric(
                                pixel, b, &v0, &v1, &v2, w0, w1, w2,
                            );
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

#[inline]
pub(crate) fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

#[cfg(test)]
mod tests {
    use crate::obj_render::obj_sprite_dimensions;
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
