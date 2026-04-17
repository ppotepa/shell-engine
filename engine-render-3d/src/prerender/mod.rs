pub mod camera_basis;
pub mod frame_item;
pub mod frame_schedule;
pub mod lighting;
pub mod obj_prerender;
pub mod object_motion;
pub mod object_specs;
pub mod pipeline;
pub mod render_item;
pub mod runtime_builder;
pub mod runtime_store;
pub mod scene3d_atlas;
pub mod scene3d_prerender;
pub mod scene_sources;
pub mod tween_eval;
pub mod warmup;
pub mod work_items;

pub use camera_basis::look_at_basis;
pub use frame_item::build_scene3d_frame_item_at;
pub use frame_schedule::{clip_progress_at, expand_frame_samples, FrameSample};
pub use lighting::{extract_light_params, parse_hex_color, LightParams};
pub use obj_prerender::{
    prerender_obj_sprites_with, AnimSpriteFrames, ObjPrerenderStatus, ObjPrerenderedFrames,
    ObjSpriteDimensionsFn, PrerenderedCanvas, PrerenderedFrame, RenderObjToCanvasFn,
    YAW_FRAME_COUNT, YAW_STEP_DEG,
};
pub use object_motion::{resolve_object_frame_motion, ObjectFrameMotion};
pub use object_specs::{build_object_specs, ObjectRenderSpec};
pub use render_item::{
    render_scene3d_work_item, render_work_item_buffer_with, render_work_item_canvas_with,
    Scene3DColorCanvas,
};
pub use runtime_builder::build_scene3d_runtime_store;
pub use runtime_store::{with_runtime_store, Scene3DRuntimeEntry, Scene3DRuntimeStore};
pub use scene3d_atlas::{with_atlas, Scene3DAtlas};
pub use scene3d_prerender::{prerender_scene3d_atlas_with, render_scene3d_frame_at_with};
pub use scene_sources::{collect_scene3d_sources, load_and_resolve_scene3d};
pub use tween_eval::{
    evaluate_tween_values, resolve_camera_frame_state, CameraFrameState, TweenValues,
};
pub use warmup::warmup_scene_meshes;
pub use work_items::{build_work_items, Scene3DWorkItem};
