use super::cutscene::CutsceneFilterRegistry;
use super::render_scene::{
    compile_render_scene_document_with_loader_and_source as compile_render_scene_document_with_loader_and_source_impl,
    compile_render_scene_document_with_loader_and_source_and_filters as compile_render_scene_document_with_loader_and_source_and_filters_impl,
};
use super::{compile_2d_layers, compile_3d_viewports, CompiledRenderScene};
use engine_core::render_types::RenderScene;
use engine_core::scene::Scene;

/// Builds a render-scene view from an already compiled runtime scene.
pub fn build_render_scene_from_scene(scene: &Scene) -> RenderScene {
    RenderScene {
        layers_2d: compile_2d_layers(scene),
        viewports_3d: compile_3d_viewports(scene),
    }
}

/// Compiles authored scene YAML into runtime and render-scene forms.
pub fn compile_render_scene_document<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    compile_render_scene_document_with_loader_and_source_impl(
        content,
        scene_source_path,
        object_loader,
    )
}

/// Same as [`compile_render_scene_document`] but allows custom cutscene filters.
pub fn compile_render_scene_document_with_filters<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
    cutscene_filters: &CutsceneFilterRegistry,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    compile_render_scene_document_with_loader_and_source_and_filters_impl(
        content,
        scene_source_path,
        object_loader,
        cutscene_filters,
    )
}

#[cfg(test)]
mod tests {
    use super::build_render_scene_from_scene;
    use crate::compile::{
        compile_render_scene_document_with_loader_and_source,
        compile_scene_document_with_loader_and_source,
    };

    #[test]
    fn render_scene_builder_matches_existing_compile_path() {
        let raw = r#"
id: parity
title: Parity
layers:
  - name: mixed
    sprites:
      - type: text
        content: HELLO
      - type: obj
        id: mesh
        source: /assets/3d/cube.obj
"#;

        let from_existing =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("compile render scene");
        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("compile scene");
        let from_builder = build_render_scene_from_scene(&scene);

        assert_eq!(from_existing.render_scene, from_builder);
    }
}
