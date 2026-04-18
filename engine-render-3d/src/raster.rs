//! OBJ mesh rasterization pipeline: vertex projection, triangle rasterization,
//! shared-buffer rendering, and canvas-to-buffer blitting.
//!
//! This module owns the core render-domain logic for Scene3D work items,
//! independent of the compositor's frame assembly concerns.

use std::cell::RefCell;
use std::time::Instant;

use engine_asset::{load_render_mesh, ObjFace, ObjMesh};
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::TonemapOperator;
use rayon::prelude::*;

use crate::api::Render3dPipeline;
use crate::effects::atmosphere::apply_atmosphere_overlay_barycentric;
use crate::effects::biome::{land_biome_signals, polar_ice_mask_ocean_from_view};
use crate::effects::noise::fbm_3d_octaves;
use crate::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use crate::effects::terrain::{
    apply_crater_overlay_rgb, compute_terrain_noise_at, displace_sphere_vertex,
    land_elevation_relief, normal_perturb_shade, ocean_shade_from_local, ocean_specular_add,
    snow_line_mask, CraterParams,
};
use crate::geom::clip::{clip_line_to_viewport, clipped_depths, Viewport};
use crate::geom::math::{dot3, normalize3, rotate_xyz};
use crate::geom::raster::edge;
use crate::geom::types::ProjectedVertex;
use crate::prerender::ObjPrerenderedFrames;
use crate::shading::{
    apply_point_light_tint, apply_shading, apply_tone_palette, color_to_rgb,
    face_shading_with_specular, flicker_multiplier, mix_rgb, quantize_shade,
};
use crate::ObjRenderParams;
use engine_core::scene::SpriteSizePreset;

/// Safety cap on face count after early backface culling (front-facing only).
const MAX_OBJ_FACE_RENDER: usize = 2_000_000;
const MIN_PROJECTED_FACE_DOUBLE_AREA: f32 = 0.01;

/// Minimum vertex/face count to use parallel processing.
/// Below this, serial is faster due to rayon thread spawn overhead.
const VERTEX_PARALLEL_THRESHOLD: usize = 64;

