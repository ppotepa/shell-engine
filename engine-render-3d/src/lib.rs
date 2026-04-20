pub mod api;
pub mod effects;
pub mod frame_input;
pub mod frame_profiles;
pub mod geom;
pub mod mesh;
pub mod obj_render_params;
pub mod pipeline;
pub mod prerender;
pub mod raster;
pub mod scene;
pub mod shading;

pub use effects::passes::context::RenderPassContext;
pub use frame_input::Render3dFrameInput;
pub use frame_profiles::{
    FrameAtmosphereProfile, FrameEnvironmentProfile, FrameGeometry3D, FrameLightingProfile,
    FramePointLightProfile, FramePostProcessProfile, FrameSurfaceProfile,
};
pub use obj_render_params::ObjRenderParams;
pub use pipeline::{
    prepare_render3d_item, render_prepared_render3d_item_to_buffer, GeneratedWorldFrameProducer,
    MeshFrameProducer, PreparedRender3dItem, PreparedRender3dRuntime, PreparedRender3dSource,
    Render3dProducer, SceneClipFrameProducer,
};
pub use raster::{
    blit_color_canvas, blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba,
    obj_sprite_dimensions, render_obj_content, render_obj_to_canvas, render_obj_to_rgba_canvas,
    render_obj_to_shared_buffers, reset_obj_raster_frame_metrics, take_obj_raster_frame_metrics,
    try_blit_prerendered, virtual_dimensions, virtual_dimensions_multiplier,
};
