use engine_animation::SceneStage;
use engine_core::scene::{Layer, LayerSpace, SceneSpace, Sprite};
use engine_render_3d::pipeline::{extract_render3d_sprite_spec, Render3dSpriteSpec};

use crate::scene_compositor::{FrameAssemblyInputs, PreparedLayerFrame};

/// A 2D sprite reference prepared for rendering — confirmed non-3D at preparation time.
pub struct PreparedSprite2d<'a> {
    pub sprite: &'a Sprite,
    pub sprite_idx: usize,
}

/// A 3D sprite reference with its render spec pre-extracted at preparation time.
pub struct PreparedSprite3d<'a> {
    pub sprite: &'a Sprite,
    pub sprite_idx: usize,
    pub spec: Render3dSpriteSpec<'a>,
}

/// Layer render inputs with sprites pre-classified into 2D and 3D render lists.
///
/// Carries the same visibility/camera decisions as `PreparedLayerFrame` but additionally
/// separates sprites into 2D and 3D buckets so the compositor can consume pre-extracted
/// render information instead of raw authored sprite detail at dispatch time.
pub struct PreparedLayerInput<'a> {
    pub layer_index: usize,
    /// Full layer reference — still needed for effects, name, and other layer-level metadata.
    pub layer: &'a Layer,
    pub sprites_2d: Vec<PreparedSprite2d<'a>>,
    pub sprites_3d: Vec<PreparedSprite3d<'a>>,
    pub uses_2d_camera: bool,
    pub authored_visible: bool,
    pub has_active_effects: bool,
}

impl<'a> PreparedLayerInput<'a> {
    /// Convert to a `PreparedLayerFrame` for backwards-compatible compositor paths
    /// that still drive composite_layers via the existing prepared-frame slice contract.
    pub fn as_layer_frame(&self) -> PreparedLayerFrame<'a> {
        PreparedLayerFrame {
            index: self.layer_index,
            layer: self.layer,
            uses_2d_camera: self.uses_2d_camera,
            authored_visible: self.authored_visible,
            has_active_effects: self.has_active_effects,
        }
    }
}

/// Prepare sprite-classified render inputs for one layer.
///
/// Returns `None` when the layer is excluded (e.g. UI layer when UI is disabled).
pub fn prepare_layer_input<'a>(
    index: usize,
    layer: &'a Layer,
    layer_timed_visibility: &[bool],
    ui_enabled: bool,
    scene_space: SceneSpace,
    current_stage: &SceneStage,
) -> Option<PreparedLayerInput<'a>> {
    if layer.ui && !ui_enabled {
        return None;
    }

    let resolved_space = match layer.space {
        LayerSpace::Inherit => {
            if layer.ui {
                LayerSpace::Screen
            } else {
                match scene_space {
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
        || layer_timed_visibility.get(index).copied().unwrap_or(false);

    // Classify sprites as 2D or 3D at preparation time.
    let mut sprites_2d = Vec::new();
    let mut sprites_3d = Vec::new();
    for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
        if let Some(spec) = extract_render3d_sprite_spec(sprite) {
            sprites_3d.push(PreparedSprite3d {
                sprite,
                sprite_idx,
                spec,
            });
        } else {
            sprites_2d.push(PreparedSprite2d { sprite, sprite_idx });
        }
    }

    Some(PreparedLayerInput {
        layer_index: index,
        layer,
        sprites_2d,
        sprites_3d,
        uses_2d_camera,
        authored_visible: layer.visible,
        has_active_effects,
    })
}

/// Prepare sprite-classified render inputs for all layers in a frame.
///
/// Mirrors `prepare_layer_frames` but additionally classifies each layer's sprites into
/// 2D and 3D buckets, allowing the compositor to use pre-extracted render information
/// instead of raw authored sprite detail when dispatching renders.
pub fn prepare_frame_layer_inputs<'a>(
    layers: &'a [Layer],
    layer_timed_visibility: &[bool],
    ui_enabled: bool,
    scene_space: SceneSpace,
    current_stage: &SceneStage,
) -> Vec<PreparedLayerInput<'a>> {
    layers
        .iter()
        .enumerate()
        .filter_map(|(index, layer)| {
            prepare_layer_input(
                index,
                layer,
                layer_timed_visibility,
                ui_enabled,
                scene_space,
                current_stage,
            )
        })
        .collect()
}

/// Derive a `PreparedLayerFrame` slice from prepared layer inputs.
///
/// Used by compositor paths that still consume `PreparedLayerFrame` internally.
pub fn layer_frames_from_prepared<'a>(
    prepared: &'a [PreparedLayerInput<'a>],
) -> Vec<PreparedLayerFrame<'a>> {
    prepared
        .iter()
        .map(PreparedLayerInput::as_layer_frame)
        .collect()
}

/// Convenience: prepare frame inputs from a `FrameAssemblyInputs` reference.
///
/// Reads the raw layers and frame-level flags — does not consume the prepared field.
pub fn prepare_frame_layer_inputs_from_frame<'a>(
    frame: &'a FrameAssemblyInputs<'a>,
    current_stage: &SceneStage,
) -> Vec<PreparedLayerInput<'a>> {
    prepare_frame_layer_inputs(
        frame.layers,
        frame.layer_timed_visibility,
        frame.ui_enabled,
        frame.scene_space,
        current_stage,
    )
}
