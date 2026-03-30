//! Thin terminal-facing re-export of the shared generic rasterizer implementation.
//!
//! This keeps existing module paths stable while ensuring generic glyph logic has
//! a single source of truth in `engine-render`.

pub use engine_render::generic::*;
