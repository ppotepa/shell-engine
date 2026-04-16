//! 3D scene rendering system for Shell Engine.
//!
//! Provides:
//! - Scene3D asset format and parsing
//! - 3D-to-2D rasterization via OBJ model rendering
//! - Prerender frame caching with thread-locals
//! - Scene3D reference resolution (materials, cameras, lights)

pub mod obj_frame_cache;
pub mod obj_prerender;
pub mod scene3d_atlas;
pub mod scene3d_format;
pub mod scene3d_resolve;

// Re-export public types
pub use scene3d_atlas::Scene3DAtlas;
pub use scene3d_format::Scene3DDefinition;
pub use scene3d_resolve::{resolve_scene3d_refs, Scene3DAssetResolver};
