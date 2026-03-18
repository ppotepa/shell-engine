//! Source abstraction for runtime asset loading.
//!
//! This isolates "where bytes come from" from "how bytes are decoded" so the
//! current mod-asset pipeline can later grow URL/generated adapters without
//! rewriting image/mesh/font consumers.

use std::path::{Path, PathBuf};

use crate::asset_cache::AssetCache;
use crate::repositories::{create_asset_repository, AnyAssetRepository, AssetRepository};
use crate::EngineError;

/// Supported runtime source categories.
///
/// Today only mod-local assets are enabled, but the enum provides the stable
/// expansion point for future URL/generated adapters.
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
    fn read_bytes(&self, source: &SourceRef) -> Result<Vec<u8>, EngineError>;
    fn has_source(&self, source: &SourceRef) -> Result<bool, EngineError>;
    fn cache_key(&self, source: &SourceRef) -> String;
}

/// Decodes typed assets from raw bytes and optional nested source lookups.
pub trait SourceAdapter<T> {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        loader: &dyn SourceLoader,
    ) -> Result<T, EngineError>;
}

/// Current concrete source loader backed by the mod directory/zip repository.
#[derive(Debug, Clone)]
pub struct ModAssetSourceLoader {
    mod_source: PathBuf,
    repo: AnyAssetRepository,
}

impl ModAssetSourceLoader {
    pub fn new(mod_source: &Path) -> Result<Self, EngineError> {
        Ok(Self {
            mod_source: mod_source.to_path_buf(),
            repo: create_asset_repository(mod_source)?,
        })
    }
}

impl SourceLoader for ModAssetSourceLoader {
    fn read_bytes(&self, source: &SourceRef) -> Result<Vec<u8>, EngineError> {
        match source.kind() {
            SourceKind::ModAsset => self.repo.read_asset_bytes(source.value()),
        }
    }

    fn has_source(&self, source: &SourceRef) -> Result<bool, EngineError> {
        match source.kind() {
            SourceKind::ModAsset => self.repo.has_asset(source.value()),
        }
    }

    fn cache_key(&self, source: &SourceRef) -> String {
        format!(
            "{}::{}",
            self.mod_source.display(),
            source.normalized_value()
        )
    }
}

static SOURCE_BYTES_CACHE: AssetCache<Vec<u8>> = AssetCache::new();

/// Loads raw bytes with a shared source cache.
pub fn load_source_bytes(loader: &impl SourceLoader, source: &SourceRef) -> Option<Vec<u8>> {
    let key = loader.cache_key(source);
    SOURCE_BYTES_CACHE.get_or_load(key, || loader.read_bytes(source).ok())
}

/// Returns `true` when the source exists according to the concrete loader.
pub fn has_source(loader: &impl SourceLoader, source: &SourceRef) -> bool {
    loader.has_source(source).unwrap_or(false)
}

/// Loads and decodes a typed asset with a caller-owned typed cache.
pub fn load_decoded_source<T: Clone>(
    cache: &AssetCache<T>,
    loader: &impl SourceLoader,
    source: &SourceRef,
    adapter: &impl SourceAdapter<T>,
) -> Option<T> {
    let key = loader.cache_key(source);
    cache.get_or_load(key, || {
        let bytes = load_source_bytes(loader, source)?;
        adapter.decode(source, &bytes, loader).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::{has_source, load_source_bytes, ModAssetSourceLoader, SourceRef};
    use std::fs;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn normalizes_mod_asset_refs() {
        let source = SourceRef::mod_asset("assets/images/logo.png");
        assert_eq!(source.value(), "/assets/images/logo.png");
        assert_eq!(source.normalized_value(), "assets/images/logo.png");
    }

    #[test]
    fn loads_bytes_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/images")).expect("images dir");
        fs::write(mod_dir.join("assets/images/tiny.bin"), b"abc").expect("write bytes");

        let loader = ModAssetSourceLoader::new(&mod_dir).expect("loader");
        let source = SourceRef::mod_asset("/assets/images/tiny.bin");
        assert!(has_source(&loader, &source));
        assert_eq!(load_source_bytes(&loader, &source), Some(b"abc".to_vec()));
    }

    #[test]
    fn loads_bytes_from_zip_source() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("zip file");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("assets/images/tiny.bin", SimpleFileOptions::default())
            .expect("zip entry");
        std::io::Write::write_all(&mut writer, b"abc").expect("write");
        writer.finish().expect("finish zip");

        let loader = ModAssetSourceLoader::new(&zip_path).expect("loader");
        let source = SourceRef::mod_asset("/assets/images/tiny.bin");
        assert!(has_source(&loader, &source));
        assert_eq!(load_source_bytes(&loader, &source), Some(b"abc".to_vec()));
    }
}
