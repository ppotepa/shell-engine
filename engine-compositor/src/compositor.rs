use std::collections::HashMap;

use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_effects::apply_effect;
use engine_pipeline::LayerCompositor;

use crate::layer_compositor::{composite_layers, LayerCompositeInputs, PreparedLayerRenderInputs};
use crate::CompositeParams;

fn composite_scene(
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    buffer.fill(params.bg);
    let scene_state = params
        .prepared
        .object_states
        .get(params.prepared.target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return HashMap::new();
    }

    let mut object_regions = HashMap::with_capacity(params.frame.layers.len() + 4);
    object_regions.insert(
        params
            .prepared
            .target_resolver
            .scene_object_id()
            .to_string(),
        offset_region(
            buffer.width,
            buffer.height,
            scene_state.offset_x,
            scene_state.offset_y,
        ),
    );

    let scene_w = buffer.width;
    let scene_h = buffer.height;

    let mut layer_inputs = LayerCompositeInputs {
        layers: params.frame.layers,
        layer_timed_visibility: params.frame.layer_timed_visibility,
        ui_enabled: params.frame.ui_enabled,
        scene_w,
        scene_h,
        scene_space: params.frame.scene_space,
        target_resolver: Some(params.prepared.target_resolver),
        object_regions: &mut object_regions,
        scene_origin_x: scene_state.offset_x,
        scene_origin_y: scene_state.offset_y,
        object_states: params.prepared.object_states,
        current_stage: params.prepared.current_stage,
        step_idx: params.prepared.step_idx,
        elapsed_ms: params.prepared.elapsed_ms,
        scene_elapsed_ms: params.prepared.scene_elapsed_ms,
        camera_x: params.prepared.camera.camera_x,
        camera_y: params.prepared.camera.camera_y,
        camera_zoom: params.prepared.camera.camera_zoom,
        render: PreparedLayerRenderInputs {
            asset_root: params.prepared.asset_root,
            obj_camera_states: params.prepared.obj_camera_states,
            scene_camera_3d: params.prepared.camera.scene_camera_3d,
            celestial_catalogs: params.prepared.celestial_catalogs,
            is_pixel_backend: params.prepared.is_pixel_backend,
            default_font: params.prepared.default_font,
        },
    };
    composite_layers(&mut layer_inputs, layer, buffer);

    if params.frame.scene_effects.is_empty() {
        return object_regions;
    }
    let full_region = Region::full(buffer);
    for effect in params.frame.scene_effects {
        let region = params.prepared.target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(
            effect,
            params.prepared.scene_effect_progress,
            region,
            buffer,
        );
    }
    object_regions
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

pub fn dispatch_composite(
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    composite_scene(params, layer, buffer)
}
