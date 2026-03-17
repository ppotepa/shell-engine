use serde_yaml::{Mapping, Value};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::scene::Scene;
use crate::scene_compiler::compile_scene_document_with_loader_and_source;
use crate::EngineError;

pub trait SceneRepository {
    fn load_scene(&self, scene_path: &str) -> Result<Scene, EngineError>;
    fn discover_scene_paths(&self) -> Result<Vec<String>, EngineError>;
}

pub trait AssetRepository {
    fn read_asset_bytes(&self, asset_path: &str) -> Result<Vec<u8>, EngineError>;
    fn has_asset(&self, asset_path: &str) -> Result<bool, EngineError>;
    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError>;
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

    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        match self {
            Self::Fs(repo) => repo.list_assets_under(asset_prefix),
            Self::Zip(repo) => repo.list_assets_under(asset_prefix),
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

        let mut paths = Vec::new();
        walk_scene_paths(&root, &mut paths).map_err(|source| EngineError::ManifestRead {
            path: root.clone(),
            source,
        })?;
        paths.sort();

        let mut normalized = Vec::with_capacity(paths.len());
        for path in paths {
            let rel = relative_to_mod(&self.mod_source, &path);
            if !is_discoverable_scene_path(&rel) {
                continue;
            }
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

    fn list_yaml_entries_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        let prefix = asset_prefix.trim_start_matches('/').trim_end_matches('/');
        let prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}/")
        };
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
            if !name.starts_with(&prefix) {
                continue;
            }
            if !is_yaml_path(&name) {
                continue;
            }
            out.push(name);
        }
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
            if !is_discoverable_scene_path(&name) {
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

    fn list_assets_under(&self, asset_prefix: &str) -> Result<Vec<String>, EngineError> {
        let normalized_prefix = Self::normalized(asset_prefix)
            .trim_end_matches('/')
            .to_string();
        let prefix = if normalized_prefix.is_empty() {
            String::new()
        } else {
            format!("{normalized_prefix}/")
        };

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
            if !name.starts_with(&prefix) {
                continue;
            }
            out.push(format!("/{name}"));
        }
        out.sort();
        Ok(out)
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

fn assemble_fs_scene_package(scene_file: &Path, root_content: &str) -> Result<String, EngineError> {
    let mut root = parse_yaml_value(root_content, scene_file)?;
    let Some(package_dir) = scene_file.parent() else {
        return Ok(root_content.to_string());
    };

    merge_sequence_dir(&mut root, &package_dir.join("layers"), "layers")?;
    merge_mapping_dir(&mut root, &package_dir.join("templates"), "templates")?;
    merge_sequence_dir(&mut root, &package_dir.join("objects"), "objects")?;

    to_yaml_string(&root, scene_file)
}

fn assemble_zip_scene_package(
    repo: &ZipSceneRepository,
    scene_path: &str,
    root_content: &str,
) -> Result<String, EngineError> {
    let mut root = parse_yaml_value(root_content, &repo.mod_source)?;
    let Some((package_dir, _)) = scene_path.rsplit_once('/') else {
        return Ok(root_content.to_string());
    };

    merge_zip_sequence_dir(repo, &mut root, package_dir, "layers")?;
    merge_zip_mapping_dir(repo, &mut root, package_dir, "templates")?;
    merge_zip_sequence_dir(repo, &mut root, package_dir, "objects")?;

    to_yaml_string(&root, &repo.mod_source)
}

fn merge_sequence_dir(root: &mut Value, dir: &Path, key: &str) -> Result<(), EngineError> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = Vec::new();
    for file in yaml_files_under(dir).map_err(|source| EngineError::ManifestRead {
        path: dir.to_path_buf(),
        source,
    })? {
        let raw = fs::read_to_string(&file).map_err(|source| EngineError::ManifestRead {
            path: file.clone(),
            source,
        })?;
        let value = parse_yaml_value(&raw, &file)?;
        let Some(seq) = value.as_sequence() else {
            continue;
        };
        entries.extend(seq.iter().cloned());
    }
    append_sequence_entries(root, key, entries);
    Ok(())
}

