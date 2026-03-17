//! Lightweight mod manifest summary deserialized from `mod.yaml`.

use serde::Deserialize;

/// Parsed summary of the top-level `mod.yaml` manifest fields.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModManifestSummary {
    pub name: Option<String>,
    pub version: Option<String>,
    pub entrypoint: Option<String>,
}
