use crate::document::{RenderScene3dDocument, Viewport3dSpriteDocument};
use engine_core::render_types::{SpriteRef, Viewport3DRef};
use engine_core::scene::{Scene, Sprite};

/// Compiles 3D viewport references for intermediate render scene construction.
pub fn compile_3d_viewports(scene: &Scene) -> Vec<Viewport3DRef> {
    let render_scene_3d = build_render_scene3d_document(scene);
    compile_3d_viewports_from_document(&render_scene_3d)
}

fn collect_3d_sprites(
    sprites: &[Sprite],
    layer_index: usize,
    path: &mut Vec<usize>,
    out: &mut Vec<Viewport3dSpriteDocument>,
) {
    for (sprite_index, sprite) in sprites.iter().enumerate() {
        path.push(sprite_index);
        match sprite {
            Sprite::Obj { .. } | Sprite::Planet { .. } | Sprite::Scene3D { .. } => {
                out.push(Viewport3dSpriteDocument {
                    id: sprite.id().map(str::to_string),
                    layer_index,
                    sprite_path: path.clone(),
                    sprite_id: sprite.id().map(str::to_string),
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

fn build_render_scene3d_document(scene: &Scene) -> RenderScene3dDocument {
    let mut viewports_3d = Vec::new();
    for (layer_index, layer) in scene.layers.iter().enumerate() {
        collect_3d_sprites(
            &layer.sprites,
            layer_index,
            &mut Vec::new(),
            &mut viewports_3d,
        );
    }

    RenderScene3dDocument {
        viewports_3d,
        ..RenderScene3dDocument::default()
    }
}

fn compile_3d_viewports_from_document(document: &RenderScene3dDocument) -> Vec<Viewport3DRef> {
    document
        .viewports_3d
        .iter()
        .map(|viewport| Viewport3DRef {
            id: viewport.id.clone(),
            sprite: SpriteRef {
                layer_index: viewport.layer_index,
                sprite_path: viewport.sprite_path.clone(),
                sprite_id: viewport.sprite_id.clone(),
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::compile_3d_viewports;
    use crate::compile::compile_scene_document_with_loader_and_source;
    use crate::document::RenderScene3dDocument;

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

    #[test]
    fn compiles_viewports_from_render_scene3d_document_entries() {
        let document: RenderScene3dDocument = serde_yaml::from_str(
            r#"
viewports-3d:
  - id: viewport-a
    layer_index: 2
    sprite_path: [1, 3]
    sprite_id: sprite-a
"#,
        )
        .expect("document parse");

        let viewports = super::compile_3d_viewports_from_document(&document);
        assert_eq!(viewports.len(), 1);
        assert_eq!(viewports[0].id.as_deref(), Some("viewport-a"));
        assert_eq!(viewports[0].sprite.layer_index, 2);
        assert_eq!(viewports[0].sprite.sprite_path, vec![1, 3]);
        assert_eq!(viewports[0].sprite.sprite_id.as_deref(), Some("sprite-a"));
    }
}
