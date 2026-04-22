//! Game object and game state types.
//!
//! Provides:
//! - GameObjectKind: Discriminant for scene tree nodes
//! - GameObject: Runtime scene tree node with parent/child relationships
//! - GameState: Mutable game state (flags, variables, etc.)
//! - GameplayWorld: Shared gameplay entity store for mod scripts and gameplay systems
//! - Diagnostics for tracking entity lifecycle and object growth

pub mod collision;
pub mod components;
pub mod diagnostics;
pub mod game_object;
pub mod game_state;
pub mod gameplay;
pub mod strategy;

pub use components::{
    AngularBody, AngularMotor3D, Assembly3D, AtmosphereAffected2D, AttachmentBundle3D,
    BootstrapAssembly3D, BootstrapPreset3D, CharacterMotor3D, Collider2D, ColliderShape,
    ComponentBundle3D, ControlBundle3D, ControlIntent3D, DespawnReason, EntityTimers,
    FlightMotor3D, FollowAnchor2D, FollowAnchor3D, GravityAffected2D, GravityMode2D,
    LifecyclePolicy, Lifetime, LinearBrake, LinearMotor3D, MotorBundle3D, Ownership,
    ParticleColorRamp, ParticlePhysics, ParticleThreadMode, PhysicsBody2D, PhysicsBody3D,
    ReferenceFrameBinding3D, ReferenceFrameMode, ReferenceFrameState3D, SpatialBundle3D,
    SpatialKind, Transform2D, Transform3D, VehicleRuntimePrimitives, VehicleStateCache,
    VisualBinding, WrapBounds,
};
pub use diagnostics::{EntityCountSnapshot, EntityEventLog};
pub use engine_vehicle::{
    BrakePhase, MotionFrame, MotionFrameInput, VehicleFacing, VehicleProfile, VehicleProfileInput,
    VehicleTelemetry, VehicleTelemetryInput,
};
pub use game_object::{GameObject, GameObjectKind};
pub use game_state::GameState;
pub use gameplay::{GameplayEntity, GameplayWorld};

pub use collision::{
    apply_collision_response, apply_particle_bounce, collision_system, particle_collision_system,
    BroadphaseKind, CollisionHit, CollisionStrategies, NarrowphaseKind, WrapStrategy,
};
pub use strategy::{
    GameplayStrategies, MotorApplyStrategy3D, NoopMotorApply3D, NoopPhysicsIntegration3D,
    NoopReferenceFrameResolution3D, ParallelEulerIntegration, PhysicsIntegrationStrategy,
    PhysicsIntegrationStrategy3D, ReferenceFrameResolutionStrategy3D, SimpleEulerIntegration,
};

#[inline]
pub fn point_gravity_accel_2d(dx: f32, dy: f32, gravity_mu: f32) -> Option<(f32, f32)> {
    let dist_sq = dx * dx + dy * dy;
    if gravity_mu <= 0.0 || dist_sq <= 1.0 {
        return None;
    }
    let dist = dist_sq.sqrt();
    let accel = gravity_mu / dist_sq;
    Some((dx / dist * accel, dy / dist * accel))
}

#[inline]
pub fn point_gravity_accel_3d(
    dx: f32,
    dy: f32,
    dz: f32,
    gravity_mu: f32,
) -> Option<(f32, f32, f32)> {
    let dist_sq = dx * dx + dy * dy + dz * dz;
    if gravity_mu <= 0.0 || dist_sq <= 1.0 {
        return None;
    }
    let dist = dist_sq.sqrt();
    let accel = gravity_mu / dist_sq;
    Some((dx / dist * accel, dy / dist * accel, dz / dist * accel))
}
