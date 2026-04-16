use super::cutscene::CutsceneFilterRegistry;
use super::scene::{
    compile_scene_and_authoring_input_with_loader_and_source_and_filters,
    CompiledSceneAuthoringInput,
};
use super::{compile_2d_layers, compile_3d::compile_3d_viewports_with_authored};
use crate::document::RenderScene3dDocument;
use engine_core::render_types::RenderScene;
use engine_core::scene::Scene;

#[derive(Debug, Clone)]
pub struct CompiledRenderScene {
    pub runtime_scene: Scene,
    pub render_scene: RenderScene,
}

#[allow(dead_code)]
pub fn compile_render_scene_document_with_loader<F>(
    content: &str,
    object_loader: F,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let filters = CutsceneFilterRegistry::with_builtin_filters();
    compile_render_scene_with_source_and_filters(content, "/", object_loader, &filters)
}

pub fn compile_render_scene_document_with_loader_and_source<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let filters = CutsceneFilterRegistry::with_builtin_filters();
    compile_render_scene_with_source_and_filters(
        content,
        scene_source_path,
        object_loader,
        &filters,
    )
}

pub fn compile_render_scene_document_with_loader_and_source_and_filters<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
    cutscene_filters: &CutsceneFilterRegistry,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    compile_render_scene_with_source_and_filters(
        content,
        scene_source_path,
        object_loader,
        cutscene_filters,
    )
}

fn compile_render_scene_with_source_and_filters<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
    cutscene_filters: &CutsceneFilterRegistry,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let compiled = compile_scene_and_authoring_input_with_loader_and_source_and_filters(
        content,
        scene_source_path,
        object_loader,
        cutscene_filters,
    )?;
    Ok(build_compiled_render_scene(compiled))
}

fn build_compiled_render_scene(compiled: CompiledSceneAuthoringInput) -> CompiledRenderScene {
    let CompiledSceneAuthoringInput {
        scene,
        authored_render_scene_3d,
    } = compiled;
    let render_scene =
        build_render_scene_from_scene_with_authored(&scene, authored_render_scene_3d.as_ref());
    CompiledRenderScene {
        runtime_scene: scene,
        render_scene,
    }
}

fn build_render_scene_from_scene_with_authored(
    scene: &Scene,
    authored_3d: Option<&RenderScene3dDocument>,
) -> RenderScene {
    RenderScene {
        layers_2d: compile_2d_layers(scene),
        viewports_3d: compile_3d_viewports_with_authored(scene, authored_3d),
    }
}

pub fn build_render_scene_from_scene(scene: &Scene) -> RenderScene {
    build_render_scene_from_scene_with_authored(scene, None)
}

#[cfg(test)]
mod tests {
    use super::compile_render_scene_document_with_loader_and_source;
    use engine_core::render_types::{Layer2DRef, SpriteRef, Viewport3DRef};

    #[test]
    fn compiles_intermediate_render_scene_with_2d_layers_and_3d_viewports() {
        let raw = r#"
id: mixed-render-scene
title: Mixed Render Scene
layers:
  - name: mixed
    sprites:
      - type: text
        id: title
        content: "HELLO"
      - type: obj
        id: mesh-view
        source: /assets/3d/sphere.obj
  - name: scene-only
    sprites:
      - type: scene3_d
        id: clip-view
        src: /assets/3d/sample.scene3d.yml
        frame: idle
"#;
        let compiled =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("render scene should compile");

        assert_eq!(compiled.runtime_scene.id, "mixed-render-scene");
        assert_eq!(
            compiled.render_scene.layers_2d,
            vec![Layer2DRef { layer_index: 0 }]
        );
        assert_eq!(
            compiled.render_scene.viewports_3d,
            vec![
                Viewport3DRef {
                    id: Some("mesh-view".to_string()),
                    sprite: SpriteRef {
                        layer_index: 0,
                        sprite_path: vec![1],
                        sprite_id: Some("mesh-view".to_string()),
                    },
                },
                Viewport3DRef {
                    id: Some("clip-view".to_string()),
                    sprite: SpriteRef {
                        layer_index: 1,
                        sprite_path: vec![0],
                        sprite_id: Some("clip-view".to_string()),
                    },
                },
            ]
        );
    }

    #[test]
    fn compiles_authored_obj_planet_and_scene3d_sprites_into_viewports() {
        let raw = r#"
id: authored-3d
title: Authored 3D
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

        let compiled =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("render scene should compile");

        assert_eq!(
            compiled.render_scene.viewports_3d,
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

    #[test]
    fn uses_authored_viewports_3d_when_declared() {
        let raw = r#"
id: authored-viewports
title: Authored Viewports
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: HELLO
viewports-3d:
  - id: authored-camera
    layer_index: 0
    sprite_path: [0]
    sprite_id: authored-camera
"#;

        let compiled =
            compile_render_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
                .expect("render scene should compile");

        assert_eq!(
            compiled.render_scene.viewports_3d,
            vec![Viewport3DRef {
                id: Some("authored-camera".to_string()),
                sprite: SpriteRef {
                    layer_index: 0,
                    sprite_path: vec![0],
                    sprite_id: Some("authored-camera".to_string()),
                },
            }]
        );
    }
}
