//! Scripting domain APIs organized by subsystem (physics, audio, UI, gameplay, etc.)
//!
//! Each domain module encapsulates Script API types and registration logic for its subsystem,
//! replacing the monolithic init_rhai_engine approach with domain-driven design.

pub mod audio;
pub mod debug;
pub mod game;
pub mod gameplay;
pub mod gameplay_impl;
pub mod helpers;
pub mod io;
pub mod scene;
pub mod ui;

use rhai::Engine as RhaiEngine;

/// Register all scripting domain APIs with the Rhai engine.
///
/// Called once during engine initialization. Each domain module handles its own
/// type registration and function binding, keeping concerns separated and easier
/// to maintain and extend.
pub(crate) fn register_all_domains(engine: &mut RhaiEngine) {
    audio::register_with_rhai(engine);
    debug::register_with_rhai(engine);
    game::register_with_rhai(engine);
    gameplay::register_with_rhai(engine);
    io::register_with_rhai(engine);
    scene::register_with_rhai(engine);
    ui::register_with_rhai(engine);
}
