use engine_core::assets::AssetRoot;

use super::Scene3DWorkItem;

#[derive(Debug, Clone)]
pub struct Scene3DColorCanvas {
    pub virtual_w: u16,
    pub virtual_h: u16,
    pub colors: Vec<Option<[u8; 3]>>,
}

pub fn render_work_item_canvas_with<F>(
    item: &Scene3DWorkItem,
    asset_root: &AssetRoot,
    virtual_w: u16,
    virtual_h: u16,
    mut render_object: F,
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
    let canvas_size = virtual_w as usize * virtual_h as usize;
    if canvas_size == 0 {
        return None;
    }

    let mut canvas = vec![None; canvas_size];
    let mut depth_buf = vec![f32::INFINITY; canvas_size];

    for obj in item.objects.iter().filter(|o| !o.wireframe) {
        render_object(
            &obj.mesh,
            item.viewport_w,
            item.viewport_h,
            obj.params.clone(),
            obj.wireframe,
            obj.backface_cull,
            obj.fg,
            Some(asset_root),
            &mut canvas[..],
            &mut depth_buf[..],
        );
    }

    for obj in item.objects.iter().filter(|o| o.wireframe) {
        render_object(
            &obj.mesh,
            item.viewport_w,
            item.viewport_h,
            obj.params.clone(),
            obj.wireframe,
            obj.backface_cull,
            obj.fg,
            Some(asset_root),
            &mut canvas[..],
            &mut depth_buf[..],
        );
    }

    Some(Scene3DColorCanvas {
        virtual_w,
        virtual_h,
        colors: canvas,
    })
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
    FB: FnOnce(
        &mut engine_core::buffer::Buffer,
        &Scene3DColorCanvas,
        u16,
        u16,
    ),
{
    let mut buf = engine_core::buffer::Buffer::new(item.viewport_w, item.viewport_h);
    let (virtual_w, virtual_h) = virtual_dimensions(item.viewport_w, item.viewport_h);
    let Some(canvas) =
        render_work_item_canvas_with(item, asset_root, virtual_w, virtual_h, render_object)
    else {
        return Some(buf);
    };

    blit_canvas(&mut buf, &canvas, item.viewport_w, item.viewport_h);
    Some(buf)
}
