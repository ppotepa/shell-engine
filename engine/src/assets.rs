use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AssetRoot {
    mod_source: PathBuf,
}

impl AssetRoot {
    pub fn new(mod_source: PathBuf) -> Self {
        Self { mod_source }
    }

    pub fn mod_source(&self) -> &Path {
        &self.mod_source
    }

    pub fn resolve(&self, asset_path: &str) -> PathBuf {
        resolve_asset_path(&self.mod_source, asset_path)
    }
}

pub fn resolve_asset_path(mod_source: &Path, asset_path: &str) -> PathBuf {
    mod_source.join(asset_path.trim_start_matches('/'))
}
