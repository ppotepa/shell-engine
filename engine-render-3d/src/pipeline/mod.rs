pub mod generated_world_profile;
pub mod generated_world_renderer;
pub mod generated_world_sprite_renderer;
pub mod generated_world_sprite_spec;
pub mod obj_sprite_renderer;
pub mod obj_sprite_spec;
pub mod render3d_sprite_spec;
pub mod renderer;
pub mod scene_clip_renderer;
pub mod scene_clip_sprite_spec;
pub mod sprite_mapping;
pub mod stages;
pub mod view_lighting;
pub mod world_lod;

pub use generated_world_profile::{
    build_generated_world_render_profile, generated_world_atmosphere_visibility,
};
pub use generated_world_renderer::{
    render_generated_world_sprite_with, reset_generated_world_pass_metrics,
    take_generated_world_pass_metrics, BlitRgbaCanvasFn, CompositeRgbaOverFn,
    GeneratedWorldPassMetrics, GeneratedWorldRenderCallbacks, GeneratedWorldRenderProfile,
    RenderObjToRgbaCanvasFn,
};
pub use generated_world_sprite_renderer::{
    render_generated_world_sprite_to_buffer, GeneratedWorldSpriteRenderRuntime,
};
pub use generated_world_sprite_spec::{
    extract_generated_world_sprite_spec, GeneratedWorldSpriteSpec,
};
pub use obj_sprite_renderer::{
    render_obj_sprite_to_buffer, ObjSpriteRenderRuntime, SpriteRenderArea,
};
pub use obj_sprite_spec::{extract_obj_sprite_spec, ObjSpriteSpec};
pub use render3d_sprite_spec::{extract_render3d_sprite_spec, Render3dSpriteSpec};
pub use renderer::{
    render_scene3d_work_item_buffer_with, render_scene3d_work_item_canvas_with, Scene3DColorCanvas,
};
pub use scene_clip_renderer::{render_scene_clip_sprite_to_buffer, SceneClipRenderRuntime};
pub use scene_clip_sprite_spec::{extract_scene_clip_sprite_spec, SceneClipSpriteSpec};
pub use sprite_mapping::map_sprite_to_node3d;
pub use view_lighting::{resolve_view_lighting, ViewLightingParams};
pub use world_lod::apply_world_lod_to_source;
