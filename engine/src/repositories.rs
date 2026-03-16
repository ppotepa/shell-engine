use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::scene::Scene;
use crate::EngineError;

pub trait SceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError>;
    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError>;
}

pub trait AssetRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError>;
    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError>;
}

#[derive(Debug, Clone)]
pub enum AnySceneRepository {
    Fs(FsSceneRepository),
    Zip(ZipSceneRepository),
}

impl SceneRepository for AnySceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError> {
        match self {
            Self::Fs(repo) => repo.load_scene(scene_path),
            Self::Zip(repo) => repo.load_scene(scene_path),
        }
    }

    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError> {
        match self {
            Self::Fs(repo) => repo.discover_scene_paths(),
            Self::Zip(repo) => repo.discover_scene_paths(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnyAssetRepository {
    Fs(FsSceneRepository),
    Zip(ZipSceneRepository),
}

impl AssetRepository for AnyAssetRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError> {
        match self {
            Self::Fs(repo) => repo.read_asset_bytes(asset_path),
            Self::Zip(repo) => repo.read_asset_bytes(asset_path),
        }
    }

    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError> {
        match self {
            Self::Fs(repo) => repo.has_asset(asset_path),
            Self::Zip(repo) => repo.has_asset(asset_path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsSceneRepository {
    mod_source: PathBuf,
}

impl FsSceneRepository {
    pub fn new(mod_source: impl Into<PathBuf>) -> Self {
        Self {
            mod_source: mod_source.into(),
        }
    }

    fn scene_abs_path(&self, scene_path: &str) -> PathBuf {
        let normalized = scene_path.trim_start_matches('/');
        self.mod_source.join(normalized)
    }
}

impl SceneRepository for FsSceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError> {
        let full_path = self.scene_abs_path(scene_path);
        let content =
            fs::read_to_string(&full_path).map_err(|source| EngineError::ManifestRead {
                path: full_path.clone(),
                source,
            })?;

        serde_yaml::from_str::<Scene>(&content).map_err(|source| EngineError::InvalidModYaml {
            path: full_path,
            source,
        })
    }

    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError> {
        let root = self.mod_source.join("scenes");
        if !root.exists() {
            return Ok(Vec::new());
        }

        let mut paths = Vec::new();
        walk_scene_paths(&root, &mut paths).map_err(|source| EngineError::ManifestRead {
            path: root.clone(),
            source,
        })?;
        paths.sort();

        let mut normalized = Vec::with_capacity(paths.len());
        for path in paths {
            let rel = path
                .strip_prefix(&self.mod_source)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");
            normalized.push(format!("/{rel}"));
        }
        Ok(normalized)
    }
}

impl AssetRepository for FsSceneRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError> {
        let full_path = self.scene_abs_path(asset_path);
        fs::read(&full_path).map_err(|source| EngineError::ManifestRead {
            path: full_path,
            source,
        })
    }

    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError> {
        Ok(self.scene_abs_path(asset_path).exists())
    }
}

#[derive(Debug, Clone)]
pub struct ZipSceneRepository {
    mod_source: PathBuf,
}

impl ZipSceneRepository {
    pub fn new(mod_source: impl Into<PathBuf>) -> Self {
        Self {
            mod_source: mod_source.into(),
        }
    }

    fn normalized(scene_path: &str) -> &str {
        scene_path.trim_start_matches('/')
    }

    fn open_archive(&self) -> Result<ZipArchive<fs::File>, EngineError> {
        let file =
            fs::File::open(&self.mod_source).map_err(|source| EngineError::ManifestRead {
                path: self.mod_source.clone(),
                source,
            })?;
        ZipArchive::new(file).map_err(|source| EngineError::ZipArchive {
            path: self.mod_source.clone(),
            source,
        })
    }
}

impl SceneRepository for ZipSceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError> {
        let normalized = Self::normalized(scene_path);
        let mut archive = self.open_archive()?;
        let mut scene_file =
            archive
                .by_name(normalized)
                .map_err(|_| EngineError::MissingSceneEntrypoint {
                    mod_source: self.mod_source.clone(),
                    entrypoint: scene_path.to_string(),
                })?;
        let mut content = String::new();
        scene_file
            .read_to_string(&mut content)
            .map_err(|source| EngineError::ManifestRead {
                path: self.mod_source.clone(),
                source,
            })?;

        serde_yaml::from_str::<Scene>(&content).map_err(|source| EngineError::InvalidModYaml {
            path: self.mod_source.clone(),
            source,
        })
    }

    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError> {
        let mut archive = self.open_archive()?;
        let mut out = Vec::new();
        for idx in 0..archive.len() {
            let entry = archive
                .by_index(idx)
                .map_err(|source| EngineError::ZipArchive {
                    path: self.mod_source.clone(),
                    source,
                })?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().replace('\\', "/");
            if !name.starts_with("scenes/") {
                continue;
            }
            if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
                continue;
            }
            out.push(format!("/{name}"));
        }
        out.sort();
        Ok(out)
    }
}

impl AssetRepository for ZipSceneRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError> {
        let normalized = Self::normalized(asset_path);
        let mut archive = self.open_archive()?;
        let mut file =
            archive
                .by_name(normalized)
                .map_err(|_| EngineError::MissingSceneEntrypoint {
                    mod_source: self.mod_source.clone(),
                    entrypoint: asset_path.to_string(),
                })?;
        let mut out = Vec::new();
        file.read_to_end(&mut out)
            .map_err(|source| EngineError::ManifestRead {
                path: self.mod_source.clone(),
                source,
            })?;
        Ok(out)
    }

    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError> {
        let normalized = Self::normalized(asset_path);
        let mut archive = self.open_archive()?;
        let present = archive.by_name(normalized).is_ok();
        Ok(present)
    }
}

pub fn create_scene_repository(mod_source: &Path) -> Result<AnySceneRepository, EngineError> {
    if !mod_source.exists() {
        return Err(EngineError::SourceNotFound(mod_source.to_path_buf()));
    }
    if mod_source.is_dir() {
        return Ok(AnySceneRepository::Fs(FsSceneRepository::new(mod_source)));
    }
    if is_zip_file(mod_source) {
        return Ok(AnySceneRepository::Zip(ZipSceneRepository::new(mod_source)));
    }
    Err(EngineError::UnsupportedSource(mod_source.to_path_buf()))
}

pub fn create_asset_repository(mod_source: &Path) -> Result<AnyAssetRepository, EngineError> {
    if !mod_source.exists() {
        return Err(EngineError::SourceNotFound(mod_source.to_path_buf()));
    }
    if mod_source.is_dir() {
        return Ok(AnyAssetRepository::Fs(FsSceneRepository::new(mod_source)));
    }
    if is_zip_file(mod_source) {
        return Ok(AnyAssetRepository::Zip(ZipSceneRepository::new(mod_source)));
    }
    Err(EngineError::UnsupportedSource(mod_source.to_path_buf()))
}

fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

#[cfg(test)]
mod tests {
    use super::{create_scene_repository, SceneRepository, ZipSceneRepository};
    use std::fs;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn zip_repository_discovers_and_loads_scenes() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file("mod.yaml", opts)
            .expect("start mod manifest");
        std::io::Write::write_all(
            &mut writer,
            b"name: test\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
        )
        .expect("write manifest");
        writer
            .start_file("scenes/intro.yml", opts)
            .expect("start scene");
        std::io::Write::write_all(
            &mut writer,
            b"id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write scene");
        writer.finish().expect("finish zip");

        let repo = ZipSceneRepository::new(zip_path);
        let paths = repo.discover_scene_paths().expect("discover scenes");
        assert_eq!(paths, vec!["/scenes/intro.yml".to_string()]);
        let scene = repo.load_scene("/scenes/intro.yml").expect("load scene");
        assert_eq!(scene.id, "intro");
    }

    #[test]
    fn create_scene_repository_supports_directory_and_zip() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::write(
            mod_dir.join("scenes/intro.yml"),
            "id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write scene");
        let dir_repo = create_scene_repository(&mod_dir).expect("dir repo");
        assert_eq!(
            dir_repo.discover_scene_paths().expect("discover"),
            vec!["/scenes/intro.yml".to_string()]
        );

        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("scenes/intro.yml", opts)
            .expect("start scene");
        std::io::Write::write_all(
            &mut writer,
            b"id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write scene");
        writer.finish().expect("finish zip");

        let zip_repo = create_scene_repository(&zip_path).expect("zip repo");
        assert_eq!(
            zip_repo.discover_scene_paths().expect("discover"),
            vec!["/scenes/intro.yml".to_string()]
        );
    }
}

fn walk_scene_paths(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_scene_paths(&path, out)?;
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml") {
            out.push(path);
        }
    }
    Ok(())
}
