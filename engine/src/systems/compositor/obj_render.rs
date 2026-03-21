use std::cell::Cell;

use crossterm::style::Color;

use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::obj_frame_cache::{BakeCacheKey, ObjFrameCache};
use crate::scene::{SceneRenderedMode, SpriteSizePreset};

use super::obj_loader::{load_obj_mesh, ObjFace};

// Thread-local pointer to the current frame's ObjFrameCache (set by compositor, cleared after).
// SAFETY: only set during `with_frame_cache` and never accessed across threads.
thread_local! {
    static FRAME_CACHE_PTR: Cell<*const ObjFrameCache> = Cell::new(std::ptr::null());
}

/// Set the thread-local frame cache pointer for the duration of `f`.
pub(super) fn with_frame_cache<R>(cache: Option<&ObjFrameCache>, f: impl FnOnce() -> R) -> R {
    let ptr = cache.map(|c| c as *const _).unwrap_or(std::ptr::null());
    FRAME_CACHE_PTR.with(|cell| cell.set(ptr));
    let result = f();
    FRAME_CACHE_PTR.with(|cell| cell.set(std::ptr::null()));
    result
}

/// Borrow the current frame's `ObjFrameCache` if one was set.
fn current_frame_cache<'a>() -> Option<&'a ObjFrameCache> {
    FRAME_CACHE_PTR.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            None
        } else {
            // SAFETY: ptr was set from a reference valid for the duration of `with_frame_cache`.
            Some(unsafe { &*ptr })
        }
    })
}

#[derive(Debug, Clone, Copy)]
struct ProjectedVertex {
    x: f32,
    y: f32,
    depth: f32,
    view: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjRenderParams {
    pub scale: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    /// Static initial rotation offsets (x=pitch, y=yaw, z=roll) from `rotation-x/y/z` YAML.
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotate_y_deg_per_sec: f32,
    pub camera_distance: f32,
    pub fov_degrees: f32,
    pub near_clip: f32,
    pub light_direction_x: f32,
    pub light_direction_y: f32,
    pub light_direction_z: f32,
    pub light_2_direction_x: f32,
    pub light_2_direction_y: f32,
    pub light_2_direction_z: f32,
    pub light_2_intensity: f32,
    pub light_point_x: f32,
    pub light_point_y: f32,
    pub light_point_z: f32,
    pub light_point_intensity: f32,
    pub light_point_colour: Option<Color>,
    pub light_point_flicker_depth: f32,
    pub light_point_flicker_hz: f32,
    pub light_point_orbit_hz: f32,
    pub light_point_snap_hz: f32,
    pub light_point_2_x: f32,
    pub light_point_2_y: f32,
    pub light_point_2_z: f32,
    pub light_point_2_intensity: f32,
    pub light_point_2_colour: Option<Color>,
    pub light_point_2_flicker_depth: f32,
    pub light_point_2_flicker_hz: f32,
    pub light_point_2_orbit_hz: f32,
    pub light_point_2_snap_hz: f32,
    pub cel_levels: u8,
    pub shadow_colour: Option<Color>,
    pub midtone_colour: Option<Color>,
    pub highlight_colour: Option<Color>,
    pub tone_mix: f32,
    pub scene_elapsed_ms: u64,
    /// Camera pan offset in view-space units (applied before projection).
    pub camera_pan_x: f32,
    pub camera_pan_y: f32,
    /// Additional camera look rotation (accumulated from mouse). Yaw = horizontal, pitch = vertical.
    pub camera_look_yaw: f32,
    pub camera_look_pitch: f32,
    /// Vertical clip region (normalised 0.0–1.0). Rows outside [min, max) are skipped.
    pub clip_y_min: f32,
    pub clip_y_max: f32,
}

pub(super) fn obj_sprite_dimensions(
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
pub(crate) fn render_obj_to_canvas(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    mode: SceneRenderedMode,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 3]>>, u16, u16)> {
    let root = asset_root?;
    let mesh = load_obj_mesh(root, source)?;
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    if target_w < 2 || target_h < 2 {
        return None;
    }
    let (virtual_w, virtual_h) = virtual_dimensions(mode, target_w, target_h);
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
    // Snap: uses deterministic hash of (snap_index, seed) to pick a pseudo-random angle instantly.
    fn snap_angle(elapsed_s: f32, snap_hz: f32, seed: u32) -> f32 {
        let snap_index = (elapsed_s * snap_hz) as u32;
        let h = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
        (h as f32 / u32::MAX as f32) * std::f32::consts::TAU
    }

    let orbit_radius_1 = (params.light_point_x.powi(2) + params.light_point_z.powi(2)).sqrt().max(0.0001);
    let (light_1_x, light_1_z) = if params.light_point_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_snap_hz, 0x9e37_79b9);
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else if params.light_point_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else {
        (params.light_point_x, params.light_point_z)
    };

