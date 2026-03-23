//! Scene preparation step: pre-renders all frames declared in `.scene3d.yml` files
//! referenced by `Sprite::Scene3D` sprites in the active scene.
//!
//! Each named frame is rendered to a `Buffer` and stored in [`Scene3DAtlas`].
//! The atlas is registered as a scoped world resource so it lives for exactly
//! one scene.
//!
//! Clip frames (animated) generate sub-frames named `{frame_id}-{n}` (0-indexed),
//! where N = `clip.keyframes`.  The caller (behavior) picks the frame by name.
//!
//! All frames across all `.scene3d.yml` files are rendered in parallel via rayon.

use std::collections::{HashMap, HashSet};

use crossterm::style::Color;
use rayon::prelude::*;

use crate::buffer::Buffer;
use crate::render_policy;
use crate::scene::{Layer, Scene, SceneRenderedMode, Sprite};
use crate::scene3d_atlas::Scene3DAtlas;
use crate::scene3d_format::{load_scene3d, FrameDef, LightKind, Scene3DDefinition, SurfaceMode};
use crate::scene3d_resolve::resolve_scene3d_refs;
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::systems::compositor::obj_render::{
    blit_color_canvas, render_obj_to_shared_buffers,
    virtual_dimensions, ObjRenderParams,
};
use crate::world::World;

// ── Scene preparation step ─────────────────────────────────────────────────────

/// Renders every named frame of every `.scene3d.yml` referenced by the scene
/// into the [`Scene3DAtlas`] world resource before the scene is activated.
pub struct Scene3DPrerenderStep;

impl ScenePreparationStep for Scene3DPrerenderStep {
    fn name(&self) -> &'static str {
        "scene3d-prerender"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        let sources = collect_scene3d_sources(&scene.layers);
        if sources.is_empty() {
            return;
        }

        let Some(asset_root) = world.asset_root().cloned() else {
            engine_core::logging::warn(
                "engine.scene3d",
                format!("scene={}: no asset_root, skipping scene3d prerender", scene.id),
            );
            return;
        };

        let inherited_mode = scene.rendered_mode;
        let scene_id = scene.id.clone();

        engine_core::logging::info(
            "engine.scene3d",
            format!(
                "scene={scene_id}: prerendering {} scene3d source(s) (parallel)",
                sources.len()
            ),
        );

        // Build work items: one per (src, frame_id) pair.
        let work_items: Vec<WorkItem> = sources
            .iter()
            .flat_map(|src| {
                let path = asset_root.resolve(src);
                let path_str = path.to_string_lossy();
                let mut def = match load_scene3d(&path_str) {
                    Ok(d) => d,
                    Err(e) => {
                        engine_core::logging::warn(
                            "engine.scene3d",
                            format!("scene={scene_id}: failed to load {src}: {e}"),
                        );
                        return vec![];
                    }
                };
                resolve_scene3d_refs(&mut def, src, &asset_root);
                build_work_items(src, def, inherited_mode)
            })
            .collect();

        let total = work_items.len();
        engine_core::logging::info(
            "engine.scene3d",
            format!("scene={scene_id}: rendering {total} scene3d frame(s)"),
        );

        // Render all frames in parallel — each is fully independent.
        let rendered: Vec<(String, String, Buffer)> = work_items
            .into_par_iter()
            .filter_map(|item| {
                let buf = render_frame(&item, &asset_root)?;
                Some((item.src, item.frame_id, buf))
            })
            .collect();

        let count = rendered.len();
        let mut atlas = Scene3DAtlas::new();
        for (src, frame_id, buf) in rendered {
            atlas.insert(&src, &frame_id, buf);
        }

        engine_core::logging::info(
            "engine.scene3d",
            format!("scene={scene_id}: scene3d prerender complete ({count}/{total} frames cached)"),
        );

        world.register_scoped(atlas);
    }
}

// ── Collection ─────────────────────────────────────────────────────────────────

/// Collect all unique `src` values from `Sprite::Scene3D` in the layer tree.
fn collect_scene3d_sources(layers: &[Layer]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for layer in layers {
        collect_sources_from_sprites(&layer.sprites, &mut seen, &mut out);
    }
    out
}

fn collect_sources_from_sprites(sprites: &[Sprite], seen: &mut HashSet<String>, out: &mut Vec<String>) {
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

// ── Work items ─────────────────────────────────────────────────────────────────

struct WorkItem {
    src: String,
    frame_id: String,
    viewport_w: u16,
    viewport_h: u16,
    mode: SceneRenderedMode,
    /// Objects to render, in order (back-to-front).
    objects: Vec<ObjectRenderSpec>,
}

struct ObjectRenderSpec {
    mesh: String,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
}

/// Expand a `Scene3DDefinition` into one `WorkItem` per frame (or N per clip).
fn build_work_items(
    src: &str,
    def: Scene3DDefinition,
    inherited_mode: SceneRenderedMode,
) -> Vec<WorkItem> {
    let mode = render_policy::resolve_renderer_mode(inherited_mode, def.viewport.rendered_mode);

    let vw = def.viewport.width;
    let vh = def.viewport.height;

    // Pre-bake light params that are shared across frames.
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
                    &light_params,
                    &[],  // no tweens
                    0.0,  // t = 0
                );
                items.push(WorkItem {
                    src: src.to_string(),
                    frame_id: frame_id.clone(),
                    viewport_w: vw,
                    viewport_h: vh,
                    mode,
                    objects,
                });
            }
            FrameDef::Clip(clip_def) => {
                let n = clip_def.clip.keyframes.max(1);
                for kf in 0..n {
                    let t = if n <= 1 {
                        0.0f32
                    } else {
                        kf as f32 / (n - 1) as f32
                    };
                    let sub_frame_id = format!("{frame_id}-{kf}");
                    let objects = build_object_specs(
                        &clip_def.show,
                        &def.objects,
                        &def.materials,
                        &def.camera,
                        &light_params,
                        &clip_def.clip.tweens,
                        t,
                    );
                    items.push(WorkItem {
                        src: src.to_string(),
                        frame_id: sub_frame_id,
                        viewport_w: vw,
                        viewport_h: vh,
                        mode,
                        objects,
                    });
                }
            }
        }
    }

    items
}

