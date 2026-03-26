//! Asset source loading backed by mod repositories.

use std::path::{Path, PathBuf};

use engine_core::asset_source::{SourceKind, SourceLoader, SourceRef};
use engine_error::EngineError;

use crate::{create_asset_repository, AssetRepository};

/// Current concrete source loader backed by the mod directory/zip repository.
#[derive(Debug, Clone)]
pub struct ModAssetSourceLoader {
    mod_source: PathBuf,
    repo: crate::AnyAssetRepository,
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
    fn read_bytes(&self, source: &SourceRef) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        match source.kind() {
            SourceKind::ModAsset => self.repo.read_asset_bytes(source.value()).map_err(|e| Box::new(e) as _),
        }
    }

    fn has_source(&self, source: &SourceRef) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match source.kind() {
            SourceKind::ModAsset => self.repo.has_asset(source.value()).map_err(|e| Box::new(e) as _),
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

#[cfg(test)]
mod tests {
    use super::{ModAssetSourceLoader, SourceRef};
    use engine_core::asset_source::{has_source, load_source_bytes};
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
        assert_eq!(load_source_bytes(&loader, &source).as_deref(), Some(&b"abc".to_vec()));
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
        assert_eq!(load_source_bytes(&loader, &source).as_deref(), Some(&b"abc".to_vec()));
    }
}
