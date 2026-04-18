//! Common render-time context and helpers shared across sprite variants.

use crate::ObjPrerenderedFrames;
use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::animations::AnimationDispatcher;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::scene::LayerStages;
use engine_core::scene::ResolvedViewProfile;
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_core::spatial::SpatialContext;
use std::collections::HashMap;

use super::super::effect_applicator::apply_sprite_effects;

// One AnimationDispatcher per thread — avoids rebuilding the HashMap registry every sprite.
thread_local! {
    static ANIM_DISPATCHER: AnimationDispatcher = AnimationDispatcher::new();
}

/// Shared mutable state needed while recursively rendering sprites.
pub(crate) struct RenderCtx<'a> {
    pub(crate) asset_root: Option<&'a AssetRoot>,
    pub(crate) scene_elapsed_ms: u64,
    pub(crate) current_stage: &'a SceneStage,
    pub(crate) step_idx: usize,
    pub(crate) elapsed_ms: u64,
    pub(crate) layer_buf: &'a mut Buffer,
    pub(crate) obj_camera_states: &'a HashMap<String, ObjCameraState>,
    pub(crate) scene_camera_3d: &'a SceneCamera3D,
    pub(crate) spatial_context: SpatialContext,
    pub(crate) celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub(crate) is_pixel_backend: bool,
    pub(crate) default_font: Option<&'a str>,
    pub(crate) ui_font_scale: f32,
    pub(crate) ui_layout_scale_x: f32,
    pub(crate) ui_layout_scale_y: f32,
    pub(crate) prerender_frames: Option<&'a ObjPrerenderedFrames>,
    pub(crate) resolved_view_profile: &'a ResolvedViewProfile,
}

/// Returns `Some(appear_at)` when the sprite should be rendered, `None` to skip.
/// Called per sprite per frame — inline to eliminate function call overhead.
#[inline]
pub(crate) fn check_visibility(
    hide_on_leave: bool,
    appear_at_ms: Option<u64>,
    disappear_at_ms: Option<u64>,
    current_stage: &SceneStage,
    scene_elapsed_ms: u64,
) -> Option<u64> {
    if hide_on_leave && matches!(current_stage, SceneStage::OnLeave) {
        return None;
    }
    let appear_at = appear_at_ms.unwrap_or(0);
    if scene_elapsed_ms < appear_at {
        return None;
    }
    if let Some(disappear_at) = disappear_at_ms {
        if scene_elapsed_ms >= disappear_at {
            return None;
        }
    }
    Some(appear_at)
}

/// Checks if sprite bounds are completely outside viewport.
/// Used for OPT-36: sprite culling acceleration.
#[inline(always)]
pub(crate) fn is_sprite_offscreen(
    x: i32,
    y: i32,
    w: u16,
    h: u16,
    scene_w: u16,
    scene_h: u16,
) -> bool {
    let x_end = x + w as i32;
    let y_end = y + h as i32;
    x_end <= 0 || x >= scene_w as i32 || y_end <= 0 || y >= scene_h as i32
}

/// Computes the aggregate animation offset for the current elapsed time.
#[inline]
pub(crate) fn sprite_transform_offset(
    animations: &[engine_core::scene::Animation],
    elapsed_ms: u64,
) -> (i32, i32) {
    ANIM_DISPATCHER.with(|d| {
        let t = d.compute_transform(animations, elapsed_ms);
        (t.dx as i32, t.dy as i32)
    })
}

/// Resolves the final draw position after animation and runtime object offsets.
/// Called per sprite to compute position — inline for hot path.
#[inline]
pub(crate) fn compute_draw_pos(
    base_x: i32,
    base_y: i32,
    animations: &[engine_core::scene::Animation],
    sprite_elapsed: u64,
    object_state: &ObjectRuntimeState,
) -> (u16, u16) {
    let (dx, dy) = sprite_transform_offset(animations, sprite_elapsed);
    let draw_x = base_x
        .saturating_add(dx)
        .saturating_add(object_state.offset_x)
        .max(0) as u16;
    let draw_y = base_y
        .saturating_add(dy)
        .saturating_add(object_state.offset_y)
        .max(0) as u16;
    (draw_x, draw_y)
}

/// Stores the object region and applies stage effects for the rendered sprite.
#[inline]
pub(crate) fn finalize_sprite(
    object_id: Option<&str>,
    sprite_region: Region,
    sprite_elapsed: u64,
    stages: &LayerStages,
    ctx: &mut RenderCtx<'_>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
) {
    if let Some(id) = object_id {
        object_regions.insert(id.to_string(), sprite_region);
    }
    apply_sprite_effects(
        stages,
        ctx.current_stage,
        ctx.step_idx,
        ctx.elapsed_ms,
        sprite_elapsed,
        sprite_region,
        target_resolver,
        object_regions,
        ctx.layer_buf,
    );
}
