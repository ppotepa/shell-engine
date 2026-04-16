use engine_core::color::Color;

use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_render_3d::prerender::{
    render_scene3d_frame_at_with, render_work_item_buffer_with, Scene3DWorkItem,
};

use crate::{blit_color_canvas, render_obj_to_shared_buffers, virtual_dimensions};
use engine_render_3d::prerender::Scene3DRuntimeEntry;

/// Compositor-provided Scene3D work-item raster callback.
///
/// Engine-side Scene3D prerender orchestration lives in `engine-render-3d` and
/// invokes this callback for each prepared work item.
pub fn render_scene3d_work_item(item: &Scene3DWorkItem, asset_root: &AssetRoot) -> Option<Buffer> {
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
/// `clip_name` must be the bare clip frame key (e.g. `"main"`), **not** a keyframe id
/// like `"main-7"`. Returns `None` if the clip is not found or the scene has no objects.
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
        render_scene3d_work_item,
    )
}
