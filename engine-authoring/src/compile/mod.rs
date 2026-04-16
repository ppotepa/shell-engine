//! Authored-to-runtime compilation pipeline.
//!
//! This module will normalize aliases, expand templates/objects, and compile
//! authored YAML documents into `engine_core::scene::Scene`.

mod compile_2d;
mod compile_3d;
mod compile_render_scene;
mod cutscene;
mod render_scene;
mod scene;

pub use compile_2d::compile_2d_layers;
pub use compile_3d::compile_3d_viewports;
pub use compile_render_scene::{
    build_render_scene_from_scene, compile_render_scene_document,
    compile_render_scene_document_with_filters,
};
pub use cutscene::{
    CutsceneCompileFilter, CutsceneCompileFrame, CutsceneFilterRegistry, DurationScaleFilter,
};
pub use render_scene::{
    compile_render_scene_document_with_loader,
    compile_render_scene_document_with_loader_and_source,
    compile_render_scene_document_with_loader_and_source_and_filters, CompiledRenderScene,
};
pub use scene::{
    compile_scene_document_with_loader, compile_scene_document_with_loader_and_source,
    compile_scene_document_with_loader_and_source_and_filters,
};
