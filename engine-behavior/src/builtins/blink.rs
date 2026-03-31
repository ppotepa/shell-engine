//! Blink behavior: alternates visibility on a configurable cycle.

use engine_core::scene::BehaviorParams;

use crate::{
    emit_visibility, resolve_target, Behavior, BehaviorCommand, BehaviorContext, GameObject, Scene,
};

/// Alternates an object's visibility on a configurable on/off cycle.
pub struct BlinkBehavior {
    target: Option<String>,
    visible_ms: u64,
    hidden_ms: u64,
    phase_ms: u64,
}

impl BlinkBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            visible_ms: params.visible_ms.unwrap_or(250),
            hidden_ms: params.hidden_ms.unwrap_or(250),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BlinkBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let cycle = self.visible_ms.saturating_add(self.hidden_ms);
        let visible = if cycle == 0 {
            true
        } else {
            let t = ctx.scene_elapsed_ms.saturating_add(self.phase_ms) % cycle;
            t < self.visible_ms || self.hidden_ms == 0
        };
        emit_visibility(commands, resolve_target(&self.target, object), visible);
    }
}
