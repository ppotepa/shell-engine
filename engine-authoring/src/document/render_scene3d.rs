use super::{
    AtmosphereProfileDocument, CameraProfileDocument, MaterialDocument, Viewport3dDocument,
    Viewport3dSpriteDocument, WorldProfileDocument,
};
use serde::{Deserialize, Serialize};

/// Authored document scaffold for Scene3D-oriented inputs.
///
/// `viewports_3d` is the preferred authored surface for render-scene 3D
/// viewport references. When this field is absent, authoring derives viewport
/// references from scene sprite traversal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderScene3dDocument {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub viewport: Option<Viewport3dDocument>,
    #[serde(default)]
    pub camera: Option<CameraProfileDocument>,
    #[serde(default)]
    pub world: Option<WorldProfileDocument>,
    #[serde(default)]
    pub atmosphere: Option<AtmosphereProfileDocument>,
    #[serde(default)]
    pub materials: Vec<MaterialDocument>,
    #[serde(default, rename = "viewports-3d", alias = "viewports_3d")]
    pub viewports_3d: Vec<Viewport3dSpriteDocument>,
}

#[cfg(test)]
mod tests {
    use super::RenderScene3dDocument;

    #[test]
    fn deserializes_minimal_render_scene3d_document() {
        let raw = r#"
id: intro-viewport
viewport:
  width: 80
  height: 45
"#;

        let document: RenderScene3dDocument =
            serde_yaml::from_str(raw).expect("render scene 3d document");
        assert_eq!(document.id.as_deref(), Some("intro-viewport"));
        assert_eq!(document.viewport.as_ref().and_then(|v| v.width), Some(80));
        assert_eq!(document.viewport.as_ref().and_then(|v| v.height), Some(45));
    }

    #[test]
    fn deserializes_viewports_3d_entries() {
        let raw = r#"
id: scene-3d
viewports-3d:
  - id: main
    layer_index: 1
    sprite_path: [0, 2]
    sprite_id: main
"#;

        let document: RenderScene3dDocument =
            serde_yaml::from_str(raw).expect("render scene 3d document");
        assert_eq!(document.viewports_3d.len(), 1);
        assert_eq!(document.viewports_3d[0].id.as_deref(), Some("main"));
        assert_eq!(document.viewports_3d[0].layer_index, 1);
        assert_eq!(document.viewports_3d[0].sprite_path, vec![0, 2]);
        assert_eq!(document.viewports_3d[0].sprite_id.as_deref(), Some("main"));
    }
}
