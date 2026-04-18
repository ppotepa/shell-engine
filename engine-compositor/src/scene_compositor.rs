use crate::ObjPrerenderedFrames;
use engine_animation::SceneStage;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::color::Color;
use engine_core::scene::{Effect, Layer, LayerSpace, SceneSpace};
use engine_core::scene::ResolvedViewProfile;
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_core::spatial::SpatialContext;
use std::collections::HashMap;
use engine_render_3d::pipeline::extract_render3d_sprite_spec;

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
    /// Pre-classified layer inputs with sprites split into 2D and 3D buckets.
    ///
    /// When `Some`, the compositor uses this prepared layer path instead of
    /// dispatching directly against raw sprite arrays.
    #[cfg(feature = "render-3d")]
    pub prepared_layer_inputs: Option<Vec<crate::prepared_frame::PreparedLayerInput<'a>>>,
}

/// Layer-level frame inputs prepared before composition.
pub struct PreparedLayerFrame<'a> {
    pub index: usize,
    pub layer: &'a Layer,
    pub uses_2d_camera: bool,
    pub authored_visible: bool,
    pub has_active_effects: bool,
    pub has_3d: bool,
}

/// Camera inputs prepared by engine runtime before frame assembly.
pub struct PreparedCameraInputs<'a> {
    pub scene_camera_3d: &'a SceneCamera3D,
    /// World-space camera origin. Non-UI layer origins are shifted by `(-camera_x, -camera_y)`.
    pub camera_x: i32,
    pub camera_y: i32,
    /// 2D camera zoom factor (default 1.0). Non-UI layers are scaled by this factor.
    pub camera_zoom: f32,
    /// Scene-wide spatial contract (units + axis convention).
    pub spatial_context: SpatialContext,
}

/// Per-frame runtime and render state prepared by engine before compositor dispatch.
pub struct PreparedCompositeInputs<'a> {
    pub camera: PreparedCameraInputs<'a>,
    pub resolved_view_profile: &'a ResolvedViewProfile,
    pub ui_font_scale: f32,
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
    pub prerender_frames: Option<&'a ObjPrerenderedFrames>,
}

/// Prepare per-layer timing flags used by compositor assembly decisions.
pub fn prepare_layer_timed_visibility(layers: &[Layer]) -> Vec<bool> {
    layers
        .iter()
        .map(engine_render_2d::layer_has_timed_sprites)
        .collect()
}

/// Prepare per-layer frame records so compositor assembly consumes precomputed
/// visibility/effects/camera-space decisions instead of interpreting layer data inline.
pub fn prepare_layer_frames<'a>(
    frame: &'a FrameAssemblyInputs<'a>,
    current_stage: &SceneStage,
) -> Vec<PreparedLayerFrame<'a>> {
    frame
        .layers
        .iter()
        .enumerate()
        .filter_map(|(index, layer)| {
            if layer.ui && !frame.ui_enabled {
                return None;
            }

            let resolved_space = match layer.space {
                LayerSpace::Inherit => {
                    if layer.ui {
                        LayerSpace::Screen
                    } else {
                        match frame.scene_space {
                            SceneSpace::TwoD => LayerSpace::TwoD,
                            SceneSpace::ThreeD => LayerSpace::ThreeD,
                        }
                    }
                }
                other => other,
            };
            let uses_2d_camera = matches!(resolved_space, LayerSpace::TwoD);

            let stage_ref = match current_stage {
                SceneStage::OnEnter => &layer.stages.on_enter,
                SceneStage::OnIdle => &layer.stages.on_idle,
                SceneStage::OnLeave => &layer.stages.on_leave,
                SceneStage::Done => &layer.stages.on_idle,
            };
            let has_active_effects = stage_ref.steps.iter().any(|s| !s.effects.is_empty())
                || frame
                    .layer_timed_visibility
                    .get(index)
                    .copied()
                    .unwrap_or(false);
            let has_3d = layer
                .sprites
                .iter()
                .any(|sprite| extract_render3d_sprite_spec(sprite).is_some());

            Some(PreparedLayerFrame {
                index,
                layer,
                uses_2d_camera,
                authored_visible: layer.visible,
                has_active_effects,
                has_3d,
            })
        })
        .collect()
}
