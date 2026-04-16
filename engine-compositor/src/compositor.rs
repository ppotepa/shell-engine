use std::collections::HashMap;

use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_effects::apply_effect;
use engine_pipeline::LayerCompositor;

use crate::layer_compositor::composite_layers;
use crate::CompositeParams;

fn composite_scene(
    params: &CompositeParams<'_>,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    buffer.fill(params.bg);
    let scene_state = params
        .runtime
        .object_states
        .get(params.runtime.target_resolver.scene_object_id())
        .cloned()
        .unwrap_or_default();
    if !scene_state.visible {
        return HashMap::new();
    }

    let mut object_regions = HashMap::with_capacity(params.frame.layers.len() + 4);
    object_regions.insert(
        params.runtime.target_resolver.scene_object_id().to_string(),
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
        params.frame.layers,
        params.frame.ui_enabled,
        scene_w,
        scene_h,
        params.frame.scene_space,
        params.render.asset_root,
        Some(params.runtime.target_resolver),
        &mut object_regions,
        scene_state.offset_x,
        scene_state.offset_y,
        params.runtime.object_states,
        params.runtime.current_stage,
        params.runtime.step_idx,
        params.runtime.elapsed_ms,
        params.runtime.scene_elapsed_ms,
        params.runtime.obj_camera_states,
        params.frame.scene_camera_3d,
        params.render.celestial_catalogs,
        params.render.is_pixel_backend,
        params.render.default_font,
        params.frame.camera_x,
        params.frame.camera_y,
        params.frame.camera_zoom,
        layer,
        buffer,
    );

    let scene_progress = if params.frame.scene_step_dur == 0 {
        0.0_f32
    } else {
        (params.runtime.elapsed_ms as f32 / params.frame.scene_step_dur as f32).clamp(0.0, 1.0)
    };
    if params.frame.scene_effects.is_empty() {
        return object_regions;
    }
    let full_region = Region::full(buffer);
    for effect in params.frame.scene_effects {
        let region = params.runtime.target_resolver.effect_region(
            effect.params.target.as_deref(),
            full_region,
            &object_regions,
        );
        apply_effect(effect, scene_progress, region, buffer);
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
