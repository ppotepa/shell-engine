//! OBJ mesh rasterization pipeline: vertex projection, triangle rasterization,
//! shared-buffer rendering, and canvas-to-buffer blitting.
//!
//! This module owns the core render-domain logic for Scene3D work items,
//! independent of the compositor's frame assembly concerns.

use std::cell::RefCell;
use std::time::Instant;

use engine_asset::{load_render_mesh, ObjMesh};
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::TonemapOperator;
use crate::api::Render3dPipeline;
use crate::effects::passes::postprocess::apply_rgb_post_passes;
use crate::geom::clip::{clip_line_to_viewport, Viewport};
use crate::geom::raster::edge;
use crate::geom::types::ProjectedVertex;
use crate::pipeline::stages::classify::{classify_and_sort_faces_into, FaceClassificationConfig};
use crate::pipeline::stages::edges::{
    draw_outline_edges_flat, render_wireframe_rgb,
};
use crate::pipeline::stages::flat::render_flat_rgb_solid;
use crate::pipeline::stages::frame_context::FrameShadingContext;
use crate::pipeline::stages::gouraud::prepare_visible_gouraud_faces_into;
use crate::pipeline::stages::light_motion::animate_point_lights;
use crate::pipeline::stages::project::{
    project_mesh_with_viewport, project_vertices_into, ProjectionPoseConfig,
    ProjectionStageConfig, ProjectionStageInput, TerrainNoisePolicy,
};
use crate::pipeline::stages::raster_exec::{
    execute_gouraud_rgb_faces, execute_gouraud_rgb_faces_parallel_strips,
    execute_gouraud_rgba_faces_parallel_strips,
};
use crate::pipeline::stages::shade::{
    prepare_gouraud_faces_into,
};
use crate::prerender::ObjPrerenderedFrames;
use crate::shading::color_to_rgb;
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

#[inline]
fn take_projected_buffer(capacity: usize) -> Vec<Option<ProjectedVertex>> {
    OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(capacity);
        taken
    })
}

#[inline]
fn take_sorted_faces_buffer(capacity: usize) -> Vec<(f32, usize)> {
    OBJ_SORTED_FACE_INDEX.with(|v| {
        let mut pool = v.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.reserve(capacity);
        taken
    })
}

#[inline]
fn take_shaded_gouraud_buffer(capacity: usize) -> Vec<crate::pipeline::stages::shade::GouraudFace> {
    OBJ_SHADED_GOURAUD.with(|g| {
        let mut pool = g.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.reserve(capacity);
        taken
    })
}

#[inline]
fn take_shaded_flat_buffer(capacity: usize) -> Vec<crate::pipeline::stages::shade::FlatFace> {
    OBJ_SHADED_FLAT.with(|g| {
        let mut pool = g.borrow_mut();
        let mut taken = std::mem::take(&mut *pool);
        taken.clear();
        taken.reserve(capacity);
        taken
    })
}

#[inline]
fn take_rgb_canvas_buffer(canvas_size: usize) -> Vec<Option<[u8; 3]>> {
    OBJ_CANVAS.with(|c| {
        let mut v = c.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, None);
        taken
    })
}

#[inline]
fn take_rgba_canvas_buffer(canvas_size: usize) -> Vec<Option<[u8; 4]>> {
    OBJ_CANVAS_RGBA.with(|c| {
        let mut v = c.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, None);
        taken
    })
}

#[inline]
fn take_depth_buffer(canvas_size: usize) -> Vec<f32> {
    OBJ_DEPTH.with(|d| {
        let mut v = d.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, f32::INFINITY);
        taken
    })
}

#[inline]
fn release_sorted_faces_buffer(sorted_faces: Vec<(f32, usize)>) {
    OBJ_SORTED_FACE_INDEX.with(|v| *v.borrow_mut() = sorted_faces);
}

#[inline]
fn release_shaded_gouraud_buffer(
    shaded_gouraud: Vec<crate::pipeline::stages::shade::GouraudFace>,
) {
    OBJ_SHADED_GOURAUD.with(|g| *g.borrow_mut() = shaded_gouraud);
}

