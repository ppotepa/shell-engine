use std::sync::Arc;

use engine_core::color::Color;
use rayon::prelude::*;

use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::{Layer, SpriteSizePreset};
use engine_render_3d::pipeline::{extract_obj_sprite_spec, ObjSpriteSpec};
use engine_render_3d::prerender::{
    AnimSpriteFrames, ObjPrerenderedFrames, PrerenderedFrame, YAW_FRAME_COUNT, YAW_STEP_DEG,
};

use crate::{obj_sprite_dimensions, render_obj_to_canvas, ObjRenderParams};

pub fn prerender_scene_sprites(
    layers: &[Layer],
    scene_id: &str,
    asset_root: &AssetRoot,
) -> Option<ObjPrerenderedFrames> {
    let targets = collect_targets(layers);
    let anim_targets = collect_anim_targets(layers);
    if targets.is_empty() && anim_targets.is_empty() {
        logging::info(
            "engine.prerender",
            format!("scene={scene_id}: no prerenderable OBJ sprites, skipping"),
        );
        return None;
    }

    // ── Static sprites ──────────────────────────────────────────────────────
    if !targets.is_empty() {
        logging::info(
            "engine.prerender",
            format!(
                "scene={scene_id}: prerendering {} static OBJ sprites (parallel)",
                targets.len()
            ),
        );
    }

    let results: Vec<(String, PrerenderedFrame)> = targets
        .par_iter()
        .filter_map(|target| {
            let (canvas, virtual_w, virtual_h) = render_obj_to_canvas(
                &target.source,
                target.width,
                target.height,
                target.size,
                target.params.clone(),
                target.wireframe,
                target.backface_cull,
                target.fg,
                Some(asset_root),
            )?;
            let (target_w, target_h) =
                obj_sprite_dimensions(target.width, target.height, target.size);
            let rendered_yaw = target.params.rotation_y + target.params.yaw_deg;
            let rendered_pitch = target.params.pitch_deg;
            Some((
                target.sprite_id.clone(),
                PrerenderedFrame {
                    canvas: Arc::new(canvas),
                    virtual_w,
                    virtual_h,
                    target_w,
                    target_h,
                    rendered_yaw,
                    rendered_pitch,
                },
            ))
        })
        .collect();

    let static_count = results.len();
    let mut frames = ObjPrerenderedFrames::new();
    for (id, frame) in results {
        frames.insert(id, frame);
    }

    // ── Animated sprites: bake all YAW_FRAME_COUNT yaw keyframes ─────────────
    if !anim_targets.is_empty() {
        logging::info(
            "engine.prerender",
            format!(
                "scene={scene_id}: baking {} animated OBJ sprites × {} yaw frames (parallel)",
                anim_targets.len(),
                YAW_FRAME_COUNT
            ),
        );
    }

    // Flatten to (sprite_id, yaw_step_index, job) for maximum rayon parallelism.
    let anim_jobs: Vec<(String, usize, &AnimPrerenderTarget)> = anim_targets
        .iter()
        .flat_map(|target| {
            (0..YAW_FRAME_COUNT).map(move |step| (target.sprite_id.clone(), step, target))
        })
        .collect();

    type BakeItem = (String, usize, Arc<Vec<Option<[u8; 3]>>>, u16, u16, u16, u16);
    let anim_results: Vec<BakeItem> = anim_jobs
        .par_iter()
        .filter_map(|(sprite_id, step, target)| {
            let yaw_deg = (*step as u16 * YAW_STEP_DEG) as f32;
            let mut params = target.base_params.clone();
            params.yaw_deg = yaw_deg;
            params.rotation_y = 0.0;
            params.rotate_y_deg_per_sec = 0.0;

            let (canvas, virtual_w, virtual_h) = render_obj_to_canvas(
                &target.source,
                target.width,
                target.height,
                target.size,
                params,
                target.wireframe,
                target.backface_cull,
                target.fg,
                Some(asset_root),
            )?;
            let (target_w, target_h) =
                obj_sprite_dimensions(target.width, target.height, target.size);
            Some((
                sprite_id.clone(),
                *step,
                Arc::new(canvas),
                virtual_w,
                virtual_h,
                target_w,
                target_h,
            ))
        })
        .collect();

    // Assemble per-sprite: 72 canvases in step order.
    type SlotVec = Vec<Option<Arc<Vec<Option<[u8; 3]>>>>>;
    let mut anim_by_id: std::collections::HashMap<String, (u16, u16, u16, u16, SlotVec)> =
        std::collections::HashMap::new();

    for (sprite_id, step, canvas, vw, vh, tw, th) in anim_results {
        let entry = anim_by_id
            .entry(sprite_id)
            .or_insert_with(|| (vw, vh, tw, th, vec![None; YAW_FRAME_COUNT]));
        if step < entry.4.len() {
            entry.4[step] = Some(canvas);
        }
    }

    let anim_count = anim_by_id.len();
    for (sprite_id, (vw, vh, tw, th, slots)) in anim_by_id {
        let canvases: Vec<Arc<Vec<Option<[u8; 3]>>>> = slots
            .into_iter()
            .map(|s| s.unwrap_or_else(|| Arc::new(Vec::new())))
            .collect();
        frames.insert_anim(
            sprite_id,
            AnimSpriteFrames {
                canvases,
                virtual_w: vw,
                virtual_h: vh,
                target_w: tw,
                target_h: th,
            },
        );
    }

    logging::info(
        "engine.prerender",
        format!(
            "scene={scene_id}: prerender complete ({static_count} static + {anim_count} animated sprites cached)"
        ),
    );

    Some(frames)
}

