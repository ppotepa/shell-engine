//! Scripting domain APIs organized by subsystem (physics, audio, UI, gameplay, etc.)
//!
//! Each domain module encapsulates Script API types and registration logic for its subsystem,
//! replacing the monolithic init_rhai_engine approach with domain-driven design.

pub mod audio;
pub mod debug;
pub mod ephemeral;
pub mod game;
pub mod gameplay;
pub mod gameplay_impl;
pub mod gui;
pub mod io;
pub mod palette;
pub mod physics;
pub mod scene;
pub mod ui;
pub mod world;

use rhai::Engine as RhaiEngine;

/// Register all scripting domain APIs with the Rhai engine.
pub(crate) fn register_all_domains(engine: &mut RhaiEngine) {
    audio::register_with_rhai(engine);
    debug::register_with_rhai(engine);
    game::register_with_rhai(engine);
    gameplay::register_with_rhai(engine);
    gui::register_with_rhai(engine);
    io::register_with_rhai(engine);
    palette::register_with_rhai(engine);
    physics::register_with_rhai(engine);
    scene::register_with_rhai(engine);
    ui::register_with_rhai(engine);
    world::register_with_rhai(engine);
    engine_api::register_effects_api(engine);
    engine_api::register_collision_api(engine);
}
