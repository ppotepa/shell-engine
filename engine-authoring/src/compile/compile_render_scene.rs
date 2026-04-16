use super::cutscene::CutsceneFilterRegistry;
pub use super::render_scene::build_render_scene_from_scene;
use super::render_scene::{
    compile_render_scene_document_with_loader_and_source as compile_render_scene_document_with_loader_and_source_impl,
    compile_render_scene_document_with_loader_and_source_and_filters as compile_render_scene_document_with_loader_and_source_and_filters_impl,
};
use super::CompiledRenderScene;

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
    use super::{build_render_scene_from_scene, compile_render_scene_document};
    use crate::compile::{
        compile_render_scene_document_with_loader_and_source,
        compile_scene_document_with_loader_and_source,
    };
    use engine_core::render_types::{SpriteRef, Viewport3DRef};

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
      - type: planet
        id: planet
        body-id: earth
      - type: scene3_d
        id: clip
        src: /assets/3d/clip.scene3d.yml
        frame: idle
"#;

        let from_existing =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("compile render scene");
        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("compile scene");
        let from_builder = build_render_scene_from_scene(&scene);

        assert_eq!(from_existing.render_scene, from_builder);
    }

    #[test]
    fn wrapper_compile_path_keeps_authored_3d_forms_on_single_intermediate_builder() {
        let raw = r#"
id: wrapper-3d
title: Wrapper 3D
layers:
  - name: world
    sprites:
      - type: obj
        id: mesh-view
        source: /assets/3d/sphere.obj
      - type: panel
        children:
          - type: planet
            id: planet-view
            body-id: earth
          - type: scene3_d
            id: cinematic-view
            src: /assets/3d/intro.scene3d.yml
            frame: idle
"#;

        let from_wrapper =
            compile_render_scene_document(raw, "test/scene.yml", |_| None).expect("wrapper path");
        let from_loader =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("loader path");

        assert_eq!(from_wrapper.render_scene, from_loader.render_scene);
        assert_eq!(
            from_wrapper.render_scene.viewports_3d,
            vec![
                Viewport3DRef {
                    id: Some("mesh-view".to_string()),
                    sprite: SpriteRef {
                        layer_index: 0,
                        sprite_path: vec![0],
                        sprite_id: Some("mesh-view".to_string()),
                    },
                },
                Viewport3DRef {
                    id: Some("planet-view".to_string()),
                    sprite: SpriteRef {
                        layer_index: 0,
                        sprite_path: vec![1, 0],
                        sprite_id: Some("planet-view".to_string()),
                    },
                },
                Viewport3DRef {
                    id: Some("cinematic-view".to_string()),
                    sprite: SpriteRef {
                        layer_index: 0,
                        sprite_path: vec![1, 1],
                        sprite_id: Some("cinematic-view".to_string()),
                    },
                },
            ]
        );
    }
}
