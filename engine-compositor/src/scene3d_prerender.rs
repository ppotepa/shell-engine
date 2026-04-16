use std::collections::{HashMap, HashSet};

use engine_core::color::Color;
use rayon::prelude::*;

use engine_3d::scene3d_format::{
    load_scene3d, CameraDef, FrameDef, LightDef, LightKind, MaterialDef, ObjectDef,
    Scene3DDefinition, SurfaceMode, TweenDef,
};
use engine_3d::scene3d_resolve::{resolve_scene3d_refs, Scene3DAssetResolver};
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::logging;
use engine_core::scene::{Layer, Scene, Sprite};
use engine_core::scene_runtime_types::SceneCamera3D;

use crate::scene3d_runtime_store::{Scene3DRuntimeEntry, Scene3DRuntimeStore};
use crate::{
    blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions, ObjRenderParams,
    Scene3DAtlas,
};

pub fn prerender_scene3d_atlas(scene: &Scene, asset_root: &AssetRoot) -> Option<Scene3DAtlas> {
    let sources = collect_scene3d_sources(&scene.layers);
    if sources.is_empty() {
        return None;
    }

    let scene_id = scene.id.clone();

    logging::info(
        "engine.scene3d",
        format!(
            "scene={scene_id}: prerendering {} scene3d source(s) (parallel)",
            sources.len()
        ),
    );

    let resolver = AssetRootResolver { asset_root };
    let work_items: Vec<WorkItem> = sources
        .iter()
        .flat_map(|src| {
            let path = asset_root.resolve(src);
            let path_str = path.to_string_lossy();
            let mut def = match load_scene3d(&path_str) {
                Ok(d) => d,
                Err(e) => {
                    logging::warn(
                        "engine.scene3d",
                        format!("scene={scene_id}: failed to load {src}: {e}"),
                    );
                    return Vec::new();
                }
            };
            resolve_scene3d_refs(&mut def, src, &resolver);
            build_work_items(src, def)
        })
        .collect();

    let total = work_items.len();
    logging::info(
        "engine.scene3d",
        format!("scene={scene_id}: rendering {total} scene3d frame(s)"),
    );

    let rendered: Vec<(String, String, Buffer)> = work_items
        .into_par_iter()
        .filter_map(|item| {
            let buf = render_frame(&item, asset_root)?;
            Some((item.src, item.frame_id, buf))
        })
        .collect();

    let count = rendered.len();
    let mut atlas = Scene3DAtlas::new();
    for (src, frame_id, buf) in rendered {
        atlas.insert(&src, &frame_id, buf);
    }

    logging::info(
        "engine.scene3d",
        format!("scene={scene_id}: scene3d prerender complete ({count}/{total} frames cached)"),
    );

    Some(atlas)
}

struct AssetRootResolver<'a> {
    asset_root: &'a AssetRoot,
}

impl Scene3DAssetResolver for AssetRootResolver<'_> {
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = self.asset_root.resolve(asset_path);
        Ok(std::fs::read_to_string(full)?)
    }
}

fn collect_scene3d_sources(layers: &[Layer]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for layer in layers {
        collect_sources_from_sprites(&layer.sprites, &mut seen, &mut out);
    }
    out
}