// Thread-local pooled buffer for vertex projection — avoids per-frame allocation.
thread_local! {
    static OBJ_PROJECTED: RefCell<Vec<Option<ProjectedVertex>>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    static OBJ_CANVAS: RefCell<Vec<Option<[u8; 3]>>> = const { RefCell::new(Vec::new()) };
    static OBJ_CANVAS_RGBA: RefCell<Vec<Option<[u8; 4]>>> = const { RefCell::new(Vec::new()) };
    static OBJ_DEPTH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static OBJ_SORTED_FACE_INDEX: RefCell<Vec<(f32, usize)>> = const { RefCell::new(Vec::new()) };
    static OBJ_HALO_EDGE_PIXELS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
    static OBJ_HALO_OCCUPIED_SCAN: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OBJ_HALO_NEAREST_SQ: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static OBJ_SHADED_GOURAUD: RefCell<Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3], f32, f32, f32)>>
        = const { RefCell::new(Vec::new()) };
    static OBJ_SHADED_FLAT: RefCell<Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3])>>
        = const { RefCell::new(Vec::new()) };
    static OBJ_LAST_RASTER_STATS: RefCell<ObjRasterStats> = const {
        RefCell::new(ObjRasterStats {
            triangles_processed: 0,
            faces_drawn: 0,
            viewport_area_px: 0,
        })
    };
    static OBJ_RASTER_FRAME_METRICS: RefCell<ObjRasterFrameMetrics> = const {
        RefCell::new(ObjRasterFrameMetrics {
            rgb_us: 0.0,
            rgba_us: 0.0,
            halo_us: 0.0,
            rgb_calls: 0,
            rgba_calls: 0,
            triangles_processed: 0,
            faces_drawn: 0,
            viewport_area_px: 0,
        })
    };
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjRasterStats {
    pub triangles_processed: u32,
    pub faces_drawn: u32,
    pub viewport_area_px: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjRasterFrameMetrics {
    pub rgb_us: f32,
    pub rgba_us: f32,
    pub halo_us: f32,
    pub rgb_calls: u32,
    pub rgba_calls: u32,
    pub triangles_processed: u32,
    pub faces_drawn: u32,
    pub viewport_area_px: u32,
}

#[inline]
fn set_last_obj_raster_stats(stats: ObjRasterStats) {
    OBJ_LAST_RASTER_STATS.with(|cell| *cell.borrow_mut() = stats);
}

/// Returns and clears the latest OBJ raster stats captured by
/// `render_obj_to_canvas` or `render_obj_to_rgba_canvas`.
pub fn take_last_obj_raster_stats() -> ObjRasterStats {
    OBJ_LAST_RASTER_STATS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

#[inline]
fn accumulate_obj_raster_frame_metrics(delta: ObjRasterFrameMetrics) {
    OBJ_RASTER_FRAME_METRICS.with(|cell| {
        let mut acc = cell.borrow_mut();
        acc.rgb_us += delta.rgb_us;
        acc.rgba_us += delta.rgba_us;
        acc.halo_us += delta.halo_us;
        acc.rgb_calls = acc.rgb_calls.saturating_add(delta.rgb_calls);
        acc.rgba_calls = acc.rgba_calls.saturating_add(delta.rgba_calls);
        acc.triangles_processed = acc
            .triangles_processed
            .saturating_add(delta.triangles_processed);
        acc.faces_drawn = acc.faces_drawn.saturating_add(delta.faces_drawn);
        acc.viewport_area_px = acc.viewport_area_px.max(delta.viewport_area_px);
    });
}

pub fn reset_obj_raster_frame_metrics() {
    OBJ_RASTER_FRAME_METRICS.with(|cell| *cell.borrow_mut() = ObjRasterFrameMetrics::default());
}

pub fn take_obj_raster_frame_metrics() -> ObjRasterFrameMetrics {
    OBJ_RASTER_FRAME_METRICS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

// ── Dimension helpers ─────────────────────────────────────────────────────────

/// Returns `(target_w, target_h)` — virtual canvas equals the output size for this pipeline.
#[inline]
pub fn virtual_dimensions(target_w: u16, target_h: u16) -> (u16, u16) {
    (target_w, target_h)
}

/// Virtual-to-frame pixel multiplier per axis (always 1:1 for this pipeline).
#[inline]
pub fn virtual_dimensions_multiplier() -> (u16, u16) {
    (1, 1)
}

// ── Color conversion ──────────────────────────────────────────────────────────

#[inline]
pub(crate) fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

#[inline]
fn tonemap_channel(value: f32, tonemap: TonemapOperator) -> f32 {
    match tonemap {
        TonemapOperator::Linear => value,
        TonemapOperator::Reinhard => value / (1.0 + value),
        TonemapOperator::AcesApprox => {
            let a = 2.51;
            let b = 0.03;
            let c = 2.43;
            let d = 0.59;
            let e = 0.14;
            ((value * (a * value + b)) / (value * (c * value + d) + e)).clamp(0.0, 1.0)
        }
    }
}

#[inline]
fn grade_rgb(
    rgb: [u8; 3],
    exposure: f32,
    gamma: f32,
    tonemap: TonemapOperator,
    shadow_contrast: f32,
) -> [u8; 3] {
    let inv_gamma = (1.0 / gamma.max(0.1)).clamp(0.05, 10.0);
    let exposure = exposure.max(0.0);
    let shadow_contrast = shadow_contrast.clamp(0.25, 4.0);
    let map = |channel: u8| -> u8 {
        let linear = (channel as f32 / 255.0) * exposure;
        let mapped = tonemap_channel(linear, tonemap).clamp(0.0, 1.0);
        let contrasted = mapped.powf(shadow_contrast).clamp(0.0, 1.0);
        let corrected = contrasted.powf(inv_gamma).clamp(0.0, 1.0);
        (corrected * 255.0).round() as u8
    };
    [map(rgb[0]), map(rgb[1]), map(rgb[2])]
}

fn apply_canvas_grading(
    canvas: &mut [Option<[u8; 3]>],
    exposure: f32,
    gamma: f32,
    tonemap: TonemapOperator,
    shadow_contrast: f32,
) {
    if (exposure - 1.0).abs() < f32::EPSILON
        && (gamma - 2.2).abs() < f32::EPSILON
        && matches!(tonemap, TonemapOperator::Linear)
        && (shadow_contrast - 1.0).abs() < f32::EPSILON
    {
        return;
    }
    for pixel in canvas.iter_mut() {
        if let Some(rgb) = pixel.as_mut() {
            *rgb = grade_rgb(*rgb, exposure, gamma, tonemap, shadow_contrast);
        }
    }
}

fn apply_rgba_canvas_grading(
    canvas: &mut [Option<[u8; 4]>],
    exposure: f32,
    gamma: f32,
    tonemap: TonemapOperator,
    shadow_contrast: f32,
) {
    if (exposure - 1.0).abs() < f32::EPSILON
        && (gamma - 2.2).abs() < f32::EPSILON
        && matches!(tonemap, TonemapOperator::Linear)
        && (shadow_contrast - 1.0).abs() < f32::EPSILON
    {
        return;
    }
    for pixel in canvas.iter_mut() {
        if let Some(rgba) = pixel.as_mut() {
            let graded = grade_rgb(
                [rgba[0], rgba[1], rgba[2]],
                exposure,
                gamma,
                tonemap,
                shadow_contrast,
            );
            rgba[0] = graded[0];
            rgba[1] = graded[1];
            rgba[2] = graded[2];
        }
    }
}

// ── Canvas blit ───────────────────────────────────────────────────────────────

/// Write a flat RGB color canvas into a terminal [`Buffer`].
///
/// Respects SDL2 pixel bypass when `buf.pixel_canvas` is active.
/// The `clip_row_min`/`clip_row_max` range is in virtual-pixel rows.
#[allow(clippy::too_many_arguments)]
pub fn blit_color_canvas(
    buf: &mut Buffer,
    canvas: &[Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    target_w: u16,
    target_h: u16,
    x: i32,
    y: i32,
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
        let base_vx = x * virt_mult.0 as i32;
        let base_vy = y * virt_mult.1 as i32;
        for vy in 0..virtual_h {
            for vx in 0..virtual_w {
                let Some(rgb) = px(vx, vy) else { continue };
                let px_x = base_vx + vx as i32;
                let px_y = base_vy + vy as i32;
                if px_x >= 0
                    && px_y >= 0
                    && (px_x as usize) < pc.width as usize
                    && (px_y as usize) < pc.height as usize
                {
                    let px_x = px_x as usize;
                    let px_y = px_y as usize;
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
            let tx = x + ox as i32;
            let ty = y + oy as i32;
            if tx < 0 || ty < 0 || tx >= buf.width as i32 || ty >= buf.height as i32 {
                continue;
            }
            let symbol = if wireframe { draw_char } else { '█' };
            let fg_out = rgb_to_color(rgb);
            buf.set(tx as u16, ty as u16, symbol, fg_out, bg_color);
        }
    }
}

// ── Line rasterizers ──────────────────────────────────────────────────────────

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

// ── Triangle rasterizers ──────────────────────────────────────────────────────

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

// ── Face depth helper ─────────────────────────────────────────────────────────

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

// ── Setup helpers (light/camera param extraction) ─────────────────────────────

#[inline]
fn normalized_light_and_view_dirs(params: &ObjRenderParams) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let light_dir_norm = normalize3([
        params.light_direction_x,
        params.light_direction_y,
        params.light_direction_z,
    ]);
    let light_2_dir_norm = normalize3([
        params.light_2_direction_x,
        params.light_2_direction_y,
        params.light_2_direction_z,
    ]);
    let view_dir = normalize3([
        -params.view_forward_x,
        -params.view_forward_y,
        -params.view_forward_z,
    ]);
    (light_dir_norm, light_2_dir_norm, view_dir)
}

#[inline]
fn build_biome_params(
    params: &ObjRenderParams,
    light_dir_norm: [f32; 3],
    view_dir: [f32; 3],
) -> Option<PlanetBiomeParams> {
    let has_biome = params.polar_ice_color.is_some()
        || params.desert_color.is_some()
        || params.atmo_color.is_some()
        || params.atmo_rayleigh_color.is_some()
        || params.atmo_haze_color.is_some()
        || params.atmo_absorption_color.is_some()
        || params.night_light_color.is_some();
    if !has_biome {
        return None;
    }
    Some(PlanetBiomeParams {
        polar_ice_color: params.polar_ice_color,
        polar_ice_start: params.polar_ice_start,
        polar_ice_end: params.polar_ice_end,
        desert_color: params.desert_color,
        desert_strength: params.desert_strength,
        atmo_color: params.atmo_color,
        atmo_height: params.atmo_height,
        atmo_density: params.atmo_density,
        atmo_strength: params.atmo_strength,
        atmo_rayleigh_amount: params.atmo_rayleigh_amount,
        atmo_rayleigh_color: params.atmo_rayleigh_color,
        atmo_rayleigh_falloff: params.atmo_rayleigh_falloff,
        atmo_haze_amount: params.atmo_haze_amount,
        atmo_haze_color: params.atmo_haze_color,
        atmo_haze_falloff: params.atmo_haze_falloff,
        atmo_absorption_amount: params.atmo_absorption_amount,
        atmo_absorption_color: params.atmo_absorption_color,
        atmo_absorption_height: params.atmo_absorption_height,
        atmo_absorption_width: params.atmo_absorption_width,
        atmo_forward_scatter: params.atmo_forward_scatter,
        atmo_limb_boost: params.atmo_limb_boost,
        atmo_terminator_softness: params.atmo_terminator_softness,
        atmo_night_glow: params.atmo_night_glow,
        atmo_night_glow_color: params.atmo_night_glow_color,
        atmo_haze_night_leak: params.atmo_haze_night_leak,
        atmo_rim_power: params.atmo_rim_power,
        atmo_haze_strength: params.atmo_haze_strength,
        atmo_haze_power: params.atmo_haze_power,
        atmo_veil_strength: params.atmo_veil_strength,
        atmo_veil_power: params.atmo_veil_power,
        night_light_color: params.night_light_color,
        night_light_threshold: params.night_light_threshold,
        night_light_intensity: params.night_light_intensity,
        sun_dir: light_dir_norm,
        view_dir,
        camera_pos: [
            params.camera_world_x,
            params.camera_world_y,
            params.camera_world_z,
        ],
    })
}

#[inline]
fn build_terrain_extra_params(params: &ObjRenderParams) -> Option<PlanetTerrainParams> {
    if params.terrain_color.is_none()
        || (params.normal_perturb_strength <= 0.0
            && params.ocean_specular <= 0.0
            && params.crater_density <= 0.0
            && params.snow_line_altitude <= 0.0
            && params.ocean_noise_scale == 4.0
            && params.ocean_color_rgb.is_none())
    {
        return None;
    }
    Some(PlanetTerrainParams {
        noise_scale: params.terrain_noise_scale,
        normal_perturb: params.normal_perturb_strength,
        ocean_specular: params.ocean_specular,
        crater_density: params.crater_density,
        crater_rim_height: params.crater_rim_height,
        snow_line: params.snow_line_altitude,
        ocean_noise_scale: params.ocean_noise_scale,
        ocean_color_override: params.ocean_color_rgb,
    })
}

// ── Core mesh projection + rasterization ──────────────────────────────────────

/// Project vertices and render a single mesh into provided canvas and depth buffers.
///
/// Both `canvas` and `depth_buf` must be pre-sized to `virtual_w * virtual_h` elements.
/// Multiple meshes can share the same canvas/depth_buf for proper cross-mesh depth testing
/// (e.g. wire+solid portrait pairs in scene3d prerender).
#[allow(clippy::too_many_arguments)]
fn render_mesh_projected(
    mesh: &ObjMesh,
    virtual_w: u16,
    virtual_h: u16,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
) {
    fn snap_angle(elapsed_s: f32, snap_hz: f32, seed: u32) -> f32 {
        let snap_index = (elapsed_s * snap_hz) as u32;
        let h = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
        (h as f32 / u32::MAX as f32) * std::f32::consts::TAU
    }

    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;
    let point_1_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_flicker_hz,
        params.light_point_flicker_depth,
        0.37,
    );
    let point_2_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_2_flicker_hz,
        params.light_point_2_flicker_depth,
        1.91,
    );

    let orbit_radius_1 = (params.light_point_x.powi(2) + params.light_point_z.powi(2))
        .sqrt()
        .max(0.0001);
    let (light_1_x, light_1_z) = if params.light_point_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_snap_hz, 0x9e37_79b9);
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else if params.light_point_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else {
        (params.light_point_x, params.light_point_z)
    };

    let orbit_radius_2 = (params.light_point_2_x.powi(2) + params.light_point_2_z.powi(2))
        .sqrt()
        .max(0.0001);
    let (light_2_x, light_2_z) = if params.light_point_2_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_2_snap_hz, 0x6c62_272d);
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else if params.light_point_2_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_2_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else {
        (params.light_point_2_x, params.light_point_2_z)
    };

    let yaw = (params.yaw_deg
        + params.rotation_y
        + params.rotate_y_deg_per_sec * elapsed_s
        + params.camera_look_yaw)
        .to_radians();
    let pitch = (params.pitch_deg + params.rotation_x + params.camera_look_pitch).to_radians();
    let roll = (params.roll_deg + params.rotation_z).to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let near_clip = params.near_clip.max(0.000001);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = virtual_w as f32 / virtual_h as f32;

    let viewport = Viewport {
        min_x: 0,
        min_y: 0,
        max_x: virtual_w as i32 - 1,
        max_y: virtual_h as i32 - 1,
    };
    let clip_row_min = (params.clip_y_min.clamp(0.0, 1.0) * virtual_h as f32).floor() as i32;
    let clip_row_max = (params.clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as i32 - 1;
    let clipped_viewport = Viewport {
        min_x: viewport.min_x,
        min_y: viewport.min_y.max(clip_row_min),
        max_x: viewport.max_x,
        max_y: viewport.max_y.min(clip_row_max),
    };
    if clipped_viewport.min_y > clipped_viewport.max_y {
        return;
    }

    let center = mesh.center;
    let mut projected = OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(mesh.vertices.len());
        taken
    });
    mesh.vertices
        .par_iter()
        .map(|v| {
            let centered_raw = [
                (v[0] - center[0]) * model_scale,
                (v[1] - center[1]) * model_scale,
                (v[2] - center[2]) * model_scale,
            ];
            let terrain_noise_val =
                if params.terrain_color.is_some() || params.terrain_displacement > 0.0 {
                    compute_terrain_noise_at(centered_raw, &params)
                } else {
                    0.0
                };
            let centered = if params.terrain_displacement > 0.0 {
                displace_sphere_vertex(centered_raw, terrain_noise_val, params.terrain_displacement)
            } else {
                centered_raw
            };
            let rotated = rotate_xyz(centered, pitch, yaw, roll);
            let translated = [
                rotated[0] + params.object_translate_x,
                rotated[1] + params.object_translate_y,
                rotated[2] + params.object_translate_z,
            ];
            let rel = [
                translated[0] - params.camera_world_x,
                translated[1] - params.camera_world_y,
                translated[2] - params.camera_world_z,
            ];
            let cam_x = rel[0] * params.view_right_x
                + rel[1] * params.view_right_y
                + rel[2] * params.view_right_z
                - params.camera_pan_x;
            let cam_y =
                rel[0] * params.view_up_x + rel[1] * params.view_up_y + rel[2] * params.view_up_z
                    - params.camera_pan_y;
            let view_z = rel[0] * params.view_forward_x
                + rel[1] * params.view_forward_y
                + rel[2] * params.view_forward_z;
            if view_z <= near_clip {
                return None;
            }
            let ndc_x = (cam_x / aspect) * inv_tan / view_z;
            let ndc_y = cam_y * inv_tan / view_z;
            if !ndc_x.is_finite() || !ndc_y.is_finite() {
                return None;
            }
            Some(ProjectedVertex {
                x: (ndc_x + 1.0) * 0.5 * (virtual_w as f32 - 1.0),
                y: (1.0 - (ndc_y + 1.0) * 0.5) * (virtual_h as f32 - 1.0),
                depth: view_z,
                view: translated,
                normal: [0.0, 0.0, 1.0],
                local: centered,
                terrain_noise: terrain_noise_val,
            })
        })
        .collect_into_vec(&mut projected);

    if params.smooth_shading && !mesh.smooth_normals.is_empty() {
        for (i, pv_opt) in projected.iter_mut().enumerate() {
            if let Some(pv) = pv_opt.as_mut() {
                if let Some(&n) = mesh.smooth_normals.get(i) {
                    pv.normal = rotate_xyz(n, pitch, yaw, roll);
                }
            }
        }
    }

    if wireframe {
        let line_color = color_to_rgb(fg);
        let (depth_near, depth_far) = {
            let mut near = f32::INFINITY;
            let mut far = f32::NEG_INFINITY;
            for pv in projected.iter().flatten() {
                near = near.min(pv.depth);
                far = far.max(pv.depth);
            }
            if (far - near).abs() < f32::EPSILON {
                (near, near + 1.0)
            } else {
                (near, far)
            }
        };

        let mut drawn_edges = 0usize;
        for (a, b) in &mesh.edges {
            if drawn_edges > 12_000 {
                break;
            }
            let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                continue;
            };
            let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                continue;
            };
            let x0 = pa.x.round() as i32;
            let y0 = pa.y.round() as i32;
            let x1 = pb.x.round() as i32;
            let y1 = pb.y.round() as i32;
            if let Some((cx0, cy0, cx1, cy1)) =
                clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
            {
                let (cz0, cz1) =
                    clipped_depths(x0, y0, x1, y1, cx0, cy0, cx1, cy1, pa.depth, pb.depth);
                draw_line_depth(
                    canvas, depth_buf, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color, cz0,
                    cz1, depth_near, depth_far,
                );
                drawn_edges += 1;
            }
        }
    } else {
        let (light_dir_norm, light_2_dir_norm, view_dir) = normalized_light_and_view_dirs(&params);
        let half_dir_1 = normalize3([
            light_dir_norm[0] + view_dir[0],
            light_dir_norm[1] + view_dir[1],
            light_dir_norm[2] + view_dir[2],
        ]);
        let half_dir_2 = normalize3([
            light_2_dir_norm[0] + view_dir[0],
            light_2_dir_norm[1] + view_dir[1],
            light_2_dir_norm[2] + view_dir[2],
        ]);

        let mut sorted_faces = OBJ_SORTED_FACE_INDEX.with(|v| {
            let mut pool = v.borrow_mut();
            let mut taken = std::mem::take(&mut *pool);
            taken.clear();
            taken.reserve(mesh.faces.len());
            taken
        });
        for (face_idx, face) in mesh.faces.iter().enumerate() {
            let v0 = projected.get(face.indices[0]).and_then(|p| *p);
            let v1 = projected.get(face.indices[1]).and_then(|p| *p);
            let v2 = projected.get(face.indices[2]).and_then(|p| *p);
            let (Some(v0), Some(v1), Some(v2)) = (v0, v1, v2) else {
                continue;
            };
            let projected_area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
            if backface_cull && projected_area < 0.0 {
                continue;
            }
            if projected_area.abs() < MIN_PROJECTED_FACE_DOUBLE_AREA {
                continue;
            }
            let key = if params.depth_sort_faces {
                face_avg_depth(&projected, face)
            } else {
                0.0
            };
            sorted_faces.push((key, face_idx));
        }
        if params.depth_sort_faces {
            sorted_faces.sort_unstable_by(|a, b| {
                b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let face_limit = sorted_faces.len().min(MAX_OBJ_FACE_RENDER);
        let light_point_y = params.light_point_y;
        let light_point_2_y = params.light_point_2_y;
        let light_2_intensity = params.light_2_intensity;
        let light_point_intensity = params.light_point_intensity;
        let light_point_2_intensity = params.light_point_2_intensity;
        let cel_levels = params.cel_levels;
        let tone_mix = params.tone_mix;
        let shadow_colour = params.shadow_colour;
        let midtone_colour = params.midtone_colour;
        let highlight_colour = params.highlight_colour;
        let light_point_colour = params.light_point_colour;
        let light_point_2_colour = params.light_point_2_colour;
        let unlit = params.unlit;
        let ambient = params.ambient;
        let light_point_falloff = params.light_point_falloff;
        let light_point_2_falloff = params.light_point_2_falloff;
        let smooth_shading = params.smooth_shading;
        let latitude_bands = params.latitude_bands;
        let latitude_band_depth = params.latitude_band_depth;
        let fg_rgb = color_to_rgb(fg);

        let biome_params = build_biome_params(&params, light_dir_norm, view_dir);
        let planet_terrain_extra = build_terrain_extra_params(&params);

        let drawn_faces = if smooth_shading {
            let ka_lum_ambient = ambient.max(params.ambient_floor);
            let light_2_strength = light_2_intensity.clamp(0.0, 2.0);

            let shade_at_vertex = |normal: [f32; 3]| -> f32 {
                let lambert_1 = dot3(normal, light_dir_norm).max(0.0);
                let lambert_2 = dot3(normal, light_2_dir_norm).max(0.0) * light_2_strength;
                let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
                (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
            };

            let shaded_gouraud: Vec<(
                ProjectedVertex,
                ProjectedVertex,
                ProjectedVertex,
                [u8; 3],
                f32,
                f32,
                f32,
            )> = sorted_faces[..face_limit]
                .par_iter()
                .filter_map(|(_, face_idx)| {
                    let face = &mesh.faces[*face_idx];
                    let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                    let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                    let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                    let (s0, s1, s2) = if unlit {
                        (1.0, 1.0, 1.0)
                    } else {
                        (
                            shade_at_vertex(v0.normal),
                            shade_at_vertex(v1.normal),
                            shade_at_vertex(v2.normal),
                        )
                    };
                    let base_color = if unlit { fg_rgb } else { face.color };
                    Some((v0, v1, v2, base_color, s0, s1, s2))
                })
                .collect();

            let count = shaded_gouraud.len();
            for (v0, v1, v2, base_color, s0, s1, s2) in &shaded_gouraud {
                rasterize_triangle_gouraud(
                    canvas,
                    depth_buf,
                    virtual_w,
                    virtual_h,
                    *v0,
                    *v1,
                    *v2,
                    *base_color,
                    *s0,
                    *s1,
                    *s2,
                    shadow_colour,
                    midtone_colour,
                    highlight_colour,
                    tone_mix,
                    cel_levels,
                    latitude_bands,
                    latitude_band_depth,
                    params.terrain_color,
                    params.terrain_threshold,
                    params.marble_depth,
                    params.terrain_relief,
                    params.below_threshold_transparent,
                    biome_params,
                    planet_terrain_extra,
                    clipped_viewport.min_y,
                    clipped_viewport.max_y,
                    0,
                );
            }
            count
        } else {
            let shaded_faces: Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3])> =
                sorted_faces[..face_limit]
                    .par_iter()
                    .filter_map(|(_, face_idx)| {
                        let face = &mesh.faces[*face_idx];
                        let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                        let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                        let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                        if unlit {
                            return Some((v0, v1, v2, fg_rgb));
                        }
                        let shading = face_shading_with_specular(
                            v0.view,
                            v1.view,
                            v2.view,
                            face.ka,
                            face.ks,
                            face.ns,
                            light_dir_norm,
                            light_2_dir_norm,
                            half_dir_1,
                            half_dir_2,
                            light_2_intensity,
                            [light_1_x, light_point_y, light_1_z],
                            light_point_intensity * point_1_flicker,
                            [light_2_x, light_point_2_y, light_2_z],
                            light_point_2_intensity * point_2_flicker,
                            cel_levels,
                            tone_mix,
                            ambient,
                            view_dir,
                            light_point_falloff,
                            light_point_2_falloff,
                        );
                        let shaded_base = apply_shading(face.color, shading.0);
                        let toned_color = apply_tone_palette(
                            shaded_base,
                            shading.1,
                            shadow_colour,
                            midtone_colour,
                            highlight_colour,
                            tone_mix,
                        );
                        let shaded_color = apply_point_light_tint(
                            toned_color,
                            light_point_colour,
                            shading.2,
                            light_point_2_colour,
                            shading.3,
                        );
                        Some((v0, v1, v2, shaded_color))
                    })
                    .collect();

            let count = shaded_faces.len();
            for (v0, v1, v2, shaded_color) in &shaded_faces {
                rasterize_triangle(
                    canvas,
                    depth_buf,
                    virtual_w,
                    virtual_h,
                    *v0,
                    *v1,
                    *v2,
                    *shaded_color,
                    clipped_viewport.min_y,
                    clipped_viewport.max_y,
                );
            }
            count
        };

        OBJ_SORTED_FACE_INDEX.with(|v| *v.borrow_mut() = sorted_faces);
        if drawn_faces == 0 {
            let line_color = color_to_rgb(fg);
            for (a, b) in &mesh.edges {
                let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                    continue;
                };
                let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                    continue;
                };
                let x0 = pa.x.round() as i32;
                let y0 = pa.y.round() as i32;
                let x1 = pb.x.round() as i32;
                let y1 = pb.y.round() as i32;
                if let Some((cx0, cy0, cx1, cy1)) =
                    clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
                {
                    draw_line_flat(canvas, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color);
                }
            }
        }
    }
    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Render an OBJ mesh source into a pre-allocated shared canvas and depth buffer.
