//! Layout helpers for compositor containers and sprite measurement adapters.
//! Public layout types live in the `engine-layout` crate; this module re-exports
//! them and keeps the compositor-private measurement helpers locally.

pub use engine_layout::*;

pub(crate) mod measure;
pub(crate) use measure::measure_sprite_for_layout;
pub(crate) use measure::with_render_context;
