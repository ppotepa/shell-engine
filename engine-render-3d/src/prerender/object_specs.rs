use std::collections::HashMap;

use engine_3d::scene3d_format::{CameraDef, MaterialDef, ObjectDef, SurfaceMode, TweenDef};
use engine_core::color::Color;
use engine_core::scene_runtime_types::SceneCamera3D;

use crate::ObjRenderParams;

use super::{
    evaluate_tween_values, parse_hex_color, resolve_camera_frame_state,
    resolve_object_frame_motion, LightParams,
};

#[derive(Debug, Clone)]
pub struct ObjectRenderSpec {
    pub mesh: String,
    pub params: ObjRenderParams,
    pub wireframe: bool,
    pub backface_cull: bool,
    pub fg: Color,
}

pub fn build_object_specs(
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
    let tween_values = evaluate_tween_values(tweens, t);
    let camera_state = resolve_camera_frame_state(camera, camera_override, &tween_values);

    objects
        .iter()
        .filter(|obj| show.contains(&obj.id))
        .filter_map(|obj| {
            let mat = materials.get(&obj.material)?;
            let obj_tweens = tween_values.get(&obj.id);
            let motion = resolve_object_frame_motion(obj, obj_tweens, clip_orbit_origin);

            let tf = &obj.transform;

            let (view_right, view_up, view_forward) = (
                camera_state.view_right,
                camera_state.view_up,
                camera_state.view_forward,
            );
            let cam_pos = camera_state.camera_position;

            let wireframe = mat.surface_mode == SurfaceMode::Wireframe;
            let fg = mat
                .fg_colour
                .as_deref()
                .and_then(parse_hex_color)
                .unwrap_or(Color::White);
            let (rot_x, rot_y, rot_z) = tf.resolved_rotation();
            let params = ObjRenderParams {
                scale: tf.resolved_scale(),
                yaw_deg: rot_y + motion.yaw_offset,
                pitch_deg: rot_x,
                roll_deg: rot_z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                rotation_z: 0.0,
                rotate_y_deg_per_sec: 0.0,
                camera_distance: camera_state.camera_distance,
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
                object_translate_x: motion.translation_x,
                object_translate_y: motion.translation_y,
                object_translate_z: motion.translation_z,
                clip_y_min: motion.clip_y_min,
                clip_y_max: motion.clip_y_max,
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
                ambient_floor: 0.06,
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
                atmo_height: 0.12,
                atmo_density: 0.0,
                atmo_strength: 0.0,
                atmo_rayleigh_amount: 0.0,
                atmo_rayleigh_color: None,
                atmo_rayleigh_falloff: 0.32,
                atmo_haze_amount: 0.0,
                atmo_haze_color: None,
                atmo_haze_falloff: 0.18,
                atmo_absorption_amount: 0.0,
                atmo_absorption_color: None,
                atmo_absorption_height: 0.55,
                atmo_absorption_width: 0.18,
                atmo_forward_scatter: 0.72,
                atmo_limb_boost: 1.0,
                atmo_terminator_softness: 1.0,
                atmo_night_glow: 0.0,
                atmo_night_glow_color: None,
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
