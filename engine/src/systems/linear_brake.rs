//! Linear braking system.
//!
//! Applies velocity damping to all entities with a [`LinearBrake`] component.
//! Runs inside the physics tick, after arcade_controller and angular_body,
//! before the physics integrator.

use engine_game::GameplayWorld;

pub fn linear_brake_system(world: &GameplayWorld, dt_ms: u64) {
    world.tick_linear_brakes(dt_ms);
}
