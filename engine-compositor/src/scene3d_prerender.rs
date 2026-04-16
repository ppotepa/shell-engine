use engine_core::color::Color;

use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::scene::Scene;
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_render_3d::prerender::{
    prerender_scene3d_atlas_with, render_scene3d_frame_at_with, render_work_item_buffer_with,
    Scene3DWorkItem,
};

use crate::{
    blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions, Scene3DAtlas,
};
use engine_render_3d::prerender::Scene3DRuntimeEntry;

pub fn prerender_scene3d_atlas(scene: &Scene, asset_root: &AssetRoot) -> Option<Scene3DAtlas> {
    prerender_scene3d_atlas_with(scene, asset_root, render_frame)
}

fn render_frame(item: &Scene3DWorkItem, asset_root: &AssetRoot) -> Option<Buffer> {
    render_work_item_buffer_with(
        item,
        asset_root,
        virtual_dimensions,
        render_obj_to_shared_buffers,
        |buf, canvas, viewport_w, viewport_h| {
            blit_color_canvas(
                buf,
                &canvas.colors,
                canvas.virtual_w,
                canvas.virtual_h,
                viewport_w,
                viewport_h,
                0,
                0,
                false,
                '#',
                Color::White,
                Color::Reset,
                0,
                canvas.virtual_h as usize,
            );
        },
    )
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
