//! `engine-mesh` — procedural 3D mesh generation for Shell Quest.
//!
//! Provides a [`Mesh`] type and primitive generators:
//!
//! | Generator | Function | Notes |
//! |-----------|----------|-------|
//! | Cube-sphere | [`primitives::cube_sphere`] | Uniform triangles, no pole singularity |
//! | UV-sphere   | [`primitives::uv_sphere`]   | Classic lat/lon, matches original sphere.obj |
//!
//! ## Integration with `engine-compositor`
//!
//! `engine-compositor` converts `Mesh` to `ObjMesh` and injects it into the
//! global mesh cache under a synthetic key:
//!
//! ```text
//! mesh-source: cube-sphere://64   (in scene YAML)
//!     → engine-compositor parses scheme + param
//!     → calls engine_mesh::primitives::cube_sphere(64)
//!     → converts to ObjMesh, stores under "cube-sphere://64"
//! ```
//!
//! This means `engine-mesh` has **zero engine dependencies** and can be used
//! in tools, tests, and future renderers without pulling in the full pipeline.

pub mod mesh;
pub mod primitives;

pub use mesh::Mesh;
