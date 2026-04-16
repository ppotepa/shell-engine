//! Compositor types, PostFX passes, and rendering strategy.
//!
//! This crate provides:
//! - PostFX: post-processing effect passes (CRT distort, bloom, burn-in, etc.)
//! - OBJ prerender frame store and Scene3D atlas
//! - Scene compositor strategy pattern types and dispatch
//! - CompositorProvider trait for decoupling from engine's World type
//! - BufferPool: reusable buffer allocation for efficient frame rendering

pub mod access;
pub mod buffer_pool;
pub mod compositor;
pub mod effect_applicator;
mod generated_world_render_adapter;
mod layer_compositor;
pub mod obj_prerender;
mod obj_render;
mod obj_render_adapter;
pub mod obj_render_helpers;
mod prerender;
pub mod provider;
pub mod render;
mod scene3d_prerender;
mod scene_clip_render_adapter;
mod scene_compositor;
mod sprite_renderer_2d;
pub mod systems;

pub use access::CompositorAccess;
pub use buffer_pool::{
    acquire_buffer, pool_stats, BufferPool, BufferPoolConfig, PoolStats, PooledBuffer,
};
pub use compositor::dispatch_composite;
pub use engine_render_3d::prerender::Scene3DAtlas;
pub use engine_render_3d::prerender::{
    build_scene3d_runtime_store, with_runtime_store, Scene3DRuntimeStore,
};
pub(crate) use obj_render::{
    blit_color_canvas, blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba,
    obj_sprite_dimensions, render_obj_content, render_obj_to_canvas, render_obj_to_rgba_canvas,
    render_obj_to_shared_buffers, try_blit_prerendered, ObjRenderParams,
};
pub use obj_render::{virtual_dimensions, with_prerender_frames};
pub use prerender::prerender_scene_sprites;
pub use provider::CompositorProvider;
pub use scene3d_prerender::render_scene3d_work_item;
pub use scene_compositor::{
    prepare_layer_timed_visibility, CompositeParams, FrameAssemblyInputs, PreparedCameraInputs,
    PreparedCompositeInputs,
};
pub use systems::postfx;

/// Clear the per-frame vector primitive collector (call before compositing).
pub fn clear_vector_primitives() {
    engine_render_2d::clear_vector_primitives();
}

/// Take collected vector primitives (call after compositing).
pub fn take_vector_primitives() -> Vec<engine_render::VectorPrimitive> {
    engine_render_2d::take_vector_primitives()
}
