use crate::document::RenderScene3dDocument;

/// Validation diagnostics for Scene3D-oriented authored documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Render3dDiagnostic {
    /// Viewport block exists but one or both dimensions are absent.
    IncompleteViewportDimensions,
    /// Multiple materials share the same id.
    DuplicateMaterialId { id: String },
    /// Multiple viewport entries point to the same sprite path.
    DuplicateViewportSpriteRef {
        layer_index: usize,
        sprite_path: Vec<usize>,
    },
}

/// Validates authored RenderScene3D scaffolding.
pub fn validate_render_scene3d_document(doc: &RenderScene3dDocument) -> Vec<Render3dDiagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(viewport) = &doc.viewport {
        if viewport.width.is_none() || viewport.height.is_none() {
            diagnostics.push(Render3dDiagnostic::IncompleteViewportDimensions);
        }
    }

    let mut ids = std::collections::BTreeSet::<String>::new();
    for material in &doc.materials {
        if !ids.insert(material.id.clone()) {
            diagnostics.push(Render3dDiagnostic::DuplicateMaterialId {
                id: material.id.clone(),
            });
        }
    }

    let mut sprite_refs = std::collections::BTreeSet::<(usize, Vec<usize>)>::new();
    for viewport in &doc.viewports_3d {
        let key = (viewport.layer_index, viewport.sprite_path.clone());
        if !sprite_refs.insert(key.clone()) {
            diagnostics.push(Render3dDiagnostic::DuplicateViewportSpriteRef {
                layer_index: key.0,
                sprite_path: key.1,
            });
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::{validate_render_scene3d_document, Render3dDiagnostic};
    use crate::document::RenderScene3dDocument;

    #[test]
    fn flags_incomplete_viewport_dimensions() {
        let raw = r#"
viewport:
  width: 80
"#;
        let doc: RenderScene3dDocument = serde_yaml::from_str(raw).expect("document");
        let diagnostics = validate_render_scene3d_document(&doc);
        assert_eq!(
            diagnostics,
            vec![Render3dDiagnostic::IncompleteViewportDimensions]
        );
    }

    #[test]
    fn flags_duplicate_material_ids() {
        let raw = r#"
materials:
  - id: hull
  - id: hull
"#;
        let doc: RenderScene3dDocument = serde_yaml::from_str(raw).expect("document");
        let diagnostics = validate_render_scene3d_document(&doc);
        assert_eq!(
            diagnostics,
            vec![Render3dDiagnostic::DuplicateMaterialId {
                id: "hull".to_string()
            }]
        );
    }

    #[test]
    fn flags_duplicate_viewport_sprite_refs() {
        let raw = r#"
viewports-3d:
  - id: one
    layer_index: 0
    sprite_path: [1, 2]
  - id: two
    layer_index: 0
    sprite_path: [1, 2]
"#;
        let doc: RenderScene3dDocument = serde_yaml::from_str(raw).expect("document");
        let diagnostics = validate_render_scene3d_document(&doc);
        assert_eq!(
            diagnostics,
            vec![Render3dDiagnostic::DuplicateViewportSpriteRef {
                layer_index: 0,
                sprite_path: vec![1, 2]
            }]
        );
    }
}
