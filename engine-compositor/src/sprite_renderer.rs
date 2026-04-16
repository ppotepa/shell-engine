use engine_core::color::Color;
use std::borrow::Cow;
use std::sync::Arc;

use crate::obj_render::parse_terrain_params_from_uri;
use engine_animation::SceneStage;
use engine_celestial::{BodyDef, CelestialCatalogs};
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::markup::strip_markup;
use engine_core::scene::{CameraSource, Layer, Sprite};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_render_2d::{
    compute_flex_cells, compute_grid_cells, dim_colour, image_sprite_dimensions,
    intersect_clip_rect, measure_sprite_for_layout, push_vector_primitive,
    render_children_in_cells, render_image_content, render_panel_box, render_text_content,
    resolve_x, resolve_y, text_sprite_dimensions, with_render_context, ClipRect, RenderArea,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// OPT-36: Sprite culling acceleration - skip rendering sprites completely outside viewport

// OPT-7: Thread-local cache for pre-rendered text glow buffers.
// Stores Arc<Buffer> so cache hits are a refcount increment, not a full Buffer clone.
type GlowCacheKey = u64;

thread_local! {
    static GLOW_CACHE: std::cell::RefCell<HashMap<GlowCacheKey, Arc<Buffer>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Hash a `engine_core::color::Color` without allocating (avoids `format!("{:?}", col)`).
#[inline(always)]
fn hash_color<H: Hasher>(col: Color, h: &mut H) {
    match col {
        Color::Reset => 0u8.hash(h),
        Color::Black => 1u8.hash(h),
        Color::DarkGrey => 2u8.hash(h),
        Color::Red => 3u8.hash(h),
        Color::DarkRed => 4u8.hash(h),
        Color::Green => 5u8.hash(h),
        Color::DarkGreen => 6u8.hash(h),
        Color::Yellow => 7u8.hash(h),
        Color::DarkYellow => 8u8.hash(h),
        Color::Blue => 9u8.hash(h),
        Color::DarkBlue => 10u8.hash(h),
        Color::Magenta => 11u8.hash(h),
        Color::DarkMagenta => 12u8.hash(h),
        Color::Cyan => 13u8.hash(h),
        Color::DarkCyan => 14u8.hash(h),
        Color::White => 15u8.hash(h),
        Color::Grey => 16u8.hash(h),
        Color::Rgb { r, g, b } => {
            17u8.hash(h);
            r.hash(h);
            g.hash(h);
            b.hash(h);
        }
    }
}

#[inline(always)]
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

use super::render::{
    check_visibility, compute_draw_pos, finalize_sprite, is_sprite_offscreen,
    sprite_transform_offset, RenderCtx,
};
use crate::{
    blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba, obj_sprite_dimensions,
    render_obj_content, render_obj_to_canvas, render_obj_to_rgba_canvas, try_blit_prerendered,
    ObjRenderParams,
};

/// Render all sprites in a layer onto `layer_buf`.
#[allow(clippy::too_many_arguments)]
pub fn render_sprites(
    layer_idx: usize,
    layer: &Layer,
    scene_w: u16,
    scene_h: u16,
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
    scene_camera_3d: &SceneCamera3D,
    celestial_catalogs: Option<&CelestialCatalogs>,
    is_pixel_backend: bool,
    default_font: Option<&str>,
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
        scene_camera_3d,
        celestial_catalogs,
        is_pixel_backend,
        default_font,
    };
    let root_area = RenderArea {
        origin_x: root_origin_x,
        origin_y: root_origin_y,
        width: scene_w,
        height: scene_h,
    };

    // Fast-path: skip if layer has zero area
    if scene_w == 0 || scene_h == 0 {
        return;
    }

    // Reuse one path Vec across sprites; Grid extends/truncates it in-place per child.
    let mut sprite_path: Vec<usize> = Vec::with_capacity(8);
    with_render_context(is_pixel_backend, default_font, || {
        for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
            sprite_path.clear();
            sprite_path.push(sprite_idx);
            render_sprite(
                layer_idx,
                &mut sprite_path,
                sprite,
                root_area,
                None,
                target_resolver,
                object_regions,
                object_states,
                &mut ctx,
            );
        }
    });
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn render_sprite(
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    sprite: &Sprite,
    area: RenderArea,
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

    // Use runtime state for visibility when available (it is initialised from the authored model
    // in construction, so toggling it at runtime via scene.set works). Fall back to authored
    // model visibility only when there is no runtime state (no resolver / no ID).
    let is_visible = if object_id.is_some() {
        object_state.visible
    } else {
        sprite.visible()
    };
    if !is_visible {
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
        Sprite::Text { .. } => render_text_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            ctx,
        ),
        Sprite::Image { .. } => render_image_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            ctx,
        ),
        Sprite::Vector { .. } => render_vector_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            ctx,
        ),
        Sprite::Panel { .. } => render_panel_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            layer_idx,
            sprite_path,
            object_states,
            ctx,
        ),
        Sprite::Grid { .. } => render_grid_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            layer_idx,
            sprite_path,
            object_states,
            ctx,
        ),
        Sprite::Flex { .. } => render_flex_sprite(
            sprite,
            area,
            clip_rect,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            layer_idx,
            sprite_path,
            object_states,
            ctx,
        ),
        Sprite::Obj { .. } => render_obj_sprite(
            sprite,
            area,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            ctx,
        ),
        Sprite::Planet { .. } => render_planet_sprite(
            sprite,
            area,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            sprite_elapsed,
            ctx,
        ),
        Sprite::Scene3D { .. } => {
            render_scene3d_sprite(sprite, area, object_id, &object_state, object_regions, ctx)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_text_sprite(
    sprite: &Sprite,
    area: RenderArea,
    clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Text {
        content,
        x,
        y,
        size,
        font,
        force_font_mode,
        align_x,
        align_y,
        fg_colour,
        bg_colour,
        reveal_ms,
        glow,
        text_transform,
        scale_x,
        scale_y,
        ..
    } = sprite
    else {
        return;
    };
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
    let resolved_font = engine_render_policy::resolve_text_font_spec(
        font.as_deref(),
        force_font_mode.as_deref(),
        *size,
        ctx.is_pixel_backend,
        ctx.default_font,
    );
    let mod_source = ctx.asset_root.map(|root| root.mod_source());
    let (sprite_width, sprite_height) = text_sprite_dimensions(
        mod_source,
        &rendered_content,
        resolved_font.as_deref(),
        fg,
        sprite_bg,
        *scale_x,
        *scale_y,
    );

    let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
    let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
    let (draw_x, draw_y) = compute_draw_pos(
        base_x,
        base_y,
        sprite.animations(),
        sprite_elapsed,
        object_state,
    );

    // OPT-36: Cull sprites completely outside viewport
    if is_sprite_offscreen(
        draw_x as i32,
        draw_y as i32,
        sprite_width,
        sprite_height,
        ctx.layer_buf.width,
        ctx.layer_buf.height,
    ) {
        return;
    }

    let clip = clip_rect;

    if let Some(glow_opts) = glow.as_ref() {
        let glow_col = glow_opts
            .colour
            .as_ref()
            .map(Color::from)
            .unwrap_or_else(|| dim_colour(fg));
        let radius = glow_opts.radius.max(1) as i32;
        let glow_content = strip_markup(&rendered_content);
        let glow_key = glow_cache_key(
            &glow_content,
            radius,
            glow_col,
            resolved_font.as_deref(),
            sprite_bg,
            sprite_width,
            sprite_height,
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
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let gx = (pad as i32 + dx).max(0) as u16;
                    let gy = (pad as i32 + dy).max(0) as u16;
                    render_text_content(
                        mod_source,
                        &glow_content,
                        resolved_font.as_deref(),
                        glow_col,
                        sprite_bg,
                        gx,
                        gy,
                        None,
                        &mut scratch,
                        text_transform,
                        *scale_x,
                        *scale_y,
                    );
                }
            }
            let arc = Arc::new(scratch);
            let mut c = cache.borrow_mut();
            if c.len() >= 128 {
                c.clear();
            }
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
                        if tx_i < cr.x
                            || tx_i >= cr.x + cr.width as i32
                            || ty_i < cr.y
                            || ty_i >= cr.y + cr.height as i32
                        {
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
        &rendered_content,
        resolved_font.as_deref(),
        fg,
        sprite_bg,
        draw_x,
        draw_y,
        clip,
        ctx.layer_buf,
        text_transform,
        *scale_x,
        *scale_y,
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

#[allow(clippy::too_many_arguments)]
fn render_image_sprite(
    sprite: &Sprite,
    area: RenderArea,
    _clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Image {
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
        align_x,
        align_y,
        ..
    } = sprite
    else {
        return;
    };
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
        ctx.asset_root,
    );
    let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
    let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
    let (draw_x, draw_y) = compute_draw_pos(
        base_x,
        base_y,
        sprite.animations(),
        sprite_elapsed,
        object_state,
    );
    render_image_content(
        source,
        target_width,
        target_height,
        target_size,
        *spritesheet_columns,
        *spritesheet_rows,
        *frame_index,
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

#[allow(clippy::too_many_arguments)]
fn render_vector_sprite(
    sprite: &Sprite,
    area: RenderArea,
    _clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Vector {
        points,
        closed,
        draw_char,
        x,
        y,
        align_x,
        align_y,
        fg_colour,
        bg_colour,
        ..
    } = sprite
    else {
        return;
    };
    let Some(bounds) = engine_vector::bounds(points) else {
        return;
    };
    let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
    let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
    let glyph = draw_char
        .as_deref()
        .and_then(|value| value.chars().next())
        .unwrap_or('*');

    let base_x = area.origin_x + resolve_x(*x, align_x, area.width, bounds.width);
    let base_y = area.origin_y + resolve_y(*y, align_y, area.height, bounds.height);
    let (draw_x, draw_y) = compute_draw_pos(
        base_x,
        base_y,
        sprite.animations(),
        sprite_elapsed,
        object_state,
    );
    // Vector sprites are authored around their local origin, so gameplay-driven
    // rotation should pivot around (0,0) instead of an AABB corner.
    let origin_x = i32::from(draw_x);
    let origin_y = i32::from(draw_y);

    // If the entity has a non-zero heading, rotate points around local origin.
    let rotated: Vec<[i32; 2]>;
    let draw_points: &[[i32; 2]] = if object_state.heading.abs() > f32::EPSILON {
        let (sin_h, cos_h) = object_state.heading.sin_cos();
        rotated = points
            .iter()
            .map(|p| {
                let fx = p[0] as f32;
                let fy = p[1] as f32;
                [
                    (fx * cos_h - fy * sin_h).round() as i32,
                    (fx * sin_h + fy * cos_h).round() as i32,
                ]
            })
            .collect();
        &rotated
    } else {
        points
    };

    if *closed && !matches!(bg, Color::Reset) {
        engine_vector::fill_polygon(ctx.layer_buf, draw_points, origin_x, origin_y, '█', bg, bg);
    }
    engine_vector::draw_polyline(
        ctx.layer_buf,
        draw_points,
        *closed,
        origin_x,
        origin_y,
        glyph,
        fg,
        bg,
    );

    // Collect resolved vector for SDL2 native rendering.
    push_vector_primitive(engine_render::VectorPrimitive {
        points: draw_points
            .iter()
            .map(|p| [(origin_x + p[0]) as f32, (origin_y + p[1]) as f32])
            .collect(),
        closed: *closed,
        fg: fg.to_rgb(),
        bg: if *closed && !matches!(bg, Color::Reset) {
            Some(bg.to_rgb())
        } else {
            None
        },
    });

    let sprite_region = Region {
        x: draw_x,
        y: draw_y,
        width: bounds.width,
        height: bounds.height,
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

#[allow(clippy::too_many_arguments)]
fn render_panel_sprite(
    sprite: &Sprite,
    area: RenderArea,
    clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    object_states: &HashMap<String, ObjectRuntimeState>,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Panel {
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
        align_x,
        align_y,
        bg_colour,
        border_colour,
        shadow_colour,
        children,
        ..
    } = sprite
    else {
        return;
    };
    // Panels with no children still render their background box —
    // this supports background-only panels (e.g. HUD decoration behind a flat sprite layer).
    // Only skip rendering if there are no children AND no explicit dimensions are set.
    if children.is_empty() && width.is_none() && height.is_none() {
        return;
    }
    let (auto_w, auto_h) = measure_sprite_for_layout(sprite, ctx.asset_root);
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

    let panel_bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
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
        .unwrap_or(Color::Reset);
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

#[allow(clippy::too_many_arguments)]
fn render_grid_sprite(
    sprite: &Sprite,
    area: RenderArea,
    clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    object_states: &HashMap<String, ObjectRuntimeState>,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Grid {
        x,
        y,
        width,
        height,
        gap_x,
        gap_y,
        align_x,
        align_y,
        columns,
        rows,
        children,
        ..
    } = sprite
    else {
        return;
    };
    // Fast-path: skip empty grids
    if children.is_empty() {
        return;
    }
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
        ctx.asset_root,
        &measure_sprite_for_layout,
    );
    render_children_in_cells(
        layer_idx,
        sprite_path,
        children,
        &child_cells,
        draw_x,
        draw_y,
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

#[allow(clippy::too_many_arguments)]
fn render_flex_sprite(
    sprite: &Sprite,
    area: RenderArea,
    clip_rect: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    object_states: &HashMap<String, ObjectRuntimeState>,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Flex {
        x,
        y,
        width,
        height,
        gap,
        direction,
        align_x,
        align_y,
        children,
        ..
    } = sprite
    else {
        return;
    };
    // Fast-path: skip empty flex containers
    if children.is_empty() {
        return;
    }
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
        ctx.asset_root,
        &measure_sprite_for_layout,
    );
    render_children_in_cells(
        layer_idx,
        sprite_path,
        children,
        &child_cells,
        draw_x,
        draw_y,
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

#[allow(clippy::too_many_arguments)]
fn render_obj_sprite(
    sprite: &Sprite,
    area: RenderArea,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Obj {
        id,
        source,
        x,
        y,
        size,
        width,
        height,
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
        smooth_shading,
        ambient,
        latitude_bands,
        latitude_band_depth,
        terrain_color,
        terrain_threshold,
        terrain_noise_scale,
        terrain_noise_octaves,
        marble_depth,
        below_threshold_transparent,
        polar_ice_color,
        polar_ice_start,
        polar_ice_end,
        desert_color,
        desert_strength,
        atmo_color: _atmo_color,
        atmo_height,
        atmo_density,
        atmo_strength: _atmo_strength,
        atmo_rayleigh_amount,
        atmo_rayleigh_color,
        atmo_rayleigh_falloff,
        atmo_haze_amount,
        atmo_haze_color,
        atmo_haze_falloff,
        atmo_absorption_amount,
        atmo_absorption_color,
        atmo_absorption_height,
        atmo_absorption_width,
        atmo_forward_scatter,
        atmo_limb_boost,
        atmo_terminator_softness,
        atmo_night_glow,
        atmo_night_glow_color,
        atmo_rim_power: _atmo_rim_power,
        atmo_haze_strength: _atmo_haze_strength,
        atmo_haze_power: _atmo_haze_power,
        atmo_veil_strength: _atmo_veil_strength,
        atmo_veil_power: _atmo_veil_power,
        atmo_halo_strength: _atmo_halo_strength,
        atmo_halo_width: _atmo_halo_width,
        atmo_halo_power: _atmo_halo_power,
        night_light_color,
        night_light_threshold,
        night_light_intensity,
        draw_char,
        align_x,
        align_y,
        fg_colour,
        bg_colour,
        world_x,
        world_y,
        world_z,
        cam_world_x,
        cam_world_y,
        cam_world_z,
        view_right_x,
        view_right_y,
        view_right_z,
        view_up_x,
        view_up_y,
        view_up_z,
        view_fwd_x,
        view_fwd_y,
        view_fwd_z,
        camera_source,
        terrain_plane_amplitude,
        terrain_plane_frequency,
        terrain_plane_roughness,
        terrain_plane_octaves,
        terrain_plane_seed_x,
        terrain_plane_seed_z,
        terrain_plane_lacunarity,
        terrain_plane_ridge,
        terrain_plane_plateau,
        terrain_plane_sea_level,
        terrain_plane_scale_x,
        terrain_plane_scale_z,
        world_gen_shape,
        world_gen_base,
        world_gen_coloring,
        world_gen_seed,
        world_gen_ocean_fraction,
        world_gen_continent_scale,
        world_gen_continent_warp,
        world_gen_continent_octaves,
        world_gen_mountain_scale,
        world_gen_mountain_strength,
        world_gen_mountain_ridge_octaves,
        world_gen_moisture_scale,
        world_gen_ice_cap_strength,
        world_gen_lapse_rate,
        world_gen_rain_shadow,
        world_gen_displacement_scale,
        world_gen_subdivisions,
        ..
    } = sprite
    else {
        return;
    };

    let effective_source_buf: String;
    let effective_source: &str = if (source.starts_with("terrain-plane://") || source.starts_with("terrain-sphere://") || source.starts_with("earth-sphere://"))
        && (terrain_plane_amplitude.is_some()
            || terrain_plane_frequency.is_some()
            || terrain_plane_roughness.is_some()
            || terrain_plane_octaves.is_some()
            || terrain_plane_seed_x.is_some()
            || terrain_plane_seed_z.is_some()
            || terrain_plane_lacunarity.is_some()
            || terrain_plane_ridge.is_some()
            || terrain_plane_plateau.is_some()
            || terrain_plane_sea_level.is_some()
            || terrain_plane_scale_x.is_some()
            || terrain_plane_scale_z.is_some())
    {
        let scheme = if source.starts_with("terrain-sphere://") {
            "terrain-sphere"
        } else if source.starts_with("earth-sphere://") {
            "earth-sphere"
        } else {
            "terrain-plane"
        };
        let mut params = parse_terrain_params_from_uri(source);
        if let Some(v) = terrain_plane_amplitude  { params.amplitude  = *v; }
        if let Some(v) = terrain_plane_frequency  { params.frequency  = *v; }
        if let Some(v) = terrain_plane_roughness  { params.roughness  = *v; }
        if let Some(v) = terrain_plane_octaves    { params.octaves    = *v; }
        if let Some(v) = terrain_plane_seed_x     { params.seed_x     = *v; }
        if let Some(v) = terrain_plane_seed_z     { params.seed_z     = *v; }
        if let Some(v) = terrain_plane_lacunarity { params.lacunarity = *v; }
        if let Some(v) = terrain_plane_ridge      { params.ridge      = *v; }
        if let Some(v) = terrain_plane_plateau    { params.plateau    = *v; }
        if let Some(v) = terrain_plane_sea_level  { params.sea_level  = *v; }
        if let Some(v) = terrain_plane_scale_x    { params.scale_x    = *v; }
        if let Some(v) = terrain_plane_scale_z    { params.scale_z    = *v; }
        let grid = source
            .splitn(3, "//")
            .nth(1)
            .unwrap_or("32")
            .split('?')
            .next()
            .unwrap_or("32");
        effective_source_buf = format!(
            "{scheme}://{}?amp={}&freq={}&oct={}&rough={}&sx={}&sz={}&lac={}&ridge={}&plat={}&sea={}&scx={}&scz={}",
            grid,
            params.amplitude, params.frequency, params.octaves, params.roughness,
            params.seed_x, params.seed_z, params.lacunarity,
            if params.ridge { 1 } else { 0 },
            params.plateau, params.sea_level, params.scale_x, params.scale_z
        );
        &effective_source_buf
    } else if source.starts_with("world://")
        && (world_gen_seed.is_some()
            || world_gen_ocean_fraction.is_some()
            || world_gen_continent_scale.is_some()
            || world_gen_continent_warp.is_some()
            || world_gen_continent_octaves.is_some()
            || world_gen_mountain_scale.is_some()
            || world_gen_mountain_strength.is_some()
            || world_gen_mountain_ridge_octaves.is_some()
            || world_gen_moisture_scale.is_some()
            || world_gen_ice_cap_strength.is_some()
            || world_gen_lapse_rate.is_some()
            || world_gen_rain_shadow.is_some()
            || world_gen_displacement_scale.is_some()
            || world_gen_subdivisions.is_some()
            || world_gen_shape.is_some()
            || world_gen_base.is_some()
            || world_gen_coloring.is_some())
    {
        let mut p = engine_worldgen::parse_world_params_from_uri(source);
        if let Some(v) = world_gen_shape               { p.shape = engine_worldgen::parse_world_shape(v); }
        if let Some(v) = world_gen_base                { p.base = engine_worldgen::parse_world_base(v); }
        if let Some(v) = world_gen_coloring            { p.coloring = engine_worldgen::parse_world_coloring(v); }
        if let Some(v) = world_gen_subdivisions        { p.subdivisions = *v; }
        if let Some(v) = world_gen_seed                { p.planet.seed = *v; }
        if let Some(v) = world_gen_ocean_fraction      { p.planet.ocean_fraction = *v; }
        if let Some(v) = world_gen_continent_scale     { p.planet.continent_scale = *v; }
        if let Some(v) = world_gen_continent_warp      { p.planet.continent_warp = *v; }
        if let Some(v) = world_gen_continent_octaves   { p.planet.continent_octaves = *v; }
        if let Some(v) = world_gen_mountain_scale      { p.planet.mountain_scale = *v; }
        if let Some(v) = world_gen_mountain_strength   { p.planet.mountain_strength = *v; }
        if let Some(v) = world_gen_mountain_ridge_octaves { p.planet.mountain_ridge_octaves = *v; }
        if let Some(v) = world_gen_moisture_scale      { p.planet.moisture_scale = *v; }
        if let Some(v) = world_gen_ice_cap_strength    { p.planet.ice_cap_strength = *v; }
        if let Some(v) = world_gen_lapse_rate          { p.planet.lapse_rate = *v; }
        if let Some(v) = world_gen_rain_shadow         { p.planet.rain_shadow = *v; }
        if let Some(v) = world_gen_displacement_scale  { p.displacement_scale = *v; }
        effective_source_buf = engine_worldgen::world_uri_from_params(&p);
        &effective_source_buf
    } else {
        source.as_str()
    };
    let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
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
        object_state,
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
        .cloned()
        .unwrap_or_default();

    // Prerender fast path: check if this sprite has a cached frame.
    let sprite_id_opt = id.as_deref();
    let elapsed_s = sprite_elapsed as f32 / 1000.0;
    let live_total_yaw = rotation_y.unwrap_or(0.0)
        + yaw_deg.unwrap_or(0.0)
        + rotate_y_deg_per_sec.unwrap_or(0.0) * elapsed_s;
    let current_pitch = pitch_deg.unwrap_or(0.0);
    let clip_min = clip_y_min.unwrap_or(0.0);
    let clip_max = clip_y_max.unwrap_or(1.0);
    if let Some(sid) = sprite_id_opt {
        if try_blit_prerendered(
            sid,
            live_total_yaw,
            current_pitch,
            clip_min,
            clip_max,
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

    let use_scene_camera = *camera_source == CameraSource::Scene;
    let scene_camera = ctx.scene_camera_3d;
    render_obj_content(
        effective_source,
        Some(sprite_width),
        Some(sprite_height),
        *size,
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
            object_translate_x: world_x.unwrap_or(0.0),
            object_translate_y: world_y.unwrap_or(0.0),
            object_translate_z: world_z.unwrap_or(0.0),
            clip_y_min: clip_y_min.unwrap_or(0.0),
            clip_y_max: clip_y_max.unwrap_or(1.0),
            // Cockpit camera override: when cam_world_x/y/z are set, use them; otherwise fall
            // back to the legacy (0, 0, -camera_distance) position.
            camera_world_x: if use_scene_camera {
                scene_camera.eye[0]
            } else {
                cam_world_x.unwrap_or(0.0)
            },
            camera_world_y: if use_scene_camera {
                scene_camera.eye[1]
            } else {
                cam_world_y.unwrap_or(0.0)
            },
            camera_world_z: if use_scene_camera {
                scene_camera.eye[2]
            } else {
                cam_world_z.unwrap_or(-camera_distance.unwrap_or(3.0))
            },
            view_right_x: if use_scene_camera {
                scene_camera.right()[0]
            } else {
                view_right_x.unwrap_or(1.0)
            },
            view_right_y: if use_scene_camera {
                scene_camera.right()[1]
            } else {
                view_right_y.unwrap_or(0.0)
            },
            view_right_z: if use_scene_camera {
                scene_camera.right()[2]
            } else {
                view_right_z.unwrap_or(0.0)
            },
            view_up_x: if use_scene_camera {
                scene_camera.up[0]
            } else {
                view_up_x.unwrap_or(0.0)
            },
            view_up_y: if use_scene_camera {
                scene_camera.up[1]
            } else {
                view_up_y.unwrap_or(1.0)
            },
            view_up_z: if use_scene_camera {
                scene_camera.up[2]
            } else {
                view_up_z.unwrap_or(0.0)
            },
            view_forward_x: if use_scene_camera {
                scene_camera.forward()[0]
            } else {
                view_fwd_x.unwrap_or(0.0)
            },
            view_forward_y: if use_scene_camera {
                scene_camera.forward()[1]
            } else {
                view_fwd_y.unwrap_or(0.0)
            },
            view_forward_z: if use_scene_camera {
                scene_camera.forward()[2]
            } else {
                view_fwd_z.unwrap_or(1.0)
            },
            unlit: false,
            ambient: ambient.unwrap_or(0.15),
            light_point_falloff: 0.7,
            light_point_2_falloff: 0.7,
            smooth_shading: smooth_shading.unwrap_or(false),
            latitude_bands: latitude_bands.unwrap_or(0),
            latitude_band_depth: latitude_band_depth.unwrap_or(0.0),
            terrain_color: terrain_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            terrain_threshold: terrain_threshold.unwrap_or(0.5),
            terrain_noise_scale: terrain_noise_scale.unwrap_or(2.5),
            terrain_noise_octaves: terrain_noise_octaves.unwrap_or(2),
            marble_depth: marble_depth.unwrap_or(0.0),
            terrain_relief: 0.0,
            noise_seed: 0.0,
            warp_strength: 0.0,
            warp_octaves: 2,
            noise_lacunarity: 2.0,
            noise_persistence: 0.5,
            normal_perturb_strength: 0.0,
            ocean_specular: 0.0,
            crater_density: 0.0,
            crater_rim_height: 0.35,
            snow_line_altitude: 0.0,
            terrain_displacement: 0.0,
            below_threshold_transparent: *below_threshold_transparent,
            cloud_alpha_softness: 0.0,
            polar_ice_color: polar_ice_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            polar_ice_start: polar_ice_start.unwrap_or(0.78),
            polar_ice_end: polar_ice_end.unwrap_or(0.92),
            desert_color: desert_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            desert_strength: desert_strength.unwrap_or(0.0),
            atmo_color: None,
            atmo_height: atmo_height.unwrap_or(0.12),
            atmo_density: atmo_density.unwrap_or(0.0),
            atmo_strength: 0.0,
            atmo_rayleigh_amount: atmo_rayleigh_amount.unwrap_or(0.0),
            atmo_rayleigh_color: atmo_rayleigh_color
                .as_ref()
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([124, 200, 255])),
            atmo_rayleigh_falloff: atmo_rayleigh_falloff.unwrap_or(0.32),
            atmo_haze_amount: atmo_haze_amount.unwrap_or(0.0),
            atmo_haze_color: atmo_haze_color
                .as_ref()
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([212, 225, 240])),
            atmo_haze_falloff: atmo_haze_falloff.unwrap_or(0.18),
            atmo_absorption_amount: atmo_absorption_amount.unwrap_or(0.0),
            atmo_absorption_color: atmo_absorption_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            atmo_absorption_height: atmo_absorption_height.unwrap_or(0.55),
            atmo_absorption_width: atmo_absorption_width.unwrap_or(0.18),
            atmo_forward_scatter: atmo_forward_scatter.unwrap_or(0.72),
            atmo_limb_boost: atmo_limb_boost.unwrap_or(1.0),
            atmo_terminator_softness: atmo_terminator_softness.unwrap_or(1.0),
            atmo_night_glow: atmo_night_glow.unwrap_or(0.0),
            atmo_night_glow_color: atmo_night_glow_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            atmo_rim_power: 4.5,
            atmo_haze_strength: 0.0,
            atmo_haze_power: 1.8,
            atmo_veil_strength: 0.0,
            atmo_veil_power: 1.6,
            atmo_halo_strength: 0.0,
            atmo_halo_width: 0.12,
            atmo_halo_power: 2.2,
            ocean_noise_scale: 4.0,
            ocean_color_rgb: None,
            night_light_color: night_light_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            night_light_threshold: night_light_threshold.unwrap_or(0.82),
            night_light_intensity: night_light_intensity.unwrap_or(0.0),
            heightmap: None,
            heightmap_w: 0,
            heightmap_h: 0,
            heightmap_blend: 0.0,
            // Opaque OBJ/world meshes rely on the depth buffer; transparent layers use dedicated RGBA paths.
            depth_sort_faces: false,
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

const DEFAULT_PLANET_MESH_SOURCE: &str = "cube-sphere://64";
const DEFAULT_PLANET_CLOUD_COLOR: &str = "#eaf2f8";
const DEFAULT_PLANET_CLOUD_2_COLOR: &str = "#d7e2ec";

#[allow(clippy::too_many_arguments)]
fn render_planet_sprite(
    sprite: &Sprite,
    area: RenderArea,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Planet {
        body_id,
        preset,
        mesh_source,
        x,
        y,
        size,
        width,
        height,
        scale,
        yaw_deg,
        pitch_deg,
        roll_deg,
        spin_deg,
        cloud_spin_deg,
        cloud2_spin_deg,
        observer_altitude_km,
        camera_distance,
        camera_source,
        fov_degrees,
        near_clip,
        sun_dir_x,
        sun_dir_y,
        sun_dir_z,
        align_x,
        align_y,
        ..
    } = sprite
    else {
        return;
    };

    let Some(catalogs) = ctx.celestial_catalogs else {
        return;
    };
    let Some(body) = catalogs.bodies.get(body_id) else {
        return;
    };
    let preset_id = preset.as_deref().or(body.planet_type.as_deref());
    let Some(planet) = preset_id.and_then(|id| catalogs.planet_types.get(id)) else {
        return;
    };

    let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
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
        object_state,
    );

    let use_scene_camera = *camera_source == CameraSource::Scene;
    let scene_camera = ctx.scene_camera_3d;
    let sun_dir = [
        sun_dir_x.unwrap_or(planet.sun_dir_x as f32),
        sun_dir_y.unwrap_or(planet.sun_dir_y as f32),
        sun_dir_z.unwrap_or(planet.sun_dir_z as f32),
    ];
    let surface_scale = scale.unwrap_or(1.0);
    let (cloud_scale, cloud2_scale) = planet_cloud_scales(body, surface_scale);
    let mesh_path = mesh_source.as_deref().unwrap_or(DEFAULT_PLANET_MESH_SOURCE);
    let base_yaw = yaw_deg.unwrap_or(0.0);
    let pitch = pitch_deg.unwrap_or(0.0);
    let roll = roll_deg.unwrap_or(0.0);
    let camera_distance = camera_distance.unwrap_or(3.0);
    let fov_degrees = fov_degrees.unwrap_or(60.0);
    let near_clip = near_clip.unwrap_or(0.001);
    let atmo_visibility = planet_atmosphere_visibility(body, observer_altitude_km.unwrap_or(0.0));

    let mut surface_params = build_planet_base_params(
        surface_scale,
        base_yaw + spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    surface_params.ambient = planet.ambient as f32;
    surface_params.smooth_shading = true;
    surface_params.latitude_bands = planet.latitude_bands;
    surface_params.latitude_band_depth = planet.latitude_band_depth as f32;
    surface_params.terrain_displacement = planet.terrain_displacement as f32;
    surface_params.terrain_color = colour_rgb(Some(planet.land_color.as_str()));
    surface_params.terrain_threshold = planet.terrain_threshold as f32;
    surface_params.terrain_noise_scale = planet.terrain_noise_scale as f32;
    surface_params.terrain_noise_octaves = planet.terrain_noise_octaves;
    surface_params.marble_depth = planet.marble_depth as f32;
    surface_params.terrain_relief = planet.terrain_relief as f32;
    surface_params.polar_ice_color = planet
        .polar_ice_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.polar_ice_start = planet.polar_ice_start as f32;
    surface_params.polar_ice_end = planet.polar_ice_end as f32;
    surface_params.desert_color = planet
        .desert_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.desert_strength = planet.desert_strength as f32;
    surface_params.atmo_color = None;
    surface_params.atmo_height = 0.12;
    surface_params.atmo_density = (planet.atmo_strength as f32 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_strength = 0.0;
    surface_params.atmo_rayleigh_amount = (planet.atmo_strength as f32 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_rayleigh_color = planet
        .atmo_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.atmo_rayleigh_falloff = 0.32;
    surface_params.atmo_haze_amount = (planet.atmo_strength as f32 * 0.45 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_haze_color = surface_params.atmo_rayleigh_color;
    surface_params.atmo_haze_falloff = 0.18;
    surface_params.atmo_absorption_amount = 0.0;
    surface_params.atmo_absorption_color = None;
    surface_params.atmo_absorption_height = 0.55;
    surface_params.atmo_absorption_width = 0.18;
    surface_params.atmo_forward_scatter = 0.72;
    surface_params.atmo_limb_boost = 1.35;
    surface_params.atmo_terminator_softness = 1.05;
    surface_params.atmo_night_glow = 0.0;
    surface_params.atmo_night_glow_color = None;
    surface_params.atmo_rim_power = 4.5;
    surface_params.atmo_haze_strength = 0.0;
    surface_params.atmo_haze_power = 1.8;
    surface_params.atmo_veil_strength = 0.0;
    surface_params.atmo_veil_power = 1.6;
    surface_params.atmo_halo_strength = 0.0;
    surface_params.atmo_halo_width = 0.12;
    surface_params.atmo_halo_power = 2.2;
    surface_params.night_light_color = planet
        .night_light_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.night_light_threshold = planet.night_light_threshold as f32;
    surface_params.night_light_intensity = planet.night_light_intensity as f32;
    // Wire tone palette — this is what gives the ocean its colour (shadow→midtone→highlight
    // maps the Lambertian shade range onto authored ocean colours). Without tone_mix > 0 the
    // ocean renders as shaded white/grey from the sphere mesh face colour.
    surface_params.shadow_colour = planet
        .shadow_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::Black));
    surface_params.midtone_colour = planet
        .midtone_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::White));
    surface_params.highlight_colour = planet
        .highlight_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::White));
    surface_params.tone_mix = planet.tone_mix as f32;
    surface_params.cel_levels = planet.cel_levels;
    surface_params.noise_seed = planet.noise_seed as f32;
    surface_params.heightmap = planet.generated_heightmap.clone();
    surface_params.heightmap_w = planet.generated_heightmap_w;
    surface_params.heightmap_h = planet.generated_heightmap_h;
    surface_params.heightmap_blend = planet.heightmap_blend as f32;
    surface_params.warp_strength = planet.warp_strength as f32;
    surface_params.warp_octaves = planet.warp_octaves;
    surface_params.noise_lacunarity = planet.noise_lacunarity as f32;
    surface_params.noise_persistence = planet.noise_persistence as f32;
    surface_params.normal_perturb_strength = planet.normal_perturb_strength as f32;
    surface_params.ocean_specular = planet.ocean_specular as f32;
    surface_params.ocean_noise_scale = planet.ocean_noise_scale as f32;
    surface_params.crater_density = planet.crater_density as f32;
    surface_params.crater_rim_height = planet.crater_rim_height as f32;
    surface_params.snow_line_altitude = planet.snow_line_altitude as f32;

    // ── RGBA compositing pipeline: surface → cloud1 → cloud2 → blit ──────────
    let ocean_fg = colour_value(Some(planet.ocean_color.as_str()), Color::White);
    let (ocean_r, ocean_g, ocean_b) = ocean_fg.to_rgb();
    surface_params.ocean_color_rgb = Some([ocean_r, ocean_g, ocean_b]);

    // 1. Render surface (opaque) to RGB canvas, then convert to RGBA.
    let Some((surface_rgb, virtual_w, virtual_h)) = render_obj_to_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        *size,
        surface_params,
        false,
        false,
        ocean_fg,
        ctx.asset_root,
    ) else {
        return;
    };
    let mut composited = convert_canvas_to_rgba(surface_rgb);

    // 2. Cloud layer 1 — soft alpha edges, per-pixel noise.
    let cloud_colour = planet
        .cloud_color
        .as_deref()
        .unwrap_or(DEFAULT_PLANET_CLOUD_COLOR);
    let cloud_rgb = colour_rgb(Some(cloud_colour));
    let cloud_threshold = (planet.cloud_threshold as f32).clamp(0.0, 0.999);
    let mut cloud_params = build_planet_base_params(
        cloud_scale,
        base_yaw + cloud_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    cloud_params.ambient = planet.cloud_ambient as f32;
    cloud_params.smooth_shading = true;
    cloud_params.terrain_color = cloud_rgb;
    cloud_params.terrain_threshold = cloud_threshold;
    cloud_params.terrain_noise_scale = planet.cloud_noise_scale as f32;
    cloud_params.terrain_noise_octaves = planet.cloud_noise_octaves.max(1);
    cloud_params.marble_depth = (planet.marble_depth as f32 * 0.5).max(0.003);
    cloud_params.below_threshold_transparent = true;
    cloud_params.cloud_alpha_softness = 0.12;

    if let Some((cloud1_rgba, _, _)) = render_obj_to_rgba_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        *size,
        cloud_params,
        false,
        colour_value(Some(cloud_colour), Color::White),
        ctx.asset_root,
    ) {
        composite_rgba_over(&mut composited, &cloud1_rgba);
    }

    // 3. Cloud layer 2 — sparse high-altitude breakup.
    let cloud2_colour = DEFAULT_PLANET_CLOUD_2_COLOR;
    let mut cloud2_params = build_planet_base_params(
        cloud2_scale,
        base_yaw + 180.0 + cloud2_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    cloud2_params.ambient = 0.004;
    cloud2_params.smooth_shading = true;
    cloud2_params.terrain_color = colour_rgb(Some(cloud2_colour));
    cloud2_params.terrain_threshold = (cloud_threshold + 0.12).min(0.992);
    cloud2_params.terrain_noise_scale = (planet.cloud_noise_scale as f32 * 0.35).max(1.1);
    cloud2_params.terrain_noise_octaves = planet.cloud_noise_octaves.clamp(1, 2);
    cloud2_params.marble_depth = (planet.marble_depth as f32 * 0.2).max(0.002);
    cloud2_params.below_threshold_transparent = true;
    cloud2_params.cloud_alpha_softness = 0.08;

    if let Some((cloud2_rgba, _, _)) = render_obj_to_rgba_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        *size,
        cloud2_params,
        false,
        colour_value(Some(cloud2_colour), Color::White),
        ctx.asset_root,
    ) {
        composite_rgba_over(&mut composited, &cloud2_rgba);
    }

    // 4. Blit composited RGBA canvas to buffer.
    let (target_w, _target_h) =
        obj_sprite_dimensions(Some(sprite_width), Some(sprite_height), *size);
    blit_rgba_canvas(
        ctx.layer_buf,
        &composited,
        virtual_w,
        virtual_h,
        target_w,
        sprite_height,
        draw_x,
        draw_y,
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

fn build_planet_base_params(
    scale: f32,
    yaw_deg: f32,
    pitch_deg: f32,
    roll_deg: f32,
    camera_distance: f32,
    fov_degrees: f32,
    near_clip: f32,
    scene_elapsed_ms: u64,
    use_scene_camera: bool,
    scene_camera: &SceneCamera3D,
    sun_dir: [f32; 3],
) -> ObjRenderParams {
    ObjRenderParams {
        scale,
        yaw_deg,
        pitch_deg,
        roll_deg,
        rotation_x: 0.0,
        rotation_y: 0.0,
        rotation_z: 0.0,
        rotate_y_deg_per_sec: 0.0,
        camera_distance,
        fov_degrees,
        near_clip,
        light_direction_x: sun_dir[0],
        light_direction_y: sun_dir[1],
        light_direction_z: sun_dir[2],
        light_2_direction_x: 0.0,
        light_2_direction_y: 0.0,
        light_2_direction_z: -1.0,
        light_2_intensity: 0.0,
        light_point_x: 0.0,
        light_point_y: 2.0,
        light_point_z: 0.0,
        light_point_intensity: 0.0,
        light_point_colour: None,
        light_point_flicker_depth: 0.0,
        light_point_flicker_hz: 0.0,
        light_point_orbit_hz: 0.0,
        light_point_snap_hz: 0.0,
        light_point_2_x: 0.0,
        light_point_2_y: 0.0,
        light_point_2_z: 0.0,
        light_point_2_intensity: 0.0,
        light_point_2_colour: None,
        light_point_2_flicker_depth: 0.0,
        light_point_2_flicker_hz: 0.0,
        light_point_2_orbit_hz: 0.0,
        light_point_2_snap_hz: 0.0,
        cel_levels: 0,
        shadow_colour: None,
        midtone_colour: None,
        highlight_colour: None,
        tone_mix: 0.0,
        scene_elapsed_ms,
        camera_pan_x: 0.0,
        camera_pan_y: 0.0,
        camera_look_yaw: 0.0,
        camera_look_pitch: 0.0,
        object_translate_x: 0.0,
        object_translate_y: 0.0,
        object_translate_z: 0.0,
        clip_y_min: 0.0,
        clip_y_max: 1.0,
        camera_world_x: if use_scene_camera {
            scene_camera.eye[0]
        } else {
            0.0
        },
        camera_world_y: if use_scene_camera {
            scene_camera.eye[1]
        } else {
            0.0
        },
        camera_world_z: if use_scene_camera {
            scene_camera.eye[2]
        } else {
            -camera_distance
        },
        view_right_x: if use_scene_camera {
            scene_camera.right()[0]
        } else {
            1.0
        },
        view_right_y: if use_scene_camera {
            scene_camera.right()[1]
        } else {
            0.0
        },
        view_right_z: if use_scene_camera {
            scene_camera.right()[2]
        } else {
            0.0
        },
        view_up_x: if use_scene_camera {
            scene_camera.up[0]
        } else {
            0.0
        },
        view_up_y: if use_scene_camera {
            scene_camera.up[1]
        } else {
            1.0
        },
        view_up_z: if use_scene_camera {
            scene_camera.up[2]
        } else {
            0.0
        },
        view_forward_x: if use_scene_camera {
            scene_camera.forward()[0]
        } else {
            0.0
        },
        view_forward_y: if use_scene_camera {
            scene_camera.forward()[1]
        } else {
            0.0
        },
        view_forward_z: if use_scene_camera {
            scene_camera.forward()[2]
        } else {
            1.0
        },
        unlit: false,
        ambient: 0.05,
        light_point_falloff: 0.7,
        light_point_2_falloff: 0.7,
        smooth_shading: true,
        latitude_bands: 0,
        latitude_band_depth: 0.0,
        terrain_displacement: 0.0,
        terrain_color: None,
        terrain_threshold: 0.5,
        terrain_noise_scale: 2.5,
        terrain_noise_octaves: 2,
        marble_depth: 0.0,
        terrain_relief: 0.0,
        noise_seed: 0.0,
        warp_strength: 0.0,
        warp_octaves: 2,
        noise_lacunarity: 2.0,
        noise_persistence: 0.5,
        normal_perturb_strength: 0.0,
        ocean_specular: 0.0,
        crater_density: 0.0,
        crater_rim_height: 0.35,
        snow_line_altitude: 0.0,
        below_threshold_transparent: false,
        cloud_alpha_softness: 0.0,
        polar_ice_color: None,
        polar_ice_start: 0.78,
        polar_ice_end: 0.92,
        desert_color: None,
        desert_strength: 0.0,
        atmo_color: None,
        atmo_height: 0.12,
        atmo_density: 0.0,
        atmo_strength: 0.0,
        atmo_rayleigh_amount: 0.0,
        atmo_rayleigh_color: None,
        atmo_rayleigh_falloff: 0.32,
        atmo_haze_amount: 0.0,
        atmo_haze_color: None,
        atmo_haze_falloff: 0.18,
        atmo_absorption_amount: 0.0,
        atmo_absorption_color: None,
        atmo_absorption_height: 0.55,
        atmo_absorption_width: 0.18,
        atmo_forward_scatter: 0.72,
        atmo_limb_boost: 1.0,
        atmo_terminator_softness: 1.0,
        atmo_night_glow: 0.0,
        atmo_night_glow_color: None,
        atmo_rim_power: 4.5,
        atmo_haze_strength: 0.0,
        atmo_haze_power: 1.8,
        atmo_veil_strength: 0.0,
        atmo_veil_power: 1.6,
        atmo_halo_strength: 0.0,
        atmo_halo_width: 0.12,
        atmo_halo_power: 2.2,
        ocean_noise_scale: 4.0,
        ocean_color_rgb: None,
        night_light_color: None,
        night_light_threshold: 0.82,
        night_light_intensity: 0.0,
        heightmap: None,
        heightmap_w: 0,
        heightmap_h: 0,
        heightmap_blend: 0.0,
        depth_sort_faces: false,
    }
}

fn colour_value(raw: Option<&str>, fallback: Color) -> Color {
    raw.and_then(engine_core::scene::color::parse_colour_str)
        .map(|value| Color::from(&value))
        .unwrap_or(fallback)
}

fn colour_rgb(raw: Option<&str>) -> Option<[u8; 3]> {
    let colour = raw.and_then(engine_core::scene::color::parse_colour_str)?;
    let (r, g, b) = Color::from(&colour).to_rgb();
    Some([r, g, b])
}

fn body_radius_km(body: &BodyDef) -> Option<f32> {
    body.radius_km.map(|value| value as f32).or_else(|| {
        body.km_per_px
            .map(|km_per_px| (body.radius_px * km_per_px) as f32)
    })
}

fn planet_cloud_scales(body: &BodyDef, surface_scale: f32) -> (f32, f32) {
    let Some(radius_km) = body_radius_km(body).filter(|value| *value > f32::EPSILON) else {
        return (surface_scale, surface_scale);
    };
    let cloud_bottom = body.cloud_bottom_km.unwrap_or(0.0) as f32;
    let cloud_top = body.cloud_top_km.unwrap_or(cloud_bottom as f64) as f32;
    let cloud_mid = ((cloud_bottom + cloud_top) * 0.5).max(0.0);
    let cloud_high = (cloud_top + (cloud_top - cloud_bottom).max(6.0) * 1.5).max(cloud_mid);
    (
        surface_scale * (1.0 + cloud_mid / radius_km),
        surface_scale * (1.0 + cloud_high / radius_km),
    )
}

fn planet_atmosphere_visibility(body: &BodyDef, observer_altitude_km: f32) -> f32 {
    let top_km = body
        .atmosphere_top_km
        .map(|value| value as f32)
        .or_else(|| {
            body.atmosphere_top
                .zip(body.km_per_px)
                .map(|(top_px, km_per_px)| (top_px * km_per_px) as f32)
        })
        .unwrap_or(0.0);
    if top_km <= f32::EPSILON {
        return 1.0;
    }
    (1.0 - (observer_altitude_km / (top_km * 8.0)).clamp(0.0, 0.65)).clamp(0.35, 1.0)
}

fn render_scene3d_sprite(
    sprite: &Sprite,
    area: RenderArea,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    object_regions: &mut HashMap<String, Region>,
    ctx: &mut RenderCtx<'_>,
) {
    let Sprite::Scene3D {
        src,
        frame,
        x,
        y,
        camera_source,
        ..
    } = sprite
    else {
        return;
    };
    use crate::scene3d_atlas::Scene3DAtlas;
    use crate::scene3d_runtime_store::Scene3DRuntimeStore;
    use engine_render::rasterizer::blit;
    let draw_x = area
        .origin_x
        .saturating_add(*x)
        .saturating_add(object_state.offset_x)
        .max(0) as u16;
    let draw_y = area
        .origin_y
        .saturating_add(*y)
        .saturating_add(object_state.offset_y)
        .max(0) as u16;

    // Real-time path: if the frame string names a clip (no "-N" suffix with a numeric keyframe
    // index), look up the parsed scene definition and render the current animation frame live.
    // This gives true 60fps 3D animation without startup prerender cost for clip frames.
    let rendered_realtime = if let (Some(entry), Some(asset_root)) =
        (Scene3DRuntimeStore::current_get(src), ctx.asset_root)
    {
        if entry.def.frames.contains_key(frame.as_str()) {
            let buf = crate::scene3d_prerender::render_scene3d_frame_at(
                entry,
                frame,
                ctx.scene_elapsed_ms,
                asset_root,
                (*camera_source == CameraSource::Scene).then_some(ctx.scene_camera_3d),
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

    // Fallback: look up prerendered buffer from world-scoped atlas via thread-local pointer.
    // Used for static frames and when no runtime store is available.
    if !rendered_realtime && *camera_source != CameraSource::Scene {
        if let Some(buf) = Scene3DAtlas::current_get(src, frame) {
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

