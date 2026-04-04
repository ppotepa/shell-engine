//! Lightweight parsed fields from `mod.yaml` registered into the engine world
//! so systems can read them without re-parsing the manifest.

/// Manifest fields that need to be accessible from engine systems at runtime.
#[derive(Debug, Clone, Default)]
pub struct ModManifestData {
    /// The `default_palette:` field from `mod.yaml`, if present.
    pub default_palette: Option<String>,
}

impl ModManifestData {
    pub fn from_manifest(manifest: &serde_yaml::Value) -> Self {
        let default_palette = manifest
            .get("default_palette")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Self { default_palette }
    }
}
