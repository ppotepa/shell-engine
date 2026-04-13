//! GPU acceleration support for 3D mesh rendering.
//! Provides WGPU-based context, mesh management, and render pipelines.
//!
//! Feature: gpu (optional, enabled by default for supported platforms)
//!
//! ## Phases
//! - Phase 1 (Foundation): GPU context, mesh upload, basic shaders ✓
//! - Phase 2 (Features): Lighting, cel shading, wireframe, depth buffering
//! - Phase 3 (Optimization): Mesh caching, batching, async readback
//! - Phase 4 (Integration): Feature flags, fallback, benchmarking

pub mod context;
pub mod mesh;
pub mod render;

pub use context::GpuContext;
pub use mesh::{GpuMesh, Vertex, RenderParams};
pub use render::{convert_rgba_to_rgb_samples, render_obj_gpu};
