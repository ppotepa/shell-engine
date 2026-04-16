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
pub mod layer_compositor;
pub mod layout;
pub mod obj_loader;
pub mod obj_prerender;
pub mod obj_render;
pub mod obj_render_helpers;
pub mod prerender;
pub mod provider;
pub mod render;
pub mod scene3d_atlas;
pub mod scene3d_prerender;
pub mod scene3d_runtime_store;
pub mod scene_compositor;
pub mod sprite_renderer;
pub mod systems;
pub mod warmup;

pub use access::CompositorAccess;
pub use buffer_pool::{
    acquire_buffer, pool_stats, BufferPool, BufferPoolConfig, PoolStats, PooledBuffer,
};
pub use compositor::dispatch_composite;
pub use layout::{
    compute_flex_cells, compute_grid_cells, parse_track_spec, resolve_x, resolve_y, GridCellRect,
    RenderArea, TrackSpec,
};
pub use obj_render::{
    blit_color_canvas, blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba,
    obj_sprite_dimensions, render_obj_content, render_obj_to_canvas, render_obj_to_rgba_canvas,
    render_obj_to_shared_buffers, try_blit_prerendered, virtual_dimensions, with_prerender_frames,
    ObjRenderParams,
};
pub use prerender::prerender_scene_sprites;
pub use provider::CompositorProvider;
pub use scene3d_atlas::Scene3DAtlas;
pub use scene3d_prerender::{
    build_scene3d_runtime_store, prerender_scene3d_atlas, render_scene3d_frame_at,
};
pub use scene3d_runtime_store::{with_runtime_store, Scene3DRuntimeStore};
pub use scene_compositor::CompositeParams;
pub use systems::postfx;
pub use warmup::warmup_scene_meshes;

/// Clear the per-frame vector primitive collector (call before compositing).
pub fn clear_vector_primitives() {
    sprite_renderer::VECTOR_PRIMITIVES.with(|v| v.borrow_mut().clear());
}

/// Take collected vector primitives (call after compositing).
pub fn take_vector_primitives() -> Vec<engine_render::VectorPrimitive> {
    sprite_renderer::VECTOR_PRIMITIVES.with(|v| std::mem::take(&mut *v.borrow_mut()))
}
