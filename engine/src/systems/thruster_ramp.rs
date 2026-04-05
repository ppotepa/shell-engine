//! Thruster ramp system.
//!
//! Advances [`ThrusterRamp`] state for all entities that have one.
//! Reads `ArcadeController`, `AngularBody`, `LinearBrake`, and `PhysicsBody2D`
//! (all generic engine components — no mod knowledge).
//! Writes normalised factor outputs that scripts can read for VFX dispatch.
//!
//! Runs inside `gameplay_system`, after `linear_brake_system` and physics
//! integration, before the behavior (script) system.

use engine_game::GameplayWorld;

pub fn thruster_ramp_system(world: &GameplayWorld, dt_ms: u64) {
    world.tick_thruster_ramps(dt_ms);
}
