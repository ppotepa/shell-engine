//! Asset resolution helpers — [`AssetRoot`] anchors all asset paths to the mod source directory.

use std::path::{Path, PathBuf};

/// Holds the mod source root used to resolve relative asset paths.
#[derive(Debug, Clone)]
pub struct AssetRoot {
    mod_source: PathBuf,
}

impl AssetRoot {
    /// Creates a new [`AssetRoot`] anchored at `mod_source`.
    pub fn new(mod_source: PathBuf) -> Self {
        Self { mod_source }
    }

    /// Returns the mod source directory this root is anchored to.
    pub fn mod_source(&self) -> &Path {
        &self.mod_source
    }

    /// Joins `asset_path` (stripping a leading `/`) onto the mod source root.
    pub fn resolve(&self, asset_path: &str) -> PathBuf {
        resolve_asset_path(&self.mod_source, asset_path)
    }
}

/// Joins `asset_path` onto `mod_source`, stripping the leading `/` from the asset path.
pub fn resolve_asset_path(mod_source: &Path, asset_path: &str) -> PathBuf {
    mod_source.join(asset_path.trim_start_matches('/'))
}
