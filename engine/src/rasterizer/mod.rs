//! Thin engine-facing compatibility wrapper around the shared render rasterizer.
//!
//! This preserves existing engine module paths while keeping font loading,
//! generic glyph rasterization, and text rasterization behavior owned by
//! `engine-render`.

pub use engine_render::generic;
pub use engine_render::rasterizer::*;
