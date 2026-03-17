//! Compilation helpers that turn authored scene YAML into the runtime `Scene`
//! model, including object expansion before typed deserialization.

use crate::scene::Scene;

/// Compiles authored scene YAML into a runtime [`Scene`].
///
/// # Purpose
///
/// This is the authored-scene entry point used by repositories after they have
/// assembled any scene package fragments. It expands `objects:` references,
/// merges authored overrides from `with:`, and then hands the normalized YAML to
/// [`SceneDocument`] for the final authored-to-runtime conversion.
///
/// `scene_source_path` is used to resolve relative object references inside a
/// scene package.
pub fn compile_scene_document_with_loader_and_source<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    engine_authoring::compile::compile_scene_document_with_loader_and_source(
        content,
        scene_source_path,
        object_loader,
    )
}
