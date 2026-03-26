//! Typed access to the audio runtime via [`World`].

use crate::audio::AudioRuntime;
use engine_core::world::World;

/// Typed access to the [`AudioRuntime`] resource in World.
pub trait AudioAccess {
    fn audio_runtime(&self) -> Option<&AudioRuntime>;
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime>;
}

impl AudioAccess for World {
    fn audio_runtime(&self) -> Option<&AudioRuntime> {
        self.get::<AudioRuntime>()
    }
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
        self.get_mut::<AudioRuntime>()
    }
}