fn merge_mapping_dir(root: &mut Value, dir: &Path, key: &str) -> Result<(), EngineError> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = Mapping::new();
    for file in yaml_files_under(dir).map_err(|source| EngineError::ManifestRead {
        path: dir.to_path_buf(),
        source,
    })? {
        let raw = fs::read_to_string(&file).map_err(|source| EngineError::ManifestRead {
            path: file.clone(),
            source,
        })?;
        let value = parse_yaml_value(&raw, &file)?;
        let Some(map) = value.as_mapping() else {
            continue;
        };
        for (k, v) in map {
            entries.insert(k.clone(), v.clone());
        }
    }
    merge_mapping_entries(root, key, entries);
    Ok(())
}

fn merge_zip_sequence_dir(
    repo: &ZipSceneRepository,
    root: &mut Value,
    package_dir: &str,
    key: &str,
) -> Result<(), EngineError> {
    let prefix = format!("{package_dir}/{key}");
    let mut entries = Vec::new();
    for file in repo.list_yaml_entries_under(&prefix)? {
        let raw = repo.read_text_file(&file)?;
        let value = parse_yaml_value(&raw, &repo.mod_source)?;
        let Some(seq) = value.as_sequence() else {
            continue;
        };
        entries.extend(seq.iter().cloned());
    }
    append_sequence_entries(root, key, entries);
    Ok(())
}

fn merge_zip_mapping_dir(
    repo: &ZipSceneRepository,
    root: &mut Value,
    package_dir: &str,
    key: &str,
) -> Result<(), EngineError> {
    let prefix = format!("{package_dir}/{key}");
    let mut entries = Mapping::new();
    for file in repo.list_yaml_entries_under(&prefix)? {
        let raw = repo.read_text_file(&file)?;
        let value = parse_yaml_value(&raw, &repo.mod_source)?;
        let Some(map) = value.as_mapping() else {
            continue;
        };
        for (k, v) in map {
            entries.insert(k.clone(), v.clone());
        }
    }
    merge_mapping_entries(root, key, entries);
    Ok(())
}

