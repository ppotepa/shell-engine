use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModManifestSummary {
    pub name: Option<String>,
    pub version: Option<String>,
    pub entrypoint: Option<String>,
}