fn collect_sources_from_sprites(
    sprites: &[Sprite],
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    for sprite in sprites {
        match sprite {
            Sprite::Scene3D { src, .. } => {
                if seen.insert(src.clone()) {
                    out.push(src.clone());
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                collect_sources_from_sprites(children, seen, out);
            }
            _ => {}
        }
    }
}

fn look_at_basis(
    eye: [f32; 3],
    target: [f32; 3],
    world_up: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let fwd = {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        [d[0] / len, d[1] / len, d[2] / len]
    };
    // right = normalize(cross(fwd, world_up))
    let right = {
        let d = [
            fwd[1] * world_up[2] - fwd[2] * world_up[1],
            fwd[2] * world_up[0] - fwd[0] * world_up[2],
            fwd[0] * world_up[1] - fwd[1] * world_up[0],
        ];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        [d[0] / len, d[1] / len, d[2] / len]
    };
    // up = cross(right, fwd)
    let up = [
        right[1] * fwd[2] - right[2] * fwd[1],
        right[2] * fwd[0] - right[0] * fwd[2],
        right[0] * fwd[1] - right[1] * fwd[0],
    ];
    (right, up, fwd)
}

struct WorkItem {
    src: String,
    frame_id: String,
    viewport_w: u16,
    viewport_h: u16,
    objects: Vec<ObjectRenderSpec>,
}

struct ObjectRenderSpec {
    mesh: String,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
}

fn build_work_items(
    src: &str,
    def: Scene3DDefinition,
) -> Vec<WorkItem> {
    let vw = def.viewport.width;
    let vh = def.viewport.height;
    let light_params = extract_light_params(&def.lights);

    let mut items = Vec::new();
    for (frame_id, frame_def) in &def.frames {
        match frame_def {
            FrameDef::Static(static_def) => {
                let objects = build_object_specs(
                    &static_def.show,
                    &def.objects,
                    &def.materials,
                    &def.camera,
                    None,
                    &light_params,
                    None,
                    &[],
                    0.0,
                );
                items.push(WorkItem {
                    src: src.to_string(),
                    frame_id: frame_id.clone(),
                    viewport_w: vw,
                    viewport_h: vh,
                    objects,
                });
            }
            FrameDef::Clip(clip_def) => {
                let n = clip_def.clip.keyframes.max(1);
                for kf in 0..n {
                    let t = if n <= 1 {
                        0.0
                    } else {
                        kf as f32 / (n - 1) as f32
                    };
                    let objects = build_object_specs(
                        &clip_def.show,
                        &def.objects,
                        &def.materials,
                        &def.camera,
                        None,
                        &light_params,
                        clip_def.clip.orbit_origin,
                        &clip_def.clip.tweens,
                        t,
                    );
                    items.push(WorkItem {
                        src: src.to_string(),
                        frame_id: format!("{frame_id}-{kf}"),
                        viewport_w: vw,
                        viewport_h: vh,
                        objects,
                    });
                }
            }
        }
    }

    items
}

struct LightParams {
    dir1: [f32; 3],
    dir2: [f32; 3],
    dir2_intensity: f32,
    point1: [f32; 3],
    point1_intensity: f32,
    point1_colour: Option<Color>,
    point1_snap_hz: f32,
    point1_falloff: f32,
    point2: [f32; 3],
    point2_intensity: f32,
    point2_colour: Option<Color>,
    point2_snap_hz: f32,
    point2_falloff: f32,
    ambient: f32,
}

fn extract_light_params(lights: &[LightDef]) -> LightParams {
    let mut p = LightParams {
        dir1: [-0.45, 0.70, -0.85],
        dir2: [0.0, 0.0, -1.0],
        dir2_intensity: 0.0,
        point1: [0.0, 2.0, 0.0],
        point1_intensity: 0.0,
        point1_colour: None,
        point1_snap_hz: 0.0,
        point1_falloff: 0.7,
        point2: [0.0, 0.0, 0.0],
        point2_intensity: 0.0,
        point2_colour: None,
        point2_snap_hz: 0.0,
        point2_falloff: 0.7,
        ambient: 0.0,
    };

    let mut dir_count = 0u8;
    let mut point_count = 0u8;
    for light in lights {
        match light.kind {
            LightKind::Directional => {
                let dir = light.direction.unwrap_or([-0.45, 0.70, -0.85]);
                if dir_count == 0 {
                    p.dir1 = dir;
                    dir_count += 1;
                } else if dir_count == 1 {
                    p.dir2 = dir;
                    p.dir2_intensity = light.intensity;
                    dir_count += 1;
                }
            }
            LightKind::Point => {
                let pos = light.position.unwrap_or([0.0, 2.0, 0.0]);
                let colour = light.colour.as_deref().and_then(parse_hex_color);
                if point_count == 0 {
                    p.point1 = pos;
                    p.point1_intensity = light.intensity;
                    p.point1_colour = colour;
                    p.point1_snap_hz = light.snap_hz;
                    p.point1_falloff = light.falloff_constant;
                    point_count += 1;
                } else if point_count == 1 {
                    p.point2 = pos;
                    p.point2_intensity = light.intensity;
                    p.point2_colour = colour;
                    p.point2_snap_hz = light.snap_hz;
                    p.point2_falloff = light.falloff_constant;
                    point_count += 1;
                }
            }
            LightKind::Ambient => {
                // Sum multiple ambient sources; max avoids unintended brightness stacking.
                p.ambient = (p.ambient + light.intensity).min(1.0);
            }
        }
    }

    p
}

fn build_object_specs(
    show: &[String],
    objects: &[ObjectDef],
    materials: &HashMap<String, MaterialDef>,
    camera: &CameraDef,
    camera_override: Option<&SceneCamera3D>,
    lights: &LightParams,
    clip_orbit_origin: Option<[f32; 3]>,
    tweens: &[TweenDef],
    t: f32,
) -> Vec<ObjectRenderSpec> {
    let mut tween_values: HashMap<String, HashMap<String, f32>> = HashMap::new();
    for tw in tweens {
        let et = tw.easing.apply(t);
        let value = tw.from + (tw.to - tw.from) * et;
        tween_values
            .entry(tw.object.clone())
            .or_default()
            .insert(tw.property.clone(), value);
    }

    // ── Camera orbit tween support ──────────────────────────────────────────
    // Object id "camera" with property "orbit_angle_deg" rotates the camera
    // around camera.look_at in the horizontal plane, preserving radial distance
    // and elevation angle from the initial position.
    let base_cam_pos = camera_override
        .map(|camera| camera.eye)
        .unwrap_or_else(|| camera.position.unwrap_or([0.0, 0.0, camera.distance]));
    let look_at = camera_override
        .map(|camera| camera.look_at)
        .or(camera.look_at);
    let effective_cam_pos =
        if let (Some(cam_tw), Some(look_at)) = (tween_values.get("camera"), look_at) {
            if let Some(&orbit_angle_deg) = cam_tw.get("orbit_angle_deg") {
                let dx = base_cam_pos[0] - look_at[0];
                let dy = base_cam_pos[1] - look_at[1];
                let dz = base_cam_pos[2] - look_at[2];
                let horiz_r = (dx * dx + dz * dz).sqrt();
                let elevation = dy.atan2(horiz_r); // preserve vertical angle
                let base_phase = dz.atan2(dx); // initial azimuth
                let theta = base_phase + orbit_angle_deg.to_radians();
                let total_r = (dx * dx + dy * dy + dz * dz).sqrt();
                [
                    look_at[0] + total_r * elevation.cos() * theta.cos(),
                    look_at[1] + total_r * elevation.sin(),
                    look_at[2] + total_r * elevation.cos() * theta.sin(),
                ]
            } else {
                base_cam_pos
            }
        } else {
            base_cam_pos
        };

    let camera_distance = if camera_override.is_some() {
        ((effective_cam_pos[0] - look_at.unwrap_or([0.0, 0.0, 0.0])[0]).powi(2)
            + (effective_cam_pos[1] - look_at.unwrap_or([0.0, 0.0, 0.0])[1]).powi(2)
            + (effective_cam_pos[2] - look_at.unwrap_or([0.0, 0.0, 0.0])[2]).powi(2))
        .sqrt()
        .max(0.001)
    } else {
        (effective_cam_pos[0].powi(2) + effective_cam_pos[1].powi(2) + effective_cam_pos[2].powi(2))
            .sqrt()
            .max(camera.distance.abs())
    };

    let (global_view_right, global_view_up, global_view_forward) = if let Some(look_at) = look_at {
        let up = camera_override
            .map(|camera| camera.up)
            .unwrap_or([0.0, 1.0, 0.0]);
        look_at_basis(effective_cam_pos, look_at, up)
    } else {
        ([1.0f32, 0.0, 0.0], [0.0f32, 1.0, 0.0], [0.0f32, 0.0, 1.0])
    };

    objects
        .iter()
        .filter(|obj| show.contains(&obj.id))
        .filter_map(|obj| {
            let mat = materials.get(&obj.material)?;
            let obj_tweens = tween_values.get(&obj.id);

            let yaw_offset = obj_tweens
                .and_then(|m| m.get("yaw_offset"))
                .copied()
                .unwrap_or(0.0);
            let clip_y_min = obj_tweens
                .and_then(|m| m.get("clip_y_min"))
                .copied()
                .unwrap_or(0.0);
            let clip_y_max = obj_tweens
                .and_then(|m| m.get("clip_y_max"))
                .copied()
                .unwrap_or(1.0);
            let base_translation = obj.transform.translation.unwrap_or([0.0, 0.0, 0.0]);
            let translation_x = obj_tweens
                .and_then(|m| m.get("translation_x"))
                .copied()
                .unwrap_or(base_translation[0]);
            let translation_y = obj_tweens
                .and_then(|m| m.get("translation_y"))
                .copied()
                .unwrap_or(base_translation[1]);
            let translation_z = obj_tweens
                .and_then(|m| m.get("translation_z"))
                .copied()
                .unwrap_or(base_translation[2]);
            let orbit_angle_deg = obj_tweens.and_then(|m| m.get("orbit_angle_deg")).copied();

            let (translation_x, translation_y, translation_z) =
                if let Some(orbit_angle_deg) = orbit_angle_deg {
                    let origin = clip_orbit_origin.unwrap_or([0.0, 0.0, 0.0]);
                    let orbit_center_x = obj_tweens
                        .and_then(|m| m.get("orbit_center_x"))
                        .copied()
                        .unwrap_or(origin[0]);
                    let orbit_center_y = obj_tweens
                        .and_then(|m| m.get("orbit_center_y"))
                        .copied()
                        .unwrap_or(origin[1]);
                    let orbit_center_z = obj_tweens
                        .and_then(|m| m.get("orbit_center_z"))
                        .copied()
                        .unwrap_or(origin[2]);

                    let dx0 = translation_x - orbit_center_x;
                    let dz0 = translation_z - orbit_center_z;
                    let derived_radius = (dx0 * dx0 + dz0 * dz0).sqrt();
                    let orbit_radius = obj_tweens
                        .and_then(|m| m.get("orbit_radius"))
                        .copied()
                        .unwrap_or(derived_radius);
                    let orbit_phase_deg = obj_tweens
                        .and_then(|m| m.get("orbit_phase_deg"))
                        .copied()
                        .unwrap_or_else(|| dz0.atan2(dx0).to_degrees());

                    let theta = (orbit_phase_deg + orbit_angle_deg).to_radians();
                    (
                        orbit_center_x + orbit_radius * theta.cos(),
                        translation_y + (orbit_center_y - origin[1]),
                        orbit_center_z + orbit_radius * theta.sin(),
                    )
                } else {
                    (translation_x, translation_y, translation_z)
                };

            let tf = &obj.transform;

            let (view_right, view_up, view_forward) =
                (global_view_right, global_view_up, global_view_forward);
            let cam_pos = effective_cam_pos;

            let wireframe = mat.surface_mode == SurfaceMode::Wireframe;
            let fg = mat
                .fg_colour
                .as_deref()
                .and_then(parse_hex_color)
                .unwrap_or(Color::White);
            let (rot_x, rot_y, rot_z) = tf.resolved_rotation();
            let params = ObjRenderParams {
                scale: tf.resolved_scale(),
                yaw_deg: rot_y + yaw_offset,
                pitch_deg: rot_x,
                roll_deg: rot_z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                rotation_z: 0.0,
                rotate_y_deg_per_sec: 0.0,
                camera_distance,
                fov_degrees: camera.fov_degrees,
                near_clip: camera.near_clip,
                light_direction_x: lights.dir1[0],
                light_direction_y: lights.dir1[1],
                light_direction_z: lights.dir1[2],
                light_2_direction_x: lights.dir2[0],
                light_2_direction_y: lights.dir2[1],
                light_2_direction_z: lights.dir2[2],
                light_2_intensity: lights.dir2_intensity,
                light_point_x: lights.point1[0],
                light_point_y: lights.point1[1],
                light_point_z: lights.point1[2],
                light_point_intensity: lights.point1_intensity,
                light_point_colour: lights.point1_colour,
                light_point_flicker_depth: 0.0,
                light_point_flicker_hz: 0.0,
                light_point_orbit_hz: 0.0,
                light_point_snap_hz: lights.point1_snap_hz,
                light_point_2_x: lights.point2[0],
                light_point_2_y: lights.point2[1],
                light_point_2_z: lights.point2[2],
                light_point_2_intensity: lights.point2_intensity,
                light_point_2_colour: lights.point2_colour,
                light_point_2_flicker_depth: 0.0,
                light_point_2_flicker_hz: 0.0,
                light_point_2_orbit_hz: 0.0,
                light_point_2_snap_hz: lights.point2_snap_hz,
                cel_levels: mat.cel_levels,
                shadow_colour: mat.shadow_colour.as_deref().and_then(parse_hex_color),
                midtone_colour: mat.midtone_colour.as_deref().and_then(parse_hex_color),
                highlight_colour: mat.highlight_colour.as_deref().and_then(parse_hex_color),
                tone_mix: mat.tone_mix,
                scene_elapsed_ms: 0,
                camera_pan_x: 0.0,
                camera_pan_y: 0.0,
                camera_look_yaw: 0.0,
                camera_look_pitch: 0.0,
                object_translate_x: translation_x,
                object_translate_y: translation_y,
                object_translate_z: translation_z,
                clip_y_min,
                clip_y_max,
                camera_world_x: cam_pos[0],
                camera_world_y: cam_pos[1],
                camera_world_z: cam_pos[2],
                view_right_x: view_right[0],
                view_right_y: view_right[1],
                view_right_z: view_right[2],
                view_up_x: view_up[0],
                view_up_y: view_up[1],
                view_up_z: view_up[2],
                view_forward_x: view_forward[0],
                view_forward_y: view_forward[1],
                view_forward_z: view_forward[2],
                unlit: mat.surface_mode == SurfaceMode::Unlit,
                ambient: lights.ambient,
                light_point_falloff: lights.point1_falloff,
                light_point_2_falloff: lights.point2_falloff,
                smooth_shading: false,
                latitude_bands: 0,
                latitude_band_depth: 0.0,
                terrain_color: None,
                terrain_threshold: 0.5,
                terrain_noise_scale: 2.5,
                terrain_noise_octaves: 2,
                marble_depth: 0.0,
                terrain_relief: 0.0,
                noise_seed: 0.0,
                warp_strength: 0.0,
                warp_octaves: 2,
                noise_lacunarity: 2.0,
                noise_persistence: 0.5,
                normal_perturb_strength: 0.0,
                ocean_specular: 0.0,
                crater_density: 0.0,
                crater_rim_height: 0.35,
                snow_line_altitude: 0.0,
                terrain_displacement: 0.0,
                below_threshold_transparent: false,
                cloud_alpha_softness: 0.0,
                polar_ice_color: None,
                polar_ice_start: 0.78,
                polar_ice_end: 0.92,
                desert_color: None,
                desert_strength: 0.0,
                atmo_color: None,
                atmo_strength: 0.0,
                atmo_rim_power: 4.5,
                atmo_haze_strength: 0.0,
                atmo_haze_power: 1.8,
                atmo_veil_strength: 0.0,
                atmo_veil_power: 1.6,
                atmo_halo_strength: 0.0,
                atmo_halo_width: 0.12,
                atmo_halo_power: 2.2,
                ocean_noise_scale: 4.0,
                ocean_color_rgb: None,
                night_light_color: None,
                night_light_threshold: 0.82,
                night_light_intensity: 0.0,
                heightmap: None,
                heightmap_w: 0,
                heightmap_h: 0,
                heightmap_blend: 0.0,
                depth_sort_faces: false,
            };

            Some(ObjectRenderSpec {
                mesh: obj.mesh.clone(),
                params,
                wireframe,
                backface_cull: mat.backface_cull,
                fg,
            })
        })
        .collect()
}

fn render_frame(item: &WorkItem, asset_root: &AssetRoot) -> Option<Buffer> {
    let mut buf = Buffer::new(item.viewport_w, item.viewport_h);
    let (virtual_w, virtual_h) = virtual_dimensions(item.viewport_w, item.viewport_h);
    let canvas_size = virtual_w as usize * virtual_h as usize;
    if canvas_size == 0 {
        return Some(buf);
    }

    let mut canvas = vec![None; canvas_size];
    let mut depth_buf = vec![f32::INFINITY; canvas_size];

    for obj in item.objects.iter().filter(|o| !o.wireframe) {
        render_obj_to_shared_buffers(
            &obj.mesh,
            item.viewport_w,
            item.viewport_h,
            obj.params.clone(),
            obj.wireframe,
            obj.backface_cull,
            obj.fg,
            Some(asset_root),
            &mut canvas,
            &mut depth_buf,
        );
    }
    for obj in item.objects.iter().filter(|o| o.wireframe) {
        render_obj_to_shared_buffers(
            &obj.mesh,
            item.viewport_w,
            item.viewport_h,
            obj.params.clone(),
            obj.wireframe,
            obj.backface_cull,
            obj.fg,
            Some(asset_root),
            &mut canvas,
            &mut depth_buf,
        );
    }

    blit_color_canvas(
        &mut buf,
        &canvas,
        virtual_w,
        virtual_h,
        item.viewport_w,
        item.viewport_h,
        0,
        0,
        false,
        '#',
        Color::White,
        Color::Reset,
        0,
        virtual_h as usize,
    );

    Some(buf)
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb { r, g, b })
}