fn append_sequence_entries(root: &mut Value, key: &str, entries: Vec<Value>) {
    if entries.is_empty() {
        return;
    }
    let Some(root_map) = root.as_mapping_mut() else {
        return;
    };
    let value = root_map
        .entry(Value::String(key.to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(seq) = value.as_sequence_mut() else {
        return;
    };
    seq.extend(entries);
}

fn merge_mapping_entries(root: &mut Value, key: &str, entries: Mapping) {
    if entries.is_empty() {
        return;
    }
    let Some(root_map) = root.as_mapping_mut() else {
        return;
    };
    let value = root_map
        .entry(Value::String(key.to_string()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let Some(map) = value.as_mapping_mut() else {
        return;
    };
    for (k, v) in entries {
        map.insert(k, v);
    }
}

fn parse_yaml_value(raw: &str, path: &Path) -> Result<Value, EngineError> {
    serde_yaml::from_str(raw).map_err(|source| EngineError::InvalidModYaml {
        path: path.to_path_buf(),
        source,
    })
}

fn to_yaml_string(value: &Value, path: &Path) -> Result<String, EngineError> {
    let mut out = serde_yaml::to_string(value).map_err(|source| EngineError::InvalidModYaml {
        path: path.to_path_buf(),
        source,
    })?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn relative_to_mod(mod_root: &Path, path: &Path) -> String {
    path.strip_prefix(mod_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_discoverable_scene_path(path: &str) -> bool {
    path.starts_with("scenes/") && is_yaml_path(path) && !is_reserved_scene_partial_path(path)
}

fn is_scene_package_manifest(path: &str) -> bool {
    let trimmed = path.trim_start_matches('/');
    trimmed.starts_with("scenes/") && trimmed.ends_with("/scene.yml")
}

fn is_reserved_scene_partial_path(path: &str) -> bool {
    let trimmed = path.trim_start_matches('/');
    let segments: Vec<&str> = trimmed.split('/').collect();
    if segments.first() != Some(&"scenes") {
        return false;
    }
    if segments.get(1) == Some(&"shared") {
        return true;
    }
    if segments.len() < 4 {
        return false;
    }
    matches!(
        segments[2],
        "layers" | "sprites" | "templates" | "objects" | "effects"
    )
}

fn is_yaml_path(path: &str) -> bool {
    path.ends_with(".yml") || path.ends_with(".yaml")
}

#[cfg(test)]
mod tests {
    use super::{
        create_asset_repository, create_scene_repository, AssetRepository, SceneRepository,
        ZipSceneRepository,
    };
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
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("assets/fonts/test/8px/ascii/manifest.yaml", opts)
            .expect("start manifest");
        std::io::Write::write_all(&mut writer, b"glyphs: []\n").expect("write manifest");
        writer
            .start_file("assets/fonts/test/8px/ascii/a.txt", opts)
            .expect("start glyph");
        std::io::Write::write_all(&mut writer, b"A\n").expect("write glyph");
        writer.finish().expect("finish zip");

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
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("objects/suzan.yml", opts)
            .expect("start object");
        std::io::Write::write_all(
            &mut writer,
            br#"
name: suzan
sprites:
  - type: text
    content: "$label"
"#,
        )
        .expect("write object");
        writer
            .start_file("scenes/intro.yml", opts)
            .expect("start scene");
        std::io::Write::write_all(
            &mut writer,
            br#"
id: intro
title: Intro
layers: []
objects:
  - use: suzan
    with:
      label: ZIP
next: null
"#,
        )
        .expect("write scene");
        writer.finish().expect("finish zip");

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
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("objects/banner.yml", opts)
            .expect("start object");
        std::io::Write::write_all(
            &mut writer,
            br#"
name: banner
sprites:
  - type: text
    content: "$label"
"#,
        )
        .expect("write object");
        writer
            .start_file("scenes/intro/scene.yml", opts)
            .expect("start scene root");
        std::io::Write::write_all(
            &mut writer,
            br#"
id: intro-package
title: Intro Package
next: null
"#,
        )
        .expect("write scene root");
        writer
            .start_file("scenes/intro/layers/base.yml", opts)
            .expect("start layer partial");
        std::io::Write::write_all(
            &mut writer,
            br#"
- name: base
  sprites:
    - type: text
      content: ZIP
"#,
        )
        .expect("write layer partial");
        writer
            .start_file("scenes/intro/objects/banner.yml", opts)
            .expect("start object partial");
        std::io::Write::write_all(
            &mut writer,
            br#"
- use: banner
  with:
    label: PACKAGE
"#,
        )
        .expect("write object partial");
        writer.finish().expect("finish zip");

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
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("scenes/intro/scene.yml", opts)
            .expect("start scene root");
        std::io::Write::write_all(
            &mut writer,
            br#"
id: intro-package
title: Intro Package
next: null
"#,
        )
        .expect("write scene root");
        writer
            .start_file("scenes/intro/objects/banner.yml", opts)
            .expect("start object partial");
        std::io::Write::write_all(
            &mut writer,
            br#"
- use: ../shared/objects/banner.yml
"#,
        )
        .expect("write object partial");
        writer
            .start_file("scenes/shared/objects/banner.yml", opts)
            .expect("start shared object");
        std::io::Write::write_all(
            &mut writer,
            br#"
name: banner
sprites:
  - type: text
    content: SHARED-ZIP
"#,
        )
        .expect("write shared object");
        writer.finish().expect("finish zip");

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
        let file = fs::File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();
        writer
            .start_file("scenes/intro/scene.yml", opts)
            .expect("start scene root");
        std::io::Write::write_all(
            &mut writer,
            b"id: intro\ntitle: Intro\nlayers: []\nnext: null\n",
        )
        .expect("write scene root");
        writer
            .start_file("scenes/shared/common.yml", opts)
            .expect("start shared");
        std::io::Write::write_all(
            &mut writer,
            b"id: shared\ntitle: Shared\nlayers: []\nnext: null\n",
        )
        .expect("write shared");
        writer
            .start_file("scenes/intro/layers/base.yml", opts)
            .expect("start partial");
        std::io::Write::write_all(&mut writer, b"- name: base\n  sprites: []\n")
            .expect("write partial");
        writer.finish().expect("finish zip");

        let repo = create_scene_repository(&zip_path).expect("repo");
        assert_eq!(
            repo.discover_scene_paths().expect("discover scenes"),
            vec!["/scenes/intro/scene.yml".to_string()]
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