    let orbit_radius_2 = (params.light_point_2_x.powi(2) + params.light_point_2_z.powi(2)).sqrt().max(0.0001);
    let (light_2_x, light_2_z) = if params.light_point_2_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_2_snap_hz, 0x6c62_272d);
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else if params.light_point_2_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_2_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else {
        (params.light_point_2_x, params.light_point_2_z)
    };
    // Combine static rotation-y/x/z offsets with yaw-deg/pitch-deg/roll-deg + orbit + camera look.
    let yaw = (params.yaw_deg
        + params.rotation_y
        + params.rotate_y_deg_per_sec * elapsed_s
        + params.camera_look_yaw)
        .to_radians();
    let pitch = (params.pitch_deg + params.rotation_x + params.camera_look_pitch).to_radians();
    let roll = (params.roll_deg + params.rotation_z).to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let camera_distance = params.camera_distance.max(0.1);
    let near_clip = params.near_clip.max(0.000001);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = virtual_w as f32 / virtual_h as f32;

    let viewport = Viewport {
        min_x: 0,
        min_y: 0,
        max_x: virtual_w as i32 - 1,
        max_y: virtual_h as i32 - 1,
    };
    // Vertical clip region (normalised 0.0–1.0 → pixel rows).
    let clip_row_min = (params.clip_y_min.clamp(0.0, 1.0) * virtual_h as f32).floor() as i32;
    let clip_row_max = (params.clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as i32 - 1;
    let clipped_viewport = Viewport {
        min_x: viewport.min_x,
        min_y: viewport.min_y.max(clip_row_min),
        max_x: viewport.max_x,
        max_y: viewport.max_y.min(clip_row_max),
    };
    let projected: Vec<Option<ProjectedVertex>> = mesh
        .vertices
        .iter()
        .map(|v| {
            let centered = [
                (v[0] - mesh.center[0]) * model_scale,
                (v[1] - mesh.center[1]) * model_scale,
                (v[2] - mesh.center[2]) * model_scale,
            ];
            let rotated = rotate_xyz(centered, pitch, yaw, roll);
            // Apply camera pan: shift the scene in view-space (equivalent to moving camera).
            let panned = [
                rotated[0] - params.camera_pan_x,
                rotated[1] - params.camera_pan_y,
                rotated[2],
            ];
            let view_z = panned[2] + camera_distance;
            if view_z <= near_clip {
                return None;
            }
            let ndc_x = (panned[0] / aspect) * inv_tan / view_z;
            let ndc_y = panned[1] * inv_tan / view_z;
            if !ndc_x.is_finite() || !ndc_y.is_finite() {
                return None;
            }
            Some(ProjectedVertex {
                x: (ndc_x + 1.0) * 0.5 * (virtual_w as f32 - 1.0),
                y: (1.0 - (ndc_y + 1.0) * 0.5) * (virtual_h as f32 - 1.0),
                depth: view_z,
                view: panned,
            })
        })
        .collect();

    let mut canvas: Vec<Option<[u8; 3]>> = vec![None; virtual_w as usize * virtual_h as usize];
    if wireframe {
        let line_color = color_to_rgb(fg);
        let mut depth_buf = vec![f32::INFINITY; canvas.len()];

        // Depth range from all projected vertices for brightness mapping.
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
            if let Some((cx0, cy0, cx1, cy1)) = clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport) {
                let (cz0, cz1) = clipped_depths(x0, y0, x1, y1, cx0, cy0, cx1, cy1, pa.depth, pb.depth);
                draw_line_depth(
                    &mut canvas,
                    &mut depth_buf,
                    virtual_w,
                    virtual_h,
                    cx0, cy0, cx1, cy1,
                    line_color,
                    cz0, cz1,
                    depth_near, depth_far,
                );
                drawn_edges += 1;
            }
        }
    } else {
        let mut depth = vec![f32::INFINITY; canvas.len()];
        let mut drawn_faces = 0usize;
        // Pre-compute normalized light directions once per render (not per face).
        let light_dir_norm = normalize3([params.light_direction_x, params.light_direction_y, params.light_direction_z]);
        let light_2_dir_norm = normalize3([params.light_2_direction_x, params.light_2_direction_y, params.light_2_direction_z]);
        // Sort faces back-to-front for correct painter's-algorithm blending when
        // depth-buffering alone isn't enough (avoids most z-fighting glitches).
        // Pre-compute depth keys to avoid redundant work inside the comparator.
        let mut sorted_faces: Vec<(f32, &ObjFace)> = mesh
            .faces
            .iter()
            .map(|f| (face_avg_depth(&projected, f), f))
            .collect();
        sorted_faces.sort_unstable_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });

        for &(_, face) in &sorted_faces {
            if drawn_faces > 20_000 {
                break;
            }
            let Some(v0) = projected.get(face.indices[0]).and_then(|p| *p) else {
                continue;
            };
            let Some(v1) = projected.get(face.indices[1]).and_then(|p| *p) else {
                continue;
            };
            let Some(v2) = projected.get(face.indices[2]).and_then(|p| *p) else {
                continue;
            };
            // Back-face culling: skip faces whose screen-space winding is clockwise.
            // Opt-in via `backface-cull: true` in YAML; disabled by default for OBJ
            // files with inconsistent winding.
            if backface_cull {
                let screen_area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
                if screen_area < 0.0 {
                    continue;
                }
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
                params.light_2_intensity,
                [light_1_x, params.light_point_y, light_1_z],
                params.light_point_intensity * point_1_flicker,
                [
                    light_2_x,
                    params.light_point_2_y,
                    light_2_z,
                ],
                params.light_point_2_intensity * point_2_flicker,
                params.cel_levels,
                params.tone_mix,
            );
            let shaded_base = apply_shading(face.color, shading.0);
            let toned_color = apply_tone_palette(
                shaded_base,
                shading.1,
                params.shadow_colour,
                params.midtone_colour,
                params.highlight_colour,
                params.tone_mix,
            );
            let shaded_color = apply_point_light_tint(
                toned_color,
                params.light_point_colour,
                shading.2,
                params.light_point_2_colour,
                shading.3,
            );
            rasterize_triangle(
                &mut canvas,
                &mut depth,
                virtual_w,
                virtual_h,
                v0,
                v1,
                v2,
                shaded_color,
                clipped_viewport.min_y,
                clipped_viewport.max_y,
            );
            drawn_faces += 1;
        }

        // Fallback if model has no valid faces/materials.
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
                if let Some((cx0, cy0, cx1, cy1)) = clip_line_to_viewport(x0, y0, x1, y1, clipped_viewport)
                {
                    draw_line_flat(
                        &mut canvas,
                        virtual_w,
                        virtual_h,
                        cx0, cy0, cx1, cy1,
                        line_color,
                    );
                }
            }
        }
    }

    Some((canvas, virtual_w, virtual_h))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_obj_content(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    mode: SceneRenderedMode,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    draw_char: char,
    fg: Color,
    bg: Color,
    asset_root: Option<&AssetRoot>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    let (virtual_w, virtual_h) = virtual_dimensions(mode, target_w, target_h);

    // Fast path: try the pre-baked frame cache first.
    if let Some(cache) = current_frame_cache() {
        // Total yaw = rotation_y (static offset) + yaw_deg (from animation/YAML)
        let total_yaw = params.rotation_y + params.yaw_deg;
        let yaw_step = ObjFrameCache::snap_yaw(total_yaw);
        let key = BakeCacheKey {
            source: source.to_string(),
            wireframe,
            yaw_step,
        };
        if let Some(canvas) = cache.get(&key) {
            let clip_min = params.clip_y_min;
            let clip_max = params.clip_y_max;
            if clip_min <= 0.0 && clip_max >= 1.0 {
                blit_color_canvas(
                    buf, mode, canvas, virtual_w, virtual_h, target_w, target_h, x, y,
                    wireframe, draw_char, fg, bg,
                );
            } else {
                let clip_min_row = (clip_min.clamp(0.0, 1.0) * virtual_h as f32) as usize;
                let clip_max_row =
                    (clip_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as usize;
                let vw = virtual_w as usize;
                let mut masked: Vec<Option<[u8; 3]>> = (**canvas).clone();
                let canvas_len = masked.len();
                for vy in 0..virtual_h as usize {
                    if vy < clip_min_row || vy >= clip_max_row {
                        let row_start = vy * vw;
                        if row_start >= canvas_len {
                            break;
                        }
                        let row_end = (row_start + vw).min(canvas_len);
                        for px in &mut masked[row_start..row_end] {
                            *px = None;
                        }
                    }
                }
                blit_color_canvas(
                    buf, mode, &masked, virtual_w, virtual_h, target_w, target_h, x, y,
                    wireframe, draw_char, fg, bg,
                );
            }
            return;
        }
        // Cache exists but key not found — prerender scene should never fall through to live 3D.
        engine_core::logging::warn(
            "engine.obj_render",
            format!("cache miss for {source} wire={wireframe} yaw={yaw_step} — skipping live render"),
        );
        return;
    }

    // Fallback: live render.
    let Some((canvas, virtual_w, virtual_h)) = render_obj_to_canvas(
        source, width, height, size, mode, params, wireframe, backface_cull, fg, asset_root,
    ) else {
        return;
    };
    blit_color_canvas(
        buf, mode, &canvas, virtual_w, virtual_h, target_w, target_h, x, y, wireframe, draw_char,
        fg, bg,
    );
}

/// Render a pre-baked OBJ sprite from the frame cache.
///
/// Looks up `(source, wireframe, snap_yaw(rotation_y + yaw_deg))` in the thread-local cache.
/// Applies `clip_y_min / clip_y_max` row masking (used for the scanline materialize effect).
/// Silently does nothing if the cache is absent or the key is missing.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_baked_obj_content(
    source: &str,
    wireframe: bool,
    width: Option<u16>,
    height: Option<u16>,
    mode: SceneRenderedMode,
    yaw_deg: f32,
    rotation_y: f32,
    clip_y_min: f32,
    clip_y_max: f32,
    draw_char: char,
    fg: Color,
    bg: Color,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let Some(cache) = current_frame_cache() else {
        return;
    };
    let total_yaw = rotation_y + yaw_deg;
    let yaw_step = ObjFrameCache::snap_yaw(total_yaw);
    let key = BakeCacheKey {
        source: source.to_string(),
        wireframe,
        yaw_step,
    };
    let Some(canvas) = cache.get(&key) else {
        return;
    };

    let (target_w, target_h) = obj_sprite_dimensions(width, height, None);
    let (virtual_w, virtual_h) = virtual_dimensions(mode, target_w, target_h);

    let clip_min_row = (clip_y_min.clamp(0.0, 1.0) * virtual_h as f32) as usize;
    let clip_max_row = (clip_y_max.clamp(0.0, 1.0) * virtual_h as f32).ceil() as usize;

    if clip_min_row == 0 && clip_max_row >= virtual_h as usize {
        blit_color_canvas(
            buf, mode, canvas, virtual_w, virtual_h, target_w, target_h, x, y, wireframe,
            draw_char, fg, bg,
        );
    } else {
        let mut masked: Vec<Option<[u8; 3]>> = (**canvas).clone();
        let vw = virtual_w as usize;
        let canvas_len = masked.len();
        for vy in 0..virtual_h as usize {
            if vy < clip_min_row || vy >= clip_max_row {
                let row_start = vy * vw;
                if row_start >= canvas_len {
                    break;
                }
                let row_end = (row_start + vw).min(canvas_len);
                for px in &mut masked[row_start..row_end] {
                    *px = None;
                }
            }
        }
        blit_color_canvas(
            buf, mode, &masked, virtual_w, virtual_h, target_w, target_h, x, y, wireframe,
            draw_char, fg, bg,
        );
    }
}