// ── Object spec building ────────────────────────────────────────────────────────

struct LightParams {
    dir1: [f32; 3],
    dir2: [f32; 3],
    dir2_intensity: f32,
    point1: [f32; 3],
    point1_intensity: f32,
    point1_colour: Option<Color>,
    point1_snap_hz: f32,
    point2: [f32; 3],
    point2_intensity: f32,
    point2_colour: Option<Color>,
    point2_snap_hz: f32,
}

fn extract_light_params(lights: &[crate::scene3d_format::LightDef]) -> LightParams {
    let mut p = LightParams {
        dir1: [-0.45, 0.70, -0.85],
        dir2: [0.0, 0.0, -1.0],
        dir2_intensity: 0.0,
        point1: [0.0, 2.0, 0.0],
        point1_intensity: 0.0,
        point1_colour: None,
        point1_snap_hz: 0.0,
        point2: [0.0, 0.0, 0.0],
        point2_intensity: 0.0,
        point2_colour: None,
        point2_snap_hz: 0.0,
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
                    point_count += 1;
                } else if point_count == 1 {
                    p.point2 = pos;
                    p.point2_intensity = light.intensity;
                    p.point2_colour = colour;
                    p.point2_snap_hz = light.snap_hz;
                    point_count += 1;
                }
            }
            LightKind::Ambient => {} // not yet implemented in ObjRenderParams
        }
    }

    p
}

fn build_object_specs(
    show: &[String],
    objects: &[crate::scene3d_format::ObjectDef],
    materials: &HashMap<String, crate::scene3d_format::MaterialDef>,
    camera: &crate::scene3d_format::CameraDef,
    lights: &LightParams,
    tweens: &[crate::scene3d_format::TweenDef],
    t: f32,
) -> Vec<ObjectRenderSpec> {
    // Per-object tween values at this t.
    let mut tween_values: HashMap<String, HashMap<String, f32>> = HashMap::new();
    for tw in tweens {
        let et = tw.easing.apply(t);
        let value = tw.from + (tw.to - tw.from) * et;
        tween_values
            .entry(tw.object.clone())
            .or_default()
            .insert(tw.property.clone(), value);
    }

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

            let tf = &obj.transform;
            let cam_pos = camera.position.unwrap_or([0.0, 0.0, camera.distance]);
            let camera_distance = (cam_pos[0].powi(2) + cam_pos[1].powi(2) + cam_pos[2].powi(2))
                .sqrt()
                .max(camera.distance.abs());

            let wireframe = mat.surface_mode == SurfaceMode::Wireframe;
            let fg = mat.fg_colour.as_deref().and_then(parse_hex_color).unwrap_or(Color::White);

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

// ── Frame rendering ─────────────────────────────────────────────────────────────

fn render_frame(item: &WorkItem, asset_root: &crate::assets::AssetRoot) -> Option<Buffer> {
    let mut buf = Buffer::new(item.viewport_w, item.viewport_h);
    let (virtual_w, virtual_h) = virtual_dimensions(item.mode, item.viewport_w, item.viewport_h);
    let canvas_size = virtual_w as usize * virtual_h as usize;
    if canvas_size == 0 {
        return Some(buf);
    }

    let mut canvas: Vec<Option<[u8; 3]>> = vec![None; canvas_size];
    let mut depth_buf: Vec<f32> = vec![f32::INFINITY; canvas_size];

    // Render solid objects first (fill depth buffer), then wireframe on top.
    // Shared depth buffer ensures wire edges behind solid faces are correctly culled.
    for obj in item.objects.iter().filter(|o| !o.wireframe) {
        render_obj_to_shared_buffers(
            &obj.mesh, item.viewport_w, item.viewport_h, item.mode,
            obj.params, obj.wireframe, obj.backface_cull, obj.fg,
            Some(asset_root), &mut canvas, &mut depth_buf,
        );
    }
    for obj in item.objects.iter().filter(|o| o.wireframe) {
        render_obj_to_shared_buffers(
            &obj.mesh, item.viewport_w, item.viewport_h, item.mode,
            obj.params, obj.wireframe, obj.backface_cull, obj.fg,
            Some(asset_root), &mut canvas, &mut depth_buf,
        );
    }

    blit_color_canvas(
        &mut buf, item.mode, &canvas, virtual_w, virtual_h,
        item.viewport_w, item.viewport_h, 0, 0, false, '#',
        Color::White, Color::Reset, 0, virtual_h as usize,
    );

    Some(buf)
}

// ── Helpers ─────────────────────────────────────────────────────────────────────

/// Parse `#rrggbb` → `Color::Rgb`.
fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgb { r, g, b })
    } else {
        None
    }
}
