//! Game object and game state types.
//!
//! Provides:
//! - GameObjectKind: Discriminant for scene tree nodes
//! - GameObject: Runtime scene tree node with parent/child relationships
//! - GameState: Mutable game state (flags, variables, etc.)
//! - GameplayWorld: Shared gameplay entity store for mod scripts and gameplay systems

pub mod game_object;
pub mod game_state;
pub mod gameplay;
pub mod components;
pub mod prefabs;
pub mod strategy;
pub mod collision;

pub use game_object::{GameObject, GameObjectKind};
pub use game_state::GameState;
pub use gameplay::{GameplayEntity, GameplayWorld};
pub use components::{
    Collider2D, ColliderShape, EntityTimers, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding, WrapBounds,
};
pub use prefabs::{PrefabSpec, SpawnParams};
pub use strategy::{GameplayStrategies, PhysicsIntegrationStrategy, SimpleEulerIntegration};
pub use collision::{CollisionStrategies, CollisionHit, BroadphaseKind, NarrowphaseKind, WrapStrategy};
