use super::{diagnostics::Diagnostics, mod_manifest::ModManifestSummary, scene_index::SceneIndex};

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AssetIndex {
    pub manifest: Option<ModManifestSummary>,
    pub project_yamls: Vec<String>,
    pub is_valid_project: bool,
    pub scenes: SceneIndex,
    pub images: Vec<String>,
    pub fonts: Vec<String>,
    pub effects: Vec<String>,
    pub diagnostics: Diagnostics,
}
