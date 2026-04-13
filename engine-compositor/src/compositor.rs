use std::collections::HashMap;

use engine_animation::SceneStage;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene::{Effect, Layer, SceneSpace};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_effects::apply_effect;
use engine_pipeline::LayerCompositor;

use crate::layer_compositor::composite_layers;
use crate::CompositeParams;

#[allow(clippy::too_many_arguments)]
fn composite_scene(
    bg: Color,
    layers: &[Layer],
    ui_enabled: bool,
    asset_root: Option<&AssetRoot>,
    target_resolver: &TargetResolver,
    object_states: &HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &HashMap<String, ObjCameraState>,
    current_stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    scene_space: SceneSpace,
    scene_camera_3d: &SceneCamera3D,
    celestial_catalogs: Option<&engine_celestial::CelestialCatalogs>,
    scene_effects: &[Effect],
    scene_step_dur: u64,
    is_pixel_backend: bool,
    default_font: Option<&str>,
    camera_x: i32,
    camera_y: i32,
    camera_zoom: f32,
    layer: &dyn LayerCompositor,
    buffer: &mut Buffer,
) -> HashMap<String, Region> {
    buffer.fill(bg);
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
        scene_space,
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
        scene_camera_3d,
        celestial_catalogs,
        is_pixel_backend,
        default_font,
        camera_x,
        camera_y,
        camera_zoom,
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
    composite_scene(
        params.bg,
        params.layers,
        params.ui_enabled,
        params.asset_root,
        params.target_resolver,
        params.object_states,
        params.obj_camera_states,
        params.current_stage,
        params.step_idx,
        params.elapsed_ms,
        params.scene_elapsed_ms,
        params.scene_space,
        params.scene_camera_3d,
        params.celestial_catalogs,
        params.scene_effects,
        params.scene_step_dur,
        params.is_pixel_backend,
        params.default_font,
        params.camera_x,
        params.camera_y,
        params.camera_zoom,
        layer,
        buffer,
    )
}
