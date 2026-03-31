//! Follow behavior: lock position and visibility to a target object.

use engine_core::scene::BehaviorParams;

use crate::{
    emit_offset, emit_visibility, Behavior, BehaviorCommand, BehaviorContext, GameObject, Scene,
};

/// Locks an object's position to match the current frame position of a named target.
pub struct FollowBehavior {
    target: Option<String>,
    offset_x: i32,
    offset_y: i32,
}

impl FollowBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            offset_x: params.amplitude_x.unwrap_or(0),
            offset_y: params.amplitude_y.unwrap_or(0),
        }
    }
}

impl Behavior for FollowBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let Some(target) = self.target.as_deref() else {
            return;
        };
        let Some(target_state) = ctx.resolved_object_state(target) else {
            return;
        };
        emit_visibility(commands, object.id.clone(), target_state.visible);
        emit_offset(
            commands,
            object.id.clone(),
            target_state.offset_x.saturating_add(self.offset_x),
            target_state.offset_y.saturating_add(self.offset_y),
        );
    }
}
