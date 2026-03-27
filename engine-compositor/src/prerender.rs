use std::sync::Arc;

use engine_core::color::Color;
use rayon::prelude::*;

use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::{Layer, SceneRenderedMode, Sprite, SpriteSizePreset};

use crate::obj_prerender::{ObjPrerenderedFrames, PrerenderedFrame};
use crate::{obj_sprite_dimensions, render_obj_to_canvas, ObjRenderParams};

pub fn prerender_scene_sprites(
    layers: &[Layer],
    scene_mode: SceneRenderedMode,
    scene_id: &str,
    asset_root: &AssetRoot,
) -> Option<ObjPrerenderedFrames> {
    let targets = collect_targets(layers, scene_mode);
    if targets.is_empty() {
        logging::info(
            "engine.prerender",
            format!("scene={scene_id}: no prerenderable OBJ sprites, skipping"),
        );
        return None;
    }

    logging::info(
        "engine.prerender",
        format!(
            "scene={scene_id}: prerendering {} OBJ sprites (parallel)",
            targets.len()
        ),
    );

    let results: Vec<(String, PrerenderedFrame)> = targets
        .par_iter()
        .filter_map(|target| {
            let (canvas, virtual_w, virtual_h) = render_obj_to_canvas(
                &target.source,
                target.width,
                target.height,
                target.size,
                target.mode,
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

    let count = results.len();
    let mut frames = ObjPrerenderedFrames::new();
    for (id, frame) in results {
        frames.insert(id, frame);
    }

    logging::info(
        "engine.prerender",
        format!("scene={scene_id}: prerender complete ({count} sprites cached)"),
    );

    Some(frames)
}

struct PrerenderTarget {
    sprite_id: String,
    source: String,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    mode: SceneRenderedMode,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
}

#[inline]
fn collect_targets(layers: &[Layer], scene_mode: SceneRenderedMode) -> Vec<PrerenderTarget> {
    let mut targets = Vec::new();
    for layer in layers {
        collect_from_sprites(&layer.sprites, scene_mode, &mut targets);
    }
    targets
}

#[inline]
fn collect_from_sprites(
    sprites: &[Sprite],
    mode: SceneRenderedMode,
    out: &mut Vec<PrerenderTarget>,
) {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                source,
                size,
                width,
                height,
                force_renderer_mode,
                surface_mode,
                backface_cull,
                scale,
                yaw_deg,
                pitch_deg,
                roll_deg,
                rotation_x,
                rotation_y,
                rotation_z,
                rotate_y_deg_per_sec,
                camera_distance,
                fov_degrees,
                near_clip,
                light_direction_x,
                light_direction_y,
                light_direction_z,
                light_2_direction_x,
                light_2_direction_y,
                light_2_direction_z,
                light_2_intensity,
                light_point_x,
                light_point_y,
                light_point_z,
                light_point_intensity,
                light_point_colour,
                light_point_flicker_depth,
                light_point_flicker_hz,
                light_point_orbit_hz,
                light_point_snap_hz,
                light_point_2_x,
                light_point_2_y,
                light_point_2_z,
                light_point_2_intensity,
                light_point_2_colour,
                light_point_2_flicker_depth,
                light_point_2_flicker_hz,
                light_point_2_orbit_hz,
                light_point_2_snap_hz,
                cel_levels,
                shadow_colour,
                midtone_colour,
                highlight_colour,
                tone_mix,
                fg_colour,
                prerender,
                clip_y_min: _,
                clip_y_max: _,
                ..
            } => {
                if !prerender {
                    continue;
                }
                if rotate_y_deg_per_sec.unwrap_or(0.0).abs() > f32::EPSILON {
                    continue;
                }

                let resolved_mode =
                    engine_render_policy::resolve_renderer_mode(mode, *force_renderer_mode);
                let is_wireframe = surface_mode
                    .as_deref()
                    .map(|s| s.trim().eq_ignore_ascii_case("wireframe"))
                    .unwrap_or(false);
                let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);

                let params = ObjRenderParams {
                    scale: scale.unwrap_or(1.0),
                    yaw_deg: yaw_deg.unwrap_or(0.0),
                    pitch_deg: pitch_deg.unwrap_or(0.0),
                    roll_deg: roll_deg.unwrap_or(0.0),
                    rotation_x: rotation_x.unwrap_or(0.0),
                    rotation_y: rotation_y.unwrap_or(0.0),
                    rotation_z: rotation_z.unwrap_or(0.0),
                    rotate_y_deg_per_sec: 0.0,
                    camera_distance: camera_distance.unwrap_or(3.0),
                    fov_degrees: fov_degrees.unwrap_or(60.0),
                    near_clip: near_clip.unwrap_or(0.001),
                    light_direction_x: light_direction_x.unwrap_or(-0.45),
                    light_direction_y: light_direction_y.unwrap_or(0.70),
                    light_direction_z: light_direction_z.unwrap_or(-0.85),
                    light_2_direction_x: light_2_direction_x.unwrap_or(0.0),
                    light_2_direction_y: light_2_direction_y.unwrap_or(0.0),
                    light_2_direction_z: light_2_direction_z.unwrap_or(-1.0),
                    light_2_intensity: light_2_intensity.unwrap_or(0.0),
                    light_point_x: light_point_x.unwrap_or(0.0),
                    light_point_y: light_point_y.unwrap_or(2.0),
                    light_point_z: light_point_z.unwrap_or(0.0),
                    light_point_intensity: light_point_intensity.unwrap_or(0.0),
                    light_point_colour: light_point_colour.as_ref().map(Color::from),
                    light_point_flicker_depth: light_point_flicker_depth.unwrap_or(0.0),
                    light_point_flicker_hz: light_point_flicker_hz.unwrap_or(0.0),
                    light_point_orbit_hz: light_point_orbit_hz.unwrap_or(0.0),
                    light_point_snap_hz: light_point_snap_hz.unwrap_or(0.0),
                    light_point_2_x: light_point_2_x.unwrap_or(0.0),
                    light_point_2_y: light_point_2_y.unwrap_or(0.0),
                    light_point_2_z: light_point_2_z.unwrap_or(0.0),
                    light_point_2_intensity: light_point_2_intensity.unwrap_or(0.0),
                    light_point_2_colour: light_point_2_colour.as_ref().map(Color::from),
                    light_point_2_flicker_depth: light_point_2_flicker_depth.unwrap_or(0.0),
                    light_point_2_flicker_hz: light_point_2_flicker_hz.unwrap_or(0.0),
                    light_point_2_orbit_hz: light_point_2_orbit_hz.unwrap_or(0.0),
                    light_point_2_snap_hz: light_point_2_snap_hz.unwrap_or(0.0),
                    cel_levels: cel_levels.unwrap_or(0),
                    shadow_colour: shadow_colour.as_ref().map(Color::from),
                    midtone_colour: midtone_colour.as_ref().map(Color::from),
                    highlight_colour: highlight_colour.as_ref().map(Color::from),
                    tone_mix: tone_mix.unwrap_or(0.0),
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
                };

                out.push(PrerenderTarget {
                    sprite_id: id.clone(),
                    source: source.clone(),
                    width: *width,
                    height: *height,
                    size: *size,
                    mode: resolved_mode,
                    params,
                    wireframe: is_wireframe,
                    backface_cull: backface_cull.unwrap_or(false),
                    fg,
                });
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => collect_from_sprites(children, mode, out),
            _ => {}
        }
    }
}
