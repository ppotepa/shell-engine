use crossterm::style::Color;

use crate::animations::AnimationDispatcher;
use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::markup::strip_markup;
use crate::render_policy;
use crate::scene::{HorizontalAlign, Layer, SceneRenderedMode, Sprite, VerticalAlign};
use crate::scene_runtime::{ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;
use std::collections::BTreeMap;

use super::effect_applicator::apply_sprite_effects;
use super::grid_tracks::{
    parse_track_spec, resolve_track_sizes, span_size, track_start, TrackSpec,
};
use super::image_render::{image_sprite_dimensions, render_image_content};
use super::obj_render::{obj_sprite_dimensions, render_obj_content, ObjRenderParams};
use super::text_render::{dim_colour, render_text_content, text_sprite_dimensions};

#[derive(Clone, Copy)]
struct RenderArea {
    origin_x: i32,
    origin_y: i32,
    width: u16,
    height: u16,
}

struct RenderCtx<'a> {
    asset_root: Option<&'a AssetRoot>,
    scene_elapsed_ms: u64,
    current_stage: &'a SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    layer_buf: &'a mut Buffer,
}

#[derive(Clone, Copy)]
struct GridCellRect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
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
    layer_buf: &mut Buffer,
) {
    let mut ctx = RenderCtx {
        asset_root,
        scene_elapsed_ms,
        current_stage,
        step_idx,
        elapsed_ms,
        layer_buf,
    };

    let root_area = RenderArea {
        origin_x: root_origin_x,
        origin_y: root_origin_y,
        width: scene_w,
        height: scene_h,
    };

    for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
        render_sprite(
            layer_idx,
            &[sprite_idx],
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
    sprite_path: &[usize],
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

    match sprite {
        Sprite::Text {
            content,
            x,
            y,
            font,
            force_renderer_mode,
            force_font_mode,
            align_x,
            align_y,
            fg_colour,
            bg_colour,
            appear_at_ms,
            disappear_at_ms,
            reveal_ms,
            hide_on_leave,
            stages,
            animations,
            glow,
            ..
        } => {
            if *hide_on_leave && matches!(ctx.current_stage, SceneStage::OnLeave) {
                return;
            }
            let appear_at = appear_at_ms.unwrap_or(0);
            if ctx.scene_elapsed_ms < appear_at {
                return;
            }
            if let Some(disappear_at) = disappear_at_ms {
                if ctx.scene_elapsed_ms >= *disappear_at {
                    return;
                }
            }

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
            if rendered_content.is_empty() {
                return;
            }

            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let sprite_bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);

            let resolved_font = render_policy::resolve_font_spec(
                font.as_deref(),
                force_font_mode.as_deref(),
                inherited_mode,
                *force_renderer_mode,
            );
            let mod_source = ctx.asset_root.map(|root| root.mod_source());

            let (sprite_width, sprite_height) = text_sprite_dimensions(
                mod_source,
                &rendered_content,
                resolved_font.as_deref(),
                fg,
                sprite_bg,
            );

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);
            let anim_dispatcher = AnimationDispatcher::new();
            let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
            let draw_x = base_x
                .saturating_add(transform.dx as i32)
                .saturating_add(object_state.offset_x)
                .max(0) as u16;
            let draw_y = base_y
                .saturating_add(transform.dy as i32)
                .saturating_add(object_state.offset_y)
                .max(0) as u16;

            if let Some(glow_opts) = glow.as_ref() {
                let glow_col = glow_opts
                    .colour
                    .as_ref()
                    .map(Color::from)
                    .unwrap_or_else(|| dim_colour(fg));
                let radius = glow_opts.radius.max(1) as i32;
                let glow_content = strip_markup(&rendered_content);
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let gx = (draw_x as i32 + dx).max(0) as u16;
                        let gy = (draw_y as i32 + dy).max(0) as u16;
                        render_text_content(
                            mod_source,
                            &glow_content,
                            resolved_font.as_deref(),
                            glow_col,
                            sprite_bg,
                            gx,
                            gy,
                            ctx.layer_buf,
                        );
                    }
                }
            }

            render_text_content(
                mod_source,
                &rendered_content,
                resolved_font.as_deref(),
                fg,
                sprite_bg,
                draw_x,
                draw_y,
                ctx.layer_buf,
            );

            let sprite_region = Region {
                x: draw_x,
                y: draw_y,
                width: sprite_width,
                height: sprite_height,
            };
            if let Some(object_id) = object_id {
                object_regions.insert(object_id.to_string(), sprite_region);
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
        Sprite::Image {
            source,
            x,
            y,
            width,
            height,
            force_renderer_mode,
            align_x,
            align_y,
            appear_at_ms,
            disappear_at_ms,
            hide_on_leave,
            stages,
            animations,
            ..
        } => {
            if *hide_on_leave && matches!(ctx.current_stage, SceneStage::OnLeave) {
                return;
            }
            let appear_at = appear_at_ms.unwrap_or(0);
            if ctx.scene_elapsed_ms < appear_at {
                return;
            }
            if let Some(disappear_at) = disappear_at_ms {
                if ctx.scene_elapsed_ms >= *disappear_at {
                    return;
                }
            }

            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let (sprite_width, sprite_height) =
                image_sprite_dimensions(source, *width, *height, resolved_mode, ctx.asset_root);

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);
            let anim_dispatcher = AnimationDispatcher::new();
            let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
            let draw_x = base_x
                .saturating_add(transform.dx as i32)
                .saturating_add(object_state.offset_x)
                .max(0) as u16;
            let draw_y = base_y
                .saturating_add(transform.dy as i32)
                .saturating_add(object_state.offset_y)
                .max(0) as u16;

            render_image_content(
                source,
                *width,
                *height,
                resolved_mode,
                ctx.asset_root,
                draw_x,
                draw_y,
                ctx.layer_buf,
            );

            let sprite_region = Region {
                x: draw_x,
                y: draw_y,
                width: sprite_width,
                height: sprite_height,
            };
            if let Some(object_id) = object_id {
                object_regions.insert(object_id.to_string(), sprite_region);
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
        Sprite::Grid {
            x,
            y,
            width,
            height,
            gap_x,
            gap_y,
            force_renderer_mode,
            align_x,
            align_y,
            appear_at_ms,
            disappear_at_ms,
            hide_on_leave,
            stages,
            animations,
            columns,
            rows,
            children,
            ..
        } => {
            if *hide_on_leave && matches!(ctx.current_stage, SceneStage::OnLeave) {
                return;
            }
            let appear_at = appear_at_ms.unwrap_or(0);
            if ctx.scene_elapsed_ms < appear_at {
                return;
            }
            if let Some(disappear_at) = disappear_at_ms {
                if ctx.scene_elapsed_ms >= *disappear_at {
                    return;
                }
            }

            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let container_w = width.unwrap_or(area.width).max(1);
            let container_h = height.unwrap_or(area.height).max(1);

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);
            let anim_dispatcher = AnimationDispatcher::new();
            let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
            let draw_x = base_x
                .saturating_add(transform.dx as i32)
                .saturating_add(object_state.offset_x);
            let draw_y = base_y
                .saturating_add(transform.dy as i32)
                .saturating_add(object_state.offset_y);

            let child_cells = compute_grid_cells(
                columns,
                rows,
                children,
                container_w,
                container_h,
                *gap_x,
                *gap_y,
                resolved_mode,
                ctx.asset_root,
            );

            for (idx, rect) in child_cells {
                let Some(child) = children.get(idx) else {
                    continue;
                };
                let mut child_path = sprite_path.to_vec();
                child_path.push(idx);
                let child_area = RenderArea {
                    origin_x: draw_x + rect.x as i32,
                    origin_y: draw_y + rect.y as i32,
                    width: rect.width.max(1),
                    height: rect.height.max(1),
                };
                render_sprite(
                    layer_idx,
                    &child_path,
                    child,
                    child_area,
                    resolved_mode,
                    target_resolver,
                    object_regions,
                    object_states,
                    ctx,
                );
            }

            let sprite_region = Region {
                x: draw_x.max(0) as u16,
                y: draw_y.max(0) as u16,
                width: container_w,
                height: container_h,
            };
            if let Some(object_id) = object_id {
                object_regions.insert(object_id.to_string(), sprite_region);
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
        Sprite::Obj {
            source,
            x,
            y,
            width,
            height,
            scale,
            yaw_deg,
            pitch_deg,
            roll_deg,
            rotate_y_deg_per_sec,
            camera_distance,
            fov_degrees,
            draw_char,
            align_x,
            align_y,
            fg_colour,
            bg_colour,
            appear_at_ms,
            disappear_at_ms,
            hide_on_leave,
            stages,
            animations,
            ..
        } => {
            if *hide_on_leave && matches!(ctx.current_stage, SceneStage::OnLeave) {
                return;
            }
            let appear_at = appear_at_ms.unwrap_or(0);
            if ctx.scene_elapsed_ms < appear_at {
                return;
            }
            if let Some(disappear_at) = disappear_at_ms {
                if ctx.scene_elapsed_ms >= *disappear_at {
                    return;
                }
            }

            let (sprite_width, sprite_height) = obj_sprite_dimensions(*width, *height);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);
            let anim_dispatcher = AnimationDispatcher::new();
            let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
            let draw_x = base_x
                .saturating_add(transform.dx as i32)
                .saturating_add(object_state.offset_x)
                .max(0) as u16;
            let draw_y = base_y
                .saturating_add(transform.dy as i32)
                .saturating_add(object_state.offset_y)
                .max(0) as u16;
            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let draw_glyph = draw_char
                .as_deref()
                .and_then(|s| s.chars().next())
                .unwrap_or('#');
            render_obj_content(
                source,
                *width,
                *height,
                ObjRenderParams {
                    scale: scale.unwrap_or(1.0),
                    yaw_deg: yaw_deg.unwrap_or(0.0),
                    pitch_deg: pitch_deg.unwrap_or(0.0),
                    roll_deg: roll_deg.unwrap_or(0.0),
                    rotate_y_deg_per_sec: rotate_y_deg_per_sec.unwrap_or(20.0),
                    camera_distance: camera_distance.unwrap_or(3.0),
                    fov_degrees: fov_degrees.unwrap_or(60.0),
                    scene_elapsed_ms: sprite_elapsed,
                },
                draw_glyph,
                fg,
                bg,
                ctx.asset_root,
                draw_x,
                draw_y,
                ctx.layer_buf,
            );

            let sprite_region = Region {
                x: draw_x,
                y: draw_y,
                width: sprite_width,
                height: sprite_height,
            };
            if let Some(object_id) = object_id {
                object_regions.insert(object_id.to_string(), sprite_region);
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
    }
}

fn compute_grid_cells(
    columns: &[String],
    rows: &[String],
    children: &[Sprite],
    container_w: u16,
    container_h: u16,
    gap_x: u16,
    gap_y: u16,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> Vec<(usize, GridCellRect)> {
    let col_specs = if columns.is_empty() {
        vec![TrackSpec::Fr(1)]
    } else {
        columns.iter().map(|c| parse_track_spec(c)).collect()
    };
    let row_specs = if rows.is_empty() {
        vec![TrackSpec::Fr(1)]
    } else {
        rows.iter().map(|r| parse_track_spec(r)).collect()
    };

    let mut col_auto_reqs: Vec<(usize, u16)> = Vec::new();
    let mut row_auto_reqs: Vec<(usize, u16)> = Vec::new();
    let mut placements: Vec<(usize, usize, usize, usize, usize)> = Vec::new();

    for (idx, child) in children.iter().enumerate() {
        let (row, col, row_span, col_span) = child.grid_position();
        let col_idx = (col as usize)
            .saturating_sub(1)
            .min(col_specs.len().saturating_sub(1));
        let row_idx = (row as usize)
            .saturating_sub(1)
            .min(row_specs.len().saturating_sub(1));
        let col_span = col_span as usize;
        let row_span = row_span as usize;
        let col_span_clamped = col_span.max(1).min(col_specs.len().saturating_sub(col_idx));
        let row_span_clamped = row_span.max(1).min(row_specs.len().saturating_sub(row_idx));

        let (pref_w, pref_h) = measure_sprite_for_layout(child, inherited_mode, asset_root);
        if col_span_clamped == 1 {
            col_auto_reqs.push((col_idx, pref_w));
        }
        if row_span_clamped == 1 {
            row_auto_reqs.push((row_idx, pref_h));
        }

        placements.push((idx, col_idx, row_idx, col_span_clamped, row_span_clamped));
    }

    let col_sizes = resolve_track_sizes(&col_specs, container_w, gap_x, &col_auto_reqs);
    let row_sizes = resolve_track_sizes(&row_specs, container_h, gap_y, &row_auto_reqs);

    let mut out = Vec::with_capacity(placements.len());
    for (idx, col_idx, row_idx, col_span, row_span) in placements {
        let x = track_start(&col_sizes, gap_x, col_idx);
        let y = track_start(&row_sizes, gap_y, row_idx);
        let width = span_size(&col_sizes, gap_x, col_idx, col_span).max(1);
        let height = span_size(&row_sizes, gap_y, row_idx, row_span).max(1);
        out.push((
            idx,
            GridCellRect {
                x,
                y,
                width,
                height,
            },
        ));
    }
    out
}

fn measure_sprite_for_layout(
    sprite: &Sprite,
    inherited_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> (u16, u16) {
    match sprite {
        Sprite::Text {
            content,
            font,
            force_renderer_mode,
            force_font_mode,
            fg_colour,
            bg_colour,
            ..
        } => {
            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let resolved_font = render_policy::resolve_font_spec(
                font.as_deref(),
                force_font_mode.as_deref(),
                inherited_mode,
                *force_renderer_mode,
            );
            text_sprite_dimensions(
                asset_root.map(|root| root.mod_source()),
                content,
                resolved_font.as_deref(),
                fg,
                bg,
            )
        }
        Sprite::Image {
            source,
            width,
            height,
            force_renderer_mode,
            ..
        } => {
            let mode = render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            image_sprite_dimensions(source, *width, *height, mode, asset_root)
        }
        Sprite::Grid { width, height, .. } => {
            (width.unwrap_or(1).max(1), height.unwrap_or(1).max(1))
        }
        Sprite::Obj { width, height, .. } => obj_sprite_dimensions(*width, *height),
    }
}

fn resolve_x(offset_x: i32, align_x: &Option<HorizontalAlign>, area_w: u16, sprite_w: u16) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) => 0i32,
        Some(HorizontalAlign::Center) => (area_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => area_w.saturating_sub(sprite_w) as i32,
        None => 0i32,
    };
    origin.saturating_add(offset_x)
}

fn resolve_y(offset_y: i32, align_y: &Option<VerticalAlign>, area_h: u16, sprite_h: u16) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) => 0i32,
        Some(VerticalAlign::Center) => (area_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => area_h.saturating_sub(sprite_h) as i32,
        None => 0i32,
    };
    origin.saturating_add(offset_y)
}
