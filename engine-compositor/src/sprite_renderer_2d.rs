use engine_core::color::Color;
use std::borrow::Cow;
use std::sync::Arc;

use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::markup::strip_markup;
use engine_core::scene::{Layer, Sprite};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_render_2d::{
    compute_flex_cells, compute_grid_cells, dim_colour, image_sprite_dimensions,
    intersect_clip_rect, measure_sprite_for_layout, push_vector_primitive,
    render_children_in_cells, render_image_content, render_panel_box, render_text_content,
    resolve_x, resolve_y, text_sprite_dimensions, with_render_context, ClipRect, RenderArea,
};
use engine_render_3d::pipeline::{
    extract_generated_world_sprite_spec, extract_obj_sprite_spec, extract_scene_clip_sprite_spec,
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

pub(crate) trait Render3dDelegate {
    fn render_obj_sprite(
        &self,
        sprite: &Sprite,
        area: RenderArea,
        target_resolver: Option<&TargetResolver>,
        object_regions: &mut HashMap<String, Region>,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        appear_at: u64,
        sprite_elapsed: u64,
        ctx: &mut RenderCtx<'_>,
    );

    fn render_generated_world_sprite(
        &self,
        sprite: &Sprite,
        area: RenderArea,
        target_resolver: Option<&TargetResolver>,
        object_regions: &mut HashMap<String, Region>,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        sprite_elapsed: u64,
        ctx: &mut RenderCtx<'_>,
    );

    fn render_scene_clip_sprite(
        &self,
        sprite: &Sprite,
        area: RenderArea,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        object_regions: &mut HashMap<String, Region>,
        ctx: &mut RenderCtx<'_>,
    );
}

/// Render all sprites in a layer onto `layer_buf`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_sprites(
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
    render_3d: &dyn Render3dDelegate,
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
                render_3d,
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
    render_3d: &dyn Render3dDelegate,
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

    if extract_obj_sprite_spec(sprite).is_some() {
        render_3d.render_obj_sprite(
            sprite,
            area,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            appear_at,
            sprite_elapsed,
            ctx,
        );
        return;
    }
    if extract_generated_world_sprite_spec(sprite).is_some() {
        render_3d.render_generated_world_sprite(
            sprite,
            area,
            target_resolver,
            object_regions,
            object_id,
            &object_state,
            sprite_elapsed,
            ctx,
        );
        return;
    }
    if extract_scene_clip_sprite_spec(sprite).is_some() {
        render_3d.render_scene_clip_sprite(
            sprite,
            area,
            object_id,
            &object_state,
            object_regions,
            ctx,
        );
        return;
    }

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
            render_3d,
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
            render_3d,
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
            render_3d,
        ),
        Sprite::Obj { .. } | Sprite::Planet { .. } | Sprite::Scene3D { .. } => {}
    }
}
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
    render_3d: &dyn Render3dDelegate,
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
            render_3d,
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
    render_3d: &dyn Render3dDelegate,
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
        |layer_idx,
         sprite_path,
         sprite,
         area,
         clip_rect,
         target_resolver,
         object_regions,
         object_states,
         ctx| {
            render_sprite(
                layer_idx,
                sprite_path,
                sprite,
                area,
                clip_rect,
                target_resolver,
                object_regions,
                object_states,
                ctx,
                render_3d,
            );
        },
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
    render_3d: &dyn Render3dDelegate,
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
        |layer_idx,
         sprite_path,
         sprite,
         area,
         clip_rect,
         target_resolver,
         object_regions,
         object_states,
         ctx| {
            render_sprite(
                layer_idx,
                sprite_path,
                sprite,
                area,
                clip_rect,
                target_resolver,
                object_regions,
                object_states,
                ctx,
                render_3d,
            );
        },
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
