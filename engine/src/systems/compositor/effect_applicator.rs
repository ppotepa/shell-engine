use crate::buffer::Buffer;
use crate::effects::{apply_effect, Region};
use crate::scene::{Layer, LayerStages, Stage, Step};
use crate::systems::animator::SceneStage;

/// Apply effects for a sprite's lifecycle stage, driven by sprite-relative elapsed time.
pub fn apply_sprite_effects(
    stages: &LayerStages,
    stage: &SceneStage,
    _step_idx: usize,
    _elapsed_ms: u64,
    sprite_elapsed_ms: u64,
    region: Region,
    buffer: &mut Buffer,
) {
    let current_stage = match stage {
        SceneStage::OnEnter => &stages.on_enter,
        SceneStage::OnIdle  => &stages.on_idle,
        SceneStage::OnLeave => &stages.on_leave,
        SceneStage::Done    => return,
    };

    let (step, progress) = match resolve_step_by_elapsed(current_stage, sprite_elapsed_ms) {
        Some(v) => v,
        None => return,
    };

    for effect in &step.effects {
        apply_effect(effect, progress, region, buffer);
    }
}

/// Apply effects for a full layer, driven by scene or step elapsed time.
pub fn apply_layer_effects(
    layer: &Layer,
    stage: &SceneStage,
    step_idx: usize,
    elapsed_ms: u64,
    scene_elapsed_ms: u64,
    buffer: &mut Buffer,
) {
    let current_stage = match stage {
        SceneStage::OnEnter => &layer.stages.on_enter,
        SceneStage::OnIdle  => &layer.stages.on_idle,
        SceneStage::OnLeave => &layer.stages.on_leave,
        SceneStage::Done    => return,
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
        let p = if dur == 0 { 0.0 } else { (elapsed_ms as f32 / dur as f32).clamp(0.0, 1.0) };
        (step, p)
    };

    let full_region = Region::full(buffer);
    for effect in &step.effects {
        apply_effect(effect, progress, full_region, buffer);
    }
}

/// Find the active step and its normalized progress for a stage at `elapsed_ms`.
pub fn resolve_step_by_elapsed(
    stage: &Stage,
    elapsed_ms: u64,
) -> Option<(&Step, f32)> {
    if stage.steps.is_empty() {
        return None;
    }

    let effective_elapsed = if stage.looping {
        let total: u64 = stage.steps.iter().map(|s| s.duration_ms()).sum();
        if total == 0 { elapsed_ms } else { elapsed_ms % total }
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