#[derive(Clone, Copy)]
struct Viewport {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

fn virtual_dimensions(mode: SceneRenderedMode, target_w: u16, target_h: u16) -> (u16, u16) {
    match mode {
        SceneRenderedMode::Cell => (target_w, target_h),
        SceneRenderedMode::HalfBlock => (target_w, target_h.saturating_mul(2)),
        SceneRenderedMode::QuadBlock => (target_w.saturating_mul(2), target_h.saturating_mul(2)),
        SceneRenderedMode::Braille => (target_w.saturating_mul(2), target_h.saturating_mul(4)),
    }
}

/// Interpolate depths at clipped line endpoints using parametric projection.
fn clipped_depths(
    x0: i32, y0: i32, x1: i32, y1: i32,
    cx0: i32, cy0: i32, cx1: i32, cy1: i32,
    z0: f32, z1: f32,
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
fn draw_line_flat(
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
fn draw_line_depth(
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
            let t = if total_steps > 0.0 { step / total_steps } else { 0.0 };
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
fn rasterize_triangle(
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

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;
    let min_y = min_y.max(clip_min_y);
    let max_y = max_y.min(clip_max_y);

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let y = py as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) / area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) / area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) / area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = py as usize * w as usize + px as usize;
            if z < depth[idx] {
                depth[idx] = z;
                canvas[idx] = Some(color);
            }
        }
    }
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn face_avg_depth(projected: &[Option<ProjectedVertex>], face: &ObjFace) -> f32 {
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

fn face_shading_with_specular(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    ka: [f32; 3],
    ks: f32,
    ns: f32,
    light_dir: [f32; 3],
    light_2_dir: [f32; 3],
    light_2_intensity: f32,
    light_point: [f32; 3],
    light_point_intensity: f32,
    light_point_2: [f32; 3],
    light_point_2_intensity: f32,
    cel_levels: u8,
    tone_mix: f32,
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
    let point_dist = (to_point[0] * to_point[0] + to_point[1] * to_point[1] + to_point[2] * to_point[2])
        .sqrt()
        .max(0.0001);
    let point_atten = 1.0 / (1.0 + 0.7 * point_dist * point_dist);
    let to_point_2 = sub3(light_point_2, centroid);
    let point_2_dir = normalize3(to_point_2);
    let point_2_dist = (to_point_2[0] * to_point_2[0]
        + to_point_2[1] * to_point_2[1]
        + to_point_2[2] * to_point_2[2])
        .sqrt()
        .max(0.0001);
    let point_2_atten = 1.0 / (1.0 + 0.7 * point_2_dist * point_2_dist);
    // Two-sided Lambert: abs() keeps shading stable for OBJ files with inconsistent winding.
    let lambert_1 = dot3(normal, light_dir).abs();
    let lambert_2 = dot3(normal, light_2_dir).abs() * light_2_strength;
    let lambert_point = dot3(normal, point_dir).abs() * point_strength * point_atten;
    let lambert_point_2 = dot3(normal, point_2_dir).abs() * point_2_strength * point_2_atten;
    let lambert = (lambert_1 + lambert_2 + lambert_point + lambert_point_2).clamp(0.0, 1.0);
    // When tone_mix is high we intentionally reduce material influence so different OBJ
    // material packs still produce consistent silhouette lighting.
    let material_influence = (1.0 - tone_mix.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let ka_lum_material = (ka[0] * 0.299 + ka[1] * 0.587 + ka[2] * 0.114).clamp(0.03, 0.25);
    let ka_lum = 0.06 + (ka_lum_material - 0.06) * material_influence;
    // view_dir is constant [0, 0, -1] — already unit length, skip normalize.
    const VIEW_DIR: [f32; 3] = [0.0, 0.0, -1.0];
    let half_dir_1 = normalize3([
        light_dir[0] + VIEW_DIR[0],
        light_dir[1] + VIEW_DIR[1],
        light_dir[2] + VIEW_DIR[2],
    ]);
    let half_dir_2 = normalize3([
        light_2_dir[0] + VIEW_DIR[0],
        light_2_dir[1] + VIEW_DIR[1],
        light_2_dir[2] + VIEW_DIR[2],
    ]);
    let half_dir_point = normalize3([
        point_dir[0] + VIEW_DIR[0],
        point_dir[1] + VIEW_DIR[1],
        point_dir[2] + VIEW_DIR[2],
    ]);
    let half_dir_point_2 = normalize3([
        point_2_dir[0] + VIEW_DIR[0],
        point_2_dir[1] + VIEW_DIR[1],
        point_2_dir[2] + VIEW_DIR[2],
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

fn quantize_shade(value: f32, levels: u8) -> f32 {
    if levels <= 1 {
        return value.clamp(0.0, 1.0);
    }
    let levels = levels.clamp(2, 8) as f32;
    let steps = levels - 1.0;
    let v = value.clamp(0.0, 1.0);
    (v * steps).round() / steps
}

fn apply_shading(rgb: [u8; 3], shade: f32) -> [u8; 3] {
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

fn apply_tone_palette(
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

fn apply_point_light_tint(
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

fn flicker_multiplier(elapsed_s: f32, hz: f32, depth: f32, phase: f32) -> f32 {
    let d = depth.clamp(0.0, 1.0);
    if d <= f32::EPSILON {
        return 1.0;
    }
    let rate = hz.clamp(0.1, 40.0);
    let base = ((elapsed_s * std::f32::consts::TAU * rate + phase).sin() * 0.5 + 0.5).powf(1.5);
    let chatter =
        ((elapsed_s * std::f32::consts::TAU * (rate * 2.31) + phase * 1.7).sin().abs()).powf(2.3);
    let pulse = (base * 0.65 + chatter * 0.35).clamp(0.0, 1.0);
    ((1.0 - d) + d * pulse).clamp(0.0, 1.0)
}

fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

/// Convert sRGB u8 → linear f32.
fn srgb_to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear f32 → sRGB u8.
fn linear_to_srgb(v: f32) -> u8 {
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Boost saturation of a linear-space RGB triplet by `factor`.
fn saturate(lin: [f32; 3], factor: f32) -> [f32; 3] {
    let lum = lin[0] * 0.299 + lin[1] * 0.587 + lin[2] * 0.114;
    [
        (lum + (lin[0] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[1] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[2] - lum) * factor).clamp(0.0, 1.0),
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(clippy::too_many_arguments)]
fn blit_color_canvas(
    buf: &mut Buffer,
    mode: SceneRenderedMode,
    canvas: &[Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    wireframe: bool,
    draw_char: char,
    fg: Color,
    bg: Color,
) {
    let px = |vx: u16, vy: u16| -> Option<[u8; 3]> {
        if vx >= virtual_w || vy >= virtual_h {
            return None;
        }
        canvas
            .get(vy as usize * virtual_w as usize + vx as usize)
            .copied()
            .unwrap_or(None)
    };
    let bg_rgb = color_to_rgb(bg);
    let bg_color = rgb_to_color(bg_rgb);

    match mode {
        SceneRenderedMode::Cell => {
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
        SceneRenderedMode::HalfBlock => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let top = px(ox, oy * 2);
                    let bottom = px(ox, oy * 2 + 1);
                    let (symbol, fg_out, bg_out) = match (top, bottom) {
                        (None, None) => continue,
                        (Some(t), None) => ('▀', rgb_to_color(t), bg_color),
                        (None, Some(b)) => ('▄', rgb_to_color(b), bg_color),
                        (Some(t), Some(b)) => ('▀', rgb_to_color(t), rgb_to_color(b)),
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_out);
                }
            }
        }
        SceneRenderedMode::QuadBlock => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let mut mask = 0u8;
                    let mut cols = Vec::new();
                    if let Some(c) = px(ox * 2, oy * 2) {
                        mask |= 0b0001;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2) {
                        mask |= 0b0010;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2, oy * 2 + 1) {
                        mask |= 0b0100;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2 + 1) {
                        mask |= 0b1000;
                        cols.push(c);
                    }
                    let Some(symbol) = quadrant_char(mask) else {
                        continue;
                    };
                    let fg_out = if cols.is_empty() {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols))
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
        SceneRenderedMode::Braille => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let sx = ox * 2;
                    let sy = oy * 4;
                    let mut mask = 0u8;
                    let mut cols = Vec::new();
                    if let Some(c) = px(sx, sy) {
                        mask |= 1 << 0;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 1) {
                        mask |= 1 << 1;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 2) {
                        mask |= 1 << 2;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy) {
                        mask |= 1 << 3;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 1) {
                        mask |= 1 << 4;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 2) {
                        mask |= 1 << 5;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 3) {
                        mask |= 1 << 6;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 3) {
                        mask |= 1 << 7;
                        cols.push(c);
                    }
                    let Some(symbol) = braille_char(mask) else {
                        continue;
                    };
                    let fg_out = if cols.is_empty() {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols))
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
    }
}

fn average_rgb(colours: &[[u8; 3]]) -> [u8; 3] {
    if colours.is_empty() {
        return [255, 255, 255];
    }
    let mut rs = 0u32;
    let mut gs = 0u32;
    let mut bs = 0u32;
    for c in colours {
        rs += c[0] as u32;
        gs += c[1] as u32;
        bs += c[2] as u32;
    }
    let len = colours.len() as u32;
    [(rs / len) as u8, (gs / len) as u8, (bs / len) as u8]
}

fn color_to_rgb(color: Color) -> [u8; 3] {
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

fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

fn clip_line_to_viewport(
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

fn out_code(x: i32, y: i32, vp: Viewport) -> u8 {
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

fn intersect_vertical(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> Option<(i32, i32)> {
    let dx = x1 - x0;
    if dx == 0 {
        return None;
    }
    let t = (x - x0) as f32 / dx as f32;
    let y = y0 as f32 + t * (y1 - y0) as f32;
    Some((x, y.round() as i32))
}

fn intersect_horizontal(x0: i32, y0: i32, x1: i32, y1: i32, y: i32) -> Option<(i32, i32)> {
    let dy = y1 - y0;
    if dy == 0 {
        return None;
    }
    let t = (y - y0) as f32 / dy as f32;
    let x = x0 as f32 + t * (x1 - x0) as f32;
    Some((x.round() as i32, y))
}

fn quadrant_char(mask: u8) -> Option<char> {
    match mask {
        0 => None,
        1 => Some('▘'),
        2 => Some('▝'),
        3 => Some('▀'),
        4 => Some('▖'),
        5 => Some('▌'),
        6 => Some('▞'),
        7 => Some('▛'),
        8 => Some('▗'),
        9 => Some('▚'),
        10 => Some('▐'),
        11 => Some('▜'),
        12 => Some('▄'),
        13 => Some('▙'),
        14 => Some('▟'),
        15 => Some('█'),
        _ => None,
    }
}

fn braille_char(mask: u8) -> Option<char> {
    if mask == 0 {
        None
    } else {
        char::from_u32(0x2800 + mask as u32)
    }
}

fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
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
    use crate::scene::SpriteSizePreset;

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
