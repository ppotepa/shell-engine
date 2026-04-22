use std::collections::HashSet;

use engine_animation::SceneStage;
use engine_core::scene::{CameraSource, Layer, LayerSpace, SceneSpace, Sprite};
use engine_render_3d::pipeline::{
    prepare_render3d_item, PreparedRender3dItem, PreparedRender3dSource,
};

use crate::compositor::LayerPassKind;
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
    pub item: PreparedRender3dItem<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedWorld3dBatchItem {
    pub layer_index: usize,
    pub sprite_idx: usize,
}

#[derive(Debug, Default)]
pub struct PreparedWorld3dBatchPlan {
    pub items: Vec<PreparedWorld3dBatchItem>,
    pub fully_batched_layers: HashSet<usize>,
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
    pub has_3d: bool,
}

impl<'a> PreparedLayerInput<'a> {
    /// Convert to a `PreparedLayerFrame` for compositor assembly code that still
    /// consumes the prepared-frame slice contract.
    pub fn as_layer_frame(&self) -> PreparedLayerFrame<'a> {
        PreparedLayerFrame {
            index: self.layer_index,
            layer: self.layer,
            uses_2d_camera: self.uses_2d_camera,
            authored_visible: self.authored_visible,
            has_active_effects: self.has_active_effects,
            has_3d: self.has_3d,
        }
    }

    pub fn is_world3d_batch_candidate(&self) -> bool {
        !self.layer.ui
            && !self.uses_2d_camera
            && !self.has_active_effects
            && self.sprites_2d.is_empty()
            && !self.sprites_3d.is_empty()
            && self
                .sprites_3d
                .iter()
                .all(prepared_item_supports_world3d_batch)
    }
}

fn prepared_item_supports_world3d_batch(prepared: &PreparedSprite3d<'_>) -> bool {
    sprite_is_batch_safe(prepared.sprite) && prepared_item_is_batch_safe(&prepared.item)
}

fn prepared_item_is_batch_safe(item: &PreparedRender3dItem<'_>) -> bool {
    matches!(
        &item.source,
        // Phase 1 batches only shared scene-camera meshes.
        PreparedRender3dSource::Mesh(spec) if spec.camera_source == CameraSource::Scene
    )
}

fn sprite_is_batch_safe(sprite: &Sprite) -> bool {
    sprite.animations().is_empty() && !sprite_has_stage_effects(sprite)
}

