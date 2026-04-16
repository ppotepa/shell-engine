use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::color::Color;
use engine_core::scene::{Effect, Layer, SceneSpace};
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use std::collections::HashMap;

/// All scene-invariant inputs to a single compositor invocation.
///
/// Groups the 14+ parameters that were previously threaded individually through
/// `composite_scene` and its callers.
pub struct CompositeParams<'a> {
    pub bg: Color,
    pub frame: FrameAssemblyInputs<'a>,
    pub prepared: PreparedCompositeInputs<'a>,
}

/// Inputs that define how the current frame should be assembled.
pub struct FrameAssemblyInputs<'a> {
    pub layers: &'a [Layer],
    pub layer_timed_visibility: &'a [bool],
    pub ui_enabled: bool,
    pub scene_space: SceneSpace,
    pub scene_effects: &'a [Effect],
}

/// Camera inputs prepared by engine runtime before frame assembly.
pub struct PreparedCameraInputs<'a> {
    pub scene_camera_3d: &'a SceneCamera3D,
    /// World-space camera origin. Non-UI layer origins are shifted by `(-camera_x, -camera_y)`.
    pub camera_x: i32,
    pub camera_y: i32,
    /// 2D camera zoom factor (default 1.0). Non-UI layers are scaled by this factor.
    pub camera_zoom: f32,
}

/// Per-frame runtime and render state prepared by engine before compositor dispatch.
pub struct PreparedCompositeInputs<'a> {
    pub camera: PreparedCameraInputs<'a>,
    pub target_resolver: &'a TargetResolver,
    pub object_states: &'a HashMap<String, ObjectRuntimeState>,
    pub obj_camera_states: &'a HashMap<String, ObjCameraState>,
    pub current_stage: &'a SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub scene_elapsed_ms: u64,
    pub scene_effect_progress: f32,
    pub asset_root: Option<&'a AssetRoot>,
    pub celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub is_pixel_backend: bool,
    pub default_font: Option<&'a str>,
}

/// Prepare per-layer timing flags used by compositor assembly decisions.
pub fn prepare_layer_timed_visibility(layers: &[Layer]) -> Vec<bool> {
    layers
        .iter()
        .map(engine_render_2d::layer_has_timed_sprites)
        .collect()
}
