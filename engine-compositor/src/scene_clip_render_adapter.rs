use engine_core::effects::Region;
use engine_core::scene::Sprite;
use engine_core::scene_runtime_types::ObjectRuntimeState;
use engine_render_2d::RenderArea;
use engine_render_3d::pipeline::extract_scene_clip_sprite_spec;
use engine_render_3d::scene::Renderable3D;
use std::collections::HashMap;

use super::render::RenderCtx;

pub(crate) fn render_scene_clip_sprite(
    sprite: &Sprite,
    area: RenderArea,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    object_regions: &mut HashMap<String, Region>,
    ctx: &mut RenderCtx<'_>,
) {
    let Some(spec) = extract_scene_clip_sprite_spec(sprite) else {
        return;
    };
    let node = spec.node;
    let Renderable3D::SceneClip(scene_clip) = node.renderable else {
        return;
    };

    use crate::Scene3DAtlas;
    use crate::Scene3DRuntimeStore;
    use engine_render::rasterizer::blit;

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
            let buf = crate::scene3d_prerender::render_scene3d_frame_at(
                entry,
                &scene_clip.frame,
                ctx.scene_elapsed_ms,
                asset_root,
                scene_clip.use_scene_camera.then_some(ctx.scene_camera_3d),
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
