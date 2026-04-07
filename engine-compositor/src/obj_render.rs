use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use engine_core::color::Color;
use rayon::prelude::*;

use crate::obj_prerender::ObjPrerenderedFrames;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::scene::{SceneRenderedMode, SpriteSizePreset};

use super::obj_loader::{load_obj_mesh, ObjFace, ObjMesh};
use super::obj_render_helpers::*;
pub use super::obj_render_helpers::{blit_color_canvas, virtual_dimensions};

/// Minimum vertex/face count to use parallel processing.
/// Below this, serial is faster due to rayon thread spawn overhead.
const VERTEX_PARALLEL_THRESHOLD: usize = 256;

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

// Global OBJ mesh cache — parse once, reuse via Arc.
static OBJ_MESH_CACHE: OnceLock<Mutex<HashMap<String, Arc<ObjMesh>>>> = OnceLock::new();

/// Get or load an OBJ mesh from cache.
fn get_or_load_obj_mesh(asset_root: &AssetRoot, path: &str) -> Option<Arc<ObjMesh>> {
    let cache = OBJ_MESH_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Try to get from cache first.
    {
        let cache_lock = cache.lock().ok()?;
        if let Some(mesh) = cache_lock.get(path) {
            return Some(Arc::clone(mesh));
        }
    }

    // Not in cache, load it.
    let mesh_arc = load_obj_mesh(asset_root, path)?;

    // Store in cache.
    if let Ok(mut cache_lock) = cache.lock() {
        cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
    }

    Some(mesh_arc)
}

// Thread-local pooled buffers for OBJ rendering — avoids per-frame allocation.
thread_local! {
    static OBJ_CANVAS: RefCell<Vec<Option<[u8; 3]>>> = const { RefCell::new(Vec::new()) };
    static OBJ_DEPTH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static OBJ_PROJECTED: RefCell<Vec<Option<ProjectedVertex>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy)]
pub struct ObjRenderParams {
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
    /// Object-space/view-space translation applied after rotation and scale.
    pub object_translate_x: f32,
    pub object_translate_y: f32,
    pub object_translate_z: f32,
    /// Vertical clip region (normalised 0.0–1.0). Rows outside [min, max) are skipped.
    pub clip_y_min: f32,
    pub clip_y_max: f32,
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
    mode: SceneRenderedMode,
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
        let centered = [
            (v[0] - center[0]) * model_scale,
            (v[1] - center[1]) * model_scale,
            (v[2] - center[2]) * model_scale,
        ];
        let rotated = rotate_xyz(centered, pitch, yaw, roll);
        let translated = [
            rotated[0] + params.object_translate_x,
            rotated[1] + params.object_translate_y,
            rotated[2] + params.object_translate_z,
        ];
        // Apply camera pan: shift the scene in view-space (equivalent to moving camera).
        let panned = [
            translated[0] - params.camera_pan_x,
            translated[1] - params.camera_pan_y,
            translated[2],
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
    };
    
    // Use parallel only for large vertex counts
    if mesh.vertices.len() > VERTEX_PARALLEL_THRESHOLD {
        mesh.vertices.par_iter().map(project_vertex).collect_into_vec(&mut projected);
    } else {
        projected.extend(mesh.vertices.iter().map(project_vertex));
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
        // Pre-compute normalized light directions once per render (not per face).
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
        // Pre-compute Blinn-Phong half-vectors for directional lights (constant per mesh render).
        // VIEW_DIR is always [0,0,-1] in view space (camera looks down -Z).
        let half_dir_1 = normalize3([
            light_dir_norm[0],
            light_dir_norm[1],
            light_dir_norm[2] - 1.0,
        ]);
        let half_dir_2 = normalize3([
            light_2_dir_norm[0],
            light_2_dir_norm[1],
            light_2_dir_norm[2] - 1.0,
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
        let face_limit = sorted_faces.len().min(50_000);
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

        // Phase 2 (sequential): rasterize in depth-sorted order.
        // Canvas and depth buffer require exclusive access with correct ordering.
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
        let drawn_faces = shaded_faces.len();

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
        OBJ_DEPTH.with(|d| *d.borrow_mut() = depth);
    }

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
            let centered = [
                (v[0] - center[0]) * model_scale,
                (v[1] - center[1]) * model_scale,
                (v[2] - center[2]) * model_scale,
            ];
            let rotated = rotate_xyz(centered, pitch, yaw, roll);
            let translated = [
                rotated[0] + params.object_translate_x,
                rotated[1] + params.object_translate_y,
                rotated[2] + params.object_translate_z,
            ];
            let panned = [
                translated[0] - params.camera_pan_x,
                translated[1] - params.camera_pan_y,
                translated[2],
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
        .collect_into_vec(&mut projected);

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
        let half_dir_1 = normalize3([
            light_dir_norm[0],
            light_dir_norm[1],
            light_dir_norm[2] - 1.0,
        ]);
        let half_dir_2 = normalize3([
            light_2_dir_norm[0],
            light_2_dir_norm[1],
            light_2_dir_norm[2] - 1.0,
        ]);

        let mut sorted_faces: Vec<(f32, &ObjFace)> = mesh
            .faces
            .iter()
            .map(|f| (face_avg_depth(&projected, f), f))
            .collect();
        sorted_faces
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let face_limit = sorted_faces.len().min(50_000);
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

        if shaded_faces.is_empty() {
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
    mode: SceneRenderedMode,
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
    let (virtual_w, virtual_h) = virtual_dimensions(mode, target_w, target_h);
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

#[allow(clippy::too_many_arguments)]
pub fn render_obj_content(
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
    // Live render path — cache-hit is checked in sprite_renderer.rs BEFORE calling this fn.
    let Some((canvas, virtual_w, virtual_h)) = render_obj_to_canvas(
        source,
        width,
        height,
        size,
        mode,
        params,
        wireframe,
        backface_cull,
        fg,
        asset_root,
    ) else {
        return;
    };
    blit_color_canvas(
        buf,
        mode,
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
/// Checks pose tolerance: yaw within 1° and pitch within 0.5°.
/// If matched: applies clip masking and blits the canvas, returns `true`.
/// If no frames registered or pose mismatch: returns `false` → caller does live render.
#[allow(clippy::too_many_arguments)]
pub fn try_blit_prerendered(
    sprite_id: &str,
    current_total_yaw: f32,
    current_pitch: f32,
    clip_y_min: f32,
    clip_y_max: f32,
    mode: SceneRenderedMode,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> bool {
    let Some(frames) = current_prerender_frames() else {
        return false;
    };
    let Some(frame) = frames.get(sprite_id) else {
        return false;
    };

    // Pose tolerance check — wider tolerance increases cache hits.
    if (current_total_yaw - frame.rendered_yaw).abs() >= 2.0 {
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
        mode,
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
