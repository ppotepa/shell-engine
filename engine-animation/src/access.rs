//! Typed access to the animation system via [`World`].

use crate::Animator;
use engine_core::world::World;

/// Typed access to the [`Animator`] resource in World.
pub trait AnimatorAccess {
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
}

impl AnimatorAccess for World {
    fn animator(&self) -> Option<&Animator> {
        self.get::<Animator>()
    }
    fn animator_mut(&mut self) -> Option<&mut Animator> {
        self.get_mut::<Animator>()
    }
}
