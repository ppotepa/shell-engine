use std::cell::{Cell, RefCell};

use engine_core::color::Color;
use engine_render_3d::api::Render3dPipeline;
use rayon::prelude::*;

use crate::obj_prerender::ObjPrerenderedFrames;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::scene::SpriteSizePreset;

use super::obj_loader::{ObjFace, ObjMesh};
use super::obj_render_helpers::*;
pub use super::obj_render_helpers::{
    blit_color_canvas, blit_rgba_canvas, composite_rgba_over, virtual_dimensions,
};
mod atmo_shell;
mod mesh_source;
mod params;
mod setup;
mod terrain_eval;
use atmo_shell::render_atmo_shell_pass;
use mesh_source::get_or_load_obj_mesh;
pub(crate) use mesh_source::parse_terrain_params_from_uri;
pub use params::ObjRenderParams;
use setup::{
    build_biome_params, build_terrain_extra_params, normalized_light_and_view_dirs,
};
use terrain_eval::{compute_terrain_noise_at, displace_sphere_vertex};

/// Minimum vertex/face count to use parallel processing.
/// Below this, serial is faster due to rayon thread spawn overhead.
const VERTEX_PARALLEL_THRESHOLD: usize = 64;
/// Safety cap for per-mesh face shading/rasterization work in one pass.
///
/// 128 cube-sphere uses ~196k faces, so keep this above that to avoid visible truncation.
const MAX_OBJ_FACE_RENDER: usize = 250_000;

// Thread-local pointer to the current frame's ObjPrerenderedFrames (set by compositor, cleared after).
// SAFETY: only set during `with_prerender_frames` and never accessed across threads.
thread_local! {
    static PRERENDER_FRAMES_PTR: Cell<*const ObjPrerenderedFrames> = const { Cell::new(std::ptr::null()) };
}

/// Set the thread-local prerendered frames pointer for the duration of `f`.
pub fn with_prerender_frames<R>(frames: Option<&ObjPrerenderedFrames>, f: impl FnOnce() -> R) -> R {
    let ptr = frames.map(|c| c as *const _).unwrap_or(std::ptr::null());
    PRERENDER_FRAMES_PTR.with(|cell| cell.set(ptr));
    let result = f();
    PRERENDER_FRAMES_PTR.with(|cell| cell.set(std::ptr::null()));
    result
}

/// Borrow the current frame's `ObjPrerenderedFrames` if one was set.
#[inline]
fn current_prerender_frames<'a>() -> Option<&'a ObjPrerenderedFrames> {
    PRERENDER_FRAMES_PTR.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            None
        } else {
            // SAFETY: ptr was set from a reference valid for the duration of `with_prerender_frames`.
            Some(unsafe { &*ptr })
        }
    })
}

