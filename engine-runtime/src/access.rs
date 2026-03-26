//! Typed access to runtime settings via [`World`].

use crate::RuntimeSettings;
use engine_core::world::World;

/// Typed access to [`RuntimeSettings`] in World.
pub trait RuntimeAccess {
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
}

impl RuntimeAccess for World {
    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.get::<RuntimeSettings>()
    }
}
