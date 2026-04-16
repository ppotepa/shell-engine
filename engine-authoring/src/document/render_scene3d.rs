use super::{
    AtmosphereProfileDocument, CameraProfileDocument, MaterialDocument, Viewport3dDocument,
    WorldProfileDocument,
};
use serde::{Deserialize, Serialize};

/// Authored document scaffold for Scene3D-oriented inputs.
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
}
