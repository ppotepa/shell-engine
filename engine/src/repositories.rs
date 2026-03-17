//! Scene and asset repository adapters for loading authored scene packages from
//! either an unpacked mod directory or a packaged zip archive.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use engine_authoring::package::{
    assemble_scene_package, PackageError, PackageYamlFile, ScenePackagePartials,
};
use engine_authoring::repository::{
    is_discoverable_scene_path, is_scene_package_manifest, is_yaml_path,
};

use crate::scene::Scene;
use crate::scene_compiler::compile_scene_document_with_loader_and_source;
use crate::EngineError;

/// Loads authored scenes from a mod source after any package-level assembly.
pub trait SceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError>;
    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError>;
}

/// Reads non-scene assets from the same mod source as scene manifests.
pub trait AssetRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError>;
    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError>;
    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError>;
}

#[derive(Debug, Clone)]
/// Type-erased scene repository used by the engine when the backing source is
/// determined at runtime.
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
/// Type-erased asset repository paired with the selected mod source.
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

    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        match self {
            Self::Fs(repo) => repo.list_assets_under(asset_prefix),
            Self::Zip(repo) => repo.list_assets_under(asset_prefix),
        }
    }
}

#[derive(Debug, Clone)]
/// Scene and asset repository backed by an unpacked mod directory on disk.
pub struct FsSceneRepository {
    mod_source: PathBuf,
}

impl FsSceneRepository {
    /// Creates a repository rooted at an unpacked mod directory.
    pub fn new(mod_source: impl Into<PathBuf>) -> Self {
        Self {
            mod_source: mod_source.into(),
        }
    }

    fn scene_abs_path(&self, scene_path: &str) -> PathBuf {
        let normalized = scene_path.trim_start_matches('/');
        self.mod_source.join(normalized)
    }

    fn load_scene_content(&self, scene_path: &str) -> Result<(PathBuf, String), EngineError> {
        let requested_path = self.scene_abs_path(scene_path);
        let full_path = if requested_path.is_dir() {
            requested_path.join("scene.yml")
        } else {
            requested_path
        };
        let content =
            fs::read_to_string(&full_path).map_err(|source| EngineError::ManifestRead {
                path: full_path.clone(),
                source,
            })?;

        if !is_scene_package_manifest(&relative_to_mod(&self.mod_source, &full_path)) {
            return Ok((full_path, content));
        }

        let merged = assemble_fs_scene_package(&full_path, &content)?;
        Ok((full_path, merged))
    }
}

impl SceneRepository for FsSceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError> {
        let (full_path, content) = self.load_scene_content(scene_path)?;
        let scene_source_path = format!("/{}", relative_to_mod(&self.mod_source, &full_path));

