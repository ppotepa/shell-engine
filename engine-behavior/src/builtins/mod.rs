//! Builtin behaviors: reusable, game-independent behavior implementations.
//!
//! Extracted from the monolithic lib.rs, these behaviors are now organized by function:
//! - audio: scene-level audio cue scheduling
//! - blink: visibility toggling on a cycle
//! - bob: sinusoidal position offset
//! - follow: lock position/visibility to a target
//! - stage: visibility based on scene stages or time windows
//! - menu: menu navigation and carousel positioning
//! - arrows: flanking arrows for menu selection

pub mod audio;
pub mod arrows;
pub mod blink;
pub mod bob;
pub mod follow;
pub mod menu;
pub mod stage;

// Re-exports for convenience
pub use audio::SceneAudioBehavior;
pub use arrows::SelectedArrowsBehavior;
pub use blink::BlinkBehavior;
pub use bob::BobBehavior;
pub use follow::FollowBehavior;
pub use menu::{MenuCarouselBehavior, MenuCarouselObjectBehavior, MenuSelectedBehavior};
pub use stage::{StageVisibilityBehavior, TimedVisibilityBehavior};
