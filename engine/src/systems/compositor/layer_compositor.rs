use std::collections::HashMap;
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

thread_local! {
    static LAYER_SCRATCH: RefCell<Buffer> = RefCell::new(Buffer::new(0, 0));
}

/// Composite all visible layers onto the scene framebuffer.
pub fn composite_layers(
    layers: &[Layer],
    ui_enabled: bool,
    scene_w: u16,
    scene_h: u16,
    scene_rendered_mode: SceneRenderedMode,
    asset_root: Option<&AssetRoot>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    scene_origin_x: i32,
    scene_origin_y: i32,
    object_states: &HashMap<String, ObjectRuntimeState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    direct_layer: bool,
    buffer: &mut Buffer,
) {
    for (layer_idx, layer) in layers.iter().enumerate() {
        if layer.ui && !ui_enabled {
            continue;
        }
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

        // #4 opt-comp-layerscratch: DirectLayerCompositor strategy — layers without active
        // effects render directly onto the scene buffer (skip scratch fill+blit).
        // When disabled (ScratchLayerCompositor), always use the scratch path (safe default).
        let layer_has_active_effects = {
            let stage_ref = match current_stage {
                SceneStage::OnEnter => &layer.stages.on_enter,
                SceneStage::OnIdle => &layer.stages.on_idle,
                SceneStage::OnLeave => &layer.stages.on_leave,
                SceneStage::Done => &layer.stages.on_idle,
            };
            stage_ref.steps.iter().any(|s| !s.effects.is_empty())
        };
        let needs_scratch = if direct_layer { layer_has_active_effects } else { true };

        if needs_scratch {
            // Full scratch path: fill + render + effects + blit.
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

                buffer.blit_from(&layer_buf, 0, 0, 0, 0, scene_w, scene_h);
            });
        } else {
            // No effects: render sprites directly onto scene buffer (skip scratch).
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
                buffer,
            );
        }
    }
}