struct PrerenderTarget {
    sprite_id: String,
    source: String,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
}

/// Target for animated prerender: bakes all YAW_FRAME_COUNT yaw keyframes at scene load.
struct AnimPrerenderTarget {
    sprite_id: String,
    source: String,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    /// Base params — `yaw_deg` and `rotation_y` are overridden per keyframe; `rotate_y_deg_per_sec` is zeroed.
    base_params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
}

#[inline]
fn collect_targets(layers: &[Layer]) -> Vec<PrerenderTarget> {
    let mut targets = Vec::new();
    for layer in layers {
        for root in &layer.sprites {
            root.walk_recursive(&mut |sprite| {
                let Some(spec) = extract_obj_sprite_spec(sprite) else {
                    return;
                };
                let Some(id) = spec.id else {
                    return;
                };
                if !spec.prerender {
                    return;
                }
                if spec.rotate_y_deg_per_sec.unwrap_or(0.0).abs() > f32::EPSILON {
                    return;
                }
                let is_wireframe = spec
                    .surface_mode
                    .map(|mode| mode.trim().eq_ignore_ascii_case("wireframe"))
                    .unwrap_or(false);
                let fg = spec.fg_colour.map(Color::from).unwrap_or(Color::White);
                targets.push(PrerenderTarget {
                    sprite_id: id.to_string(),
                    source: spec.source.to_string(),
                    width: spec.width,
                    height: spec.height,
                    size: spec.size,
                    params: build_static_obj_prerender_params(&spec),
                    wireframe: is_wireframe,
                    backface_cull: spec.backface_cull.unwrap_or(false),
                    fg,
                });
            });
        }
    }
    targets
}

#[inline]
fn collect_anim_targets(layers: &[Layer]) -> Vec<AnimPrerenderTarget> {
    let mut targets = Vec::new();
    for layer in layers {
        for root in &layer.sprites {
            root.walk_recursive(&mut |sprite| {
                let Some(spec) = extract_obj_sprite_spec(sprite) else {
                    return;
                };
                let Some(id) = spec.id else {
                    return;
                };
                if !spec.prerender_anim {
                    return;
                }
                if spec.rotate_y_deg_per_sec.unwrap_or(0.0).abs() <= f32::EPSILON {
                    return;
                }
                let is_wireframe = spec
                    .surface_mode
                    .map(|mode| mode.trim().eq_ignore_ascii_case("wireframe"))
                    .unwrap_or(false);
                let fg = spec.fg_colour.map(Color::from).unwrap_or(Color::White);
                targets.push(AnimPrerenderTarget {
                    sprite_id: id.to_string(),
                    source: spec.source.to_string(),
                    width: spec.width,
                    height: spec.height,
                    size: spec.size,
                    base_params: build_anim_obj_prerender_params(&spec),
                    wireframe: is_wireframe,
                    backface_cull: spec.backface_cull.unwrap_or(false),
                    fg,
                });
            });
        }
    }
    targets
}

