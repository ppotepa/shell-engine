use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;

use super::Scene3DWorkItem;

pub use crate::pipeline::renderer::Scene3DColorCanvas;

pub fn render_work_item_canvas_with<F>(
    item: &Scene3DWorkItem,
    asset_root: &AssetRoot,
    virtual_w: u16,
    virtual_h: u16,
    render_object: F,
) -> Option<Scene3DColorCanvas>
where
    F: FnMut(
        &str,
        u16,
        u16,
        crate::ObjRenderParams,
        bool,
        bool,
        engine_core::color::Color,
        Option<&AssetRoot>,
        &mut [Option<[u8; 3]>],
        &mut [f32],
    ),
{
    crate::pipeline::renderer::render_scene3d_work_item_canvas_with(
        item,
        asset_root,
        virtual_w,
        virtual_h,
        render_object,
    )
}

pub fn render_work_item_buffer_with<FV, FR, FB>(
    item: &Scene3DWorkItem,
    asset_root: &AssetRoot,
    virtual_dimensions: FV,
    render_object: FR,
    blit_canvas: FB,
) -> Option<engine_core::buffer::Buffer>
where
    FV: FnOnce(u16, u16) -> (u16, u16),
    FR: FnMut(
        &str,
        u16,
        u16,
        crate::ObjRenderParams,
        bool,
        bool,
        engine_core::color::Color,
        Option<&AssetRoot>,
        &mut [Option<[u8; 3]>],
        &mut [f32],
    ),
    FB: FnOnce(&mut engine_core::buffer::Buffer, &Scene3DColorCanvas, u16, u16),
{
    crate::pipeline::renderer::render_scene3d_work_item_buffer_with(
        item,
        asset_root,
        virtual_dimensions,
        render_object,
        blit_canvas,
    )
}

/// Render a [`Scene3DWorkItem`] to a terminal [`Buffer`] using the built-in OBJ raster pipeline.
///
/// This is the default renderer for scene3d prerender and real-time clip rendering.
/// Owned by `engine-render-3d` so that neither `engine` nor the compositor
/// needs to import from `engine-compositor` for this concern.
pub fn render_scene3d_work_item(item: &Scene3DWorkItem, asset_root: &AssetRoot) -> Option<Buffer> {
    render_work_item_buffer_with(
        item,
        asset_root,
        crate::raster::virtual_dimensions,
        crate::raster::render_obj_to_shared_buffers,
        |buf, canvas, viewport_w, viewport_h| {
            crate::raster::blit_color_canvas(
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

