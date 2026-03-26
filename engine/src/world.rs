//! Engine-local World wrapper that delegates to [`engine_core::world::World`].
//!
//! This newtype exists so that engine can implement foreign provider traits
//! (AudioProvider, AnimatorProvider, etc.) on World without violating orphan rules.
//! Once provider traits are replaced by domain access traits, this wrapper can be
//! removed in favour of a direct re-export.

use std::ops::{Deref, DerefMut};

/// Engine-local World wrapper. All resource methods are available via [`Deref`]
/// to [`engine_core::world::World`].
pub struct World(pub engine_core::world::World);

impl World {
    /// Creates an empty World.
    pub fn new() -> Self {
        World(engine_core::world::World::new())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for World {
    type Target = engine_core::world::World;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
