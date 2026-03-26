//! Source abstraction for runtime asset loading.
//!
//! Type definitions, traits, and helper functions for the asset source pipeline.
//! Concrete loaders (e.g. ModAssetSourceLoader) live in the engine crate.

use std::sync::Arc;
use crate::asset_cache::AssetCache;

/// Supported runtime source categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    ModAsset,
}

/// Normalized reference to some loadable source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    kind: SourceKind,
    value: String,
}

impl SourceRef {
    /// Builds a mod-local asset reference such as `/assets/images/tux.png`.
    pub fn mod_asset(value: impl Into<String>) -> Self {
        let raw = value.into();
        let normalized = if raw.starts_with('/') {
            raw
        } else {
            format!("/{raw}")
        };
        Self {
            kind: SourceKind::ModAsset,
            value: normalized,
        }
    }

    pub fn kind(&self) -> &SourceKind {
        &self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn normalized_value(&self) -> &str {
        self.value.trim_start_matches('/')
    }
}

/// Loads raw bytes for a given source reference.
pub trait SourceLoader {
    fn read_bytes(&self, source: &SourceRef) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
    fn has_source(&self, source: &SourceRef) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    fn cache_key(&self, source: &SourceRef) -> String;
}

/// Decodes typed assets from raw bytes and optional nested source lookups.
pub trait SourceAdapter<T> {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        loader: &dyn SourceLoader,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

static SOURCE_BYTES_CACHE: AssetCache<Vec<u8>> = AssetCache::new();

/// Loads raw bytes with a shared source cache.
pub fn load_source_bytes(loader: &impl SourceLoader, source: &SourceRef) -> Option<Arc<Vec<u8>>> {
    let key = loader.cache_key(source);
    SOURCE_BYTES_CACHE.get_or_load(key, || loader.read_bytes(source).ok())
}

/// Returns `true` when the source exists according to the concrete loader.
pub fn has_source(loader: &impl SourceLoader, source: &SourceRef) -> bool {
    loader.has_source(source).unwrap_or(false)
}

/// Loads and decodes a typed asset with a caller-owned typed cache.
pub fn load_decoded_source<T>(
    cache: &AssetCache<T>,
    loader: &impl SourceLoader,
    source: &SourceRef,
    adapter: &impl SourceAdapter<T>,
) -> Option<Arc<T>> {
    let key = loader.cache_key(source);
    cache.get_or_load(key, || {
        let bytes = load_source_bytes(loader, source)?;
        adapter.decode(source, &bytes, loader).ok()
    })
}
