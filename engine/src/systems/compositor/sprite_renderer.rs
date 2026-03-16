use crossterm::style::Color;

use crate::animations::AnimationDispatcher;
use crate::assets::AssetRoot;
use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::Region;
use crate::image_loader::{self, LoadedRgbaImage};
use crate::markup::{parse_spans, strip_markup};
use crate::rasterizer;
use crate::rasterizer::generic;
use crate::render_policy;
use crate::scene::{HorizontalAlign, Layer, SceneRenderedMode, Sprite, VerticalAlign};
use crate::systems::animator::SceneStage;

use super::effect_applicator::apply_sprite_effects;

const ALPHA_THRESHOLD: u8 = 16;

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

#[derive(Clone, Copy)]
enum TrackSpec {
    Auto,
    Fr(u16),
    Fixed(u16),
}

/// Render all sprites in a layer onto `layer_buf`.
pub fn render_sprites(
    layer: &mut Layer,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    scene_elapsed_ms: u64,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    layer_buf: &mut Buffer,
) {
    layer.sprites.sort_by_key(Sprite::z_index);

    let mut ctx = RenderCtx {
        asset_root,
        scene_elapsed_ms,
        current_stage,
        step_idx,
        elapsed_ms,
        layer_buf,
    };

    let root_area = RenderArea {
        origin_x: 0,
        origin_y: 0,
        width: scene_w,
        height: scene_h,
    };

    for sprite in &layer.sprites {
        render_sprite(sprite, root_area, scene_rendered_mode, &mut ctx);
    }
}

