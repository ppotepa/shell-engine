use engine_core::render_types::Layer2DRef;
use engine_core::scene::{Scene, Sprite};

/// Compiles 2D layer references for intermediate render scene construction.
pub fn compile_2d_layers(scene: &Scene) -> Vec<Layer2DRef> {
    let mut layers_2d = Vec::new();
    for (layer_index, layer) in scene.layers.iter().enumerate() {
        if layer_has_2d_content(&layer.sprites) {
            layers_2d.push(Layer2DRef { layer_index });
        }
    }
    layers_2d
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

#[cfg(test)]
mod tests {
    use super::compile_2d_layers;
    use crate::compile::compile_scene_document_with_loader_and_source;

    #[test]
    fn collects_only_layers_with_2d_sprites() {
        let raw = r#"
id: layer-2d
title: Layer 2D
layers:
  - name: viewport
    sprites:
      - type: obj
        source: /assets/3d/cube.obj
  - name: ui
    sprites:
      - type: text
        content: HELLO
"#;

        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("scene compile");
        let layers = compile_2d_layers(&scene);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].layer_index, 1);
    }
}
