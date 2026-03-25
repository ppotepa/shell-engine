//! Game object and game state types.
//!
//! Provides:
//! - GameObjectKind: Discriminant for scene tree nodes
//! - GameObject: Runtime scene tree node with parent/child relationships
//! - GameState: Mutable game state (flags, variables, etc.)

pub mod game_object;
pub mod game_state;

pub use game_object::{GameObject, GameObjectKind};
pub use game_state::GameState;
