//! Compositor types, PostFX passes, and rendering strategy.
//!
//! This crate provides:
//! - PostFX: post-processing effect passes (CRT distort, bloom, burn-in, etc.)
//! - OBJ prerender frame store and Scene3D atlas
//! - Scene compositor strategy pattern types (Cell vs Halfblock)
//! - CompositorProvider trait for decoupling from engine's World type

pub mod provider;
pub mod scene_compositor;
pub mod systems;
pub mod obj_prerender;
pub mod scene3d_atlas;

pub use provider::CompositorProvider;
pub use scene_compositor::{
    CellSceneCompositor, CompositeParams, HalfblockSceneCompositor, SceneCompositor,
};