#[inline]
fn release_shaded_flat_buffer(shaded_faces: Vec<crate::pipeline::stages::shade::FlatFace>) {
    OBJ_SHADED_FLAT.with(|g| *g.borrow_mut() = shaded_faces);
}

#[inline]
fn finish_rgb_canvas_render(
    t_render: Instant,
    canvas: &mut [Option<[u8; 3]>],
    projected: Vec<Option<ProjectedVertex>>,
    triangles_processed: u32,
    faces_drawn: u32,
    virtual_w: u16,
    virtual_h: u16,
    params: &ObjRenderParams,
) {
    let post_pass_metrics = apply_rgb_post_passes(canvas, virtual_w, virtual_h, params);
    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
    set_last_obj_raster_stats(ObjRasterStats {
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    accumulate_obj_raster_frame_metrics(ObjRasterFrameMetrics {
        rgb_us: t_render.elapsed().as_micros() as f32,
        rgba_us: 0.0,
        halo_us: post_pass_metrics.halo_us,
        rgb_calls: 1,
        rgba_calls: 0,
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    apply_canvas_grading(
        canvas,
        params.exposure,
        params.gamma,
        params.tonemap,
        params.shadow_contrast,
    );
}

#[inline]
fn finish_rgba_canvas_render(
    t_render: Instant,
    canvas: &mut [Option<[u8; 4]>],
    projected: Vec<Option<ProjectedVertex>>,
    triangles_processed: u32,
    faces_drawn: u32,
    virtual_w: u16,
    virtual_h: u16,
    params: &ObjRenderParams,
) {
    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
    set_last_obj_raster_stats(ObjRasterStats {
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    accumulate_obj_raster_frame_metrics(ObjRasterFrameMetrics {
        rgb_us: 0.0,
        rgba_us: t_render.elapsed().as_micros() as f32,
        halo_us: 0.0,
        rgb_calls: 0,
        rgba_calls: 1,
        triangles_processed,
        faces_drawn,
        viewport_area_px: virtual_w as u32 * virtual_h as u32,
    });
    apply_rgba_canvas_grading(
        canvas,
        params.exposure,
        params.gamma,
        params.tonemap,
        params.shadow_contrast,
    );
}

fn render_gouraud_rgb_solid(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    virtual_w: u16,
    virtual_h: u16,
    clipped_viewport: Viewport,
    backface_cull: bool,
    depth_sort_faces: bool,
    frame_ctx: FrameShadingContext,
) -> (u32, u32) {
    let mut sorted_faces = take_sorted_faces_buffer(mesh.faces.len());
    let mut shaded_gouraud = take_shaded_gouraud_buffer(mesh.faces.len().min(MAX_OBJ_FACE_RENDER));
    let face_limit = prepare_visible_gouraud_faces_into(
        mesh,
        projected,
        backface_cull,
        depth_sort_faces,
        MIN_PROJECTED_FACE_DOUBLE_AREA,
        MAX_OBJ_FACE_RENDER,
        frame_ctx,
        &mut sorted_faces,
        &mut shaded_gouraud,
    );

    let row_w = virtual_w as usize;
    let num_strips = rayon::current_num_threads().max(1);
    let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
    let mut canvas_strips: Vec<(i32, &mut [Option<[u8; 3]>], &mut [f32])> = canvas
        .chunks_mut(strip_rows * row_w)
        .zip(depth.chunks_mut(strip_rows * row_w))
        .enumerate()
        .map(|(i, (cs, ds))| ((i * strip_rows) as i32, cs, ds))
        .collect();
    execute_gouraud_rgb_faces_parallel_strips(
        &mut canvas_strips,
        &shaded_gouraud,
        row_w,
        clipped_viewport.min_y,
        clipped_viewport.max_y,
        frame_ctx.gouraud_rgb_raster_context(virtual_w, virtual_h),
    );
    let faces_drawn = shaded_gouraud.len() as u32;
    release_shaded_gouraud_buffer(shaded_gouraud);
    release_sorted_faces_buffer(sorted_faces);
    (face_limit as u32, faces_drawn)
}

fn render_gouraud_rgba_solid(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    canvas: &mut [Option<[u8; 4]>],
    depth: &mut [f32],
    virtual_w: u16,
    virtual_h: u16,
    clipped_viewport: Viewport,
    backface_cull: bool,
    depth_sort_faces: bool,
    frame_ctx: FrameShadingContext,
) -> (u32, u32) {
    let mut sorted_faces = take_sorted_faces_buffer(mesh.faces.len());
    let mut shaded_gouraud = take_shaded_gouraud_buffer(mesh.faces.len().min(MAX_OBJ_FACE_RENDER));
    let face_limit = prepare_visible_gouraud_faces_into(
        mesh,
        projected,
        backface_cull,
        depth_sort_faces,
        MIN_PROJECTED_FACE_DOUBLE_AREA,
        MAX_OBJ_FACE_RENDER,
        frame_ctx,
        &mut sorted_faces,
        &mut shaded_gouraud,
    );

    let row_w = virtual_w as usize;
    let num_strips = rayon::current_num_threads().max(1);
    let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
    let mut canvas_strips: Vec<(i32, &mut [Option<[u8; 4]>], &mut [f32])> = canvas
        .chunks_mut(strip_rows * row_w)
        .zip(depth.chunks_mut(strip_rows * row_w))
        .enumerate()
        .map(|(i, (cs, ds))| ((i * strip_rows) as i32, cs, ds))
        .collect();
    execute_gouraud_rgba_faces_parallel_strips(
        &mut canvas_strips,
        &shaded_gouraud,
        row_w,
        clipped_viewport.min_y,
        clipped_viewport.max_y,
        frame_ctx.gouraud_rgba_raster_context(virtual_w, virtual_h),
    );

    let faces_drawn = shaded_gouraud.len() as u32;
    release_shaded_gouraud_buffer(shaded_gouraud);
    release_sorted_faces_buffer(sorted_faces);
    (face_limit as u32, faces_drawn)
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
    let point_lights = animate_point_lights(&params);
    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;

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
    let mut projected = take_projected_buffer(mesh.vertices.len());
    project_vertices_into(
        &mesh,
        &params,
        ProjectionStageInput {
            center,
            model_scale,
            pitch,
            yaw,
            roll,
            near_clip,
            aspect,
            inv_tan,
            virtual_w,
            virtual_h,
        },
        ProjectionStageConfig {
            terrain_noise_policy: TerrainNoisePolicy::SurfaceOrDisplacement,
            apply_smooth_normals: params.smooth_shading,
            parallel_threshold: 0,
        },
        &mut projected,
    );

    if wireframe {
        let line_color = color_to_rgb(fg);
        render_wireframe_rgb(
            mesh,
            &projected,
            canvas,
            depth_buf,
            virtual_w,
            virtual_h,
            clipped_viewport,
            line_color,
            12_000,
        );
    } else {
        let frame_ctx = FrameShadingContext::from_params(&params, fg);

        let mut sorted_faces = OBJ_SORTED_FACE_INDEX.with(|v| {
            let mut pool = v.borrow_mut();
            let mut taken = std::mem::take(&mut *pool);
            taken.clear();
            taken.reserve(mesh.faces.len());
            taken
        });
        let face_limit = classify_and_sort_faces_into(
            &mesh,
            &projected,
            FaceClassificationConfig {
                backface_cull,
                depth_sort_faces: params.depth_sort_faces,
                min_projected_face_double_area: MIN_PROJECTED_FACE_DOUBLE_AREA,
                max_faces: MAX_OBJ_FACE_RENDER,
            },
            &mut sorted_faces,
        );
        let light_point_y = params.light_point_y;
        let light_point_2_y = params.light_point_2_y;
        let light_point_intensity = params.light_point_intensity;
        let light_point_2_intensity = params.light_point_2_intensity;
        let smooth_shading = params.smooth_shading;

        let drawn_faces = if smooth_shading {
            let mut shaded_gouraud = Vec::with_capacity(face_limit);
            prepare_gouraud_faces_into(
                &mesh,
                &sorted_faces,
                face_limit,
                &projected,
                frame_ctx.unlit,
                frame_ctx.fg_rgb,
                |normal| frame_ctx.shade_at_vertex(normal),
                &mut shaded_gouraud,
            );

            let count = shaded_gouraud.len();
            execute_gouraud_rgb_faces(
                canvas,
                depth_buf,
                &shaded_gouraud,
                frame_ctx.gouraud_rgb_raster_context(virtual_w, virtual_h),
                clipped_viewport.min_y,
                clipped_viewport.max_y,
                0,
            );
            count
        } else {
            let mut shaded_faces = Vec::with_capacity(mesh.faces.len().min(MAX_OBJ_FACE_RENDER));
            let (_, faces_drawn) = render_flat_rgb_solid(
                &mesh,
                &projected,
                canvas,
                depth_buf,
                virtual_w,
                virtual_h,
                clipped_viewport.min_y,
                clipped_viewport.max_y,
                backface_cull,
                params.depth_sort_faces,
                MIN_PROJECTED_FACE_DOUBLE_AREA,
                MAX_OBJ_FACE_RENDER,
                frame_ctx.flat_stage_context(
                    [point_lights.point_1_x, light_point_y, point_lights.point_1_z],
                    light_point_intensity * point_lights.point_1_flicker,
                    [point_lights.point_2_x, light_point_2_y, point_lights.point_2_z],
                    light_point_2_intensity * point_lights.point_2_flicker,
                ),
                &mut sorted_faces,
                &mut shaded_faces,
            );
            faces_drawn as usize
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

type LoadedRenderTarget = (std::sync::Arc<ObjMesh>, u16, u16, u16, u16);

#[inline]
fn load_mesh_and_virtual_target(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    asset_root: Option<&AssetRoot>,
) -> Option<LoadedRenderTarget> {
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
    Some((mesh, target_w, target_h, virtual_w, virtual_h))
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
    let (mesh, _target_w, _target_h, virtual_w, virtual_h) =
        load_mesh_and_virtual_target(source, width, height, size, asset_root)?;

    let point_lights = animate_point_lights(&params);
    let mut projected = OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(mesh.vertices.len());
        taken
    });

    let clipped_viewport = project_mesh_with_viewport(
        &mesh,
        &params,
        virtual_w,
        virtual_h,
        ProjectionPoseConfig {
            include_animated_yaw: true,
            include_camera_look: true,
        },
        ProjectionStageConfig {
            terrain_noise_policy: TerrainNoisePolicy::SurfaceOrDisplacement,
            apply_smooth_normals: params.smooth_shading,
            parallel_threshold: VERTEX_PARALLEL_THRESHOLD,
        },
        &mut projected,
    )?;

    let canvas_size = virtual_w as usize * virtual_h as usize;
    let mut canvas = take_rgb_canvas_buffer(canvas_size);
    let mut triangles_processed = 0u32;
    let mut faces_drawn = 0u32;

    if wireframe {
        let line_color = color_to_rgb(fg);
        let mut depth_buf = take_depth_buffer(canvas_size);

        render_wireframe_rgb(
            &mesh,
            &projected,
            &mut canvas,
            &mut depth_buf,
            virtual_w,
            virtual_h,
            clipped_viewport,
            line_color,
            12_000,
        );
        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth_buf);
    } else {
        let mut depth = take_depth_buffer(canvas_size);
        let frame_ctx = FrameShadingContext::from_params(&params, fg);
        let mut sorted_faces = take_sorted_faces_buffer(mesh.faces.len());
        let light_point_y = params.light_point_y;
        let light_point_2_y = params.light_point_2_y;
        let light_point_intensity = params.light_point_intensity;
        let light_point_2_intensity = params.light_point_2_intensity;
        let smooth_shading = params.smooth_shading;

        let drawn_faces = if smooth_shading {
            let (triangles, faces_drawn) = render_gouraud_rgb_solid(
                &mesh,
                &projected,
                &mut canvas,
                &mut depth,
                virtual_w,
                virtual_h,
                clipped_viewport,
                backface_cull,
                params.depth_sort_faces,
                frame_ctx,
            );
            triangles_processed = triangles;
            faces_drawn as usize
        } else {
            let mut shaded_faces = take_shaded_flat_buffer(mesh.faces.len().min(MAX_OBJ_FACE_RENDER));
            let (triangles, faces_drawn) = render_flat_rgb_solid(
                &mesh,
                &projected,
                &mut canvas,
                &mut depth,
                virtual_w,
                virtual_h,
                clipped_viewport.min_y,
                clipped_viewport.max_y,
                backface_cull,
                params.depth_sort_faces,
                MIN_PROJECTED_FACE_DOUBLE_AREA,
                MAX_OBJ_FACE_RENDER,
                frame_ctx.flat_stage_context(
                    [point_lights.point_1_x, light_point_y, point_lights.point_1_z],
                    light_point_intensity * point_lights.point_1_flicker,
                    [point_lights.point_2_x, light_point_2_y, point_lights.point_2_z],
                    light_point_2_intensity * point_lights.point_2_flicker,
                ),
                &mut sorted_faces,
                &mut shaded_faces,
            );
            triangles_processed = triangles;
            release_shaded_flat_buffer(shaded_faces);
            faces_drawn as usize
        };
        if !smooth_shading {
            release_sorted_faces_buffer(sorted_faces);
        }

        faces_drawn = drawn_faces as u32;
        if drawn_faces == 0 {
            let line_color = color_to_rgb(fg);
            draw_outline_edges_flat(
                &mesh,
                &projected,
                &mut canvas,
                virtual_w,
                virtual_h,
                clipped_viewport,
                line_color,
            );
        }
        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    }

    finish_rgb_canvas_render(
        t_render,
        &mut canvas,
        projected,
        triangles_processed,
        faces_drawn,
        virtual_w,
        virtual_h,
        &params,
    );
    Some((canvas, virtual_w, virtual_h))
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
    let (mesh, _target_w, _target_h, virtual_w, virtual_h) =
        load_mesh_and_virtual_target(source, width, height, size, asset_root)?;

    let mut projected = take_projected_buffer(mesh.vertices.len());

    let clipped_viewport = project_mesh_with_viewport(
        &mesh,
        &params,
        virtual_w,
        virtual_h,
        ProjectionPoseConfig {
            include_animated_yaw: false,
            include_camera_look: false,
        },
        ProjectionStageConfig {
            terrain_noise_policy: TerrainNoisePolicy::SurfaceUnlessSoftCloudsOrDisplacement,
            apply_smooth_normals: true,
            parallel_threshold: VERTEX_PARALLEL_THRESHOLD,
        },
        &mut projected,
    )?;

    let canvas_size = virtual_w as usize * virtual_h as usize;
    let mut canvas = take_rgba_canvas_buffer(canvas_size);
    let mut depth = take_depth_buffer(canvas_size);

    let frame_ctx = FrameShadingContext::from_params(&params, fg);

    let (triangles_processed, faces_drawn_count) = render_gouraud_rgba_solid(
        &mesh,
        &projected,
        &mut canvas,
        &mut depth,
        virtual_w,
        virtual_h,
        clipped_viewport,
        backface_cull,
        params.depth_sort_faces,
        frame_ctx,
    );
    OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    finish_rgba_canvas_render(
        t_render,
        &mut canvas,
        projected,
        triangles_processed,
        faces_drawn_count,
        virtual_w,
        virtual_h,
        &params,
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
    use super::{grade_rgb, obj_sprite_dimensions};
    use crate::effects::passes::halo::{apply_halo_pass, HaloPassParams};
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

        apply_halo_pass(
            &mut canvas,
            w,
            h,
            HaloPassParams {
                ray_color: [124, 200, 255],
                haze_color: [236, 246, 255],
                absorption_color: [255, 214, 156],
                halo_strength: 0.75,
                halo_width: 0.22,
                halo_power: 2.2,
                rayleigh_amount: 0.7,
                haze_amount: 0.4,
                absorption_amount: 0.2,
                forward_scatter: 0.8,
                haze_night_leak: 0.0,
                night_glow: 0.0,
                night_glow_color: [90, 130, 255],
                light_intensity: 1.0,
                light_dir: [1.0, 0.2, 0.0],
                view_right: [1.0, 0.0, 0.0],
                view_up: [0.0, 1.0, 0.0],
                temporal_key: 0,
            },
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
