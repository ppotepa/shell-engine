//! Animation state and lifecycle management.
//!
//! Provides SceneStage (OnEnter/OnIdle/OnLeave/Done) and Animator
//! to track scene progression and frame-by-frame animation timing.

pub mod animator;
pub mod systems;

pub use animator::{Animator, SceneStage};
pub use systems::{animator_system, AnimatorProvider};