///
/// `canvas` and `depth_buf` must each be `target_w * target_h` elements.
/// Multiple calls may share the same buffers for cross-mesh depth testing.
#[allow(clippy::too_many_arguments)]
pub fn render_obj_to_shared_buffers(
    source: &str,
    target_w: u16,
    target_h: u16,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
) {
    let Some(root) = asset_root else {
        return;
    };
    let Some(mesh) = load_render_mesh(root, source) else {
        return;
    };
    if target_w < 2 || target_h < 2 {
        return;
    }
    let (virtual_w, virtual_h) = virtual_dimensions(target_w, target_h);
    if virtual_w < 2 || virtual_h < 2 {
        return;
    }

    render_mesh_projected(
        &mesh,
        virtual_w,
        virtual_h,
        params,
        wireframe,
        backface_cull,
        fg,
        canvas,
        depth_buf,
    );
}

// ── RGBA canvas compositing ───────────────────────────────────────────────────

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
    x: i32,
    y: i32,
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
        let base_vx = x * virt_mult.0 as i32;
        let base_vy = y * virt_mult.1 as i32;
        for vy in 0..virtual_h {
            for vx in 0..virtual_w {
                let Some(rgb) = px(vx, vy) else { continue };
                let px_x = base_vx + vx as i32;
                let px_y = base_vy + vy as i32;
                if px_x >= 0
                    && px_y >= 0
                    && (px_x as usize) < pc.width as usize
                    && (px_y as usize) < pc.height as usize
                {
                    let px_x = px_x as usize;
                    let px_y = px_y as usize;
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
            let tx = x + ox as i32;
            let ty = y + oy as i32;
            if tx < 0 || ty < 0 || tx >= buf.width as i32 || ty >= buf.height as i32 {
                continue;
            }
            buf.set(tx as u16, ty as u16, '█', rgb_to_color(rgb), bg_color);
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

// ── OBJ sprite dimensions ─────────────────────────────────────────────────────

pub fn obj_sprite_dimensions(
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
) -> (u16, u16) {
    match (width, height) {
        (Some(w), Some(h)) => (w.max(1), h.max(1)),
        (Some(w), None) => (w.max(1), 24),
        (None, Some(h)) => (64, h.max(1)),
        (None, None) => size.unwrap_or(SpriteSizePreset::Medium).obj_dimensions(),
    }
}

/// Render an OBJ mesh into a flat pixel canvas without writing to a terminal buffer.
/// Returns `(canvas, virtual_w, virtual_h)` on success, or `None` if assets are missing.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_obj_to_canvas(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 3]>>, u16, u16)> {
    let t_render = Instant::now();
    set_last_obj_raster_stats(ObjRasterStats::default());
    let root = asset_root?;
    let mesh = load_render_mesh(root, source)?;
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    if target_w < 2 || target_h < 2 {
        return None;
    }
    let (virtual_w, virtual_h) = virtual_dimensions(target_w, target_h);
    if virtual_w < 2 || virtual_h < 2 {
        return None;
    }

    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;
    let point_1_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_flicker_hz,
        params.light_point_flicker_depth,
        0.37,
    );
    let point_2_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_2_flicker_hz,
        params.light_point_2_flicker_depth,
        1.91,
    );
    // Light position: snap (teleport) wins over smooth orbit when snap_hz > 0.
    fn snap_angle(elapsed_s: f32, snap_hz: f32, seed: u32) -> f32 {
        let snap_index = (elapsed_s * snap_hz) as u32;
        let h = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
        (h as f32 / u32::MAX as f32) * std::f32::consts::TAU
    }

    let orbit_radius_1 = (params.light_point_x.powi(2) + params.light_point_z.powi(2))
        .sqrt()
        .max(0.0001);
    let orbit_radius_2 = (params.light_point_2_x.powi(2) + params.light_point_2_z.powi(2))
        .sqrt()
        .max(0.0001);

    let (light_1_x, light_1_z) = if params.light_point_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_snap_hz, 0x9e37_79b9);
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else if params.light_point_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else {
        (params.light_point_x, params.light_point_z)
    };

    let (light_2_x, light_2_z) = if params.light_point_2_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_2_snap_hz, 0x6c62_272d);
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else if params.light_point_2_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_2_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else {
        (params.light_point_2_x, params.light_point_2_z)
    };
    let yaw = (params.yaw_deg
        + params.rotation_y
        + params.rotate_y_deg_per_sec * elapsed_s
        + params.camera_look_yaw)
        .to_radians();
    let pitch = (params.pitch_deg + params.rotation_x + params.camera_look_pitch).to_radians();
    let roll = (params.roll_deg + params.rotation_z).to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let near_clip = params.near_clip.max(0.000001);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = virtual_w as f32 / virtual_h as f32;

    let viewport = Viewport {
        min_x: 0,
        min_y: 0,
        max_x: virtual_w as i32 - 1,
        max_y: virtual_h as i32 - 1,
    };
    let clip_row_min = (params.clip_y_min.clamp(0.0, 1.0) * virtual_h as f32).floor() as i32;
    let clip_row_max = (params.clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as i32 - 1;
    let clipped_viewport = Viewport {
        min_x: viewport.min_x,
        min_y: viewport.min_y.max(clip_row_min),
        max_x: viewport.max_x,
        max_y: viewport.max_y.min(clip_row_max),
    };
    if clipped_viewport.min_y > clipped_viewport.max_y {
        return None;
    }
    let center = mesh.center;
    let mut projected = OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(mesh.vertices.len());
        taken
    });

    let project_vertex = |v: &[f32; 3]| {
        let centered_raw = [
            (v[0] - center[0]) * model_scale,
            (v[1] - center[1]) * model_scale,
            (v[2] - center[2]) * model_scale,
        ];
        let terrain_noise_val =
            if params.terrain_color.is_some() || params.terrain_displacement > 0.0 {
                compute_terrain_noise_at(centered_raw, &params)
            } else {
                0.0
            };
        let centered = if params.terrain_displacement > 0.0 {
            displace_sphere_vertex(centered_raw, terrain_noise_val, params.terrain_displacement)
        } else {
            centered_raw
        };
        let rotated = rotate_xyz(centered, pitch, yaw, roll);
        let translated = [
            rotated[0] + params.object_translate_x,
            rotated[1] + params.object_translate_y,
            rotated[2] + params.object_translate_z,
        ];
        let rel = [
            translated[0] - params.camera_world_x,
            translated[1] - params.camera_world_y,
            translated[2] - params.camera_world_z,
        ];
        let cam_x = rel[0] * params.view_right_x
            + rel[1] * params.view_right_y
            + rel[2] * params.view_right_z
            - params.camera_pan_x;
        let cam_y =
            rel[0] * params.view_up_x + rel[1] * params.view_up_y + rel[2] * params.view_up_z
                - params.camera_pan_y;
        let view_z = rel[0] * params.view_forward_x
            + rel[1] * params.view_forward_y
            + rel[2] * params.view_forward_z;
        if view_z <= near_clip {
            return None;
        }
        let ndc_x = (cam_x / aspect) * inv_tan / view_z;
        let ndc_y = cam_y * inv_tan / view_z;
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return None;
        }
        Some(ProjectedVertex {
            x: (ndc_x + 1.0) * 0.5 * (virtual_w as f32 - 1.0),
            y: (1.0 - (ndc_y + 1.0) * 0.5) * (virtual_h as f32 - 1.0),
            depth: view_z,
            view: translated,
            normal: [0.0, 0.0, 1.0],
            local: centered,
            terrain_noise: terrain_noise_val,
        })
    };

    if mesh.vertices.len() > VERTEX_PARALLEL_THRESHOLD {
        mesh.vertices
            .par_iter()
            .map(project_vertex)
            .collect_into_vec(&mut projected);
    } else {
        projected.extend(mesh.vertices.iter().map(project_vertex));
    }

    if params.smooth_shading && !mesh.smooth_normals.is_empty() {
        for (i, pv_opt) in projected.iter_mut().enumerate() {
            if let Some(pv) = pv_opt.as_mut() {
                if let Some(&n) = mesh.smooth_normals.get(i) {
                    pv.normal = rotate_xyz(n, pitch, yaw, roll);
                }
            }
        }
    }

    let canvas_size = virtual_w as usize * virtual_h as usize;
    let mut canvas = OBJ_CANVAS.with(|c| {
        let mut v = c.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, None);
        taken
    });
    let mut triangles_processed = 0u32;
    let mut faces_drawn = 0u32;

    if wireframe {
        let line_color = color_to_rgb(fg);
        let mut depth_buf = OBJ_DEPTH.with(|d| {
            let mut v = d.borrow_mut();
            let mut taken = std::mem::take(&mut *v);
            taken.clear();
            taken.resize(canvas_size, f32::INFINITY);
            taken
        });

        let (depth_near, depth_far) = {
            let mut near = f32::INFINITY;
            let mut far = f32::NEG_INFINITY;
            for pv in projected.iter().flatten() {
                near = near.min(pv.depth);
                far = far.max(pv.depth);
            }
            if (far - near).abs() < f32::EPSILON {
                (near, near + 1.0)
            } else {
                (near, far)
            }
        };

        let mut drawn_edges = 0usize;
        for (a, b) in &mesh.edges {
            if drawn_edges > 12_000 {
                break;
            }
            let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                continue;
            };
            let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                continue;
            };
            let x0 = pa.x.round() as i32;
            let y0 = pa.y.round() as i32;
            let x1 = pb.x.round() as i32;
            let y1 = pb.y.round() as i32;
            if let Some((cx0, cy0, cx1, cy1)) =
                clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
            {
                let (cz0, cz1) =
                    clipped_depths(x0, y0, x1, y1, cx0, cy0, cx1, cy1, pa.depth, pb.depth);
                draw_line_depth(
                    &mut canvas,
                    &mut depth_buf,
                    virtual_w,
                    virtual_h,
                    cx0,
                    cy0,
                    cx1,
                    cy1,
                    line_color,
                    cz0,
                    cz1,
                    depth_near,
                    depth_far,
                );
                drawn_edges += 1;
            }
        }
        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth_buf);
    } else {
        let mut depth = OBJ_DEPTH.with(|d| {
            let mut v = d.borrow_mut();
            let mut taken = std::mem::take(&mut *v);
            taken.clear();
            taken.resize(canvas_size, f32::INFINITY);
            taken
        });
        let (light_dir_norm, light_2_dir_norm, view_dir) = normalized_light_and_view_dirs(&params);
        let half_dir_1 = normalize3([
            light_dir_norm[0] + view_dir[0],
            light_dir_norm[1] + view_dir[1],
            light_dir_norm[2] + view_dir[2],
        ]);
        let half_dir_2 = normalize3([
            light_2_dir_norm[0] + view_dir[0],
            light_2_dir_norm[1] + view_dir[1],
            light_2_dir_norm[2] + view_dir[2],
        ]);
        let mut sorted_faces = OBJ_SORTED_FACE_INDEX.with(|v| {
            let mut pool = v.borrow_mut();
            let mut taken = std::mem::take(&mut *pool);
            taken.clear();
            taken.reserve(mesh.faces.len());
            taken
        });
        for (face_idx, face) in mesh.faces.iter().enumerate() {
            let v0 = projected.get(face.indices[0]).and_then(|p| *p);
            let v1 = projected.get(face.indices[1]).and_then(|p| *p);
            let v2 = projected.get(face.indices[2]).and_then(|p| *p);
            let (Some(v0), Some(v1), Some(v2)) = (v0, v1, v2) else {
                continue;
            };
            let projected_area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
            if backface_cull && projected_area < 0.0 {
                continue;
            }
            if projected_area.abs() < MIN_PROJECTED_FACE_DOUBLE_AREA {
                continue;
            }
            let key = if params.depth_sort_faces {
                face_avg_depth(&projected, face)
            } else {
                0.0
            };
            sorted_faces.push((key, face_idx));
        }
        if params.depth_sort_faces {
            sorted_faces.sort_unstable_by(|a, b| {
                b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let face_limit = sorted_faces.len().min(MAX_OBJ_FACE_RENDER);
        triangles_processed = face_limit as u32;
        let light_point_y = params.light_point_y;
        let light_point_2_y = params.light_point_2_y;
        let light_2_intensity = params.light_2_intensity;
        let light_point_intensity = params.light_point_intensity;
        let light_point_2_intensity = params.light_point_2_intensity;
        let cel_levels = params.cel_levels;
        let tone_mix = params.tone_mix;
        let shadow_colour = params.shadow_colour;
        let midtone_colour = params.midtone_colour;
        let highlight_colour = params.highlight_colour;
        let light_point_colour = params.light_point_colour;
        let light_point_2_colour = params.light_point_2_colour;
        let unlit = params.unlit;
        let ambient = params.ambient;
        let light_point_falloff = params.light_point_falloff;
        let light_point_2_falloff = params.light_point_2_falloff;
        let smooth_shading = params.smooth_shading;
        let latitude_bands = params.latitude_bands;
        let latitude_band_depth = params.latitude_band_depth;
        let fg_rgb = color_to_rgb(fg);

        let biome_params = build_biome_params(&params, light_dir_norm, view_dir);
        let planet_terrain_extra = build_terrain_extra_params(&params);

        let drawn_faces = if smooth_shading {
            let ka_lum_ambient = ambient.max(params.ambient_floor);
            let light_2_strength = light_2_intensity.clamp(0.0, 2.0);

            let shade_at_vertex = |normal: [f32; 3]| -> f32 {
                let lambert_1 = dot3(normal, light_dir_norm).max(0.0);
                let lambert_2 = dot3(normal, light_2_dir_norm).max(0.0) * light_2_strength;
                let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
                (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
            };

            let mut shaded_gouraud = OBJ_SHADED_GOURAUD.with(|g| {
                let mut pool = g.borrow_mut();
                let mut taken = std::mem::take(&mut *pool);
                taken.clear();
                taken.reserve(face_limit);
                taken
            });
            shaded_gouraud.par_extend(sorted_faces[..face_limit].par_iter().filter_map(
                |(_, face_idx)| {
                    let face = &mesh.faces[*face_idx];
                    let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                    let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                    let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                    let (s0, s1, s2) = if unlit {
                        (1.0, 1.0, 1.0)
                    } else {
                        (
                            shade_at_vertex(v0.normal),
                            shade_at_vertex(v1.normal),
                            shade_at_vertex(v2.normal),
                        )
                    };
                    let base_color = if unlit { fg_rgb } else { face.color };
                    Some((v0, v1, v2, base_color, s0, s1, s2))
                },
            ));

            let row_w = virtual_w as usize;
            let num_strips = rayon::current_num_threads().max(1);
            let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
            let mut canvas_strips: Vec<(i32, &mut [Option<[u8; 3]>], &mut [f32])> = canvas
                .chunks_mut(strip_rows * row_w)
                .zip(depth.chunks_mut(strip_rows * row_w))
                .enumerate()
                .map(|(i, (cs, ds))| ((i * strip_rows) as i32, cs, ds))
                .collect();
            canvas_strips.par_iter_mut().for_each(|(strip_y0, cs, ds)| {
                let strip_y1 = *strip_y0 + (cs.len() / row_w) as i32 - 1;
                let clip_min = (*strip_y0).max(clipped_viewport.min_y);
                let clip_max = strip_y1.min(clipped_viewport.max_y);
                if clip_min > clip_max {
                    return;
                }
                for (v0, v1, v2, base_color, s0, s1, s2) in &shaded_gouraud {
                    rasterize_triangle_gouraud(
                        cs,
                        ds,
                        virtual_w,
                        virtual_h,
                        *v0,
                        *v1,
                        *v2,
                        *base_color,
                        *s0,
                        *s1,
                        *s2,
                        shadow_colour,
                        midtone_colour,
                        highlight_colour,
                        tone_mix,
                        cel_levels,
                        latitude_bands,
                        latitude_band_depth,
                        params.terrain_color,
                        params.terrain_threshold,
                        params.marble_depth,
                        params.terrain_relief,
                        params.below_threshold_transparent,
                        biome_params,
                        planet_terrain_extra,
                        clip_min,
                        clip_max,
                        *strip_y0,
                    );
                }
            });
            let count = shaded_gouraud.len();
            OBJ_SHADED_GOURAUD.with(|g| *g.borrow_mut() = shaded_gouraud);
            count
        } else {
            let mut shaded_faces = OBJ_SHADED_FLAT.with(|g| {
                let mut pool = g.borrow_mut();
                let mut taken = std::mem::take(&mut *pool);
                taken.clear();
                taken.reserve(face_limit);
                taken
            });
            shaded_faces.par_extend(sorted_faces[..face_limit].par_iter().filter_map(
                |(_, face_idx)| {
                    let face = &mesh.faces[*face_idx];
                    let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                    let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                    let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                    if unlit {
                        return Some((v0, v1, v2, fg_rgb));
                    }
                    let shading = face_shading_with_specular(
                        v0.view,
                        v1.view,
                        v2.view,
                        face.ka,
                        face.ks,
                        face.ns,
                        light_dir_norm,
                        light_2_dir_norm,
                        half_dir_1,
                        half_dir_2,
                        light_2_intensity,
                        [light_1_x, light_point_y, light_1_z],
                        light_point_intensity * point_1_flicker,
                        [light_2_x, light_point_2_y, light_2_z],
                        light_point_2_intensity * point_2_flicker,
                        cel_levels,
                        tone_mix,
                        ambient,
                        view_dir,
                        light_point_falloff,
                        light_point_2_falloff,
                    );
                    let shaded_base = apply_shading(face.color, shading.0);
                    let toned_color = apply_tone_palette(
                        shaded_base,
                        shading.1,
                        shadow_colour,
                        midtone_colour,
                        highlight_colour,
                        tone_mix,
                    );
                    let shaded_color = apply_point_light_tint(
                        toned_color,
                        light_point_colour,
                        shading.2,
                        light_point_2_colour,
                        shading.3,
                    );
                    Some((v0, v1, v2, shaded_color))
                },
            ));

            let count = shaded_faces.len();
            for (v0, v1, v2, shaded_color) in &shaded_faces {
                rasterize_triangle(
                    &mut canvas,
                    &mut depth,
                    virtual_w,
                    virtual_h,
                    *v0,
                    *v1,
                    *v2,
                    *shaded_color,
                    clipped_viewport.min_y,
                    clipped_viewport.max_y,
                );
            }
            OBJ_SHADED_FLAT.with(|g| *g.borrow_mut() = shaded_faces);
            count
        };
        OBJ_SORTED_FACE_INDEX.with(|v| *v.borrow_mut() = sorted_faces);

        faces_drawn = drawn_faces as u32;
        if drawn_faces == 0 {
            let line_color = color_to_rgb(fg);
            for (a, b) in &mesh.edges {
                let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                    continue;
                };
                let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                    continue;
                };
                let x0 = pa.x.round() as i32;
                let y0 = pa.y.round() as i32;
                let x1 = pb.x.round() as i32;
                let y1 = pb.y.round() as i32;
                if let Some((cx0, cy0, cx1, cy1)) =
                    clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
                {
                    draw_line_flat(
                        &mut canvas,
                        virtual_w,
                        virtual_h,
                        cx0,
                        cy0,
                        cx1,
                        cy1,
                        line_color,
                    );
                }
            }
        }
        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    }

    let mut halo_us = 0.0f32;
    if params.atmo_density > 0.0
        && (params.atmo_rayleigh_amount > 0.0
            || params.atmo_haze_amount > 0.0
            || params.atmo_absorption_amount > 0.0)
    {
        let ray_color = params
            .atmo_rayleigh_color
            .or(params.atmo_color)
            .unwrap_or([124, 200, 255]);
        let haze_color = params.atmo_haze_color.unwrap_or(ray_color);
        let absorption_color = params.atmo_absorption_color.unwrap_or([255, 170, 110]);
        let halo_strength = (params.atmo_density
            * (0.18
                + 0.46 * params.atmo_rayleigh_amount.clamp(0.0, 1.0)
                + 0.36 * params.atmo_haze_amount.clamp(0.0, 1.0))
            * params.atmo_limb_boost.max(0.0))
        .clamp(0.0, 0.98);
        let halo_width = (0.02
            + params.atmo_height * (0.58 + 1.05 * params.atmo_haze_amount.clamp(0.0, 1.0)))
        .clamp(0.02, 0.75);
        let halo_power = (2.4 - params.atmo_forward_scatter.clamp(0.0, 1.0) * 1.1
            + (1.0 - params.atmo_haze_amount.clamp(0.0, 1.0)) * 0.35)
            .clamp(0.55, 4.0);
        let t_halo = Instant::now();
        apply_atmosphere_halo_canvas(
            &mut canvas,
            virtual_w,
            virtual_h,
            ray_color,
            haze_color,
            absorption_color,
            halo_strength,
            halo_width,
            halo_power,
            params.atmo_rayleigh_amount,
            params.atmo_haze_amount,
            params.atmo_absorption_amount,
            params.atmo_forward_scatter,
            normalize3([
                params.light_direction_x,
                params.light_direction_y,
                params.light_direction_z,
            ]),
            [
                params.view_right_x,
                params.view_right_y,
                params.view_right_z,
            ],
            [params.view_up_x, params.view_up_y, params.view_up_z],
        );
        halo_us = t_halo.elapsed().as_micros() as f32;
    }

    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
    set_last_obj_raster_stats(ObjRasterStats {
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    accumulate_obj_raster_frame_metrics(ObjRasterFrameMetrics {
        rgb_us: t_render.elapsed().as_micros() as f32,
        rgba_us: 0.0,
        halo_us,
        rgb_calls: 1,
        rgba_calls: 0,
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    apply_canvas_grading(
        &mut canvas,
        params.exposure,
        params.gamma,
        params.tonemap,
        params.shadow_contrast,
    );
    Some((canvas, virtual_w, virtual_h))
}

#[allow(clippy::too_many_arguments)]
fn apply_atmosphere_halo_canvas(
    canvas: &mut [Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    ray_color: [u8; 3],
    haze_color: [u8; 3],
    absorption_color: [u8; 3],
    halo_strength: f32,
    halo_width: f32,
    halo_power: f32,
    rayleigh_amount: f32,
    haze_amount: f32,
    absorption_amount: f32,
    forward_scatter: f32,
    light_dir: [f32; 3],
    view_right: [f32; 3],
    view_up: [f32; 3],
) {
    if halo_strength <= 0.0 || halo_width <= 0.0 {
        return;
    }

    let w = virtual_w as usize;
    let h = virtual_h as usize;
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut count = 0usize;
    let mut edge_pixels = OBJ_HALO_EDGE_PIXELS.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken
    });
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    for y in 0..h {
        for x in 0..w {
            if canvas[y * w + x].is_none() {
                continue;
            }
            sum_x += x as f32;
            sum_y += y as f32;
            count += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);

            let left_empty = x == 0 || canvas[y * w + (x - 1)].is_none();
            let right_empty = x + 1 >= w || canvas[y * w + (x + 1)].is_none();
            let up_empty = y == 0 || canvas[(y - 1) * w + x].is_none();
            let down_empty = y + 1 >= h || canvas[(y + 1) * w + x].is_none();
            if left_empty || right_empty || up_empty || down_empty {
                edge_pixels.push((x as i32, y as i32));
            }
        }
    }
    if count == 0 || edge_pixels.is_empty() {
        OBJ_HALO_EDGE_PIXELS.with(|v| *v.borrow_mut() = edge_pixels);
        return;
    }

    let cx = sum_x / count as f32;
    let cy = sum_y / count as f32;
    let bbox_radius_x = ((max_x.saturating_sub(min_x) + 1) as f32) * 0.5;
    let bbox_radius_y = ((max_y.saturating_sub(min_y) + 1) as f32) * 0.5;
    let area_radius = (count as f32 / std::f32::consts::PI).sqrt();
    let radius = bbox_radius_x.max(bbox_radius_y).max(area_radius).max(1.0);
    let halo_px = (radius * halo_width.clamp(0.0, 1.0)).max(1.0);
    let halo_px_sq = halo_px * halo_px;
    let search = halo_px.ceil() as i32;
    let scan_min_x = min_x.saturating_sub(search as usize);
    let scan_min_y = min_y.saturating_sub(search as usize);
    let scan_max_x = (max_x + search as usize).min(w.saturating_sub(1));
    let scan_max_y = (max_y + search as usize).min(h.saturating_sub(1));

    let sx = dot3(light_dir, view_right);
    let sy = dot3(light_dir, view_up);
    let sl = (sx * sx + sy * sy).sqrt();
    let sun2d = if sl > 1e-5 {
        [sx / sl, -sy / sl]
    } else {
        [0.0, -1.0]
    };
    let haze_white = mix_rgb([255, 252, 246], haze_color, 0.20);
    let bright_ring = mix_rgb([255, 255, 252], haze_white, 0.42);
    let ray_tint = mix_rgb(haze_white, ray_color, 0.74);
    let sunset_tint = mix_rgb([255, 214, 156], absorption_color, 0.65);
    let haze_amount = haze_amount.clamp(0.0, 1.0);
    let rayleigh_amount = rayleigh_amount.clamp(0.0, 1.0);
    let absorption_amount = absorption_amount.clamp(0.0, 1.0);
    let forward_scatter = forward_scatter.clamp(0.0, 1.0);

    const CORE_LUT_SIZE: usize = 256;
    const TWILIGHT_LUT_SIZE: usize = 512;
    const DIST_LUT_SIZE: usize = 512;
    let core_center = 0.08 + 0.04 * (1.0 - haze_amount);
    let core_width = 0.08 + halo_width * 0.10;
    let twilight_width = 0.28 + 0.30 * (1.0 - forward_scatter);
    let skirt_exp = (halo_power * 0.55).max(0.3);
    let mut core_lut = [0.0f32; CORE_LUT_SIZE];
    let mut twilight_lut = [0.0f32; TWILIGHT_LUT_SIZE];
    let mut dist01_lut = [0.0f32; DIST_LUT_SIZE];
    let mut skirt_lut = [0.0f32; DIST_LUT_SIZE];
    for (i, slot) in core_lut.iter_mut().enumerate() {
        let t = i as f32 / (CORE_LUT_SIZE - 1) as f32;
        *slot = gaussian(t, core_center, core_width);
    }
    for (i, slot) in twilight_lut.iter_mut().enumerate() {
        let t = i as f32 / (TWILIGHT_LUT_SIZE - 1) as f32;
        let sun_alignment = t * 2.0 - 1.0;
        *slot = gaussian(sun_alignment, 0.0, twilight_width);
    }
    for i in 0..DIST_LUT_SIZE {
        let q = i as f32 / (DIST_LUT_SIZE - 1) as f32;
        let dist01 = q.sqrt();
        dist01_lut[i] = dist01;
        skirt_lut[i] = (1.0 - dist01).clamp(0.0, 1.0).powf(skirt_exp);
    }

    let scan_w = scan_max_x.saturating_sub(scan_min_x) + 1;
    let scan_h = scan_max_y.saturating_sub(scan_min_y) + 1;
    let scan_size = scan_w * scan_h;

    let mut occupied_scan = OBJ_HALO_OCCUPIED_SCAN.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.resize(scan_size, 0);
        taken
    });
    for y in scan_min_y..=scan_max_y {
        let row_offset = (y - scan_min_y) * scan_w;
        let canvas_row = y * w;
        for x in scan_min_x..=scan_max_x {
            if canvas[canvas_row + x].is_some() {
                occupied_scan[row_offset + (x - scan_min_x)] = 1;
            }
        }
    }

    let mut nearest_sq = OBJ_HALO_NEAREST_SQ.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.resize(scan_size, f32::INFINITY);
        taken
    });

    let edge_stride = if edge_pixels.len() > 7000 {
        6
    } else if edge_pixels.len() > 5000 {
        5
    } else if edge_pixels.len() > 3200 {
        4
    } else if edge_pixels.len() > 1800 {
        3
    } else if edge_pixels.len() > 900 {
        2
    } else {
        1
    };

    for &(ex, ey) in edge_pixels.iter().step_by(edge_stride) {
        let local_min_x = ((ex - search).max(scan_min_x as i32)) as usize;
        let local_max_x = ((ex + search).min(scan_max_x as i32)) as usize;
        let local_min_y = ((ey - search).max(scan_min_y as i32)) as usize;
        let local_max_y = ((ey + search).min(scan_max_y as i32)) as usize;
        for y in local_min_y..=local_max_y {
            let dy = y as i32 - ey;
            let row_offset = (y - scan_min_y) * scan_w;
            for x in local_min_x..=local_max_x {
                let local_idx = row_offset + (x - scan_min_x);
                if occupied_scan[local_idx] != 0 {
                    continue;
                }
                let dx = x as i32 - ex;
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq < nearest_sq[local_idx] {
                    nearest_sq[local_idx] = dist_sq;
                }
            }
        }
    }

    for y in scan_min_y..=scan_max_y {
        let row_offset = (y - scan_min_y) * scan_w;
        for x in scan_min_x..=scan_max_x {
            if occupied_scan[row_offset + (x - scan_min_x)] != 0 {
                continue;
            }
            let nearest_sq = nearest_sq[row_offset + (x - scan_min_x)];
            if !nearest_sq.is_finite() || nearest_sq > halo_px_sq {
                continue;
            }

            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dl = (dx * dx + dy * dy).sqrt().max(1e-5);
            let edge_dir = [dx / dl, dy / dl];
            let sun_alignment = edge_dir[0] * sun2d[0] + edge_dir[1] * sun2d[1];
            let day = smoothstep(-0.18, 0.92, sun_alignment);
            let q = (nearest_sq / halo_px_sq).clamp(0.0, 1.0);
            let dist_idx = ((q * (DIST_LUT_SIZE - 1) as f32) as usize).min(DIST_LUT_SIZE - 1);
            let dist01 = dist01_lut[dist_idx];
            let skirt = skirt_lut[dist_idx];
            let core_idx = ((dist01 * (CORE_LUT_SIZE - 1) as f32) as usize).min(CORE_LUT_SIZE - 1);
            let core_ring = core_lut[core_idx];
            let wide_scatter = skirt * (0.18 + 0.52 * day);
            let forward_lobe =
                skirt.powf(0.52) * smoothstep(0.10, 1.0, sun_alignment).powf(1.8) * forward_scatter;
            let twilight_t = ((sun_alignment + 1.0) * 0.5).clamp(0.0, 1.0);
            let twilight_idx =
                ((twilight_t * (TWILIGHT_LUT_SIZE - 1) as f32) as usize).min(TWILIGHT_LUT_SIZE - 1);
            let twilight_arc = twilight_lut[twilight_idx];
            let haze_alpha = (halo_strength
                * (0.12 + 0.88 * haze_amount)
                * (core_ring * (0.55 + 0.35 * day + 0.45 * forward_lobe) + wide_scatter * 0.18))
                .clamp(0.0, 0.97);
            let ray_alpha = (halo_strength
                * (0.10 + 0.90 * rayleigh_amount)
                * (wide_scatter + forward_lobe)
                * (0.35 + 0.85 * day))
                .clamp(0.0, 0.95);
            let sunset_alpha = (halo_strength
                * absorption_amount
                * twilight_arc
                * (0.10 + 0.90 * skirt)
                * (0.16 + 0.40 * day + 0.20 * forward_lobe))
                .clamp(0.0, 0.78);
            if haze_alpha <= 0.01 && ray_alpha <= 0.01 && sunset_alpha <= 0.01 {
                continue;
            }

            let mut out = [0, 0, 0];
            if haze_alpha > 0.0 {
                out = mix_rgb(out, bright_ring, haze_alpha);
            }
            if ray_alpha > 0.0 {
                out = mix_rgb(out, ray_tint, ray_alpha);
            }
            if sunset_alpha > 0.0 {
                out = mix_rgb(out, sunset_tint, sunset_alpha);
            }
            canvas[y * w + x] = Some(out);
        }
    }

    OBJ_HALO_EDGE_PIXELS.with(|v| *v.borrow_mut() = edge_pixels);
    OBJ_HALO_OCCUPIED_SCAN.with(|v| *v.borrow_mut() = occupied_scan);
    OBJ_HALO_NEAREST_SQ.with(|v| *v.borrow_mut() = nearest_sq);
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn gaussian(x: f32, center: f32, width: f32) -> f32 {
    let w = width.max(0.001);
    let z = (x - center) / w;
    (-0.5 * z * z).exp()
}

