use crate::scene::AnimationParams;

/// Position offset computed by an animation for a single frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    pub dx: i16,
    pub dy: i16,
}

/// Core sprite animation abstraction.
/// Animations modify sprite *position* (transform), not pixel colors.
/// Applied before rasterization — contrast with Effects which modify pixels after.
pub trait SpriteAnimation: Send + Sync {
    /// Compute the transform for this animation at the given elapsed time.
    /// `elapsed_ms` is time since sprite appeared. Animation loops using its own `period_ms`.
    fn compute(&self, elapsed_ms: u64, params: &AnimationParams) -> Transform;
}
