//! Engine ECS-style systems: behavior, prerender, compositor, menu, renderer, and scene-lifecycle.

pub mod prerender;
pub mod scene3d_prerender;
pub mod behavior;
pub mod compositor;
pub mod engine_io;
pub mod hot_reload;
pub use engine_compositor::systems::postfx;
pub mod renderer;
mod renderer_tests;
pub mod scene_lifecycle;
pub mod warmup;

// Re-export menu from engine-animation
pub use engine_animation::menu;
