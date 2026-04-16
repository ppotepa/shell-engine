use engine_core::effects::Region;
use engine_core::scene::Sprite;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use engine_render_2d::RenderArea;
use std::collections::HashMap;

use super::render::RenderCtx;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_obj_sprite(
    sprite: &Sprite,
    area: RenderArea,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    super::obj_render_adapter::render_obj_sprite(
        sprite,
        area,
        target_resolver,
        object_regions,
        object_id,
        object_state,
        appear_at,
        sprite_elapsed,
        ctx,
    );
}

pub(crate) fn render_scene_clip_sprite(
    sprite: &Sprite,
    area: RenderArea,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    object_regions: &mut HashMap<String, Region>,
    ctx: &mut RenderCtx<'_>,
) {
    super::scene_clip_render_adapter::render_scene_clip_sprite(
        sprite,
        area,
        object_id,
        object_state,
        object_regions,
        ctx,
    );
}
