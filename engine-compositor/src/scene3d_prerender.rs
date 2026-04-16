use engine_core::color::Color;
use rayon::prelude::*;

use engine_3d::scene3d_format::FrameDef;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::logging;
use engine_core::scene::Scene;
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_render_3d::prerender::{
    build_object_specs, build_work_items, clip_progress_at, collect_scene3d_sources,
    extract_light_params, load_and_resolve_scene3d, Scene3DWorkItem,
};

use crate::{
    blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions, Scene3DAtlas,
};
use engine_render_3d::prerender::Scene3DRuntimeEntry;

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

    let work_items: Vec<Scene3DWorkItem> = sources
        .iter()
        .flat_map(|src| {
            let def = match load_and_resolve_scene3d(asset_root, src) {
                Ok(d) => d,
                Err(e) => {
                    logging::warn(
                        "engine.scene3d",
                        format!("scene={scene_id}: failed to load {src}: {e}"),
                    );
                    return Vec::new();
                }
            };
            build_work_items(src, &def)
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

fn render_frame(item: &Scene3DWorkItem, asset_root: &AssetRoot) -> Option<Buffer> {
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

/// Render a single frame of a Scene3D clip at a given `elapsed_ms` within the clip's timeline.
///
/// `clip_name` must be the bare clip frame key (e.g. `"orbit"`), **not** a keyframe id
/// like `"orbit-7"`. Returns `None` if the clip is not found or the scene has no objects.
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
            let t = clip_progress_at(elapsed_ms, clip.clip.duration_ms as u64);
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

    let item = Scene3DWorkItem {
        src: String::new(),
        frame_id: frame_name.to_string(),
        viewport_w: entry.def.viewport.width,
        viewport_h: entry.def.viewport.height,
        objects,
    };

    render_frame(&item, asset_root)
}
