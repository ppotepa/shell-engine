//! Stage visibility behaviors: control visibility based on scene stages or time windows.

use engine_core::scene::BehaviorParams;

use crate::{
    emit_visibility, is_within_time_window, parse_stage_name, resolve_target, Behavior,
    BehaviorCommand, BehaviorContext, GameObject, Scene, SceneStage, TimeScope,
};

/// Shows the object only during the specified scene stages.
pub struct StageVisibilityBehavior {
    target: Option<String>,
    stages: Vec<SceneStage>,
}

impl StageVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let stages = params
            .stages
            .iter()
            .filter_map(|value| parse_stage_name(value))
            .collect::<Vec<_>>();
        Self {
            target: params.target.clone(),
            stages,
        }
    }
}

impl Behavior for StageVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let visible = if self.stages.is_empty() {
            true
        } else {
            self.stages.iter().any(|stage| stage == &ctx.stage)
        };
        emit_visibility(commands, resolve_target(&self.target, object), visible);
    }
}

/// Shows the object only within a configured time window relative to the scene or stage clock.
pub struct TimedVisibilityBehavior {
    target: Option<String>,
    start_ms: Option<u64>,
    end_ms: Option<u64>,
    time_scope: TimeScope,
}

impl TimedVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            start_ms: params.start_ms,
            end_ms: params.end_ms,
            time_scope: TimeScope::from_params(params),
        }
    }
}

impl Behavior for TimedVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let elapsed_ms = self.time_scope.elapsed_ms(ctx);
        emit_visibility(
            commands,
            resolve_target(&self.target, object),
            is_within_time_window(elapsed_ms, self.start_ms, self.end_ms),
        );
    }
}
