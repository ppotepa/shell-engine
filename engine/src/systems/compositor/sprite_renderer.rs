use crossterm::style::Color;
use std::borrow::Cow;
use std::sync::Arc;

use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::markup::strip_markup;
use crate::render_policy;
use crate::scene::{Layer, SceneRenderedMode, Sprite};
use crate::scene_runtime::{ObjCameraState, ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// OPT-7: Thread-local cache for pre-rendered text glow buffers.
// Stores Arc<Buffer> so cache hits are a refcount increment, not a full Buffer clone.
type GlowCacheKey = u64;

thread_local! {
    static GLOW_CACHE: RefCell<HashMap<GlowCacheKey, Arc<Buffer>>> =
        RefCell::new(HashMap::new());
}

/// Hash a `crossterm::style::Color` without allocating (avoids `format!("{:?}", col)`).
#[inline(always)]
fn hash_color<H: Hasher>(col: Color, h: &mut H) {
    match col {
        Color::Reset        => 0u8.hash(h),
        Color::Black        => 1u8.hash(h),
        Color::DarkGrey     => 2u8.hash(h),
        Color::Red          => 3u8.hash(h),
        Color::DarkRed      => 4u8.hash(h),
        Color::Green        => 5u8.hash(h),
        Color::DarkGreen    => 6u8.hash(h),
        Color::Yellow       => 7u8.hash(h),
        Color::DarkYellow   => 8u8.hash(h),
        Color::Blue         => 9u8.hash(h),
        Color::DarkBlue     => 10u8.hash(h),
        Color::Magenta      => 11u8.hash(h),
        Color::DarkMagenta  => 12u8.hash(h),
        Color::Cyan         => 13u8.hash(h),
        Color::DarkCyan     => 14u8.hash(h),
        Color::White        => 15u8.hash(h),
        Color::Grey         => 16u8.hash(h),
        Color::Rgb { r, g, b } => { 17u8.hash(h); r.hash(h); g.hash(h); b.hash(h); }
        Color::AnsiValue(v) => { 18u8.hash(h); v.hash(h); }
    }
}

fn glow_cache_key(
    content: &str,
    radius: i32,
    glow_col: Color,
    font: Option<&str>,
    sprite_bg: Color,
    sprite_w: u16,
    sprite_h: u16,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut h);
    radius.hash(&mut h);
    hash_color(glow_col, &mut h);
    font.hash(&mut h);
    hash_color(sprite_bg, &mut h);
    sprite_w.hash(&mut h);
    sprite_h.hash(&mut h);
    h.finish()
}

use super::image_render::{image_sprite_dimensions, render_image_content};
use super::layout::{
    compute_flex_cells, compute_grid_cells, measure_sprite_for_layout, resolve_x, resolve_y,
    RenderArea,
};
use super::obj_render::{obj_sprite_dimensions, try_blit_prerendered, render_obj_content, ObjRenderParams};
use super::render::{
    check_visibility, compute_draw_pos, finalize_sprite, render_children_in_cells,
    sprite_transform_offset, RenderCtx,
};
use super::text_render::{dim_colour, render_text_content, text_sprite_dimensions, ClipRect};

