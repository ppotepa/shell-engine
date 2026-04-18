//! Compositor types, PostFX passes, and rendering strategy.
//!
//! This crate provides:
//! - PostFX: post-processing effect passes (CRT distort, bloom, burn-in, etc.)
//! - OBJ prerender/scene3d precompute coordination
//! - Scene compositor strategy pattern types and dispatch
//! - CompositorProvider trait for decoupling from engine's World type
//! - BufferPool: reusable buffer allocation for efficient frame rendering

pub mod access;
pub mod buffer_pool;
pub mod compositor;
pub mod effect_applicator;
#[cfg(feature = "render-3d")]
mod generated_world_render_adapter;
mod layer_compositor;
#[cfg(feature = "render-3d")]
mod obj_render_adapter;
#[cfg(feature = "render-3d")]
pub mod prepared_frame;
#[cfg(feature = "render-3d")]
mod prerender;
pub mod provider;
mod render;
#[cfg(feature = "render-3d")]
mod scene_clip_render_adapter;
mod scene_compositor;
mod sprite_renderer_2d;
mod systems;

/// Re-export (or stub) of ObjPrerenderedFrames for use in compositor internals.
///
/// When the `render-3d` feature is enabled this is the real prerendered frame cache type.
/// When it is disabled a zero-size stub is used so the compositor structs still compile
/// (the field will always be `None` and nothing attempts to access it).
#[cfg(feature = "render-3d")]
pub use engine_render_3d::prerender::ObjPrerenderedFrames;
#[cfg(not(feature = "render-3d"))]
#[derive(Debug, Default)]
pub struct ObjPrerenderedFrames;

pub use access::CompositorAccess;
pub use buffer_pool::{
    acquire_buffer, pool_stats, BufferPool, BufferPoolConfig, PoolStats, PooledBuffer,
};
pub use compositor::{
    dispatch_composite, dispatch_composite_filtered, dispatch_composite_with_render_2d_pipeline,
    dispatch_composite_with_render_2d_pipeline_filtered, LayerPassKind,
};
#[cfg(feature = "render-3d")]
pub use prepared_frame::{
    layer_frames_from_prepared, prepare_frame_layer_inputs, prepare_frame_layer_inputs_from_frame,
    PreparedLayerInput, PreparedSprite2d, PreparedSprite3d,
};
#[cfg(feature = "render-3d")]
pub use prerender::prerender_scene_sprites;
pub use provider::CompositorProvider;
pub use scene_compositor::{
    prepare_layer_frames, prepare_layer_timed_visibility, CompositeParams, FrameAssemblyInputs,
    PreparedCameraInputs, PreparedCompositeInputs,
};
pub use systems::postfx;
