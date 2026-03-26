//! Animation state and lifecycle management.
//!
//! Provides SceneStage (OnEnter/OnIdle/OnLeave/Done) and Animator
//! to track scene progression and frame-by-frame animation timing.
//!
//! Also provides LifecycleProvider trait for decoupling scene lifecycle
//! systems from engine's World type (supports future extraction).

pub mod access;
pub mod animator;
pub mod menu;
pub mod provider;
pub mod systems;

pub use animator::{Animator, SceneStage};
pub use menu::{evaluate_menu_action, MenuAction};
pub use provider::LifecycleProvider;
pub use systems::{animator_system, AnimatorProvider};