fn render_sprite(
    sprite: &Sprite,
    area: RenderArea,
    inherited_mode: SceneRenderedMode,
    ctx: &mut RenderCtx<'_>,
) {
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

            let (sprite_width, sprite_height) =
                text_sprite_dimensions(&rendered_content, resolved_font.as_deref(), fg, sprite_bg);

            let base_x = area.origin_x + resolve_x(*x, align_x, area.width, sprite_width);
            let base_y = area.origin_y + resolve_y(*y, align_y, area.height, sprite_height);
            let sprite_elapsed = ctx.scene_elapsed_ms.saturating_sub(appear_at);
            let anim_dispatcher = AnimationDispatcher::new();
            let transform = anim_dispatcher.compute_transform(animations, sprite_elapsed);
            let draw_x = base_x.saturating_add(transform.dx as i32).max(0) as u16;
            let draw_y = base_y.saturating_add(transform.dy as i32).max(0) as u16;

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
            apply_sprite_effects(
                stages,
                ctx.current_stage,
                ctx.step_idx,
                ctx.elapsed_ms,
                sprite_elapsed,
                sprite_region,
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
            let draw_x = base_x.saturating_add(transform.dx as i32).max(0) as u16;
            let draw_y = base_y.saturating_add(transform.dy as i32).max(0) as u16;

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
            apply_sprite_effects(
                stages,
                ctx.current_stage,
                ctx.step_idx,
                ctx.elapsed_ms,
                sprite_elapsed,
                sprite_region,
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
            let draw_x = base_x.saturating_add(transform.dx as i32);
            let draw_y = base_y.saturating_add(transform.dy as i32);

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
                let child_area = RenderArea {
                    origin_x: draw_x + rect.x as i32,
                    origin_y: draw_y + rect.y as i32,
                    width: rect.width.max(1),
                    height: rect.height.max(1),
                };
                render_sprite(child, child_area, resolved_mode, ctx);
            }

            let sprite_region = Region {
                x: draw_x.max(0) as u16,
                y: draw_y.max(0) as u16,
                width: container_w,
                height: container_h,
            };
            apply_sprite_effects(
                stages,
                ctx.current_stage,
                ctx.step_idx,
                ctx.elapsed_ms,
                sprite_elapsed,
                sprite_region,
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
            text_sprite_dimensions(content, resolved_font.as_deref(), fg, bg)
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
    }
}

fn resolve_track_sizes(
    specs: &[TrackSpec],
    container: u16,
    gap: u16,
    auto_reqs: &[(usize, u16)],
) -> Vec<u16> {
    if specs.is_empty() {
        return vec![container.max(1)];
    }

    let mut sizes = vec![0u16; specs.len()];
    for (idx, spec) in specs.iter().enumerate() {
        if let TrackSpec::Fixed(px) = spec {
            sizes[idx] = *px;
        }
    }

    for (idx, pref) in auto_reqs {
        if *idx >= specs.len() {
            continue;
        }
        if matches!(specs[*idx], TrackSpec::Auto) {
            sizes[*idx] = sizes[*idx].max(*pref);
        }
    }

    let gap_total = gap.saturating_mul((specs.len().saturating_sub(1)) as u16);
    let used = sizes
        .iter()
        .copied()
        .fold(0u16, u16::saturating_add)
        .saturating_add(gap_total);
    let mut remaining = container.saturating_sub(used);

    let fr_total: u32 = specs
        .iter()
        .map(|s| match s {
            TrackSpec::Fr(w) => *w as u32,
            _ => 0,
        })
        .sum();

    if fr_total > 0 && remaining > 0 {
        let mut distributed = 0u16;
        let mut fr_indices = Vec::new();
        for (idx, spec) in specs.iter().enumerate() {
            if let TrackSpec::Fr(weight) = spec {
                fr_indices.push((idx, *weight));
                let share = ((remaining as u32) * (*weight as u32) / fr_total) as u16;
                sizes[idx] = share;
                distributed = distributed.saturating_add(share);
            }
        }

        remaining = remaining.saturating_sub(distributed);
        let mut i = 0usize;
        while remaining > 0 && !fr_indices.is_empty() {
            let (idx, _) = fr_indices[i % fr_indices.len()];
            sizes[idx] = sizes[idx].saturating_add(1);
            remaining = remaining.saturating_sub(1);
            i += 1;
        }
    }

    sizes
}

fn track_start(sizes: &[u16], gap: u16, track_idx: usize) -> u16 {
    let mut pos = 0u16;
    for (i, size) in sizes.iter().enumerate() {
        if i >= track_idx {
            break;
        }
        pos = pos.saturating_add(*size);
        pos = pos.saturating_add(gap);
    }
    pos
}

fn span_size(sizes: &[u16], gap: u16, start_idx: usize, span: usize) -> u16 {
    let end = (start_idx + span).min(sizes.len());
    if start_idx >= end {
        return 1;
    }
    let mut size = 0u16;
    for (i, s) in sizes.iter().enumerate().take(end).skip(start_idx) {
        size = size.saturating_add(*s);
        if i + 1 < end {
            size = size.saturating_add(gap);
        }
    }
    size
}

fn parse_track_spec(input: &str) -> TrackSpec {
    let spec = input.trim().to_ascii_lowercase();
    if spec.is_empty() || spec == "auto" {
        return TrackSpec::Auto;
    }
    if let Some(weight) = spec.strip_suffix("fr") {
        let w = weight.trim().parse::<u16>().unwrap_or(1).max(1);
        return TrackSpec::Fr(w);
    }
    if let Ok(px) = spec.parse::<u16>() {
        return TrackSpec::Fixed(px.max(1));
    }
    TrackSpec::Auto
}

fn render_text_content(
    content: &str,
    font: Option<&str>,
    fg: Color,
    bg: Color,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    match font {
        None => {
            let spans = parse_spans(content);
            let mut col = 0u16;
            for span in &spans {
                let span_fg = span.colour.as_ref().map(Color::from).unwrap_or(fg);
                for ch in span.text.chars() {
                    buf.set(x + col, y, ch, span_fg, bg);
                    col += 1;
                }
            }
        }
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            let spans = parse_spans(content);
            let colored_spans: Vec<(String, Color)> = spans
                .iter()
                .map(|s| {
                    (
                        s.text.clone(),
                        s.colour.as_ref().map(Color::from).unwrap_or(fg),
                    )
                })
                .collect();
            generic::rasterize_spans_mode(&colored_spans, mode, x, y, buf);
        }
        Some(font_name) => {
            let stripped = strip_markup(content);
            let text_buf = rasterizer::rasterize(&stripped, font_name, fg, bg);
            rasterizer::blit(&text_buf, buf, x, y);
        }
    }
}

fn text_sprite_dimensions(content: &str, font: Option<&str>, fg: Color, bg: Color) -> (u16, u16) {
    let visible = strip_markup(content);
    match font {
        None => (visible.chars().count() as u16, 1),
        Some(font_name) if font_name.starts_with("generic") => {
            let mode = generic::GenericMode::from_font_name(font_name);
            generic::generic_dimensions_mode(&visible, mode)
        }
        Some(font_name) => {
            let text_buf = rasterizer::rasterize(&visible, font_name, fg, bg);
            (text_buf.width, text_buf.height)
        }
    }
}