// Thread-local pooled buffers for OBJ rendering — avoids per-frame allocation.
thread_local! {
    static OBJ_CANVAS: RefCell<Vec<Option<[u8; 3]>>> = const { RefCell::new(Vec::new()) };
    static OBJ_CANVAS_RGBA: RefCell<Vec<Option<[u8; 4]>>> = const { RefCell::new(Vec::new()) };
    static OBJ_DEPTH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static OBJ_PROJECTED: RefCell<Vec<Option<ProjectedVertex>>> = const { RefCell::new(Vec::new()) };
}

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
    let root = asset_root?;
    let mesh = get_or_load_obj_mesh(root, source)?;
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
    // Snap: uses deterministic hash of (snap_index, seed) to pick a pseudo-random angle instantly.
    fn snap_angle(elapsed_s: f32, snap_hz: f32, seed: u32) -> f32 {
        let snap_index = (elapsed_s * snap_hz) as u32;
        let h = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
        (h as f32 / u32::MAX as f32) * std::f32::consts::TAU
    }

    // Pre-compute orbit radii to avoid repeated sqrt calls.
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
    if clipped_viewport.min_y > clipped_viewport.max_y {
        return None;
    }
    // Parallel vertex projection: each vertex is independent.
    // Significant win for large meshes (>1K vertices).
    let center = mesh.center;
    let mut projected = OBJ_PROJECTED.with(|p| {
        let mut v = p.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.reserve(mesh.vertices.len());
        taken
    });

    // Projection function (shared by serial and parallel paths)
    let project_vertex = |v: &[f32; 3]| {
        let centered_raw = [
            (v[0] - center[0]) * model_scale,
            (v[1] - center[1]) * model_scale,
            (v[2] - center[2]) * model_scale,
        ];
        // Compute terrain noise at the raw sphere surface position.
        // This drives both displacement (vertex position) and coloring, keeping them in sync.
        let terrain_noise_val = if params.terrain_color.is_some() || params.terrain_displacement > 0.0 {
            compute_terrain_noise_at(centered_raw, &params)
        } else {
            0.0
        };
        // Displace vertex outward along sphere normal before rotation.
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
        // Apply look_at view transform: project world-space vertex into camera space.
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

    // Use parallel only for large vertex counts
    if mesh.vertices.len() > VERTEX_PARALLEL_THRESHOLD {
        mesh.vertices
            .par_iter()
            .map(project_vertex)
            .collect_into_vec(&mut projected);
    } else {
        projected.extend(mesh.vertices.iter().map(project_vertex));
    }

    // When Gouraud shading is active, rotate smooth normals by the same rotation as vertices.
    if params.smooth_shading && !mesh.smooth_normals.is_empty() {
        for (i, pv_opt) in projected.iter_mut().enumerate() {
            if let Some(pv) = pv_opt.as_mut() {
                if let Some(&n) = mesh.smooth_normals.get(i) {
                    pv.normal = rotate_xyz(n, pitch, yaw, roll);
                }
            }
        }
    }

    // Use pooled buffers to avoid per-frame allocation.
    let canvas_size = virtual_w as usize * virtual_h as usize;
    let mut canvas = OBJ_CANVAS.with(|c| {
        let mut v = c.borrow_mut();
        let mut taken = std::mem::take(&mut *v);
        taken.clear();
        taken.resize(canvas_size, None);
        taken
    });
    if wireframe {
        let line_color = color_to_rgb(fg);
        let mut depth_buf = OBJ_DEPTH.with(|d| {
            let mut v = d.borrow_mut();
            let mut taken = std::mem::take(&mut *v);
            taken.clear();
            taken.resize(canvas_size, f32::INFINITY);
            taken
        });

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
        // Pre-compute Blinn-Phong half-vectors for directional lights (constant per mesh render).
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
        // Sort faces back-to-front for correct painter's-algorithm blending when
        // depth-buffering alone isn't enough (avoids most z-fighting glitches).
        // Pre-compute depth keys to avoid redundant work inside the comparator.
        let mut sorted_faces: Vec<(f32, &ObjFace)> = mesh
            .faces
            .iter()
            .map(|f| (face_avg_depth(&projected, f), f))
            .collect();
        sorted_faces
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Parallel shading: compute face color for each visible face independently.
        // Rasterization must remain sequential (shared canvas/depth writes with depth sort).
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
            // Gouraud path: compute per-vertex shade values using smooth normals.
            let ka_lum_ambient = ambient.max(0.06_f32);
            let light_2_strength = light_2_intensity.clamp(0.0, 2.0);

            let shade_at_vertex = |normal: [f32; 3]| -> f32 {
                let lambert_1 = dot3(normal, light_dir_norm).max(0.0);
                let lambert_2 = dot3(normal, light_2_dir_norm).max(0.0) * light_2_strength;
                let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
                (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
            };

            // Phase 1 (parallel): per-vertex shade, no per-face color computation.
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
                .filter_map(|(_, face)| {
                    let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                    let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                    let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                    if backface_cull && edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
                        return None;
                    }
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
            // Phase 2 (parallel strips): split canvas rows into N strips and rayon-parallelize.
            // Each strip gets exclusive ownership of its canvas/depth rows — no data races.
            let row_w = virtual_w as usize;
            let num_strips = rayon::current_num_threads().max(1);
            let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
            // Collect (strip_y0, canvas_strip, depth_strip) tuples from split borrows.
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
            count
        } else {
            // Phase 1 (parallel): filter visible faces and compute shaded colors.
            let shaded_faces: Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3])> =
                sorted_faces[..face_limit]
                    .par_iter()
                    .filter_map(|(_, face)| {
                        let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                        let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                        let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                        // Back-face culling check
                        if backface_cull && edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
                            return None;
                        }
                        // Unlit: render at flat fg color, skip all lighting.
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
            // Phase 2 (sequential): rasterize in depth-sorted order.
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
            count
        };

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
        // Render atmosphere shell AFTER planet — transparent overlay via alpha compositing.
        // The shell reuses the planet's already-displaced projected vertices (pushed outward by
        // atmo_margin), so the shell follows irregular terrain shape rather than a perfect sphere.
        if params.atmo_shell_scale > 1.001 {
            if let Some(biome) = &biome_params {
                if let Some(atmo_color) = biome.atmo_color {
                    // atmo_margin is the outward world-space offset above the planet surface.
                    let atmo_margin = params.scale * (params.atmo_shell_scale - 1.0);
                    render_atmo_shell_pass(
                        &mut canvas,
                        virtual_w,
                        virtual_h,
                        &projected,
                        &mesh.faces,
                        [
                            params.object_translate_x,
                            params.object_translate_y,
                            params.object_translate_z,
                        ],
                        atmo_margin,
                        inv_tan,
                        aspect,
                        near_clip,
                        [params.camera_world_x, params.camera_world_y, params.camera_world_z],
                        [params.view_right_x, params.view_right_y, params.view_right_z],
                        [params.view_up_x, params.view_up_y, params.view_up_z],
                        [params.view_forward_x, params.view_forward_y, params.view_forward_z],
                        atmo_color,
                        biome.atmo_strength,
                        biome.atmo_rim_power,
                        biome.sun_dir,
                        view_dir,
                    );
                }
            }
        }

        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    }

    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
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
    let root = asset_root?;
    let mesh = get_or_load_obj_mesh(root, source)?;
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
        let terrain_noise_val = if params.terrain_color.is_some() && params.cloud_alpha_softness <= 0.0
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

    // Rotate smooth normals.
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
    let ka_lum_ambient = params.ambient.max(0.06_f32);
    let light_2_strength = params.light_2_intensity.clamp(0.0, 2.0);

    let shade_at_vertex = |normal: [f32; 3]| -> f32 {
        let lambert_1 = dot3(normal, light_dir_norm).max(0.0);
        let lambert_2 = dot3(normal, light_2_dir_norm).max(0.0) * light_2_strength;
        let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
        (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
    };

    let biome_params = build_biome_params(&params, light_dir_norm, view_dir);

    // Sort faces back-to-front.
    let mut sorted_faces: Vec<(f32, &ObjFace)> = mesh
        .faces
        .iter()
        .map(|f| (face_avg_depth(&projected, f), f))
        .collect();
    sorted_faces
        .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let face_limit = sorted_faces.len().min(MAX_OBJ_FACE_RENDER);
    let unlit = params.unlit;

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
        .filter_map(|(_, face)| {
            let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
            let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
            let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
            if backface_cull && edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
                return None;
            }
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

    // Phase 2: rasterize with RGBA output.
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

    OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    OBJ_PROJECTED.with(|p| *p.borrow_mut() = projected);
    Some((canvas, virtual_w, virtual_h))
}

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
            let terrain_noise_val = if params.terrain_color.is_some() || params.terrain_displacement > 0.0 {
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

    // When Gouraud shading is active, rotate smooth normals by the same rotation as vertices.
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

        let mut sorted_faces: Vec<(f32, &ObjFace)> = mesh
            .faces
            .iter()
            .map(|f| (face_avg_depth(&projected, f), f))
            .collect();
        sorted_faces
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

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
            let ka_lum_ambient = ambient.max(0.06_f32);
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
                .filter_map(|(_, face)| {
                    let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                    let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                    let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                    if backface_cull && edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
                        return None;
                    }
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
                    0, // row_base: full canvas, no strip offset
                );
            }
            count
        } else {
            let shaded_faces: Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3])> =
                sorted_faces[..face_limit]
                    .par_iter()
                    .filter_map(|(_, face)| {
                        let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                        let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                        let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                        if backface_cull && edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y) < 0.0 {
                            return None;
                        }
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

/// Render a mesh into pre-allocated shared canvas and depth buffers.
///
/// Used by scene3d prerender to share depth testing across multiple objects in one frame,
/// ensuring wire edges behind solid faces are correctly culled.
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
    let Some(mesh) = get_or_load_obj_mesh(root, source) else {
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
    fn render(&self, input: ObjCanvasRenderRequest<'a>) -> Option<(Vec<Option<[u8; 3]>>, u16, u16)> {
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
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let (target_w, target_h) = obj_sprite_dimensions(width, height, size);
    // Live render path — cache-hit is checked in sprite_renderer.rs BEFORE calling this fn.
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
    // Return pooled buffers for reuse.
    OBJ_CANVAS.with(|c| *c.borrow_mut() = canvas);
}

/// Try to blit a pre-rendered OBJ sprite from the thread-local `ObjPrerenderedFrames`.
///
/// Checks animated frame cache first (snapped yaw lookup), then static pose tolerance.
/// Returns `true` if a cached frame was blitted; `false` → caller does live render.
#[allow(clippy::too_many_arguments)]
pub fn try_blit_prerendered(
    sprite_id: &str,
    live_total_yaw: f32,
    current_pitch: f32,
    clip_y_min: f32,
    clip_y_max: f32,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> bool {
    let Some(frames) = current_prerender_frames() else {
        return false;
    };

    // ── Animated keyframe lookup (highest priority) ───────────────────────────
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

    // ── Static pose tolerance check ───────────────────────────────────────────
    let Some(frame) = frames.get(sprite_id) else {
        return false;
    };

    // Pose tolerance check — wider tolerance increases cache hits.
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