        compile_scene_document_with_loader_and_source(&content, &scene_source_path, |asset_path| {
            fs::read_to_string(self.scene_abs_path(asset_path)).ok()
        })
        .map_err(|source| EngineError::InvalidModYaml {
            path: full_path,
            source,
        })
    }

    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError> {
        let root = self.mod_source.join("scenes");
        if !root.exists() {
            return Ok(Vec::new());
        }
        let paths = yaml_files_under(&root).map_err(|source| EngineError::ManifestRead {
            path: root.clone(),
            source,
        })?;
        Ok(paths
            .into_iter()
            .filter_map(|path| {
                let rel = relative_to_mod(&self.mod_source, &path);
                is_discoverable_scene_path(&rel).then(|| format!("/{rel}"))
            })
            .collect())
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

    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        let root = self.scene_abs_path(asset_prefix);
        if !root.exists() {
            return Ok(Vec::new());
        }
        let mut paths = Vec::new();
        walk_asset_paths(&root, &mut paths).map_err(|source| EngineError::ManifestRead {
            path: root.clone(),
            source,
        })?;
        paths.sort();
        Ok(paths
            .into_iter()
            .map(|path| {
                let rel = path.strip_prefix(&self.mod_source).unwrap_or(path.as_path());
                format!("/{}", rel.to_string_lossy().replace('\\', "/"))
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
/// Scene and asset repository backed by a packaged mod zip archive.
pub struct ZipSceneRepository {
    mod_source: PathBuf,
}

impl ZipSceneRepository {
    /// Creates a repository rooted at a packaged mod archive.
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

    fn read_text_file(&self, normalized_path: &str) -> Result<String, EngineError> {
        let mut archive = self.open_archive()?;
        let mut file =
            archive
                .by_name(normalized_path)
                .map_err(|_| EngineError::MissingSceneEntrypoint {
                    mod_source: self.mod_source.clone(),
                    entrypoint: format!("/{normalized_path}"),
                })?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|source| EngineError::ManifestRead {
                path: self.mod_source.clone(),
                source,
            })?;
        Ok(content)
    }

    fn zip_file_names(&self) -> Result<Vec<String>, EngineError> {
        let mut archive = self.open_archive()?;
        let mut out = Vec::with_capacity(archive.len());
        for idx in 0..archive.len() {
            let entry = archive
                .by_index(idx)
                .map_err(|source| EngineError::ZipArchive {
                    path: self.mod_source.clone(),
                    source,
                })?;
            if !entry.is_dir() {
                out.push(entry.name().replace('\\', "/"));
            }
        }
        Ok(out)
    }

    fn list_yaml_entries_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        let prefix_raw = asset_prefix.trim_start_matches('/').trim_end_matches('/');
        let prefix = if prefix_raw.is_empty() {
            String::new()
        } else {
            format!("{prefix_raw}/")
        };
        let mut out: Vec<String> = self
            .zip_file_names()?
            .into_iter()
            .filter(|n| n.starts_with(&prefix) && is_yaml_path(n))
            .collect();
        out.sort();
        Ok(out)
    }
}

impl SceneRepository for ZipSceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError> {
        let normalized = Self::normalized(scene_path);
        let content = self.read_text_file(normalized)?;
        let content = if is_scene_package_manifest(normalized) {
            assemble_zip_scene_package(self, normalized, &content)?
        } else {
            content
        };

        let scene_source_path = format!("/{normalized}");
        compile_scene_document_with_loader_and_source(&content, &scene_source_path, |asset_path| {
            let normalized_asset = Self::normalized(asset_path);
            let mut nested_archive = self.open_archive().ok()?;
            let mut file = nested_archive.by_name(normalized_asset).ok()?;
            let mut raw = String::new();
            file.read_to_string(&mut raw).ok()?;
            Some(raw)
        })
        .map_err(|source| EngineError::InvalidModYaml {
            path: self.mod_source.clone(),
            source,
        })
    }

    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError> {
        let mut out: Vec<String> = self
            .zip_file_names()?
            .into_iter()
            .filter(|n| is_discoverable_scene_path(n))
            .map(|n| format!("/{n}"))
            .collect();
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

    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        let prefix_raw = Self::normalized(asset_prefix).trim_end_matches('/');
        let prefix = if prefix_raw.is_empty() {
            String::new()
        } else {
            format!("{prefix_raw}/")
        };
        let mut out: Vec<String> = self
            .zip_file_names()?
            .into_iter()
            .filter(|n| n.starts_with(&prefix))
            .map(|n| format!("/{n}"))
            .collect();
        out.sort();
        Ok(out)
    }
}

/// Selects the scene repository implementation for a mod directory or zip file.
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

/// Selects the asset repository implementation for a mod directory or zip file.
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

fn assemble_fs_scene_package(scene_file: &Path, root_content: &str) -> Result<String, EngineError> {
    let Some(package_dir) = scene_file.parent() else {
        return Ok(root_content.to_string());
    };
    let Some(mod_root) = package_dir.parent().and_then(Path::parent) else {
        return Ok(root_content.to_string());
    };
    let partials = ScenePackagePartials {
        layers: read_fs_package_partials(mod_root, &package_dir.join("layers"))?,
        templates: read_fs_package_partials(mod_root, &package_dir.join("templates"))?,
        objects: read_fs_package_partials(mod_root, &package_dir.join("objects"))?,
    };
    let root_path = format!("/{}", relative_to_mod(mod_root, scene_file));
    assemble_scene_package(root_content, &root_path, &partials).map_err(map_package_error)
}

fn assemble_zip_scene_package(
    repo: &ZipSceneRepository,
    scene_path: &str,
    root_content: &str,
) -> Result<String, EngineError> {
    let Some((package_dir, _)) = scene_path.rsplit_once('/') else {
        return Ok(root_content.to_string());
    };
    let partials = ScenePackagePartials {
        layers: read_zip_package_partials(repo, package_dir, "layers")?,
        templates: read_zip_package_partials(repo, package_dir, "templates")?,
        objects: read_zip_package_partials(repo, package_dir, "objects")?,
    };
    assemble_scene_package(root_content, &format!("/{scene_path}"), &partials).map_err(map_package_error)
}

fn read_fs_package_partials(mod_root: &Path, dir: &Path) -> Result<Vec<PackageYamlFile>, EngineError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for file in yaml_files_under(dir).map_err(|source| EngineError::ManifestRead {
        path: dir.to_path_buf(),
        source,
    })? {
        let raw = fs::read_to_string(&file).map_err(|source| EngineError::ManifestRead {
            path: file.clone(),
            source,
        })?;
        out.push(PackageYamlFile::new(
            format!("/{}", relative_to_mod(mod_root, &file)),
            raw,
        ));
    }
    Ok(out)
}

