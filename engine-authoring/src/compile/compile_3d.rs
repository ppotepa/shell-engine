use engine_core::render_types::{SpriteRef, Viewport3DRef};
use engine_core::scene::{Scene, Sprite};

/// Compiles 3D viewport references for intermediate render scene construction.
pub fn compile_3d_viewports(scene: &Scene) -> Vec<Viewport3DRef> {
    let mut viewports_3d = Vec::new();
    for (layer_index, layer) in scene.layers.iter().enumerate() {
        collect_3d_sprites(
            &layer.sprites,
            layer_index,
            &mut Vec::new(),
            &mut viewports_3d,
        );
    }
    viewports_3d
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
    use super::compile_3d_viewports;
    use crate::compile::compile_scene_document_with_loader_and_source;

    #[test]
    fn collects_nested_3d_viewports() {
        let raw = r#"
id: nested-3d
title: Nested 3D
layers:
  - name: world
    sprites:
      - type: panel
        children:
          - type: scene3_d
            id: clip
            src: /assets/3d/clip.scene3d.yml
            frame: idle
"#;

        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("scene compile");
        let viewports = compile_3d_viewports(&scene);
        assert_eq!(viewports.len(), 1);
        assert_eq!(viewports[0].id.as_deref(), Some("clip"));
        assert_eq!(viewports[0].sprite.layer_index, 0);
        assert_eq!(viewports[0].sprite.sprite_path, vec![0, 0]);
    }
}