/// Convert an RGB canvas (from `render_obj_to_canvas`) to RGBA with alpha=255 for every painted pixel.
pub fn convert_canvas_to_rgba(rgb: Vec<Option<[u8; 3]>>) -> Vec<Option<[u8; 4]>> {
    rgb.into_iter()
        .map(|px| px.map(|[r, g, b]| [r, g, b, 255]))
        .collect()
}

/// Render an OBJ mesh into an RGBA canvas (Gouraud path only).
///
/// Produces `[u8; 4]` pixels: RGB + alpha.  When `cloud_alpha_softness > 0`,
/// pixels near the terrain threshold get smooth alpha edges (soft clouds).
/// Per-pixel noise is evaluated for cloud layers (instead of vertex-interpolated).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_obj_to_rgba_canvas(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 4]>>, u16, u16)> {
    let t_render = Instant::now();
    set_last_obj_raster_stats(ObjRasterStats::default());
    let root = asset_root?;
    let mesh = load_render_mesh(root, source)?;
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    if target_w < 2 || target_h < 2 {
        return None;
    }
    let (virtual_w, virtual_h) = virtual_dimensions(target_w, target_h);
    if virtual_w < 2 || virtual_h < 2 {
        return None;
    }

    let yaw = (params.yaw_deg + params.rotation_y).to_radians();
    let pitch = (params.pitch_deg + params.rotation_x).to_radians();
    let roll = (params.roll_deg + params.rotation_z).to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let near_clip = params.near_clip.max(0.000001);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = virtual_w as f32 / virtual_h as f32;
    let viewport = Viewport {
        min_x: 0,
        min_y: 0,
        max_x: virtual_w as i32 - 1,
        max_y: virtual_h as i32 - 1,
    };
    let clip_row_min = (params.clip_y_min.clamp(0.0, 1.0) * virtual_h as f32).floor() as i32;
    let clip_row_max = (params.clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as i32 - 1;
    let clipped_viewport = Viewport {
        min_x: viewport.min_x,
        min_y: viewport.min_y.max(clip_row_min),
        max_x: viewport.max_x,
        max_y: viewport.max_y.min(clip_row_max),
    };
    if clipped_viewport.min_y > clipped_viewport.max_y {
        return None;
    }

    let center = mesh.center;
    let mut projected = OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(mesh.vertices.len());
        taken
    });

    let project_vertex = |v: &[f32; 3]| {
        let centered_raw = [
            (v[0] - center[0]) * model_scale,
            (v[1] - center[1]) * model_scale,
            (v[2] - center[2]) * model_scale,
        ];
        let terrain_noise_val = if params.terrain_color.is_some()
            && params.cloud_alpha_softness <= 0.0
            || params.terrain_displacement > 0.0
        {
            compute_terrain_noise_at(centered_raw, &params)
        } else {
            0.0
        };
        let centered = if params.terrain_displacement > 0.0 {
            displace_sphere_vertex(centered_raw, terrain_noise_val, params.terrain_displacement)
        } else {
            centered_raw
        };
        let rotated = rotate_xyz(centered, pitch, yaw, roll);
        let translated = [
            rotated[0] + params.object_translate_x,
            rotated[1] + params.object_translate_y,
            rotated[2] + params.object_translate_z,
        ];
        let rel = [
            translated[0] - params.camera_world_x,
            translated[1] - params.camera_world_y,
            translated[2] - params.camera_world_z,
        ];
        let cam_x = rel[0] * params.view_right_x
            + rel[1] * params.view_right_y
            + rel[2] * params.view_right_z
            - params.camera_pan_x;
        let cam_y =
            rel[0] * params.view_up_x + rel[1] * params.view_up_y + rel[2] * params.view_up_z
                - params.camera_pan_y;
        let view_z = rel[0] * params.view_forward_x
            + rel[1] * params.view_forward_y
            + rel[2] * params.view_forward_z;
        if view_z <= near_clip {
            return None;
        }
        let ndc_x = (cam_x / aspect) * inv_tan / view_z;
        let ndc_y = cam_y * inv_tan / view_z;
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return None;
        }
        Some(ProjectedVertex {
            x: (ndc_x + 1.0) * 0.5 * (virtual_w as f32 - 1.0),
            y: (1.0 - (ndc_y + 1.0) * 0.5) * (virtual_h as f32 - 1.0),
            depth: view_z,
            view: translated,
            normal: [0.0, 0.0, 1.0],
            local: centered,
            terrain_noise: terrain_noise_val,
        })
    };

    if mesh.vertices.len() > VERTEX_PARALLEL_THRESHOLD {
        mesh.vertices
            .par_iter()
            .map(project_vertex)
            .collect_into_vec(&mut projected);
    } else {
        projected.extend(mesh.vertices.iter().map(project_vertex));
    }

    if !mesh.smooth_normals.is_empty() {
        for (i, pv_opt) in projected.iter_mut().enumerate() {
            if let Some(pv) = pv_opt.as_mut() {
                if let Some(&n) = mesh.smooth_normals.get(i) {
                    pv.normal = rotate_xyz(n, pitch, yaw, roll);
                }
            }
        }
    }

    let canvas_size = virtual_w as usize * virtual_h as usize;
    let mut canvas = OBJ_CANVAS_RGBA.with(|c| {
        let mut v = c.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, None);
        taken
    });
    let mut depth = OBJ_DEPTH.with(|d| {
        let mut v = d.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, f32::INFINITY);
        taken
    });

    let (light_dir_norm, light_2_dir_norm, view_dir) = normalized_light_and_view_dirs(&params);
    let fg_rgb = color_to_rgb(fg);
    let ka_lum_ambient = params.ambient.max(params.ambient_floor);
    let light_2_strength = params.light_2_intensity.clamp(0.0, 2.0);

    let shade_at_vertex = |normal: [f32; 3]| -> f32 {
        let lambert_1 = dot3(normal, light_dir_norm).max(0.0);
        let lambert_2 = dot3(normal, light_2_dir_norm).max(0.0) * light_2_strength;
        let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
        (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
    };

    let biome_params = build_biome_params(&params, light_dir_norm, view_dir);

    let mut sorted_faces = OBJ_SORTED_FACE_INDEX.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.reserve(mesh.faces.len());
        taken
    });
    for (face_idx, face) in mesh.faces.iter().enumerate() {
        let v0 = projected.get(face.indices[0]).and_then(|p| *p);
        let v1 = projected.get(face.indices[1]).and_then(|p| *p);
        let v2 = projected.get(face.indices[2]).and_then(|p| *p);
        let (Some(v0), Some(v1), Some(v2)) = (v0, v1, v2) else {
            continue;
        };
        let projected_area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
        if backface_cull && projected_area < 0.0 {
            continue;
        }
        if projected_area.abs() < MIN_PROJECTED_FACE_DOUBLE_AREA {
            continue;
        }
        let key = if params.depth_sort_faces {
            face_avg_depth(&projected, face)
        } else {
            0.0
        };
        sorted_faces.push((key, face_idx));
    }
    if params.depth_sort_faces {
        sorted_faces
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    }
    let face_limit = sorted_faces.len().min(MAX_OBJ_FACE_RENDER);
    let unlit = params.unlit;

    let mut shaded_gouraud = OBJ_SHADED_GOURAUD.with(|g| {
        let mut pool = g.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.reserve(face_limit);
        taken
    });
    shaded_gouraud.par_extend(
        sorted_faces[..face_limit]
            .par_iter()
            .filter_map(|(_, face_idx)| {
                let face = &mesh.faces[*face_idx];
                let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                let (s0, s1, s2) = if unlit {
                    (1.0, 1.0, 1.0)
                } else {
                    (
                        shade_at_vertex(v0.normal),
                        shade_at_vertex(v1.normal),
                        shade_at_vertex(v2.normal),
                    )
                };
                let base_color = if unlit { fg_rgb } else { face.color };
                Some((v0, v1, v2, base_color, s0, s1, s2))
            }),
    );

    let row_w = virtual_w as usize;
    let cel_levels = params.cel_levels;
    let num_strips = rayon::current_num_threads().max(1);
    let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
    let mut canvas_strips: Vec<(i32, &mut [Option<[u8; 4]>], &mut [f32])> = canvas
        .chunks_mut(strip_rows * row_w)
        .zip(depth.chunks_mut(strip_rows * row_w))
        .enumerate()
        .map(|(i, (cs, ds))| ((i * strip_rows) as i32, cs, ds))
        .collect();
    canvas_strips.par_iter_mut().for_each(|(strip_y0, cs, ds)| {
        let strip_y1 = *strip_y0 + (cs.len() / row_w) as i32 - 1;
        let clip_min = (*strip_y0).max(clipped_viewport.min_y);
        let clip_max = strip_y1.min(clipped_viewport.max_y);
        if clip_min > clip_max {
            return;
        }
        for (v0, v1, v2, base_color, s0, s1, s2) in &shaded_gouraud {
            rasterize_triangle_gouraud_rgba(
                cs,
                ds,
                virtual_w,
                virtual_h,
                *v0,
                *v1,
                *v2,
                *base_color,
                *s0,
                *s1,
                *s2,
                cel_levels,
                params.terrain_color,
                params.terrain_threshold,
                params.terrain_noise_scale,
                params.terrain_noise_octaves,
                params.below_threshold_transparent,
                params.cloud_alpha_softness,
                biome_params,
                clip_min,
                clip_max,
                *strip_y0,
                params.marble_depth,
                params.shadow_colour,
                params.midtone_colour,
                params.highlight_colour,
                params.tone_mix,
                params.latitude_bands,
                params.latitude_band_depth,
            );
        }
    });

    let faces_drawn_count = shaded_gouraud.len() as u32;
    OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
    OBJ_SHADED_GOURAUD.with(|g| *g.borrow_mut() = shaded_gouraud);
    OBJ_SORTED_FACE_INDEX.with(|v| *v.borrow_mut() = sorted_faces);
    set_last_obj_raster_stats(ObjRasterStats {
        triangles_processed: face_limit as u32,
        faces_drawn: faces_drawn_count,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    accumulate_obj_raster_frame_metrics(ObjRasterFrameMetrics {
        rgb_us: 0.0,
        rgba_us: t_render.elapsed().as_micros() as f32,
        halo_us: 0.0,
        rgb_calls: 0,
        rgba_calls: 1,
        triangles_processed: face_limit as u32,
        faces_drawn: faces_drawn_count,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    apply_rgba_canvas_grading(
        &mut canvas,
        params.exposure,
        params.gamma,
        params.tonemap,
        params.shadow_contrast,
    );
    Some((canvas, virtual_w, virtual_h))
}

struct ObjCanvasRenderRequest<'a> {
    source: &'a str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&'a AssetRoot>,
}

struct ObjCanvasPipeline;

impl<'a> Render3dPipeline<ObjCanvasRenderRequest<'a>, Option<(Vec<Option<[u8; 3]>>, u16, u16)>>
    for ObjCanvasPipeline
{
    fn render(
        &self,
        input: ObjCanvasRenderRequest<'a>,
    ) -> Option<(Vec<Option<[u8; 3]>>, u16, u16)> {
        render_obj_to_canvas(
            input.source,
            input.width,
            input.height,
            input.size,
            input.params,
            input.wireframe,
            input.backface_cull,
            input.fg,
            input.asset_root,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_obj_content(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    draw_char: char,
    fg: Color,
    bg: Color,
    asset_root: Option<&AssetRoot>,
    x: i32,
    y: i32,
    buf: &mut Buffer,
) {
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    let pipeline = ObjCanvasPipeline;
    let request = ObjCanvasRenderRequest {
        source,
        width,
        height,
        size,
        params,
        wireframe,
        backface_cull,
        fg,
        asset_root,
    };
    let Some((canvas, virtual_w, virtual_h)) = pipeline.render(request) else {
        return;
    };
    blit_color_canvas(
        buf,
        &canvas,
        virtual_w,
        virtual_h,
        target_w,
        target_h,
        x,
        y,
        wireframe,
        draw_char,
        fg,
        bg,
        0,
        virtual_h as usize,
    );
    OBJ_CANVAS.with(|c| *c.borrow_mut() = canvas);
}

/// Try to blit a pre-rendered OBJ sprite from the provided `ObjPrerenderedFrames`.
///
/// Checks animated frame cache first (snapped yaw lookup), then static pose tolerance.
/// Returns `true` if a cached frame was blitted; `false` → caller does live render.
#[allow(clippy::too_many_arguments)]
pub fn try_blit_prerendered(
    frames: Option<&ObjPrerenderedFrames>,
    sprite_id: &str,
    live_total_yaw: f32,
    current_pitch: f32,
    clip_y_min: f32,
    clip_y_max: f32,
    x: i32,
    y: i32,
    buf: &mut Buffer,
) -> bool {
    let Some(frames) = frames else {
        return false;
    };

    if let Some((canvas, virtual_w, virtual_h, target_w, target_h)) =
        frames.get_anim_canvas(sprite_id, live_total_yaw)
    {
        let clip_min_row = (clip_y_min.clamp(0.0, 1.0) * virtual_h as f32) as usize;
        let clip_max_row = (clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as usize;
        if clip_max_row > clip_min_row {
            blit_color_canvas(
                buf,
                canvas,
                virtual_w,
                virtual_h,
                target_w,
                target_h,
                x,
                y,
                false,
                '#',
                Color::White,
                Color::Reset,
                clip_min_row,
                clip_max_row,
            );
        }
        return true;
    }

    let Some(frame) = frames.get(sprite_id) else {
        return false;
    };

    if (live_total_yaw - frame.rendered_yaw).abs() >= 2.0 {
        return false;
    }
    if (current_pitch - frame.rendered_pitch).abs() >= 1.0 {
        return false;
    }

    let virtual_w = frame.virtual_w;
    let virtual_h = frame.virtual_h;
    let target_w = frame.target_w;
    let target_h = frame.target_h;
    let canvas = &frame.canvas;

    let clip_min_row = (clip_y_min.clamp(0.0, 1.0) * virtual_h as f32) as usize;
    let clip_max_row = (clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as usize;
    if clip_max_row <= clip_min_row {
        return true;
    }

    blit_color_canvas(
        buf,
        canvas,
        virtual_w,
        virtual_h,
        target_w,
        target_h,
        x,
        y,
        false,
        '#',
        Color::White,
        Color::Reset,
        clip_min_row,
        clip_max_row,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::{apply_atmosphere_halo_canvas, grade_rgb, obj_sprite_dimensions};
    use engine_core::scene::{SpriteSizePreset, TonemapOperator};

    #[test]
    fn atmosphere_halo_paints_pixels_outside_the_planet_silhouette() {
        let w = 48u16;
        let h = 48u16;
        let cx = 24i32;
        let cy = 24i32;
        let radius = 10i32;
        let mut canvas = vec![None; w as usize * h as usize];
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    canvas[y as usize * w as usize + x as usize] = Some([20, 40, 70]);
                }
            }
        }

        apply_atmosphere_halo_canvas(
            &mut canvas,
            w,
            h,
            [124, 200, 255],
            [236, 246, 255],
            [255, 214, 156],
            0.75,
            0.22,
            2.2,
            0.7,
            0.4,
            0.2,
            0.8,
            [1.0, 0.2, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );

        let outside_pixels = (0..h as i32)
            .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let dx = x - cx;
                let dy = y - cy;
                dx * dx + dy * dy > radius * radius
                    && canvas[y as usize * w as usize + x as usize].is_some()
            })
            .count();

        assert!(
            outside_pixels > 0,
            "expected halo pixels outside the original sphere"
        );
    }

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

    #[test]
    fn grading_changes_rgb_when_exposure_or_tonemap_is_applied() {
        let source = [180, 120, 60];
        let linear = grade_rgb(source, 1.0, 2.2, TonemapOperator::Linear, 1.0);
        let graded = grade_rgb(source, 1.35, 2.0, TonemapOperator::Reinhard, 1.0);

        assert_ne!(linear, graded);
        assert!(graded[0] >= graded[2]);
    }

    #[test]
    fn grading_changes_rgb_when_shadow_contrast_is_applied() {
        let neutral = grade_rgb([128, 128, 128], 1.0, 2.2, TonemapOperator::Linear, 1.0);
        let contrasted = grade_rgb([128, 128, 128], 1.0, 2.2, TonemapOperator::Linear, 1.8);

        assert!(contrasted[0] < neutral[0]);
    }
}