#[inline]
fn build_static_obj_prerender_params(spec: &ObjSpriteSpec<'_>) -> ObjRenderParams {
    ObjRenderParams {
        scale: spec.scale.unwrap_or(1.0),
        yaw_deg: spec.yaw_deg.unwrap_or(0.0),
        pitch_deg: spec.pitch_deg.unwrap_or(0.0),
        roll_deg: spec.roll_deg.unwrap_or(0.0),
        rotation_x: spec.rotation_x.unwrap_or(0.0),
        rotation_y: spec.rotation_y.unwrap_or(0.0),
        rotation_z: spec.rotation_z.unwrap_or(0.0),
        rotate_y_deg_per_sec: 0.0,
        camera_distance: spec.camera_distance.unwrap_or(3.0),
        fov_degrees: spec.fov_degrees.unwrap_or(60.0),
        near_clip: spec.near_clip.unwrap_or(0.001),
        light_direction_x: spec.light_direction_x.unwrap_or(-0.45),
        light_direction_y: spec.light_direction_y.unwrap_or(0.70),
        light_direction_z: spec.light_direction_z.unwrap_or(-0.85),
        light_2_direction_x: spec.light_2_direction_x.unwrap_or(0.0),
        light_2_direction_y: spec.light_2_direction_y.unwrap_or(0.0),
        light_2_direction_z: spec.light_2_direction_z.unwrap_or(-1.0),
        light_2_intensity: spec.light_2_intensity.unwrap_or(0.0),
        light_point_x: spec.light_point_x.unwrap_or(0.0),
        light_point_y: spec.light_point_y.unwrap_or(2.0),
        light_point_z: spec.light_point_z.unwrap_or(0.0),
        light_point_intensity: spec.light_point_intensity.unwrap_or(0.0),
        light_point_colour: spec.light_point_colour.map(Color::from),
        light_point_flicker_depth: spec.light_point_flicker_depth.unwrap_or(0.0),
        light_point_flicker_hz: spec.light_point_flicker_hz.unwrap_or(0.0),
        light_point_orbit_hz: spec.light_point_orbit_hz.unwrap_or(0.0),
        light_point_snap_hz: spec.light_point_snap_hz.unwrap_or(0.0),
        light_point_2_x: spec.light_point_2_x.unwrap_or(0.0),
        light_point_2_y: spec.light_point_2_y.unwrap_or(0.0),
        light_point_2_z: spec.light_point_2_z.unwrap_or(0.0),
        light_point_2_intensity: spec.light_point_2_intensity.unwrap_or(0.0),
        light_point_2_colour: spec.light_point_2_colour.map(Color::from),
        light_point_2_flicker_depth: spec.light_point_2_flicker_depth.unwrap_or(0.0),
        light_point_2_flicker_hz: spec.light_point_2_flicker_hz.unwrap_or(0.0),
        light_point_2_orbit_hz: spec.light_point_2_orbit_hz.unwrap_or(0.0),
        light_point_2_snap_hz: spec.light_point_2_snap_hz.unwrap_or(0.0),
        cel_levels: spec.cel_levels.unwrap_or(0),
        shadow_colour: spec.shadow_colour.map(Color::from),
        midtone_colour: spec.midtone_colour.map(Color::from),
        highlight_colour: spec.highlight_colour.map(Color::from),
        tone_mix: spec.tone_mix.unwrap_or(0.0),
        scene_elapsed_ms: 0,
        camera_pan_x: 0.0,
        camera_pan_y: 0.0,
        camera_look_yaw: 0.0,
        camera_look_pitch: 0.0,
        object_translate_x: 0.0,
        object_translate_y: 0.0,
        object_translate_z: 0.0,
        clip_y_min: 0.0,
        clip_y_max: 1.0,
        camera_world_x: 0.0,
        camera_world_y: 0.0,
        camera_world_z: -spec.camera_distance.unwrap_or(3.0),
        view_right_x: 1.0,
        view_right_y: 0.0,
        view_right_z: 0.0,
        view_up_x: 0.0,
        view_up_y: 1.0,
        view_up_z: 0.0,
        view_forward_x: 0.0,
        view_forward_y: 0.0,
        view_forward_z: 1.0,
        unlit: false,
        ambient: 0.0,
        light_point_falloff: 0.7,
        light_point_2_falloff: 0.7,
        smooth_shading: spec.smooth_shading.unwrap_or(false),
        latitude_bands: spec.latitude_bands.unwrap_or(0),
        latitude_band_depth: spec.latitude_band_depth.unwrap_or(0.0),
        terrain_color: spec.terrain_color.map(|value| {
            let (r, g, b) = Color::from(value).to_rgb();
            [r, g, b]
        }),
        terrain_threshold: spec.terrain_threshold.unwrap_or(0.5),
        terrain_noise_scale: spec.terrain_noise_scale.unwrap_or(2.5),
        terrain_noise_octaves: spec.terrain_noise_octaves.unwrap_or(2),
        marble_depth: spec.marble_depth.unwrap_or(0.0),
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
    }
}

#[inline]
fn build_anim_obj_prerender_params(spec: &ObjSpriteSpec<'_>) -> ObjRenderParams {
    let mut params = build_static_obj_prerender_params(spec);
    params.yaw_deg = 0.0;
    params.rotation_y = 0.0;
    params.rotate_y_deg_per_sec = 0.0;
    params
}
