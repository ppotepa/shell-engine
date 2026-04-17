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
                blit(&buf, ctx.layer_buf, draw_x, draw_y);
                if let Some(id) = object_id {
                    object_regions.insert(
                        id.to_string(),
                        engine_core::effects::Region {
                            x: draw_x,
                            y: draw_y,
                            width: buf.width,
                            height: buf.height,
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
            blit(&buf, ctx.layer_buf, draw_x, draw_y);
            if let Some(id) = object_id {
                object_regions.insert(
                    id.to_string(),
                    engine_core::effects::Region {
                        x: draw_x,
                        y: draw_y,
                        width: buf.width,
                        height: buf.height,
                    },
                );
            }
        }
    }
}

