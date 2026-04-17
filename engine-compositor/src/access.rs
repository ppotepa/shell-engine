//! Trait abstracting World access for the compositor system.
//!
//! This trait allows compositor code to work with any type that provides the required
//! scene state, without hard-coding a dependency on `engine::World`. Once implemented,
//! the trait enables compositor logic to move out of `engine` into `engine-compositor`.

use engine_animation::Animator;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_render_2d::Render2dPipeline;
use engine_runtime::RuntimeSettings;

/// Trait providing access to the resources needed by the compositor system.
pub trait CompositorAccess {
    /// Get immutable scene runtime state (objects, layers, stages, effects, etc.)
    fn scene_runtime(&self) -> Option<&dyn std::any::Any>;

    /// Get immutable animator (stage, step index, elapsed time).
    fn animator(&self) -> Option<&Animator>;

    /// Get mutable buffer (write target for compositing).
    #[allow(clippy::mut_from_ref)]
    fn buffer_mut(&self) -> Option<&mut Buffer>;

    /// Get runtime settings.
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;

    /// Get asset root (mod directory/zip path).
    fn asset_root(&self) -> Option<&AssetRoot>;

    /// Get 3D scene atlas (OBJ material/mesh precomputed data).
    fn scene3d_atlas(&self) -> Option<&dyn std::any::Any>;

    /// Get OBJ prerendered frames cache.
    fn obj_prerender_frames(&self) -> Option<&dyn std::any::Any>;

    /// Get layer compositor strategy (delegates to strategy/LayerCompositor impl).
    fn layer_compositor(&self) -> Option<&dyn std::any::Any>;

    /// Optional prepared 2D render pipeline seam.
    fn render_2d_pipeline(&self) -> Option<&dyn Render2dPipeline> {
        None
    }
}
