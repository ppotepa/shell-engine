use crate::animations::{SpriteAnimation, Transform};
use crate::scene::AnimationParams;
use std::f32::consts::PI;

/// Smooth sinusoidal bob animation.
/// Sprite oscillates along the configured axis by ±amplitude cells
/// with a full sine period of period_ms.
pub struct FloatAnimation;

impl SpriteAnimation for FloatAnimation {
    fn compute(&self, elapsed_ms: u64, params: &AnimationParams) -> Transform {
        let period = params.period_ms.max(1) as f32;
        let t = (elapsed_ms as f32 % period) / period;
        let raw = (2.0 * PI * t).sin();
        let offset = (raw * params.amplitude as f32).round() as i16;

        use crate::animations::params::AnimationAxis;
        match params.axis {
            AnimationAxis::Y => Transform { dx: 0, dy: offset },
            AnimationAxis::X => Transform { dx: offset, dy: 0 },
        }
    }
}