/// Render all sprites in a layer onto `layer_buf`.
pub fn render_sprites(
    layer_idx: usize,
    layer: &Layer,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    root_origin_x: i32,
    root_origin_y: i32,
    object_states: &HashMap<String, ObjectRuntimeState>,
    scene_elapsed_ms: u64,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    obj_camera_states: &HashMap<String, ObjCameraState>,
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
            None,
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
    clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_states: &HashMap<String, ObjectRuntimeState>,
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
        Sprite::Text {
            content,
            x,
            y,
            size,
            font,
            force_renderer_mode,
            force_font_mode,
            align_x,
            align_y,
            fg_colour,
            bg_colour,
            reveal_ms,
            glow,
            text_transform,
            ..
        } => {
            let total_chars = content.chars().count();
            // Build the visible slice without allocating: for the reveal case walk to
            // the char boundary and borrow; for the full case borrow directly.
            let rendered_content: Cow<'_, str> = match reveal_ms {
                Some(reveal) if *reveal > 0 => {
                    let since = ctx.scene_elapsed_ms - appear_at;
                    let p = (since as f32 / *reveal as f32).clamp(0.0, 1.0);
                    let visible_chars = ((total_chars as f32) * p).ceil() as usize;
                    let byte_end = content
                        .char_indices()
                        .nth(visible_chars)
                        .map(|(i, _)| i)
                        .unwrap_or(content.len());
                    Cow::Borrowed(&content[..byte_end])
                }
                _ => Cow::Borrowed(content.as_str()),
            };
            if rendered_content.is_empty() {
                return;
            }

            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let sprite_bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let resolved_font = render_policy::resolve_text_font_spec(
                font.as_deref(),
                force_font_mode.as_deref(),
                *size,
                inherited_mode,
                *force_renderer_mode,
            );
            let mod_source = ctx.asset_root.map(|root| root.mod_source());
            let (sprite_width, sprite_height) = text_sprite_dimensions(
                mod_source,
                &*rendered_content,
                resolved_font.as_deref(),
                fg,
                sprite_bg,
            );

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(
                base_x,
                base_y,
                sprite.animations(),
                sprite_elapsed,
                &object_state,
            );
            let clip = clip_rect;

            if let Some(glow_opts) = glow.as_ref() {
                let glow_col = glow_opts
                    .colour
                    .as_ref()
                    .map(Color::from)
                    .unwrap_or_else(|| dim_colour(fg));
                let radius = glow_opts.radius.max(1) as i32;
                let glow_content = strip_markup(&*rendered_content);
                let glow_key = glow_cache_key(
                    &glow_content, radius, glow_col,
                    resolved_font.as_deref(), sprite_bg,
                    sprite_width, sprite_height,
                );
                // OPT-7: Pre-render all glow offsets into a cached scratch buffer.
                // Cache stores Arc<Buffer>: hit = refcount increment, miss = one Arc wrap.
                let glow_buf: Arc<Buffer> = GLOW_CACHE.with(|cache| {
                    if let Some(cached) = cache.borrow().get(&glow_key) {
                        return Arc::clone(cached);
                    }
                    let pad = radius as u16;
                    let bw = sprite_width.saturating_add(pad * 2).max(1);
                    let bh = sprite_height.saturating_add(pad * 2).max(1);
                    let mut scratch = Buffer::new(bw, bh);
                    scratch.fill(Color::Reset);
                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            if dx == 0 && dy == 0 { continue; }
                            let gx = (pad as i32 + dx).max(0) as u16;
                            let gy = (pad as i32 + dy).max(0) as u16;
                            render_text_content(
                                mod_source, &glow_content,
                                resolved_font.as_deref(), glow_col, sprite_bg,
                                gx, gy, None, &mut scratch, text_transform,
                            );
                        }
                    }
                    let arc = Arc::new(scratch);
                    let mut c = cache.borrow_mut();
                    if c.len() >= 128 { c.clear(); }
                    c.insert(glow_key, Arc::clone(&arc));
                    arc
                });
                // Blit cached glow onto layer_buf at the correct position.
                let pad = radius as u16;
                let blit_x = (draw_x as i32 - pad as i32).max(0) as u16;
                let blit_y = (draw_y as i32 - pad as i32).max(0) as u16;
                for gy in 0..glow_buf.height {
                    for gx in 0..glow_buf.width {
                        if let Some(cell) = glow_buf.get(gx, gy) {
                            if cell.symbol == ' ' && matches!(cell.bg, Color::Reset) {
                                continue; // transparent
                            }
                            let tx = blit_x + gx;
                            let ty = blit_y + gy;
                            if let Some(cr) = clip {
                                let tx_i = tx as i32;
                                let ty_i = ty as i32;
                                if tx_i < cr.x || tx_i >= cr.x + cr.width as i32
                                    || ty_i < cr.y || ty_i >= cr.y + cr.height as i32 {
                                    continue;
                                }
                            }
                            ctx.layer_buf.set(tx, ty, cell.symbol, cell.fg, cell.bg);
                        }
                    }
                }
            }
            render_text_content(
                mod_source,
                &*rendered_content,
                resolved_font.as_deref(),
                fg,
                sprite_bg,
                draw_x,
                draw_y,
                clip,
                ctx.layer_buf,
                text_transform,
            );
            let sprite_region = Region {
                x: draw_x,
                y: draw_y,
                width: sprite_width,
                height: sprite_height,
            };
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
            );
        }

        Sprite::Image {
            source,
            spritesheet_columns,
            spritesheet_rows,
            frame_index,
            x,
            y,
            size,
            width,
            height,
            stretch_to_area,
            force_renderer_mode,
            align_x,
            align_y,
            ..
        } => {
            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let target_width = if *stretch_to_area {
                Some(area.width.max(1))
            } else {
                *width
            };
            let target_height = if *stretch_to_area {
                Some(area.height.max(1))
            } else {
                *height
            };
            let target_size = if *stretch_to_area { None } else { *size };
            let (sprite_width, sprite_height) = image_sprite_dimensions(
                source,
                target_width,
                target_height,
                target_size,
                *spritesheet_columns,
                *spritesheet_rows,
                *frame_index,
                resolved_mode,
                ctx.asset_root,
            );
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(
                base_x,
                base_y,
                sprite.animations(),
                sprite_elapsed,
                &object_state,
            );
            render_image_content(
                source,
                target_width,
                target_height,
                target_size,
                *spritesheet_columns,
                *spritesheet_rows,
                *frame_index,
                resolved_mode,
                sprite_elapsed,
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
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
            );
        }

        Sprite::Panel {
            x,
            y,
            width,
            width_percent,
            height,
            padding,
            border_width,
            corner_radius,
            shadow_x,
            shadow_y,
            force_renderer_mode,
            align_x,
            align_y,
            bg_colour,
            border_colour,
            shadow_colour,
            children,
            ..
        } => {
            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let (auto_w, auto_h) = measure_sprite_for_layout(sprite, resolved_mode, ctx.asset_root);
            let container_w = if let Some(explicit) = *width {
                explicit
            } else if let Some(percent) = *width_percent {
                let p = percent.clamp(1, 100) as u32;
                ((u32::from(area.width).saturating_mul(p)) / 100).max(1) as u16
            } else {
                auto_w.min(area.width)
            }
            .max(3);
            let container_h = height.unwrap_or(auto_h.min(area.height)).max(3);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let (dx, dy) = sprite_transform_offset(sprite.animations(), sprite_elapsed);
            let draw_x = base_x
                .saturating_add(dx)
                .saturating_add(object_state.offset_x);
            let draw_y = base_y
                .saturating_add(dy)
                .saturating_add(object_state.offset_y);

            let panel_bg = bg_colour
                .as_ref()
                .map(Color::from)
                .unwrap_or(Color::DarkGrey);
            let panel_border = border_colour
                .as_ref()
                .map(Color::from)
                .unwrap_or(Color::Rgb {
                    r: 38,
                    g: 38,
                    b: 38,
                });
            let panel_shadow = shadow_colour
                .as_ref()
                .map(Color::from)
                .unwrap_or(Color::Rgb {
                    r: 20,
                    g: 20,
                    b: 20,
                });
            render_panel_box(
                ctx.layer_buf,
                draw_x,
                draw_y,
                container_w,
                container_h,
                *border_width,
                *corner_radius,
                panel_bg,
                panel_border,
                panel_shadow,
                *shadow_x,
                *shadow_y,
            );

            let inset = (border_width.saturating_add(*padding)) as i32;
            let inner_w = container_w.saturating_sub((inset.saturating_mul(2)).max(0) as u16);
            let inner_h = container_h.saturating_sub((inset.saturating_mul(2)).max(0) as u16);
            let inner_area = RenderArea {
                origin_x: draw_x.saturating_add(inset),
                origin_y: draw_y.saturating_add(inset),
                width: inner_w.max(1),
                height: inner_h.max(1),
            };
            let panel_inner_clip = Some(ClipRect {
                x: inner_area.origin_x,
                y: inner_area.origin_y,
                width: inner_area.width,
                height: inner_area.height,
            });
            let child_clip = intersect_clip_rect(clip_rect, panel_inner_clip);
            for (child_idx, child) in children.iter().enumerate() {
                sprite_path.push(child_idx);
                render_sprite(
                    layer_idx,
                    sprite_path,
                    child,
                    inner_area,
                    resolved_mode,
                    child_clip,
                    target_resolver,
                    object_regions,
                    object_states,
                    ctx,
                );
                sprite_path.pop();
            }

            let sprite_region = Region {
                x: draw_x.max(0) as u16,
                y: draw_y.max(0) as u16,
                width: container_w,
                height: container_h,
            };
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
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
            columns,
            rows,
            children,
            ..
        } => {
            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let container_w = width.unwrap_or(area.width).max(1);
            let container_h = height.unwrap_or(area.height).max(1);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let (dx, dy) = sprite_transform_offset(sprite.animations(), sprite_elapsed);
            let draw_x = base_x
                .saturating_add(dx)
                .saturating_add(object_state.offset_x);
            let draw_y = base_y
                .saturating_add(dy)
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
            render_children_in_cells(
                layer_idx,
                sprite_path,
                children,
                &child_cells,
                draw_x,
                draw_y,
                resolved_mode,
                clip_rect,
                target_resolver,
                object_regions,
                object_states,
                ctx,
                render_sprite,
            );

            let sprite_region = Region {
                x: draw_x.max(0) as u16,
                y: draw_y.max(0) as u16,
                width: container_w,
                height: container_h,
            };
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
            );
        }

        Sprite::Flex {
            x,
            y,
            width,
            height,
            gap,
            direction,
            force_renderer_mode,
            align_x,
            align_y,
            children,
            ..
        } => {
            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let container_w = width.unwrap_or(area.width).max(1);
            let container_h = height.unwrap_or(area.height).max(1);
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, container_w);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, container_h);
            let (dx, dy) = sprite_transform_offset(sprite.animations(), sprite_elapsed);
            let draw_x = base_x
                .saturating_add(dx)
                .saturating_add(object_state.offset_x);
            let draw_y = base_y
                .saturating_add(dy)
                .saturating_add(object_state.offset_y);

            let child_cells = compute_flex_cells(
                children,
                *direction,
                container_w,
                container_h,
                *gap,
                resolved_mode,
                ctx.asset_root,
            );
            render_children_in_cells(
                layer_idx,
                sprite_path,
                children,
                &child_cells,
                draw_x,
                draw_y,
                resolved_mode,
                clip_rect,
                target_resolver,
                object_regions,
                object_states,
                ctx,
                render_sprite,
            );

            let sprite_region = Region {
                x: draw_x.max(0) as u16,
                y: draw_y.max(0) as u16,
                width: container_w,
                height: container_h,
            };
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
            );
        }

        Sprite::Obj {
            id,
            source,
            x,
            y,
            size,
            width,
            height,
            force_renderer_mode,
            surface_mode,
            backface_cull,
            clip_y_min,
            clip_y_max,
            scale,
            yaw_deg,
            pitch_deg,
            roll_deg,
            rotation_x,
            rotation_y,
            rotation_z,
            rotate_y_deg_per_sec,
            camera_distance,
            fov_degrees,
            near_clip,
            light_direction_x,
            light_direction_y,
            light_direction_z,
            light_2_direction_x,
            light_2_direction_y,
            light_2_direction_z,
            light_2_intensity,
            light_point_x,
            light_point_y,
            light_point_z,
            light_point_intensity,
            light_point_colour,
            light_point_flicker_depth,
            light_point_flicker_hz,
            light_point_orbit_hz,
            light_point_snap_hz,
            light_point_2_x,
            light_point_2_y,
            light_point_2_z,
            light_point_2_intensity,
            light_point_2_colour,
            light_point_2_flicker_depth,
            light_point_2_flicker_hz,
            light_point_2_orbit_hz,
            light_point_2_snap_hz,
            cel_levels,
            shadow_colour,
            midtone_colour,
            highlight_colour,
            tone_mix,
            draw_char,
            align_x,
            align_y,
            fg_colour,
            bg_colour,
            ..
        } => {
            let resolved_mode =
                render_policy::resolve_renderer_mode(inherited_mode, *force_renderer_mode);
            let (sprite_width, sprite_height) =
                if width.is_some() || height.is_some() || size.is_some() {
                    obj_sprite_dimensions(*width, *height, *size)
                } else {
                    (area.width.max(1), area.height.max(1))
                };
            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let (draw_x, draw_y) = compute_draw_pos(
                base_x,
                base_y,
                sprite.animations(),
                sprite_elapsed,
                &object_state,
            );

            let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
            let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
            let draw_glyph = draw_char
                .as_deref()
                .and_then(|s| s.chars().next())
                .unwrap_or('#');
            // Avoid allocating a lowercase String by using eq_ignore_ascii_case.
            let is_wireframe = surface_mode
                .as_deref()
                .map(|s| s.trim().eq_ignore_ascii_case("wireframe"))
                .unwrap_or(false);
            let camera_state = id
                .as_deref()
                .and_then(|sid| ctx.obj_camera_states.get(sid))
                .copied()
                .unwrap_or_default();

            // Prerender fast path: check if this sprite has a cached frame.
            let sprite_id_opt = id.as_deref();
            let current_total_yaw = rotation_y.unwrap_or(0.0) + yaw_deg.unwrap_or(0.0);
            let current_pitch = pitch_deg.unwrap_or(0.0);
            let clip_min = clip_y_min.unwrap_or(0.0);
            let clip_max = clip_y_max.unwrap_or(1.0);
            if let Some(sid) = sprite_id_opt {
                if try_blit_prerendered(
                    sid,
                    current_total_yaw,
                    current_pitch,
                    clip_min,
                    clip_max,
                    resolved_mode,
                    draw_x,
                    draw_y,
                    ctx.layer_buf,
                ) {
                    let sprite_region = Region {
                        x: draw_x,
                        y: draw_y,
                        width: sprite_width,
                        height: sprite_height,
                    };
                    finalize_sprite(
                        object_id,
                        sprite_region,
                        sprite_elapsed,
                        sprite.stages(),
                        ctx,
                        target_resolver,
                        object_regions,
                    );
                    return;
                }
            }

            render_obj_content(
                source,
                Some(sprite_width),
                Some(sprite_height),
                *size,
                resolved_mode,
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
                    light_direction_x: light_direction_x.unwrap_or(-0.45),
                    light_direction_y: light_direction_y.unwrap_or(0.70),
                    light_direction_z: light_direction_z.unwrap_or(-0.85),
                    light_2_direction_x: light_2_direction_x.unwrap_or(0.0),
                    light_2_direction_y: light_2_direction_y.unwrap_or(0.0),
                    light_2_direction_z: light_2_direction_z.unwrap_or(-1.0),
                    light_2_intensity: light_2_intensity.unwrap_or(0.0),
                    light_point_x: light_point_x.unwrap_or(0.0),
                    light_point_y: light_point_y.unwrap_or(2.0),
                    light_point_z: light_point_z.unwrap_or(0.0),
                    light_point_intensity: light_point_intensity.unwrap_or(0.0),
                    light_point_colour: light_point_colour.as_ref().map(Color::from),
                    light_point_flicker_depth: light_point_flicker_depth.unwrap_or(0.0),
                    light_point_flicker_hz: light_point_flicker_hz.unwrap_or(0.0),
                    light_point_orbit_hz: light_point_orbit_hz.unwrap_or(0.0),
                    light_point_snap_hz: light_point_snap_hz.unwrap_or(0.0),
                    light_point_2_x: light_point_2_x.unwrap_or(0.0),
                    light_point_2_y: light_point_2_y.unwrap_or(0.0),
                    light_point_2_z: light_point_2_z.unwrap_or(0.0),
                    light_point_2_intensity: light_point_2_intensity.unwrap_or(0.0),
                    light_point_2_colour: light_point_2_colour.as_ref().map(Color::from),
                    light_point_2_flicker_depth: light_point_2_flicker_depth.unwrap_or(0.0),
                    light_point_2_flicker_hz: light_point_2_flicker_hz.unwrap_or(0.0),
                    light_point_2_orbit_hz: light_point_2_orbit_hz.unwrap_or(0.0),
                    light_point_2_snap_hz: light_point_2_snap_hz.unwrap_or(0.0),
                    cel_levels: cel_levels.unwrap_or(0),
                    shadow_colour: shadow_colour.as_ref().map(Color::from),
                    midtone_colour: midtone_colour.as_ref().map(Color::from),
                    highlight_colour: highlight_colour.as_ref().map(Color::from),
                    tone_mix: tone_mix.unwrap_or(0.0),
                    scene_elapsed_ms: sprite_elapsed,
                    camera_pan_x: camera_state.pan_x,
                    camera_pan_y: camera_state.pan_y,
                    camera_look_yaw: camera_state.look_yaw,
                    camera_look_pitch: camera_state.look_pitch,
                    object_translate_x: 0.0,
                    object_translate_y: 0.0,
                    object_translate_z: 0.0,
                    clip_y_min: clip_y_min.unwrap_or(0.0),
                    clip_y_max: clip_y_max.unwrap_or(1.0),
                },
                is_wireframe,
                backface_cull.unwrap_or(false),
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
            finalize_sprite(
                object_id,
                sprite_region,
                sprite_elapsed,
                sprite.stages(),
                ctx,
                target_resolver,
                object_regions,
            );
        }

        Sprite::Scene3D { src, frame, x, y, .. } => {
            use crate::scene3d_atlas::Scene3DAtlas;
            use crate::rasterizer::blit;
            let draw_x = area.origin_x.saturating_add(*x).saturating_add(object_state.offset_x).max(0) as u16;
            let draw_y = area.origin_y.saturating_add(*y).saturating_add(object_state.offset_y).max(0) as u16;
            // Look up prerendered buffer from world-scoped atlas via thread-local pointer.
            if let Some(buf) = Scene3DAtlas::current_get(src, frame) {
                blit(&buf, ctx.layer_buf, draw_x, draw_y);
                if let Some(id) = object_id {
                    object_regions.insert(id.to_string(), crate::effects::Region {
                        x: draw_x,
                        y: draw_y,
                        width: buf.width,
                        height: buf.height,
                    });
                }
            }
        }
    }
}

