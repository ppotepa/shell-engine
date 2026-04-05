//! Angular body system: integrates per-frame turn input into angular velocity
//! and applies the resulting rotation to `Transform2D.heading`.
//!
//! Runs after `arcade_controller_system` and before physics integration so that
//! heading changes are visible to thrust calculations in the same frame.

use engine_game::GameplayWorld;

/// Tick all [`AngularBody`] components forward by `dt_ms` milliseconds.
pub fn angular_body_system(world: &GameplayWorld, dt_ms: u64) {
    world.tick_angular_bodies(dt_ms);
}
