//! Typed access to the event queue via [`World`].

use crate::EventQueue;
use engine_core::world::World;

/// Typed access to the [`EventQueue`] resource in World.
pub trait EventAccess {
    fn events(&self) -> Option<&EventQueue>;
    fn events_mut(&mut self) -> Option<&mut EventQueue>;
}

impl EventAccess for World {
    fn events(&self) -> Option<&EventQueue> {
        self.get::<EventQueue>()
    }
    fn events_mut(&mut self) -> Option<&mut EventQueue> {
        self.get_mut::<EventQueue>()
    }
}
