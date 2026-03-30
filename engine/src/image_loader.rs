//! Thin engine-facing compatibility wrapper around the shared render image loader.
//!
//! This preserves existing engine module paths while keeping decoded image asset
//! loading owned by `engine-render`.

pub use engine_render::image_loader::*;