fn render_image_content(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let Some(root) = asset_root else {
        return;
    };
    let Some(image) = image_loader::load_rgba_image(root.mod_source(), source) else {
        return;
    };
    let (target_w, target_h) = resolve_image_dimensions(&image, mode, req_width, req_height);
    if target_w == 0 || target_h == 0 {
        return;
    }

    match mode {
        SceneRenderedMode::Cell => rasterize_image_cell(&image, target_w, target_h, x, y, buf),
        SceneRenderedMode::HalfBlock => {
            rasterize_image_halfblock(&image, target_w, target_h, x, y, buf)
        }
        SceneRenderedMode::QuadBlock => {
            rasterize_image_quadblock(&image, target_w, target_h, x, y, buf)
        }
        SceneRenderedMode::Braille => {
            rasterize_image_braille(&image, target_w, target_h, x, y, buf)
        }
    }
}

fn image_sprite_dimensions(
    source: &str,
    req_width: Option<u16>,
    req_height: Option<u16>,
    mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
) -> (u16, u16) {
    let Some(root) = asset_root else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    let Some(image) = image_loader::load_rgba_image(root.mod_source(), source) else {
        return (req_width.unwrap_or(1), req_height.unwrap_or(1));
    };
    resolve_image_dimensions(&image, mode, req_width, req_height)
}

fn resolve_image_dimensions(
    image: &LoadedRgbaImage,
    mode: SceneRenderedMode,
    req_width: Option<u16>,
    req_height: Option<u16>,
) -> (u16, u16) {
    let (natural_w, natural_h) = natural_image_dimensions(image, mode);
    match (req_width, req_height) {
        (Some(w), Some(h)) => (w.max(1), h.max(1)),
        (Some(w), None) => {
            let h = ((natural_h as u32 * w.max(1) as u32) / natural_w.max(1) as u32).max(1);
            (w.max(1), h.min(u16::MAX as u32) as u16)
        }
        (None, Some(h)) => {
            let w = ((natural_w as u32 * h.max(1) as u32) / natural_h.max(1) as u32).max(1);
            (w.min(u16::MAX as u32) as u16, h.max(1))
        }
        (None, None) => (natural_w.max(1), natural_h.max(1)),
    }
}

fn natural_image_dimensions(image: &LoadedRgbaImage, mode: SceneRenderedMode) -> (u16, u16) {
    let w = image.width.max(1);
    let h = image.height.max(1);
    let (cell_w, cell_h) = match mode {
        SceneRenderedMode::Cell => (w, h),
        SceneRenderedMode::HalfBlock => (w, h.div_ceil(2)),
        SceneRenderedMode::QuadBlock => (w.div_ceil(2), h.div_ceil(2)),
        SceneRenderedMode::Braille => (w.div_ceil(2), h.div_ceil(4)),
    };
    (
        cell_w.min(u16::MAX as u32) as u16,
        cell_h.min(u16::MAX as u32) as u16,
    )
}

fn rasterize_image_cell(
    image: &LoadedRgbaImage,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    for oy in 0..target_h {
        for ox in 0..target_w {
            let px = sample_scaled(
                image,
                ox as u32,
                oy as u32,
                target_w as u32,
                target_h as u32,
            );
            if px[3] < ALPHA_THRESHOLD {
                continue;
            }
            buf.set(x + ox, y + oy, '█', rgb_color(px), TRUE_BLACK);
        }
    }
}

fn rasterize_image_halfblock(
    image: &LoadedRgbaImage,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_h = target_h as u32 * 2;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let top = sample_scaled(image, ox as u32, oy as u32 * 2, target_w as u32, virtual_h);
            let bottom = sample_scaled(
                image,
                ox as u32,
                oy as u32 * 2 + 1,
                target_w as u32,
                virtual_h,
            );
            let top_on = top[3] >= ALPHA_THRESHOLD;
            let bottom_on = bottom[3] >= ALPHA_THRESHOLD;
            let (symbol, fg, bg) = match (top_on, bottom_on) {
                (false, false) => continue,
                (true, false) => ('▀', rgb_color(top), TRUE_BLACK),
                (false, true) => ('▄', rgb_color(bottom), TRUE_BLACK),
                (true, true) => ('▀', rgb_color(top), rgb_color(bottom)),
            };
            buf.set(x + ox, y + oy, symbol, fg, bg);
        }
    }
}

