use crate::document::{RenderScene3dDocument, Viewport3dSpriteDocument};
use crate::validate::{validate_render_scene3d_document, Render3dDiagnostic};
use engine_core::render_types::{SpriteRef, Viewport3DRef};
use engine_core::scene::{Scene, Sprite};

/// Compiles 3D viewport references for intermediate render scene construction.
pub fn compile_3d_viewports(scene: &Scene) -> Vec<Viewport3DRef> {
    compile_3d_viewports_with_authored(scene, None)
}

/// Compiles 3D viewport references, using a two-mode strategy:
///
/// **Primary (authored):** if the scene YAML contains an explicit `viewports-3d:` block that
/// passes validation (no duplicate sprite refs), those entries are used directly. This is the
/// preferred path for scenes that explicitly declare their 3D viewport layout.
///
/// **Fallback (derived):** if no valid authored block is present, the compiled `Scene` is walked
/// to collect every `Sprite::Obj`, `Sprite::Planet`, and `Sprite::Scene3D` sprite and derive a
/// `Viewport3DRef` for each one. This is intentional — it lets simple scenes declare 3D sprites
/// without needing a hand-authored `viewports-3d:` section.
///
/// These are two modes of a **single** compilation path, not a dual path. `SceneDocument::compile`
/// only produces a `Scene` (runtime sprite tree); `Viewport3DRef` values are produced exclusively
/// here in `compile_3d`.
pub(super) fn compile_3d_viewports_with_authored(
    scene: &Scene,
    authored: Option<&RenderScene3dDocument>,
) -> Vec<Viewport3DRef> {
    if let Some(document) = authored {
        if should_compile_viewports_from_document(document) {
            return compile_3d_viewports_from_document(document);
        }
    }
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

fn should_compile_viewports_from_document(document: &RenderScene3dDocument) -> bool {
    if document.viewports_3d.is_empty() {
        return false;
    }
    !validate_render_scene3d_document(document)
        .iter()
        .any(|diagnostic| {
            matches!(
                diagnostic,
                Render3dDiagnostic::DuplicateViewportSpriteRef { .. }
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{compile_3d_viewports, compile_3d_viewports_with_authored};
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

    #[test]
    fn authored_viewports_override_scene_derived_viewports_when_present() {
        let raw = r#"
id: override-viewports
title: Override
layers:
  - name: world
    sprites:
      - type: obj
        id: mesh-view
        source: /assets/3d/sphere.obj
"#;
        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("scene compile");
        let authored: RenderScene3dDocument = serde_yaml::from_str(
            r#"
viewports-3d:
  - id: authored-only
    layer_index: 9
    sprite_path: [4, 2]
    sprite_id: authored-only
"#,
        )
        .expect("document parse");

        let viewports = compile_3d_viewports_with_authored(&scene, Some(&authored));
        assert_eq!(viewports.len(), 1);
        assert_eq!(viewports[0].id.as_deref(), Some("authored-only"));
        assert_eq!(viewports[0].sprite.layer_index, 9);
        assert_eq!(viewports[0].sprite.sprite_path, vec![4, 2]);
    }

    #[test]
    fn duplicate_authored_viewports_fall_back_to_scene_derived_viewports() {
        let raw = r#"
id: fallback-viewports
title: Fallback
layers:
  - name: world
    sprites:
      - type: obj
        id: mesh-view
        source: /assets/3d/sphere.obj
"#;
        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("scene compile");
        let authored: RenderScene3dDocument = serde_yaml::from_str(
            r#"
viewports-3d:
  - id: one
    layer_index: 0
    sprite_path: [1]
  - id: two
    layer_index: 0
    sprite_path: [1]
"#,
        )
        .expect("document parse");

        let viewports = compile_3d_viewports_with_authored(&scene, Some(&authored));
        assert_eq!(viewports.len(), 1);
        assert_eq!(viewports[0].id.as_deref(), Some("mesh-view"));
        assert_eq!(viewports[0].sprite.layer_index, 0);
        assert_eq!(viewports[0].sprite.sprite_path, vec![0]);
    }
}
