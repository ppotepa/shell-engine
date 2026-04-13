//! Engine ECS-style systems: behavior, prerender, compositor, menu, renderer, and scene-lifecycle.

pub mod angular_body;
pub mod arcade_controller;
pub mod atmosphere;
pub mod audio_sequencer;
pub mod behavior;
pub mod collision;
pub mod compositor;
pub mod free_look_camera;
pub mod gameplay;
pub mod gameplay_events;
pub mod gravity;
pub mod hot_reload;
pub mod linear_brake;
pub mod particle_physics;
pub mod particle_ramp;
pub mod prerender;
pub mod scene3d_prerender;
pub mod thruster_ramp;
pub mod visual_binding;
pub mod visual_sync;
pub use engine_compositor::systems::postfx;
pub mod renderer;
mod renderer_tests;
pub mod scene_lifecycle;
pub mod warmup;

// Re-export menu from engine-animation
pub use engine_animation::menu;
