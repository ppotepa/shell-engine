use engine_core::color::Color;

use crate::effects::atmosphere::apply_atmosphere_overlay_barycentric;
use crate::effects::biome::{land_biome_signals, polar_ice_mask_ocean_from_view};
use crate::effects::noise::fbm_3d_octaves;
use crate::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use crate::effects::terrain::{
    apply_crater_overlay_rgb, land_elevation_relief, normal_perturb_shade, ocean_shade_from_local,
    ocean_specular_add, snow_line_mask, CraterParams,
};
use crate::geom::raster::edge;
use crate::geom::types::ProjectedVertex;
use crate::shading::{apply_shading, apply_tone_palette, mix_rgb, quantize_shade};

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
                let shade = (w0 * shade0 + w1 * shade1 + w2 * shade2).clamp(0.0, 1.0);
                let shade = if use_bands {
                    let view_y = w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1];
                    let band = (view_y * latitude_bands as f32 * std::f32::consts::PI).sin();
                    (shade + band * latitude_band_depth * 0.5).clamp(0.0, 1.0)
                } else {
                    shade
                };

                let mut pixel = if let Some(tc) = terrain_color {
                    let noise =
                        w0 * v0.terrain_noise + w1 * v1.terrain_noise + w2 * v2.terrain_noise;
                    if noise > terrain_threshold {
                        let shade =
                            land_elevation_relief(shade, noise, terrain_threshold, terrain_relief);
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
                            if let Some(dc) = b.desert_color {
                                if sig.desert_mask > 0.005 {
                                    land_color = mix_rgb(land_color, dc, sig.desert_mask);
                                }
                            }
                            if let Some(ice_c) = b.polar_ice_color {
                                if sig.ice_mask > 0.005 {
                                    land_color = mix_rgb(land_color, ice_c, sig.ice_mask);
                                }
                            }

                            let cel = quantize_shade(shade, cel_levels);
                            let mut px_color = apply_shading(land_color, cel);

                            if let Some(city_c) = b.night_light_color {
                                if b.night_light_intensity > 0.0 && sig.city_mask > 0.01 {
                                    px_color =
                                        mix_rgb(px_color, city_c, sig.city_mask.clamp(0.0, 0.95));
                                }
                            }
                            px_color
                        } else {
                            let cel = quantize_shade(shade, cel_levels);
                            apply_shading(land_color, cel)
                        }
                    } else {
                        if below_threshold_transparent {
                            continue;
                        }
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
                        let lx = w0 * v0.local[0] + w1 * v1.local[0] + w2 * v2.local[0];
                        let ly = w0 * v0.local[1] + w1 * v1.local[1] + w2 * v2.local[1];
                        let lz = w0 * v0.local[2] + w1 * v1.local[2] + w2 * v2.local[2];
                        let ocean_ns = terrain_extra.map(|te| te.ocean_noise_scale).unwrap_or(4.0);
                        let ocean_base = terrain_extra
                            .and_then(|te| te.ocean_color_override)
                            .unwrap_or(base_color);
                        let os =
                            ocean_shade_from_local(shade, [lx, ly, lz], ocean_ns, marble_depth);
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

                if let Some(b) = biome {
                    pixel =
                        apply_atmosphere_overlay_barycentric(pixel, &b, &v0, &v1, &v2, w0, w1, w2);
                }

                canvas[idx] = Some(pixel);
            }
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
                            let a = edge_t * edge_t * (3.0 - 2.0 * edge_t);
                            (a * 255.0).round() as u8
                        } else {
                            255
                        };
                        let cel = quantize_shade(shade, cel_levels);
                        let pixel = apply_shading(tc, cel);

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