fn rasterize_image_quadblock(
    image: &LoadedRgbaImage,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_w = target_w as u32 * 2;
    let virtual_h = target_h as u32 * 2;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let tl = sample_scaled(image, ox as u32 * 2, oy as u32 * 2, virtual_w, virtual_h);
            let tr = sample_scaled(
                image,
                ox as u32 * 2 + 1,
                oy as u32 * 2,
                virtual_w,
                virtual_h,
            );
            let bl = sample_scaled(
                image,
                ox as u32 * 2,
                oy as u32 * 2 + 1,
                virtual_w,
                virtual_h,
            );
            let br = sample_scaled(
                image,
                ox as u32 * 2 + 1,
                oy as u32 * 2 + 1,
                virtual_w,
                virtual_h,
            );

            let mut mask = 0u8;
            let mut colours = Vec::new();
            if tl[3] >= ALPHA_THRESHOLD {
                mask |= 0b0001;
                colours.push(tl);
            }
            if tr[3] >= ALPHA_THRESHOLD {
                mask |= 0b0010;
                colours.push(tr);
            }
            if bl[3] >= ALPHA_THRESHOLD {
                mask |= 0b0100;
                colours.push(bl);
            }
            if br[3] >= ALPHA_THRESHOLD {
                mask |= 0b1000;
                colours.push(br);
            }
            let Some(symbol) = quadrant_char(mask) else {
                continue;
            };
            let fg = average_rgb(&colours);
            buf.set(x + ox, y + oy, symbol, fg, TRUE_BLACK);
        }
    }
}

fn rasterize_image_braille(
    image: &LoadedRgbaImage,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let virtual_w = target_w as u32 * 2;
    let virtual_h = target_h as u32 * 4;
    for oy in 0..target_h {
        for ox in 0..target_w {
            let sx = ox as u32 * 2;
            let sy = oy as u32 * 4;
            let samples = [
                sample_scaled(image, sx, sy, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 1, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 2, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 1, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 2, virtual_w, virtual_h),
                sample_scaled(image, sx, sy + 3, virtual_w, virtual_h),
                sample_scaled(image, sx + 1, sy + 3, virtual_w, virtual_h),
            ];
            let mut mask = 0u8;
            let mut colours = Vec::new();
            for (i, px) in samples.iter().enumerate() {
                if px[3] < ALPHA_THRESHOLD {
                    continue;
                }
                mask |= 1 << i;
                colours.push(*px);
            }
            let Some(symbol) = braille_char(mask) else {
                continue;
            };
            let fg = average_rgb(&colours);
            buf.set(x + ox, y + oy, symbol, fg, TRUE_BLACK);
        }
    }
}

fn sample_scaled(
    image: &LoadedRgbaImage,
    x: u32,
    y: u32,
    virtual_w: u32,
    virtual_h: u32,
) -> [u8; 4] {
    let vw = virtual_w.max(1);
    let vh = virtual_h.max(1);
    let sx = ((x as u64).saturating_mul(image.width as u64) / vw as u64)
        .min(image.width.saturating_sub(1) as u64) as u32;
    let sy = ((y as u64).saturating_mul(image.height as u64) / vh as u64)
        .min(image.height.saturating_sub(1) as u64) as u32;
    image.pixel(sx, sy).unwrap_or([0, 0, 0, 0])
}

fn rgb_color(px: [u8; 4]) -> Color {
    Color::Rgb {
        r: px[0],
        g: px[1],
        b: px[2],
    }
}

fn average_rgb(colours: &[[u8; 4]]) -> Color {
    if colours.is_empty() {
        return TRUE_BLACK;
    }
    let mut rs = 0u32;
    let mut gs = 0u32;
    let mut bs = 0u32;
    for c in colours {
        rs += c[0] as u32;
        gs += c[1] as u32;
        bs += c[2] as u32;
    }
    let len = colours.len() as u32;
    Color::Rgb {
        r: (rs / len) as u8,
        g: (gs / len) as u8,
        b: (bs / len) as u8,
    }
}

fn quadrant_char(mask: u8) -> Option<char> {
    match mask {
        0 => None,
        1 => Some('▘'),
        2 => Some('▝'),
        3 => Some('▀'),
        4 => Some('▖'),
        5 => Some('▌'),
        6 => Some('▞'),
        7 => Some('▛'),
        8 => Some('▗'),
        9 => Some('▚'),
        10 => Some('▐'),
        11 => Some('▜'),
        12 => Some('▄'),
        13 => Some('▙'),
        14 => Some('▟'),
        15 => Some('█'),
        _ => None,
    }
}

fn braille_char(mask: u8) -> Option<char> {
    if mask == 0 {
        None
    } else {
        char::from_u32(0x2800 + mask as u32)
    }
}

fn dim_colour(c: Color) -> Color {
    use crate::effects::utils::color::colour_to_rgb;
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * 0.25) as u8,
        g: (g as f32 * 0.25) as u8,
        b: (b as f32 * 0.25) as u8,
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
