//! Synchronous pre-render pass: renders all static OBJ sprites in a scene at every yaw step
//! into `ObjFrameCache` before the scene is activated.  Called during scene transition.

use std::sync::Arc;

use crossterm::style::Color;
use rayon::prelude::*;

use crate::obj_frame_cache::{BakeCacheKey, ObjBakeStatus, ObjFrameCache, YAW_STEP_DEG};
use crate::scene::{Layer, SceneRenderedMode, Sprite};
use crate::services::EngineWorldAccess;
use crate::systems::compositor::obj_render::{render_obj_to_canvas, ObjRenderParams};
use crate::world::World;

/// Parameters needed to bake one OBJ sprite at all yaw steps.
struct BakeTarget {
    source: String,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<crate::scene::SpriteSizePreset>,
    mode: SceneRenderedMode,
    params_base: ObjRenderParams,
    backface_cull: bool,
    fg: Color,
}

/// Collect all bakeable OBJ sprites from the layer tree.
fn collect_bake_targets(layers: &[Layer], mode: SceneRenderedMode) -> Vec<BakeTarget> {
    let mut targets = Vec::new();
    for layer in layers {
        collect_from_sprites(&layer.sprites, mode, &mut targets);
    }
    targets
}

fn collect_from_sprites(
    sprites: &[Sprite],
    mode: SceneRenderedMode,
    targets: &mut Vec<BakeTarget>,
) {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                source,
                size,
                width,
                height,
                force_renderer_mode,
                surface_mode: _,
                backface_cull,
                clip_y_min,
                clip_y_max,
                scale,
                yaw_deg: _,
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
                ..
            } => {
                // Skip if dynamically animated.
                let is_dynamic = rotate_y_deg_per_sec.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_orbit_hz.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_snap_hz.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_flicker_hz.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_2_orbit_hz.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_2_snap_hz.unwrap_or(0.0).abs() > f32::EPSILON
                    || light_point_2_flicker_hz.unwrap_or(0.0).abs() > f32::EPSILON;

                if is_dynamic {
                    continue;
                }

                let resolved_mode = crate::render_policy::resolve_renderer_mode(
                    mode,
                    *force_renderer_mode,
                );

                let fg = fg_colour
                    .as_ref()
                    .map(Color::from)
                    .unwrap_or(Color::White);

                // yaw_deg will be swept at bake time; store 0 here as placeholder.
                let params_base = ObjRenderParams {
                    scale: scale.unwrap_or(1.0),
                    yaw_deg: 0.0, // will be overridden per bake step
                    pitch_deg: pitch_deg.unwrap_or(0.0),
                    roll_deg: roll_deg.unwrap_or(0.0),
                    rotation_x: rotation_x.unwrap_or(0.0),
                    rotation_y: rotation_y.unwrap_or(0.0),
                    rotation_z: rotation_z.unwrap_or(0.0),
                    rotate_y_deg_per_sec: 0.0, // static
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
                    light_point_flicker_hz: 0.0,
                    light_point_orbit_hz: 0.0,
                    light_point_snap_hz: 0.0,
                    light_point_2_x: light_point_2_x.unwrap_or(0.0),
                    light_point_2_y: light_point_2_y.unwrap_or(0.0),
                    light_point_2_z: light_point_2_z.unwrap_or(0.0),
                    light_point_2_intensity: light_point_2_intensity.unwrap_or(0.0),
                    light_point_2_colour: light_point_2_colour.as_ref().map(Color::from),
                    light_point_2_flicker_depth: light_point_2_flicker_depth.unwrap_or(0.0),
                    light_point_2_flicker_hz: 0.0,
                    light_point_2_orbit_hz: 0.0,
                    light_point_2_snap_hz: 0.0,
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
                    clip_y_min: clip_y_min.unwrap_or(0.0),
                    clip_y_max: clip_y_max.unwrap_or(1.0),
                };

                targets.push(BakeTarget {
                    source: source.clone(),
                    width: *width,
                    height: *height,
                    size: *size,
                    mode: resolved_mode,
                    params_base,
                    backface_cull: backface_cull.unwrap_or(false),
                    fg,
                });
            }
            // Recurse into containers
            Sprite::Grid { children, .. } => {
                collect_from_sprites(children, mode, targets);
            }
            Sprite::Flex { children, .. } => {
                collect_from_sprites(children, mode, targets);
            }
            Sprite::Panel { children, .. } => {
                collect_from_sprites(children, mode, targets);
            }
            _ => {}
        }
    }
}

/// Synchronously pre-render all static OBJ sprites from the given scene layers.
///
/// Called during scene transition (before the scene is activated) when `scene.prerender == true`.
/// Blocks until every target × 72 yaw steps × 2 modes (wireframe + solid) is rendered.
/// Registers `ObjFrameCache` and `ObjBakeStatus::Ready` into the world.
pub fn prerender_scene_sprites(
    layers: &[Layer],
    scene_mode: SceneRenderedMode,
    scene_id: &str,
    world: &mut World,
) {
    let Some(asset_root) = world.asset_root().cloned() else {
        return;
    };

    let targets = collect_bake_targets(layers, scene_mode);
    if targets.is_empty() {
        engine_core::logging::info(
            "engine.bake",
            format!("scene={scene_id}: no bakeable OBJ sprites, skipping prerender"),
        );
        return;
    }

    let yaw_steps = 360u16 / YAW_STEP_DEG;
    let total = targets.len() * 2 * yaw_steps as usize;
    engine_core::logging::info(
        "engine.bake",
        format!(
            "scene={scene_id}: prerendering {} OBJ sprites × {} yaw × 2 = {} frames (sync)",
            targets.len(),
            yaw_steps,
            total,
        ),
    );

    // Pre-compute Arc<str> per target to avoid String clone per work item.
    let target_sources: Vec<Arc<str>> = targets.iter().map(|t| Arc::from(t.source.as_str())).collect();

    // Build a flat list of all (target_idx, wireframe, yaw_step) work items.
    let mut work_items: Vec<(usize, bool, u16)> = Vec::with_capacity(total);
    for (ti, _) in targets.iter().enumerate() {
        for wireframe in [false, true] {
            for step_idx in 0..yaw_steps {
                work_items.push((ti, wireframe, step_idx * YAW_STEP_DEG));
            }
        }
    }

    // Render all frames in parallel — each is fully independent.
    let results: Vec<(BakeCacheKey, Vec<Option<[u8; 3]>>)> = work_items
        .par_iter()
        .filter_map(|&(ti, wireframe, yaw_step)| {
            let target = &targets[ti];
            let mut params = target.params_base.clone();
            params.yaw_deg = yaw_step as f32;

            let (canvas, _vw, _vh) = render_obj_to_canvas(
                &target.source,
                target.width,
                target.height,
                target.size,
                target.mode,
                params,
                wireframe,
                target.backface_cull,
                target.fg,
                Some(&asset_root),
            )?;

            let total_yaw = target.params_base.rotation_y + yaw_step as f32;
            let key = BakeCacheKey {
                source: (*target_sources[ti]).to_owned(),
                wireframe,
                yaw_step: ObjFrameCache::snap_yaw(total_yaw),
            };
            Some((key, canvas))
        })
        .collect();

    let mut cache = ObjFrameCache::new();
    for (key, canvas) in results {
        cache.insert(key, canvas);
    }

    engine_core::logging::info(
        "engine.bake",
        format!("scene={scene_id}: prerender complete ({total} frames cached)"),
    );

    world.register(cache);
    world.register(ObjBakeStatus::Ready);
}
