use super::cutscene::CutsceneFilterRegistry;
use super::scene::{
    compile_scene_document_with_loader, compile_scene_document_with_loader_and_source,
    compile_scene_document_with_loader_and_source_and_filters,
};
use engine_core::render_types::{Layer2DRef, RenderScene, SpriteRef, Viewport3DRef};
use engine_core::scene::{Scene, Sprite};

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
    let runtime_scene = compile_scene_document_with_loader(content, object_loader)?;
    Ok(CompiledRenderScene {
        render_scene: build_render_scene(&runtime_scene),
        runtime_scene,
    })
}

pub fn compile_render_scene_document_with_loader_and_source<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<CompiledRenderScene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let runtime_scene =
        compile_scene_document_with_loader_and_source(content, scene_source_path, object_loader)?;
    Ok(CompiledRenderScene {
        render_scene: build_render_scene(&runtime_scene),
        runtime_scene,
    })
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
    let runtime_scene = compile_scene_document_with_loader_and_source_and_filters(
        content,
        scene_source_path,
        object_loader,
        cutscene_filters,
    )?;
    Ok(CompiledRenderScene {
        render_scene: build_render_scene(&runtime_scene),
        runtime_scene,
    })
}

fn build_render_scene(scene: &Scene) -> RenderScene {
    let mut layers_2d = Vec::new();
    let mut viewports_3d = Vec::new();

    for (layer_index, layer) in scene.layers.iter().enumerate() {
        if layer_has_2d_content(&layer.sprites) {
            layers_2d.push(Layer2DRef { layer_index });
        }
        collect_3d_sprites(
            &layer.sprites,
            layer_index,
            &mut Vec::new(),
            &mut viewports_3d,
        );
    }

    RenderScene {
        layers_2d,
        viewports_3d,
    }
}

fn layer_has_2d_content(sprites: &[Sprite]) -> bool {
    for sprite in sprites {
        match sprite {
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. } => return true,
            Sprite::Obj { .. } | Sprite::Planet { .. } | Sprite::Scene3D { .. } => {}
        }
    }
    false
}

fn collect_3d_sprites(
    sprites: &[Sprite],
    layer_index: usize,
    path: &mut Vec<usize>,
    out: &mut Vec<Viewport3DRef>,
) {
    for (sprite_index, sprite) in sprites.iter().enumerate() {
        path.push(sprite_index);
        match sprite {
            Sprite::Obj { .. } | Sprite::Planet { .. } | Sprite::Scene3D { .. } => {
                out.push(Viewport3DRef {
                    id: sprite.id().map(str::to_string),
                    sprite: SpriteRef {
                        layer_index,
                        sprite_path: path.clone(),
                        sprite_id: sprite.id().map(str::to_string),
                    },
                });
            }
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                collect_3d_sprites(children, layer_index, path, out);
            }
            Sprite::Text { .. } | Sprite::Image { .. } | Sprite::Vector { .. } => {}
        }
        path.pop();
    }
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
        src: /assets/3d/demo.scene3d.yml
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
}
