use crossterm::style::Color;

use crate::animations::AnimationDispatcher;
use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::markup::strip_markup;
use crate::render_policy;
use crate::scene::{Layer, LayerStages, SceneRenderedMode, Sprite};
use crate::scene_runtime::{ObjCameraState, ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;
use std::collections::BTreeMap;

use super::effect_applicator::apply_sprite_effects;
use super::layout::{
    compute_flex_cells, compute_grid_cells, measure_sprite_for_layout, resolve_x, resolve_y,
    RenderArea,
};
use super::image_render::{image_sprite_dimensions, render_image_content};
use super::obj_render::{obj_sprite_dimensions, render_obj_content, ObjRenderParams};
use super::text_render::{dim_colour, render_text_content, text_sprite_dimensions};

// One AnimationDispatcher per thread — avoids rebuilding the HashMap registry every sprite.
thread_local! {
    static ANIM_DISPATCHER: AnimationDispatcher = AnimationDispatcher::new();
}

struct RenderCtx<'a> {
    asset_root: Option<&'a AssetRoot>,
    scene_elapsed_ms: u64,
    current_stage: &'a SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    layer_buf: &'a mut Buffer,
    obj_camera_states: &'a BTreeMap<String, ObjCameraState>,
}

/// Render all sprites in a layer onto `layer_buf`.
pub fn render_sprites(
    layer_idx: usize,
    layer: &Layer,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut BTreeMap<String, Region>,
    root_origin_x: i32,
    root_origin_y: i32,
    object_states: &BTreeMap<String, ObjectRuntimeState>,
    scene_elapsed_ms: u64,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    obj_camera_states: &BTreeMap<String, ObjCameraState>,
    layer_buf: &mut Buffer,
) {
    let mut ctx = RenderCtx {
        asset_root,
        scene_elapsed_ms,
        current_stage,
        step_idx,
        elapsed_ms,
        layer_buf,
        obj_camera_states,
    };
    let root_area = RenderArea {
        origin_x: root_origin_x,
        origin_y: root_origin_y,
        width: scene_w,
        height: scene_h,
    };
    // Reuse one path Vec across sprites; Grid extends/truncates it in-place per child.
    let mut sprite_path: Vec<usize> = Vec::with_capacity(8);
    for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
        sprite_path.clear();
        sprite_path.push(sprite_idx);
        render_sprite(
            layer_idx,
            &mut sprite_path,
            sprite,
            root_area,
            scene_rendered_mode,
            target_resolver,
            object_regions,
            object_states,
            &mut ctx,
        );
    }
}

