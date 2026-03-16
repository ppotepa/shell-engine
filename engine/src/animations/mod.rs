pub mod animation;
pub mod builtin;
pub mod params;

pub use animation::{SpriteAnimation, Transform};
pub use params::AnimationParams;

use crate::scene::Animation;
use std::collections::HashMap;

pub struct AnimationDispatcher {
    registry: HashMap<&'static str, Box<dyn SpriteAnimation>>,
}

impl AnimationDispatcher {
    pub fn new() -> Self {
        let mut d = Self {
            registry: HashMap::new(),
        };
        d.registry
            .insert("float", Box::new(builtin::FloatAnimation));
        d
    }

    /// Compute combined transform from all active animations for a sprite.
    pub fn compute_transform(&self, animations: &[Animation], elapsed_ms: u64) -> Transform {
        let mut total = Transform::default();
        for anim in animations {
            if let Some(impl_) = self.registry.get(anim.name.as_str()) {
                let t = impl_.compute(elapsed_ms, &anim.params);
                total.dx += t.dx;
                total.dy += t.dy;
            }
        }
        total
    }
}