fn sprite_has_stage_effects(sprite: &Sprite) -> bool {
    let stages = sprite.stages();
    [&stages.on_enter, &stages.on_idle, &stages.on_leave]
        .into_iter()
        .any(|stage| stage.steps.iter().any(|step| !step.effects.is_empty()))
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
        if let Some(item) = prepare_render3d_item(sprite) {
            sprites_3d.push(PreparedSprite3d {
                sprite,
                sprite_idx,
                item,
            });
        } else {
            sprites_2d.push(PreparedSprite2d { sprite, sprite_idx });
        }
    }

    let has_3d = !sprites_3d.is_empty();

    Some(PreparedLayerInput {
        layer_index: index,
        layer,
        sprites_2d,
        sprites_3d,
        uses_2d_camera,
        authored_visible: layer.visible,
        has_active_effects,
        has_3d,
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

pub fn collect_world3d_batch_plan(
    prepared: &[PreparedLayerInput<'_>],
    pass: LayerPassKind,
) -> PreparedWorld3dBatchPlan {
    let mut plan = PreparedWorld3dBatchPlan::default();
    if matches!(pass, LayerPassKind::UiOnly) {
        return plan;
    }

    for layer in prepared
        .iter()
        .filter(|layer| pass.includes_layer_input(layer) && layer.is_world3d_batch_candidate())
    {
        plan.items.extend(
            layer
                .sprites_3d
                .iter()
                .map(|sprite| PreparedWorld3dBatchItem {
                    layer_index: layer.layer_index,
                    sprite_idx: sprite.sprite_idx,
                }),
        );
        plan.fully_batched_layers.insert(layer.layer_index);
    }

    plan
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

#[cfg(test)]
mod tests {
    use super::{collect_world3d_batch_plan, prepare_layer_input};
    use crate::compositor::LayerPassKind;
    use engine_animation::SceneStage;
    use engine_core::scene::{Layer, SceneSpace};

    fn parse_layer(yaml: &str) -> Layer {
        serde_yaml::from_str(yaml).expect("layer should parse")
    }

    #[test]
    fn collects_pure_scene_camera_world_layers_into_batch_plan() {
        let layer = parse_layer(
            r#"
name: world
space: 3d
sprites:
  - type: obj
    id: lm
    source: /assets/3d/lm.obj
    camera-source: scene
  - type: obj
    id: shadow
    source: /assets/3d/shadow.obj
    camera-source: scene
"#,
        );

        let prepared = prepare_layer_input(
            0,
            &layer,
            &[false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");
        let plan = collect_world3d_batch_plan(&[prepared], LayerPassKind::WorldOnly);

        assert_eq!(plan.items.len(), 2);
        assert!(plan.fully_batched_layers.contains(&0));
    }

    #[test]
    fn skips_mixed_or_local_camera_layers_from_batch_plan() {
        let local_camera = parse_layer(
            r#"
name: local-camera
space: 3d
sprites:
  - type: obj
    source: /assets/3d/lm.obj
"#,
        );
        let mixed = parse_layer(
            r#"
name: mixed
space: 3d
sprites:
  - type: obj
    source: /assets/3d/lm.obj
    camera-source: scene
  - type: text
    content: HUD
"#,
        );

        let prepared_local = prepare_layer_input(
            0,
            &local_camera,
            &[false, false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");
        let prepared_mixed = prepare_layer_input(
            1,
            &mixed,
            &[false, false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");

        let plan =
            collect_world3d_batch_plan(&[prepared_local, prepared_mixed], LayerPassKind::WorldOnly);

        assert!(plan.items.is_empty());
        assert!(plan.fully_batched_layers.is_empty());
    }

    #[test]
    fn skips_generated_world_or_effectful_mesh_layers_from_batch_plan() {
        let generated_world = parse_layer(
            r#"
name: worldgen
space: 3d
sprites:
  - type: planet
    body-id: moon
    camera-source: scene
"#,
        );
        let effectful_mesh = parse_layer(
            r#"
name: effectful
space: 3d
sprites:
  - type: obj
    source: /assets/3d/lm.obj
    camera-source: scene
    stages:
      on_idle:
        steps:
          - effects:
              - name: clear-to-colour
                duration: 1
"#,
        );
        let animated_mesh = parse_layer(
            r#"
name: animated
space: 3d
sprites:
  - type: obj
    source: /assets/3d/lm.obj
    camera-source: scene
    animations:
      - name: bob
"#,
        );

        let prepared_world = prepare_layer_input(
            0,
            &generated_world,
            &[false, false, false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");
        let prepared_effectful = prepare_layer_input(
            1,
            &effectful_mesh,
            &[false, false, false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");
        let prepared_animated = prepare_layer_input(
            2,
            &animated_mesh,
            &[false, false, false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");

        assert!(
            !prepared_world.is_world3d_batch_candidate(),
            "generated-world sprite should stay on the fallback path"
        );
        assert!(
            !prepared_effectful.is_world3d_batch_candidate(),
            "sprite-stage effects must keep the layer off the batch path"
        );
        assert!(
            !prepared_animated.is_world3d_batch_candidate(),
            "animated mesh sprites must keep the layer off the batch path"
        );

        let plan = collect_world3d_batch_plan(
            &[prepared_world, prepared_effectful, prepared_animated],
            LayerPassKind::WorldOnly,
        );

        assert!(plan.items.is_empty());
        assert!(plan.fully_batched_layers.is_empty());
    }

    #[test]
    fn does_not_collect_ui_only_pass_batches() {
        let layer = parse_layer(
            r#"
name: world
space: 3d
sprites:
  - type: obj
    source: /assets/3d/lm.obj
    camera-source: scene
"#,
        );

        let prepared = prepare_layer_input(
            0,
            &layer,
            &[false],
            true,
            SceneSpace::ThreeD,
            &SceneStage::OnIdle,
        )
        .expect("prepared layer");
        let plan = collect_world3d_batch_plan(&[prepared], LayerPassKind::UiOnly);

        assert!(plan.items.is_empty());
        assert!(plan.fully_batched_layers.is_empty());
    }
}
