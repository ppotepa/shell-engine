use engine_core::effects::Region;
use engine_core::scene_runtime_types::ObjectRuntimeState;
use engine_render::rasterizer::blit;
use engine_render_2d::RenderArea;
use engine_render_3d::pipeline::SceneClipSpriteSpec;
use engine_render_3d::prerender::{
    render_scene3d_frame_at_with, render_scene3d_work_item, Scene3DAtlas, Scene3DRuntimeStore,
};
use engine_render_3d::scene::Renderable3D;
use std::collections::HashMap;

use super::render::RenderCtx;

pub(crate) fn render_scene_clip_sprite(
    spec: SceneClipSpriteSpec<'_>,
    area: RenderArea,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    object_regions: &mut HashMap<String, Region>,
    ctx: &mut RenderCtx<'_>,
) {
    let stretch_to_area = spec.stretch_to_area;
    let node = spec.node;
    let Renderable3D::SceneClip(scene_clip) = node.renderable else {
        return;
    };

    let draw_x = area
        .origin_x
        .saturating_add(node.transform.translation[0].round() as i32)
        .saturating_add(object_state.offset_x)
        .max(0) as u16;
    let draw_y = area
        .origin_y
        .saturating_add(node.transform.translation[1].round() as i32)
        .saturating_add(object_state.offset_y)
        .max(0) as u16;

    let rendered_realtime = if let (Some(entry), Some(asset_root)) = (
        Scene3DRuntimeStore::current_get(&scene_clip.source),
        ctx.asset_root,
    ) {
        if entry.def.frames.contains_key(scene_clip.frame.as_str()) {
            let buf = render_scene3d_frame_at_with(
                entry,
                &scene_clip.frame,
                ctx.scene_elapsed_ms,
                asset_root,
                scene_clip.use_scene_camera.then_some(ctx.scene_camera_3d),
                render_scene3d_work_item,
            );
            if let Some(buf) = buf {
                if stretch_to_area {
                    blit_scaled_nearest(
                        &buf,
                        ctx.layer_buf,
                        draw_x,
                        draw_y,
                        area.width.max(1),
                        area.height.max(1),
                    );
                } else {
                    blit(&buf, ctx.layer_buf, draw_x, draw_y);
                }
                if let Some(id) = object_id {
                    object_regions.insert(
                        id.to_string(),
                        engine_core::effects::Region {
                            x: draw_x,
                            y: draw_y,
                            width: if stretch_to_area {
                                area.width.max(1)
                            } else {
                                buf.width
                            },
                            height: if stretch_to_area {
                                area.height.max(1)
                            } else {
                                buf.height
                            },
                        },
                    );
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if !rendered_realtime && !scene_clip.use_scene_camera {
        if let Some(buf) = Scene3DAtlas::current_get(&scene_clip.source, &scene_clip.frame) {
            if stretch_to_area {
                blit_scaled_nearest(
                    &buf,
                    ctx.layer_buf,
                    draw_x,
                    draw_y,
                    area.width.max(1),
                    area.height.max(1),
                );
            } else {
                blit(&buf, ctx.layer_buf, draw_x, draw_y);
            }
            if let Some(id) = object_id {
                object_regions.insert(
                    id.to_string(),
                    engine_core::effects::Region {
                        x: draw_x,
                        y: draw_y,
                        width: if stretch_to_area {
                            area.width.max(1)
                        } else {
                            buf.width
                        },
                        height: if stretch_to_area {
                            area.height.max(1)
                        } else {
                            buf.height
                        },
                    },
                );
            }
        }
    }
}

fn blit_scaled_nearest(
    src: &engine_core::buffer::Buffer,
    dst: &mut engine_core::buffer::Buffer,
    dst_x: u16,
    dst_y: u16,
    dst_w: u16,
    dst_h: u16,
) {
    if src.width == 0 || src.height == 0 || dst_w == 0 || dst_h == 0 {
        return;
    }

    let src_w = src.width as u32;
    let src_h = src.height as u32;
    let target_w = dst_w as u32;
    let target_h = dst_h as u32;

    for y in 0..target_h {
        let sy = ((y * src_h) / target_h).min(src_h - 1) as u16;
        let ty = dst_y.saturating_add(y as u16);
        if ty >= dst.height {
            continue;
        }
        for x in 0..target_w {
            let sx = ((x * src_w) / target_w).min(src_w - 1) as u16;
            let tx = dst_x.saturating_add(x as u16);
            if tx >= dst.width {
                continue;
            }
            if let Some(cell) = src.get(sx, sy) {
                dst.set(tx, ty, cell.symbol, cell.fg, cell.bg);
            }
        }
    }

    let (src_pc, dst_pc) = match (&src.pixel_canvas, &mut dst.pixel_canvas) {
        (Some(src_pc), Some(dst_pc)) => (src_pc, dst_pc),
        _ => return,
    };
    if src_pc.width == 0 || src_pc.height == 0 || dst_pc.width == 0 || dst_pc.height == 0 {
        return;
    }

    let src_pw = src_pc.width as u32;
    let src_ph = src_pc.height as u32;
    let dst_pw = dst_pc.width as u32;
    let dst_ph = dst_pc.height as u32;
    let base_x = dst_x as u32;
    let base_y = dst_y as u32;
    let max_x = (base_x + target_w).min(dst_pw);
    let max_y = (base_y + target_h).min(dst_ph);
    let src_stride = src_pc.width as usize * 4;
    let dst_stride = dst_pc.width as usize * 4;

    for py in base_y..max_y {
        let local_y = py - base_y;
        let sy = ((local_y * src_ph) / target_h).min(src_ph - 1) as usize;
        for px in base_x..max_x {
            let local_x = px - base_x;
            let sx = ((local_x * src_pw) / target_w).min(src_pw - 1) as usize;
            let src_i = sy * src_stride + sx * 4;
            let dst_i = py as usize * dst_stride + px as usize * 4;
            dst_pc.data[dst_i..dst_i + 4].copy_from_slice(&src_pc.data[src_i..src_i + 4]);
        }
    }
    dst_pc.dirty = true;
}
