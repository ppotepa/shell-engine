use engine_core::color::Color;

use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::scene::Scene;
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_render_3d::prerender::{
    prerender_scene3d_atlas_with, render_scene3d_frame_at_with, Scene3DWorkItem,
};

use crate::{
    blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions, Scene3DAtlas,
};
use engine_render_3d::prerender::Scene3DRuntimeEntry;

pub fn prerender_scene3d_atlas(scene: &Scene, asset_root: &AssetRoot) -> Option<Scene3DAtlas> {
    prerender_scene3d_atlas_with(scene, asset_root, render_frame)
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
    render_scene3d_frame_at_with(
        entry,
        frame_name,
        elapsed_ms,
        asset_root,
        camera_override,
        render_frame,
    )
}
