use crate::buffer::Buffer;
use crate::effects::{apply_effect, Region};
use crate::scene::{Layer, LayerStages, Stage, Step};
use crate::scene_runtime::TargetResolver;
use crate::systems::animator::SceneStage;
use std::collections::BTreeMap;

/// Apply effects for a sprite's lifecycle stage.
///
/// Timing model:
/// - on_enter/on_idle: resolved by sprite-relative elapsed (independent sprite timing).
/// - on_leave: resolved by scene stage step index + step-local elapsed, so leave effects
///   start at progress 0.0 when on_leave begins (instead of jumping to completion).
pub fn apply_sprite_effects(
    stages: &LayerStages,
    stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    sprite_elapsed_ms: u64,
    region: Region,
    target_resolver: Option<&TargetResolver>,
    object_regions: &BTreeMap<String, Region>,
    buffer: &mut Buffer,
) {
    let current_stage = match stage {
        SceneStage::OnEnter => &stages.on_enter,
        SceneStage::OnIdle => &stages.on_idle,
        SceneStage::OnLeave => &stages.on_leave,
        SceneStage::Done => return,
    };

    let (step, progress) = match stage {
        SceneStage::OnLeave => {
            let step = match current_stage.steps.get(step_idx) {
                Some(s) => s,
                None => return,
            };
            let dur = step.duration_ms();
            let p = if dur == 0 {
                0.0
            } else {
                (elapsed_ms as f32 / dur as f32).clamp(0.0, 1.0)
            };
            (step, p)
        }
        _ => match resolve_step_by_elapsed(current_stage, sprite_elapsed_ms) {
            Some(v) => v,
            None => return,
        },
    };

    for effect in &step.effects {
        let target_region = target_resolver
            .map(|resolver| {
                resolver.effect_region(effect.params.target.as_deref(), region, object_regions)
            })
            .unwrap_or(region);
        apply_effect(effect, progress, target_region, buffer);
    }
}

/// Apply effects for a full layer, driven by scene or step elapsed time.
pub fn apply_layer_effects(
    layer: &Layer,
    stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    target_resolver: Option<&TargetResolver>,
    object_regions: &BTreeMap<String, Region>,
    buffer: &mut Buffer,
) {
    let current_stage = match stage {
        SceneStage::OnEnter => &layer.stages.on_enter,
        SceneStage::OnIdle => &layer.stages.on_idle,
        SceneStage::OnLeave => &layer.stages.on_leave,
        SceneStage::Done => return,
    };

    let (step, progress) = if matches!(stage, SceneStage::OnEnter) {
        match resolve_step_by_elapsed(current_stage, scene_elapsed_ms) {
            Some(v) => v,
            None => return,
        }
    } else {
        let step = match current_stage.steps.get(step_idx) {
            Some(s) => s,
            None => return,
        };
        let dur = step.duration_ms();
        let p = if dur == 0 {
            0.0
        } else {
            (elapsed_ms as f32 / dur as f32).clamp(0.0, 1.0)
        };
        (step, p)
    };

    let full_region = Region::full(buffer);
    for effect in &step.effects {
        let target_region = target_resolver
            .map(|resolver| {
                resolver.effect_region(effect.params.target.as_deref(), full_region, object_regions)
            })
            .unwrap_or(full_region);
        apply_effect(effect, progress, target_region, buffer);
    }
}

/// Find the active step and its normalized progress for a stage at `elapsed_ms`.
pub fn resolve_step_by_elapsed(stage: &Stage, elapsed_ms: u64) -> Option<(&Step, f32)> {
    if stage.steps.is_empty() {
        return None;
    }

    let effective_elapsed = if stage.looping {
        let total: u64 = stage.steps.iter().map(|s| s.duration_ms()).sum();
        if total == 0 {
            elapsed_ms
        } else {
            elapsed_ms % total
        }
    } else {
        elapsed_ms
    };

    let mut acc = 0_u64;
    for step in &stage.steps {
        let dur = step.duration_ms();
        if dur == 0 {
            return Some((step, 1.0));
        }
        let end = acc.saturating_add(dur);
        if effective_elapsed < end {
            let local = effective_elapsed.saturating_sub(acc);
            let p = (local as f32 / dur as f32).clamp(0.0, 1.0);
            return Some((step, p));
        }
        acc = end;
    }
    // Non-looping: hold last step at completion.
    stage.steps.last().map(|s| (s, 1.0))
}

#[cfg(test)]
mod tests {
    use super::apply_layer_effects;
    use crate::buffer::Buffer;
    use crate::effects::Region;
    use crate::scene::{Effect, EffectParams, Layer, LayerStages, Stage, Step, TermColour};
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::animator::SceneStage;
    use crossterm::style::Color;
    use std::collections::BTreeMap;

    #[test]
    fn layer_effects_can_target_named_sprite_region() {
        let runtime = SceneRuntime::new(
            serde_yaml::from_str(
                r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: UI
    stages:
      on_idle:
        steps:
          - effects:
              - name: clear-to-colour
                duration: 1
                params:
                  colour: blue
                  target: title
    sprites:
      - type: text
        id: title
        content: HELLO
"#,
            )
            .expect("scene should parse"),
        );
        let resolver = runtime.target_resolver();
        let mut object_regions = BTreeMap::new();
        let title_id = resolver.resolve_alias("title").expect("title target");
        object_regions.insert(
            title_id.to_string(),
            Region {
                x: 2,
                y: 1,
                width: 3,
                height: 1,
            },
        );

        let mut buffer = Buffer::new(8, 3);
        buffer.fill(Color::Black);
        let layer = Layer {
            name: "UI".to_string(),
            z_index: 0,
            visible: true,
            stages: LayerStages {
                on_enter: Stage::default(),
                on_idle: Stage {
                    trigger: Default::default(),
                    steps: vec![Step {
                        effects: vec![Effect {
                            name: "clear-to-colour".to_string(),
                            duration: 1,
                            looping: false,
                            params: EffectParams {
                                colour: Some(TermColour::Blue),
                                target: Some("title".to_string()),
                                ..EffectParams::default()
                            },
                        }],
                        duration: Some(1),
                    }],
                    looping: false,
                },
                on_leave: Stage::default(),
            },
            behaviors: Vec::new(),
            sprites: Vec::new(),
        };

        apply_layer_effects(
            &layer,
            &SceneStage::OnIdle,
            0,
            1,
            0,
            Some(&resolver),
            &object_regions,
            &mut buffer,
        );

        assert_eq!(buffer.get(2, 1).expect("target cell").bg, Color::Blue);
        assert_eq!(buffer.get(0, 0).expect("untargeted cell").bg, Color::Black);
    }
}