fn render_panel_box(
    buffer: &mut Buffer,
    draw_x: i32,
    draw_y: i32,
    width: u16,
    height: u16,
    border_width: u16,
    corner_radius: u16,
    panel_bg: Color,
    border_color: Color,
    shadow_color: Color,
    shadow_x: i32,
    shadow_y: i32,
) {
    let rounded = corner_radius > 0 && width >= 4 && height >= 4;
    for py in 0..height {
        for px in 0..width {
            if !panel_cell_visible(px, py, width, height, rounded) {
                continue;
            }
            set_panel_cell(
                buffer,
                draw_x.saturating_add(px as i32).saturating_add(shadow_x),
                draw_y.saturating_add(py as i32).saturating_add(shadow_y),
                shadow_color,
            );
        }
    }

    for py in 0..height {
        for px in 0..width {
            if !panel_cell_visible(px, py, width, height, rounded) {
                continue;
            }
            let bw = border_width.min(width / 2).min(height / 2);
            let border = bw > 0
                && (px < bw
                    || py < bw
                    || px >= width.saturating_sub(bw)
                    || py >= height.saturating_sub(bw));
            let color = if border { border_color } else { panel_bg };
            set_panel_cell(
                buffer,
                draw_x.saturating_add(px as i32),
                draw_y.saturating_add(py as i32),
                color,
            );
        }
    }
}

fn intersect_clip_rect(a: Option<ClipRect>, b: Option<ClipRect>) -> Option<ClipRect> {
    match (a, b) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => {
            let left = a.x.max(b.x);
            let top = a.y.max(b.y);
            let right = (a.x + i32::from(a.width)).min(b.x + i32::from(b.width));
            let bottom = (a.y + i32::from(a.height)).min(b.y + i32::from(b.height));
            if right <= left || bottom <= top {
                return None;
            }
            Some(ClipRect {
                x: left,
                y: top,
                width: (right - left) as u16,
                height: (bottom - top) as u16,
            })
        }
    }
}

#[inline(always)]
fn panel_cell_visible(x: u16, y: u16, width: u16, height: u16, rounded: bool) -> bool {
    if !rounded {
        return true;
    }
    !(x == 0 && y == 0
        || x == width.saturating_sub(1) && y == 0
        || x == 0 && y == height.saturating_sub(1)
        || x == width.saturating_sub(1) && y == height.saturating_sub(1))
}

#[inline(always)]
fn set_panel_cell(buffer: &mut Buffer, x: i32, y: i32, bg: Color) {
    if x < 0 || y < 0 {
        return;
    }
    buffer.set(x as u16, y as u16, ' ', Color::Reset, bg);
}
