//! Engine ECS-style systems: behavior, prerender, compositor, menu, renderer, and scene-lifecycle.

pub mod audio_sequencer;
pub mod behavior;
pub mod compositor;
pub mod engine_io;
pub mod gameplay;
pub mod collision;
pub mod gameplay_events;
pub mod visual_binding;
pub mod hot_reload;
pub mod prerender;
pub mod scene3d_prerender;
pub use engine_compositor::systems::postfx;
pub mod renderer;
mod renderer_tests;
pub mod scene_lifecycle;
pub mod warmup;

// Re-export menu from engine-animation
pub use engine_animation::menu;