fn read_zip_package_partials(
    repo: &ZipSceneRepository,
    package_dir: &str,
    key: &str,
) -> Result<Vec<PackageYamlFile>, EngineError> {
    let mut out = Vec::new();
    for file in repo.list_yaml_entries_under(&format!("{package_dir}/{key}"))? {
        out.push(PackageYamlFile::new(format!("/{file}"), repo.read_text_file(&file)?));
    }
    Ok(out)
}

fn map_package_error(error: PackageError) -> EngineError {
    let (path, source) = error.into_parts();
    EngineError::InvalidModYaml {
        path: PathBuf::from(path),
        source,
    }
}

fn relative_to_mod(mod_root: &Path, path: &Path) -> String {
    path.strip_prefix(mod_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{
        create_asset_repository, create_scene_repository, AssetRepository, SceneRepository,
        ZipSceneRepository,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn make_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = fs::File::create(path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        for (name, content) in files {
            writer.start_file(*name, opts).expect("start file");
            std::io::Write::write_all(&mut writer, content).expect("write file");
        }
        writer.finish().expect("finish zip");
    }

    #[test]
    fn zip_repository_discovers_and_loads_scenes() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "mod.yaml",
                    b"name: test\nversion: 0.1.0\nentrypoint: /scenes/intro.yml\n",
                ),
                (
                    "scenes/intro.yml",
                    b"id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
                ),
            ],
        );

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
        make_zip(
            &zip_path,
            &[(
                "scenes/intro.yml",
                b"id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
            )],
        );

        let zip_repo = create_scene_repository(&zip_path).expect("zip repo");
        assert_eq!(
            zip_repo.discover_scene_paths().expect("discover"),
            vec!["/scenes/intro.yml".to_string()]
        );
    }

    #[test]
    fn asset_repository_lists_assets_for_directory_and_zip() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/fonts/test/8px/ascii")).expect("create font dir");
        fs::write(
            mod_dir.join("assets/fonts/test/8px/ascii/manifest.yaml"),
            "glyphs: []\n",
        )
        .expect("write manifest");
        fs::write(mod_dir.join("assets/fonts/test/8px/ascii/a.txt"), "A\n").expect("write glyph");
        let dir_repo = create_asset_repository(&mod_dir).expect("dir asset repo");
        assert_eq!(
            dir_repo
                .list_assets_under("/assets/fonts")
                .expect("list font assets"),
            vec![
                "/assets/fonts/test/8px/ascii/a.txt".to_string(),
                "/assets/fonts/test/8px/ascii/manifest.yaml".to_string()
            ]
        );

        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "assets/fonts/test/8px/ascii/manifest.yaml",
                    b"glyphs: []\n",
                ),
                ("assets/fonts/test/8px/ascii/a.txt", b"A\n"),
            ],
        );

        let zip_repo = create_asset_repository(&zip_path).expect("zip asset repo");
        assert_eq!(
            zip_repo
                .list_assets_under("/assets/fonts")
                .expect("list font assets"),
            vec![
                "/assets/fonts/test/8px/ascii/a.txt".to_string(),
                "/assets/fonts/test/8px/ascii/manifest.yaml".to_string()
            ]
        );
    }

    #[test]
    fn fs_repository_expands_object_instances_from_objects_directory() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes")).expect("create scenes");
        fs::create_dir_all(mod_dir.join("objects")).expect("create objects");
        fs::write(
            mod_dir.join("objects/suzan.yml"),
            r#"
name: suzan
sprites:
  - type: text
    content: "$label"
"#,
        )
        .expect("write object");
        fs::write(
            mod_dir.join("scenes/intro.yml"),
            r#"
id: intro
title: Intro
layers: []
objects:
  - use: suzan
    with:
      label: READY
next: null
"#,
        )
        .expect("write scene");

        let repo = create_scene_repository(&mod_dir).expect("repo");
        let scene = repo.load_scene("/scenes/intro.yml").expect("load scene");
        assert_eq!(scene.layers.len(), 1);
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text { content, .. } => assert_eq!(content, "READY"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn zip_repository_expands_object_instances_from_objects_directory() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "objects/suzan.yml",
                    b"\nname: suzan\nsprites:\n  - type: text\n    content: \"$label\"\n",
                ),
                (
                    "scenes/intro.yml",
                    b"\nid: intro\ntitle: Intro\nlayers: []\nobjects:\n  - use: suzan\n    with:\n      label: ZIP\nnext: null\n",
                ),
            ],
        );

        let repo = create_scene_repository(&zip_path).expect("repo");
        let scene = repo.load_scene("/scenes/intro.yml").expect("load scene");
        assert_eq!(scene.layers.len(), 1);
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text { content, .. } => assert_eq!(content, "ZIP"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn fs_repository_discovers_and_loads_scene_packages() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes/intro/layers")).expect("create layer dir");
        fs::create_dir_all(mod_dir.join("scenes/intro/objects")).expect("create object dir");
        fs::create_dir_all(mod_dir.join("objects")).expect("create global object dir");
        fs::write(
            mod_dir.join("objects/banner.yml"),
            r#"
name: banner
sprites:
  - type: text
    content: "$label"
"#,
        )
        .expect("write object");
        fs::write(
            mod_dir.join("scenes/intro/scene.yml"),
            r#"
id: intro-package
title: Intro Package
next: null
"#,
        )
        .expect("write scene root");
        fs::write(
            mod_dir.join("scenes/intro/layers/base.yml"),
            r#"
- name: base
  sprites:
    - type: text
      content: HELLO
"#,
        )
        .expect("write layer partial");
        fs::write(
            mod_dir.join("scenes/intro/objects/banner.yml"),
            r#"
- use: banner
  with:
    label: PACKAGE
"#,
        )
        .expect("write object partial");

        let repo = create_scene_repository(&mod_dir).expect("repo");
        assert_eq!(
            repo.discover_scene_paths().expect("discover scenes"),
            vec!["/scenes/intro/scene.yml".to_string()]
        );

        let scene = repo
            .load_scene("/scenes/intro/scene.yml")
            .expect("load scene package");
        assert_eq!(scene.id, "intro-package");
        assert_eq!(scene.layers.len(), 2);
    }

    #[test]
    fn zip_repository_discovers_and_loads_scene_packages() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "objects/banner.yml",
                    b"\nname: banner\nsprites:\n  - type: text\n    content: \"$label\"\n",
                ),
                (
                    "scenes/intro/scene.yml",
                    b"\nid: intro-package\ntitle: Intro Package\nnext: null\n",
                ),
                (
                    "scenes/intro/layers/base.yml",
                    b"\n- name: base\n  sprites:\n    - type: text\n      content: ZIP\n",
                ),
                (
                    "scenes/intro/objects/banner.yml",
                    b"\n- use: banner\n  with:\n    label: PACKAGE\n",
                ),
            ],
        );

        let repo = create_scene_repository(&zip_path).expect("repo");
        assert_eq!(
            repo.discover_scene_paths().expect("discover scenes"),
            vec!["/scenes/intro/scene.yml".to_string()]
        );

        let scene = repo
            .load_scene("/scenes/intro/scene.yml")
            .expect("load scene package");
        assert_eq!(scene.id, "intro-package");
        assert_eq!(scene.layers.len(), 2);
    }

    #[test]
    fn fs_scene_package_loads_shared_object_via_relative_ref() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes/intro/objects")).expect("create object dir");
        fs::create_dir_all(mod_dir.join("scenes/shared/objects")).expect("create shared object dir");
        fs::write(
            mod_dir.join("scenes/intro/scene.yml"),
            r#"
id: intro-package
title: Intro Package
next: null
"#,
        )
        .expect("write scene root");
        fs::write(
            mod_dir.join("scenes/intro/objects/banner.yml"),
            r#"
- use: ../shared/objects/banner.yml
"#,
        )
        .expect("write object partial");
        fs::write(
            mod_dir.join("scenes/shared/objects/banner.yml"),
            r#"
name: banner
sprites:
  - type: text
    content: SHARED-FS
"#,
        )
        .expect("write shared object");

        let repo = create_scene_repository(&mod_dir).expect("repo");
        let scene = repo
            .load_scene("/scenes/intro/scene.yml")
            .expect("load scene package");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn zip_scene_package_loads_shared_object_via_relative_ref() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "scenes/intro/scene.yml",
                    b"\nid: intro-package\ntitle: Intro Package\nnext: null\n",
                ),
                (
                    "scenes/intro/objects/banner.yml",
                    b"\n- use: ../shared/objects/banner.yml\n",
                ),
                (
                    "scenes/shared/objects/banner.yml",
                    b"\nname: banner\nsprites:\n  - type: text\n    content: SHARED-ZIP\n",
                ),
            ],
        );

        let repo = create_scene_repository(&zip_path).expect("repo");
        let scene = repo
            .load_scene("/scenes/intro/scene.yml")
            .expect("load scene package");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn fs_repository_discovery_excludes_shared_and_partial_directories() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes/shared")).expect("create shared dir");
        fs::create_dir_all(mod_dir.join("scenes/intro/layers")).expect("create layers dir");
        fs::write(
            mod_dir.join("scenes/intro/scene.yml"),
            "id: intro\ntitle: Intro\nlayers: []\nnext: null\n",
        )
        .expect("write scene root");
        fs::write(
            mod_dir.join("scenes/shared/common.yml"),
            "id: shared\ntitle: Shared\nlayers: []\nnext: null\n",
        )
        .expect("write shared");
        fs::write(
            mod_dir.join("scenes/intro/layers/base.yml"),
            "- name: base\n  sprites: []\n",
        )
        .expect("write partial");

        let repo = create_scene_repository(&mod_dir).expect("repo");
        assert_eq!(
            repo.discover_scene_paths().expect("discover scenes"),
            vec!["/scenes/intro/scene.yml".to_string()]
        );
    }

    #[test]
    fn zip_repository_discovery_excludes_shared_and_partial_directories() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        make_zip(
            &zip_path,
            &[
                (
                    "scenes/intro/scene.yml",
                    b"id: intro\ntitle: Intro\nlayers: []\nnext: null\n",
                ),
                (
                    "scenes/shared/common.yml",
                    b"id: shared\ntitle: Shared\nlayers: []\nnext: null\n",
                ),
                ("scenes/intro/layers/base.yml", b"- name: base\n  sprites: []\n"),
            ],
        );

        let repo = create_scene_repository(&zip_path).expect("repo");
        assert_eq!(
            repo.discover_scene_paths().expect("discover scenes"),
            vec!["/scenes/intro/scene.yml".to_string()]
        );
    }
}

fn walk_asset_paths(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_asset_paths(&path, out)?;
            continue;
        }
        out.push(path);
    }
    Ok(())
}

fn yaml_files_under(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_yaml_paths(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_yaml_paths(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_yaml_paths(&path, out)?;
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
