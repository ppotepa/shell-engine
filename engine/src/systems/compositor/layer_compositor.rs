use super::effect_applicator::apply_layer_effects;
use super::sprite_renderer::render_sprites;
use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::effects::Region;
use crate::scene::{Layer, SceneRenderedMode};
use crate::scene_runtime::{ObjCameraState, ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static LAYER_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
}

/// Composite all visible layers onto the scene framebuffer.
pub fn composite_layers(
    layers: &[Layer],
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut BTreeMap<String, Region>,
    scene_origin_x: i32,
    scene_origin_y: i32,
    object_states: &BTreeMap<String, ObjectRuntimeState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    obj_camera_states: &BTreeMap<String, ObjCameraState>,
    buffer: &mut Buffer,
) {
    for (layer_idx, layer) in layers.iter().enumerate() {
        let layer_state = target_resolver
            .and_then(|resolver| resolver.layer_object_id(layer_idx))
            .and_then(|object_id| object_states.get(object_id))
            .cloned()
            .unwrap_or_default();
        if !layer.visible {
            continue;
        }
        if !layer_state.visible {
            continue;
        }
        let total_origin_x = scene_origin_x.saturating_add(layer_state.offset_x);
        let total_origin_y = scene_origin_y.saturating_add(layer_state.offset_y);

        if let Some(object_id) =
            target_resolver.and_then(|resolver| resolver.layer_object_id(layer_idx))
        {
            object_regions.insert(
                object_id.to_string(),
                Region {
                    x: total_origin_x.max(0) as u16,
                    y: total_origin_y.max(0) as u16,
                    width: scene_w
                        .saturating_sub(total_origin_x.unsigned_abs().min(scene_w as u32) as u16),
                    height: scene_h
                        .saturating_sub(total_origin_y.unsigned_abs().min(scene_h as u32) as u16),
                },
            );
        }

        LAYER_SCRATCH.with(|scratch| {
            let mut layer_buf = scratch.borrow_mut();
            if layer_buf.width != scene_w || layer_buf.height != scene_h {
                layer_buf.resize(scene_w, scene_h);
            }
            layer_buf.fill(Color::Reset);

            render_sprites(
                layer_idx,
                layer,
                scene_w,
                scene_h,
                scene_rendered_mode,
                asset_root,
                target_resolver,
                object_regions,
                total_origin_x,
                total_origin_y,
                object_states,
                scene_elapsed_ms,
                current_stage,
                step_idx,
                elapsed_ms,
                obj_camera_states,
                &mut *layer_buf,
            );

            apply_layer_effects(
                layer,
                current_stage,
                step_idx,
                elapsed_ms,
                scene_elapsed_ms,
                target_resolver,
                object_regions,
                &mut *layer_buf,
            );

            // Blit layer onto scene framebuffer — skip transparent pixels
            for ly in 0..scene_h {
                for lx in 0..scene_w {
                    if let Some(cell) = layer_buf.get(lx, ly) {
                        if !(cell.symbol == ' ' && cell.bg == Color::Reset) {
                            let sym = cell.symbol;
                            let fg = cell.fg;
                            let cbg = cell.bg;
                            buffer.set(lx, ly, sym, fg, cbg);
                        }
                    }
                }
            }
        });
    }
}
