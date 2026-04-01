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
    Collider2D, ColliderShape, DespawnReason, EntityTimers, FollowAnchor2D, LifecyclePolicy,
    Lifetime, Ownership, PhysicsBody2D, Transform2D, VisualBinding, WrapBounds,
};
pub use diagnostics::{EntityCountSnapshot, EntityEventLog};
pub use game_object::{GameObject, GameObjectKind};
pub use game_state::GameState;
pub use gameplay::{GameplayEntity, GameplayWorld};

pub use collision::{
    BroadphaseKind, CollisionHit, CollisionStrategies, NarrowphaseKind, WrapStrategy,
};
pub use strategy::{GameplayStrategies, PhysicsIntegrationStrategy, SimpleEulerIntegration};