fn render_sprite(
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    sprite: &Sprite,
    area: RenderArea,
    inherited_mode: SceneRenderedMode,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut BTreeMap<String, Region>,
    object_states: &BTreeMap<String, ObjectRuntimeState>,
    ctx: &mut RenderCtx<'_>,
) {
    let object_id =
        target_resolver.and_then(|resolver| resolver.sprite_object_id(layer_idx, sprite_path));
    let object_state = object_id
        .and_then(|id| object_states.get(id))
        .cloned()
        .unwrap_or_default();
    if !object_state.visible {
        return;
    }

    // Visibility and timing are shared across all variants — resolve once before the match.
    let Some(appear_at) = check_visibility(
        sprite.hide_on_leave(),
        sprite.appear_at_ms(),
        sprite.disappear_at_ms(),
        ctx.current_stage,
        ctx.scene_elapsed_ms,
    ) else {
        return;
    };
    let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);

    match sprite {
        Sprite::Text { content, x, y, size, font, force_renderer_mode, force_font_mode,
                        align_x, align_y, fg_colour, bg_colour, reveal_ms, glow, .. } =>
        {
            let total_chars = content.chars().count();
            let rendered_content = match reveal_ms {
                Some(reveal) if *reveal > 0 => {
                    let since = ctx.scene_elapsed_ms - appear_at;
                    let p = (since as f32 / *reveal as f32).clamp(0.0, 1.0);
                    let visible_chars = ((total_chars as f32) * p).ceil() as usize;
                    content.chars().take(visible_chars).collect::<String>()
                }
                _ => content.clone(),
            };
            if rendered_content.is_empty() { return; }

            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let sprite_bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let resolved_font = render_policy::resolve_text_font_spec(
                font.as_deref(), force_font_mode.as_deref(), *size, inherited_mode, *force_renderer_mode,
            );
            let mod_source = ctx.asset_root.map(|root| root.mod_source());
            let (sprite_width, sprite_height) =
                text_sprite_dimensions(mod_source, &rendered_content, resolved_font.as_deref(), fg, sprite_bg);

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(base_x, base_y, sprite.animations(), sprite_elapsed, &object_state);

            if let Some(glow_opts) = glow.as_ref() {
                let glow_col = glow_opts.colour.as_ref().map(Color::from).unwrap_or_else(|| dim_colour(fg));
                let radius = glow_opts.radius.max(1) as i32;
                let glow_content = strip_markup(&rendered_content);
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx == 0 && dy == 0 { continue; }
                        let gx = (draw_x as i32 + dx).max(0) as u16;
                        let gy = (draw_y as i32 + dy).max(0) as u16;
                        render_text_content(mod_source, &glow_content, resolved_font.as_deref(), glow_col, sprite_bg, gx, gy, ctx.layer_buf);
                    }
                }
            }
            render_text_content(mod_source, &rendered_content, resolved_font.as_deref(), fg, sprite_bg, draw_x, draw_y, ctx.layer_buf);
            let sprite_region = Region { x: draw_x, y: draw_y, width: sprite_width, height: sprite_height };
            finalize_sprite(object_id, sprite_region, sprite_elapsed, sprite.stages(), ctx, target_resolver, object_regions);
        }

        Sprite::Image { source, x, y, size, width, height, force_renderer_mode, align_x, align_y, .. } => {
            let resolved_mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let (sprite_width, sprite_height) =
                image_sprite_dimensions(source, *width, *height, *size, resolved_mode, ctx.asset_root);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(base_x, base_y, sprite.animations(), sprite_elapsed, &object_state);
            render_image_content(source, *width, *height, *size, resolved_mode, ctx.asset_root, draw_x, draw_y, ctx.layer_buf);
            let sprite_region = Region { x: draw_x, y: draw_y, width: sprite_width, height: sprite_height };
            finalize_sprite(object_id, sprite_region, sprite_elapsed, sprite.stages(), ctx, target_resolver, object_regions);
        }

        Sprite::Grid { x, y, width, height, gap_x, gap_y, force_renderer_mode,
                        align_x, align_y, columns, rows, children, .. } =>
        {
            let resolved_mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let container_w = width.unwrap_or(area.width).max(1);
            let container_h = height.unwrap_or(area.height).max(1);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let (dx, dy) = sprite_transform_offset(sprite.animations(), sprite_elapsed);
            let draw_x = base_x.saturating_add(dx).saturating_add(object_state.offset_x);
            let draw_y = base_y.saturating_add(dy).saturating_add(object_state.offset_y);

            let child_cells = compute_grid_cells(
                columns, rows, children, container_w, container_h, *gap_x, *gap_y, resolved_mode, ctx.asset_root,
            );
            // Extend path once; update the last element per child instead of allocating per child.
            let base_path_len = sprite_path.len();
            sprite_path.push(0);
            for (idx, rect) in child_cells {
                let Some(child) = children.get(idx) else { continue; };
                *sprite_path.last_mut().unwrap() = idx;
                let child_area = RenderArea {
                    origin_x: draw_x + rect.x as i32,
                    origin_y: draw_y + rect.y as i32,
                    width: rect.width.max(1),
                    height: rect.height.max(1),
                };
                render_sprite(layer_idx, sprite_path, child, child_area, resolved_mode, target_resolver, object_regions, object_states, ctx);
            }
            sprite_path.truncate(base_path_len);

            let sprite_region = Region { x: draw_x.max(0) as u16, y: draw_y.max(0) as u16, width: container_w, height: container_h };
            finalize_sprite(object_id, sprite_region, sprite_elapsed, sprite.stages(), ctx, target_resolver, object_regions);
        }

        Sprite::Flex { x, y, width, height, gap, direction, force_renderer_mode,
                        align_x, align_y, children, .. } =>
        {
            let resolved_mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let container_w = width.unwrap_or(area.width).max(1);
            let container_h = height.unwrap_or(area.height).max(1);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let (dx, dy) = sprite_transform_offset(sprite.animations(), sprite_elapsed);
            let draw_x = base_x.saturating_add(dx).saturating_add(object_state.offset_x);
            let draw_y = base_y.saturating_add(dy).saturating_add(object_state.offset_y);

            let child_cells = compute_flex_cells(children, *direction, container_w, container_h, *gap, resolved_mode, ctx.asset_root);
            let base_path_len = sprite_path.len();
            sprite_path.push(0);
            for (idx, rect) in child_cells {
                let Some(child) = children.get(idx) else { continue; };
                *sprite_path.last_mut().unwrap() = idx;
                let child_area = RenderArea {
                    origin_x: draw_x + rect.x as i32,
                    origin_y: draw_y + rect.y as i32,
                    width: rect.width.max(1),
                    height: rect.height.max(1),
                };
                render_sprite(layer_idx, sprite_path, child, child_area, resolved_mode, target_resolver, object_regions, object_states, ctx);
            }
            sprite_path.truncate(base_path_len);

            let sprite_region = Region { x: draw_x.max(0) as u16, y: draw_y.max(0) as u16, width: container_w, height: container_h };
            finalize_sprite(object_id, sprite_region, sprite_elapsed, sprite.stages(), ctx, target_resolver, object_regions);
        }

        Sprite::Obj { id, source, x, y, size, width, height, force_renderer_mode,
                       surface_mode, scale, yaw_deg, pitch_deg, roll_deg,
                       rotation_x, rotation_y, rotation_z, rotate_y_deg_per_sec,
                       camera_distance, fov_degrees, near_clip, draw_char,
                       align_x, align_y, fg_colour, bg_colour, .. } =>
        {
            let resolved_mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
                obj_sprite_dimensions(*width, *height, *size)
            } else {
                (area.width.max(1), area.height.max(1))
            };
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(base_x, base_y, sprite.animations(), sprite_elapsed, &object_state);

            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let draw_glyph = draw_char.as_deref().and_then(|s| s.chars().next()).unwrap_or('#');
            // Avoid allocating a lowercase String by using eq_ignore_ascii_case.
            let is_wireframe = surface_mode.as_deref().map(|s| s.trim().eq_ignore_ascii_case("wireframe")).unwrap_or(false);
            let camera_state = id.as_deref().and_then(|sid| ctx.obj_camera_states.get(sid)).copied().unwrap_or_default();

            render_obj_content(
                source, Some(sprite_width), Some(sprite_height), *size, resolved_mode,
                ObjRenderParams {
                    scale: scale.unwrap_or(1.0),
                    yaw_deg: yaw_deg.unwrap_or(0.0),
                    pitch_deg: pitch_deg.unwrap_or(0.0),
                    roll_deg: roll_deg.unwrap_or(0.0),
                    rotation_x: rotation_x.unwrap_or(0.0),
                    rotation_y: rotation_y.unwrap_or(0.0),
                    rotation_z: rotation_z.unwrap_or(0.0),
                    rotate_y_deg_per_sec: rotate_y_deg_per_sec.unwrap_or(20.0),
                    camera_distance: camera_distance.unwrap_or(3.0),
                    fov_degrees: fov_degrees.unwrap_or(60.0),
                    near_clip: near_clip.unwrap_or(0.001),
                    scene_elapsed_ms: sprite_elapsed,
                    camera_pan_x: camera_state.pan_x, camera_pan_y: camera_state.pan_y,
                    camera_look_yaw: camera_state.look_yaw, camera_look_pitch: camera_state.look_pitch,
                },
                is_wireframe, draw_glyph, fg, bg, ctx.asset_root, draw_x, draw_y, ctx.layer_buf,
            );
            let sprite_region = Region { x: draw_x, y: draw_y, width: sprite_width, height: sprite_height };
            finalize_sprite(object_id, sprite_region, sprite_elapsed, sprite.stages(), ctx, target_resolver, object_regions);
        }
    }
}

// Returns `Some(appear_at)` when the sprite should be rendered, `None` to skip.
fn check_visibility(
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

fn sprite_transform_offset(animations: &[crate::scene::Animation], elapsed_ms: u64) -> (i32, i32) {
    ANIM_DISPATCHER.with(|d| {
        let t = d.compute_transform(animations, elapsed_ms);
        (t.dx as i32, t.dy as i32)
    })
}

fn compute_draw_pos(
    base_x: i32,
    base_y: i32,
    animations: &[crate::scene::Animation],
    sprite_elapsed: u64,
    object_state: &ObjectRuntimeState,
) -> (u16, u16) {
    let (dx, dy) = sprite_transform_offset(animations, sprite_elapsed);
    let draw_x = base_x.saturating_add(dx).saturating_add(object_state.offset_x).max(0) as u16;
    let draw_y = base_y.saturating_add(dy).saturating_add(object_state.offset_y).max(0) as u16;
    (draw_x, draw_y)
}

fn finalize_sprite(
    object_id: Option<&str>,
    sprite_region: Region,
    sprite_elapsed: u64,
    stages: &LayerStages,
    ctx: &mut RenderCtx<'_>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut BTreeMap<String, Region>,
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
