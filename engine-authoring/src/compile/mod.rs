//! Authored-to-runtime compilation pipeline.
//!
//! This module will normalize aliases, expand templates/objects, and compile
//! authored YAML documents into `engine_core::scene::Scene`.

mod cutscene;
mod scene;

pub use cutscene::{
    CutsceneCompileFilter, CutsceneCompileFrame, CutsceneFilterRegistry, DurationScaleFilter,
};
pub use scene::{
    compile_scene_document_with_loader, compile_scene_document_with_loader_and_source,
    compile_scene_document_with_loader_and_source_and_filters,
};
