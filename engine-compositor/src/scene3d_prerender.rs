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
use engine_core::scene::{Layer, Scene, SceneRenderedMode, Sprite};

use crate::{
    blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions, ObjRenderParams,
    Scene3DAtlas,
};

pub fn prerender_scene3d_atlas(scene: &Scene, asset_root: &AssetRoot) -> Option<Scene3DAtlas> {
    let sources = collect_scene3d_sources(&scene.layers);
    if sources.is_empty() {
        return None;
    }

    let inherited_mode = scene.rendered_mode;
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
            build_work_items(src, def, inherited_mode)
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

struct WorkItem {
    src: String,
    frame_id: String,
    viewport_w: u16,
    viewport_h: u16,
    mode: SceneRenderedMode,
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
    inherited_mode: SceneRenderedMode,
) -> Vec<WorkItem> {
    let mode =
        engine_render_policy::resolve_renderer_mode(inherited_mode, def.viewport.rendered_mode);
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
                    &light_params,
                    &[],
                    0.0,
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
                        0.0
                    } else {
                        kf as f32 / (n - 1) as f32
                    };
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
                        frame_id: format!("{frame_id}-{kf}"),
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

fn extract_light_params(lights: &[LightDef]) -> LightParams {
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
            LightKind::Ambient => {}
        }
    }

    p
}

fn build_object_specs(
    show: &[String],
    objects: &[ObjectDef],
    materials: &HashMap<String, MaterialDef>,
    camera: &CameraDef,
    lights: &LightParams,
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
    let (virtual_w, virtual_h) = virtual_dimensions(item.mode, item.viewport_w, item.viewport_h);
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
            item.mode,
            obj.params,
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
            item.mode,
            obj.params,
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
        item.mode,
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
