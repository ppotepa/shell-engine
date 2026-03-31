//! Bob behavior: sinusoidal offset animation along X/Y axes.

use engine_core::scene::BehaviorParams;

use crate::{
    emit_offset, resolve_target, sine_wave, Behavior, BehaviorCommand, BehaviorContext, GameObject,
    Scene,
};

/// Applies a sinusoidal offset to an object along the X and/or Y axes.
pub struct BobBehavior {
    target: Option<String>,
    amplitude_x: i32,
    amplitude_y: i32,
    period_ms: u64,
    phase_ms: u64,
}

impl BobBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            amplitude_x: params.amplitude_x.unwrap_or(0),
            amplitude_y: params.amplitude_y.unwrap_or(1),
            period_ms: params.period_ms.unwrap_or(2000).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BobBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let wave = sine_wave(ctx.scene_elapsed_ms, self.phase_ms, self.period_ms);
        emit_offset(
            commands,
            resolve_target(&self.target, object),
            (self.amplitude_x as f32 * wave).round() as i32,
            (self.amplitude_y as f32 * wave).round() as i32,
        );
    }
}
