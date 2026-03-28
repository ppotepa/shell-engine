use std::cell::RefCell;
use std::collections::HashMap;

use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::buffer::{Buffer, Cell, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::effects::{apply_effect, Region};
use engine_core::scene::{Effect, Layer, SceneRenderedMode};
use engine_core::scene_runtime_types::{ObjCameraState, ObjectRuntimeState, TargetResolver};
use engine_pipeline::{HalfblockPacker, LayerCompositor};

use crate::layer_compositor::composite_layers;
use crate::CompositeParams;

thread_local! {
    static HALFBLOCK_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
}

fn composite_scene(
    bg: Color,
    layers: &[Layer],
    ui_enabled: bool,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[Effect],
    scene_step_dur: u64,
    is_pixel_backend: bool,
    default_font: Option<&str>,
    layer: &dyn LayerCompositor,
    halfblock: &dyn HalfblockPacker,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    buffer.fill(bg);
    halfblock.prepare_source(buffer);
    let scene_state = object_states
        .get(target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return HashMap::new();
    }

    let mut object_regions = HashMap::with_capacity(layers.len() + 4);
    object_regions.insert(
        target_resolver.scene_object_id().to_string(),
        offset_region(
            buffer.width,
            buffer.height,
            scene_state.offset_x,
            scene_state.offset_y,
        ),
    );

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    composite_layers(
        layers,
        ui_enabled,
        scene_w,
        scene_h,
        scene_rendered_mode,
        asset_root,
        Some(target_resolver),
        &mut object_regions,
        scene_state.offset_x,
        scene_state.offset_y,
        object_states,
        current_stage,
        step_idx,
        elapsed_ms,
        scene_elapsed_ms,
        obj_camera_states,
        is_pixel_backend,
        default_font,
        layer,
        buffer,
    );

    let scene_progress = if scene_step_dur == 0 {
        0.0_f32
    } else {
        (elapsed_ms as f32 / scene_step_dur as f32).clamp(0.0, 1.0)
    };
    if scene_effects.is_empty() {
        return object_regions;
    }
    let full_region = Region::full(buffer);
    for effect in scene_effects {
        let region = target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(effect, scene_progress, region, buffer);
    }
    object_regions
}

fn composite_scene_halfblock(
    bg: Color,
    layers: &[Layer],
    ui_enabled: bool,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_effects: &[Effect],
    scene_step_dur: u64,
    is_pixel_backend: bool,
    default_font: Option<&str>,
    layer: &dyn LayerCompositor,
    halfblock: &dyn HalfblockPacker,
    target: &mut Buffer,
) -> HashMap<String, Region> {
    let needed_w = target.width;
    let needed_h = target.height.saturating_mul(2);
    HALFBLOCK_SCRATCH.with(|scratch| {
        let mut virtual_buf = scratch.borrow_mut();
        if virtual_buf.width != needed_w || virtual_buf.height != needed_h {
            virtual_buf.resize(needed_w, needed_h);
        }
        let object_regions = composite_scene(
            bg,
            layers,
            ui_enabled,
            scene_rendered_mode,
            asset_root,
            target_resolver,
            object_states,
            obj_camera_states,
            current_stage,
            step_idx,
            elapsed_ms,
            scene_elapsed_ms,
            scene_effects,
            scene_step_dur,
            is_pixel_backend,
            default_font,
            layer,
            halfblock,
            &mut virtual_buf,
        );
        pack_halfblock_buffer(&virtual_buf, target, bg, halfblock);
        object_regions
    })
}

pub fn pack_halfblock_buffer(
    source: &Buffer,
    target: &mut Buffer,
    fallback_bg: Color,
    halfblock: &dyn HalfblockPacker,
) {
    target.fill(fallback_bg);

    let Some((x_start, x_end, y_start, y_end)) = halfblock.iteration_bounds(source, target.height)
    else {
        return;
    };

    for y in y_start..=y_end {
        let top_y = y.saturating_mul(2);
        let bottom_y = top_y.saturating_add(1);
        for x in x_start..=x_end {
            let top = cell_or_blank(source, x, top_y, fallback_bg);
            let bottom = if bottom_y < source.height {
                cell_or_blank(source, x, bottom_y, fallback_bg)
            } else {
                Cell::blank(resolve_bg(top.bg, fallback_bg))
            };

            let top_bg = resolve_bg(top.bg, fallback_bg);
            let bottom_bg = resolve_bg(bottom.bg, fallback_bg);
            let out_bg = select_background(top_bg, bottom_bg, fallback_bg);
            let top_on = top.symbol != ' ';
            let bottom_on = bottom.symbol != ' ';

            let (symbol, fg, bg) = match (top_on, bottom_on) {
                (false, false) => (' ', TRUE_BLACK, out_bg),
                (true, false) => ('▀', top.fg, bottom_bg),
                (false, true) => ('▄', bottom.fg, top_bg),
                (true, true) => {
                    if top.fg == bottom.fg {
                        ('█', top.fg, out_bg)
                    } else {
                        ('▀', top.fg, bottom.fg)
                    }
                }
            };
            target.set(x, y, symbol, fg, bg);
        }
    }
}

#[inline]
fn offset_region(width: u16, height: u16, offset_x: i32, offset_y: i32) -> Region {
    let origin_x = offset_x.max(0) as u16;
    let origin_y = offset_y.max(0) as u16;
    let clipped_w = width.saturating_sub(offset_x.unsigned_abs().min(width as u32) as u16);
    let clipped_h = height.saturating_sub(offset_y.unsigned_abs().min(height as u32) as u16);
    Region {
        x: origin_x,
        y: origin_y,
        width: clipped_w,
        height: clipped_h,
    }
}

#[inline]
fn cell_or_blank(buffer: &Buffer, x: u16, y: u16, fallback_bg: Color) -> Cell {
    buffer
        .get(x, y)
        .cloned()
        .unwrap_or_else(|| Cell::blank(fallback_bg))
}

#[inline]
fn resolve_bg(colour: Color, fallback_bg: Color) -> Color {
    if matches!(colour, Color::Reset) {
        fallback_bg
    } else {
        colour
    }
}

#[inline]
fn select_background(top: Color, bottom: Color, fallback_bg: Color) -> Color {
    if top != fallback_bg {
        top
    } else {
        bottom
    }
}

pub fn dispatch_composite(
    mode: SceneRenderedMode,
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    halfblock: &dyn HalfblockPacker,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    match mode {
        SceneRenderedMode::HalfBlock => composite_scene_halfblock(
            params.bg,
            params.layers,
            params.ui_enabled,
            params.scene_rendered_mode,
            params.asset_root,
            params.target_resolver,
            params.object_states,
            params.obj_camera_states,
            params.current_stage,
            params.step_idx,
            params.elapsed_ms,
            params.scene_elapsed_ms,
            params.scene_effects,
            params.scene_step_dur,
            params.is_pixel_backend,
            params.default_font,
            layer,
            halfblock,
            buffer,
        ),
        _ => composite_scene(
            params.bg,
            params.layers,
            params.ui_enabled,
            params.scene_rendered_mode,
            params.asset_root,
            params.target_resolver,
            params.object_states,
            params.obj_camera_states,
            params.current_stage,
            params.step_idx,
            params.elapsed_ms,
            params.scene_elapsed_ms,
            params.scene_effects,
            params.scene_step_dur,
            params.is_pixel_backend,
            params.default_font,
            layer,
            halfblock,
            buffer,
        ),
    }
}