/// Build a [`Scene3DRuntimeStore`] holding the parsed `Scene3DDefinition` for every
/// `.scene3d.yml` referenced in `scene`.  Called alongside [`prerender_scene3d_atlas`] so that
/// the real-time rendering path has access to the scene data at compositor time.
pub fn build_scene3d_runtime_store(
    scene: &Scene,
    asset_root: &AssetRoot,
) -> Option<Scene3DRuntimeStore> {
    let sources = collect_scene3d_sources(&scene.layers);
    if sources.is_empty() {
        return None;
    }

    let resolver = AssetRootResolver { asset_root };
    let mut store = Scene3DRuntimeStore::new();

    for src in &sources {
        let path = asset_root.resolve(src);
        let path_str = path.to_string_lossy();
        let mut def = match load_scene3d(&path_str) {
            Ok(d) => d,
            Err(e) => {
                logging::warn(
                    "engine.scene3d",
                    format!("runtime-store: failed to load {src}: {e}"),
                );
                continue;
            }
        };
        resolve_scene3d_refs(&mut def, src, &resolver);
        store.insert(src.clone(), Scene3DRuntimeEntry { def });
    }

    if store.is_empty() {
        None
    } else {
        Some(store)
    }
}

/// Render a single frame of a Scene3D clip at a given `elapsed_ms` within the clip's timeline.
///
/// `clip_name` must be the bare clip frame key (e.g. `"solar-orbit"`), **not** a keyframe id
/// like `"solar-orbit-7"`.  Returns `None` if the clip is not found or the scene has no objects.
pub fn render_scene3d_frame_at(
    entry: &Scene3DRuntimeEntry,
    frame_name: &str,
    elapsed_ms: u64,
    asset_root: &AssetRoot,
    camera_override: Option<&SceneCamera3D>,
) -> Option<Buffer> {
    let frame_def = entry.def.frames.get(frame_name)?;
    let light_params = extract_light_params(&entry.def.lights);
    let objects = match frame_def {
        FrameDef::Static(static_def) => build_object_specs(
            &static_def.show,
            &entry.def.objects,
            &entry.def.materials,
            &entry.def.camera,
            camera_override,
            &light_params,
            None,
            &[],
            0.0,
        ),
        FrameDef::Clip(clip) => {
            let duration_ms = clip.clip.duration_ms as u64;
            let t = if duration_ms == 0 {
                0.0f32
            } else {
                (elapsed_ms % duration_ms) as f32 / duration_ms as f32
            };
            build_object_specs(
                &clip.show,
                &entry.def.objects,
                &entry.def.materials,
                &entry.def.camera,
                camera_override,
                &light_params,
                clip.clip.orbit_origin,
                &clip.clip.tweens,
                t,
            )
        }
    };

    let item = WorkItem {
        src: String::new(),
        frame_id: frame_name.to_string(),
        viewport_w: entry.def.viewport.width,
        viewport_h: entry.def.viewport.height,
        objects,
    };

    render_frame(&item, asset_root)
}
