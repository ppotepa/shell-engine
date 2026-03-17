//! File-system scanning helpers for discovering project assets and validating mod layouts.

use engine_authoring::repository::is_discoverable_scene_path;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Result of validating a directory as a Shell Quest mod project.
#[derive(Debug, Clone)]
pub struct ProjectValidation {
    pub valid: bool,
    pub code: &'static str,
    pub message: String,
}

/// Recursively collects sorted file paths under `root/rel` that match the given extension.
pub fn collect_files(root: &Path, rel: &str, ext: &str) -> Vec<String> {
    let base = root.join(rel);
    let mut out = Vec::new();
    walk(&base, ext, &mut out);
    out.sort();
    out
}

/// Validates the given directory as a Shell Quest mod project, checking `mod.yaml` and entrypoint.
pub fn validate_project_dir(dir: &Path) -> ProjectValidation {
    let mod_path = dir.join("mod.yaml");
    if !mod_path.exists() {
        return ProjectValidation {
            valid: false,
            code: "E_MOD_MISSING",
            message: "mod.yaml not found".to_string(),
        };
    }

    let Ok(raw) = fs::read_to_string(&mod_path) else {
        return ProjectValidation {
            valid: false,
            code: "E_MOD_READ",
            message: "mod.yaml cannot be read".to_string(),
        };
    };

    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&raw) else {
        return ProjectValidation {
            valid: false,
            code: "E_MOD_PARSE",
            message: "mod.yaml is not valid YAML".to_string(),
        };
    };

    let name_ok = yaml
        .get("name")
        .and_then(serde_yaml::Value::as_str)
        .is_some();
    let version_ok = yaml
        .get("version")
        .and_then(serde_yaml::Value::as_str)
        .is_some();
    let entrypoint = yaml.get("entrypoint").and_then(serde_yaml::Value::as_str);
    if !name_ok || !version_ok || entrypoint.is_none() {
        return ProjectValidation {
            valid: false,
            code: "E_MOD_FIELDS",
            message: "required fields missing: name/version/entrypoint".to_string(),
        };
    }

    let entrypoint = entrypoint.unwrap_or_default();
    if !entrypoint.starts_with('/') || !entrypoint.ends_with(".yml") {
        return ProjectValidation {
            valid: false,
            code: "E_ENTRYPOINT_FORMAT",
            message: "entrypoint must start with '/' and end with '.yml'".to_string(),
        };
    }

    let entrypoint_rel = entrypoint.trim_start_matches('/');
    if !dir.join(entrypoint_rel).exists() {
        return ProjectValidation {
            valid: false,
            code: "E_ENTRYPOINT_MISSING",
            message: format!("entrypoint file does not exist: {entrypoint}"),
        };
    }

    ProjectValidation {
        valid: true,
        code: "OK",
        message: "project manifest is valid".to_string(),
    }
}

fn walk(path: &Path, ext: &str, out: &mut Vec<String>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p: PathBuf = e.path();
        if p.is_dir() {
            walk(&p, ext, out);
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(p.display().to_string());
        }
    }
}

/// Collects all `.yml` files under `root` that reference a Shell Quest schema header.
pub fn collect_schema_project_yml_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_schema_yml(root, &mut out);
    out.sort();
    out
}

fn walk_schema_yml(path: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p: PathBuf = e.path();
        if p.is_dir() {
            walk_schema_yml(&p, out);
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        if file_uses_sq_schema(&p) {
            out.push(p.display().to_string());
        }
    }
}

fn file_uses_sq_schema(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    raw.lines()
        .take(3)
        .any(|line| line.contains("$schema=") && is_shell_quest_schema_ref(line))
}

fn is_shell_quest_schema_ref(line: &str) -> bool {
    let Some((_, schema_ref)) = line.split_once("$schema=") else {
        return false;
    };
    let schema_ref = schema_ref.trim();
    let references_sq_schema = schema_ref.contains("schemas/")
        || schema_ref.contains("/schemas/")
        || schema_ref.contains("shell-quest.local/schemas/");
    let references_schema_file = schema_ref.ends_with(".schema.yaml")
        || schema_ref.ends_with(".schema.yml")
        || schema_ref.ends_with(".yaml")
        || schema_ref.ends_with(".yml");
    references_sq_schema && references_schema_file
}

fn extract_schema_ref(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    raw.lines()
        .take(3)
        .find_map(|line| {
            line.split_once("$schema=")
                .map(|(_, schema_ref)| schema_ref.trim())
        })
        .map(str::to_string)
}

fn resolve_schema_ref_path(
    repo_root: &Path,
    yaml_path: &Path,
    schema_ref: &str,
) -> Option<PathBuf> {
    if let Some(relative) = schema_ref.strip_prefix("https://shell-quest.local/") {
        return Some(normalize_path(&repo_root.join(relative)));
    }
    if let Some(relative) = schema_ref.strip_prefix("http://shell-quest.local/") {
        return Some(normalize_path(&repo_root.join(relative)));
    }
    if schema_ref.contains("://") {
        return None;
    }
    Some(normalize_path(
        &yaml_path.parent().unwrap_or(repo_root).join(schema_ref),
    ))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Collects all game-relevant YAML/YML files under `mod_root` (mod.yaml, scenes, objects, fonts).
pub fn collect_game_yaml_files(mod_root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_game_yaml(mod_root, mod_root, &mut out);
    out.sort();
    out
}

/// Collects scene entry files (scene roots, not partial sub-directory files) under `mod_root/scenes/`.
pub fn collect_scene_entry_files(mod_root: &Path) -> Vec<String> {
    let scenes_root = mod_root.join("scenes");
    let mut out = Vec::new();
    walk_scene_entries(mod_root, &scenes_root, &mut out);
    out.sort();
    out
}

fn walk_game_yaml(root: &Path, path: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p: PathBuf = e.path();
        if p.is_dir() {
            walk_game_yaml(root, &p, out);
            continue;
        }
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if ext != "yml" && ext != "yaml" {
            continue;
        }
        if is_game_yaml(root, &p) {
            out.push(p.display().to_string());
        }
    }
}

fn is_game_yaml(root: &Path, path: &Path) -> bool {
    let rel = path.strip_prefix(root).ok();
    let rel_s = rel
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    if rel_s == "mod.yaml" {
        return true;
    }
    if is_discoverable_scene_path(&rel_s) {
        return true;
    }
    if rel_s.starts_with("objects/") && (rel_s.ends_with(".yml") || rel_s.ends_with(".yaml")) {
        return true;
    }
    if rel_s.contains("/assets/fonts/") && rel_s.ends_with("/manifest.yaml") {
        return true;
    }
    file_uses_sq_schema(path)
}

fn walk_scene_entries(root: &Path, path: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_scene_entries(root, &p, out);
            continue;
        }
        let rel_s = p
            .strip_prefix(root)
            .ok()
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if is_discoverable_scene_path(&rel_s) {
            out.push(p.display().to_string());
        }
    }
}

/// Walks up from a project YAML path to infer the mod root directory.
pub fn infer_mod_root_from_project_yml(path: &Path) -> Option<String> {
    let mut cur = path.parent()?;
    loop {
        let is_scenes = cur.file_name().and_then(|s| s.to_str()) == Some("scenes");
        if is_scenes {
            let mod_root = cur.parent()?;
            if mod_root.join("mod.yaml").exists() {
                return Some(mod_root.display().to_string());
            }
        }
        cur = cur.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_game_yaml_files, collect_scene_entry_files, collect_schema_project_yml_files,
        extract_schema_ref, resolve_schema_ref_path,
    };
    use serde_yaml::Value;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn schema_scanner_includes_object_schema_files() {
        let temp = tempdir().expect("temp dir");
        let object_yaml = temp.path().join("npc.yml");
        fs::write(
            &object_yaml,
            "# yaml-language-server: $schema=../schemas/object.yaml\nname: npc\n",
        )
        .expect("write yaml");

        let files = collect_schema_project_yml_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("npc.yml"));
    }

    #[test]
    fn schema_scanner_includes_scene_partial_schema_files() {
        let temp = tempdir().expect("temp dir");
        let partial_yaml = temp.path().join("scene.yml");
        fs::write(
            &partial_yaml,
            "# yaml-language-server: $schema=../../schemas/scene-file.schema.yaml\nid: intro\ntitle: Intro\n",
        )
        .expect("write yaml");

        let files = collect_schema_project_yml_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("scene.yml"));
    }

    #[test]
    fn schema_scanner_includes_package_partial_schema_files() {
        let temp = tempdir().expect("temp dir");
        let layers_yaml = temp.path().join("layers.yml");
        let templates_yaml = temp.path().join("templates.yml");
        let objects_yaml = temp.path().join("objects.yml");
        fs::write(
            &layers_yaml,
            "# yaml-language-server: $schema=../../schemas/layers-file.schema.yaml\n- name: bg\n  sprites: []\n",
        )
        .expect("write layers yaml");
        fs::write(
            &templates_yaml,
            "# yaml-language-server: $schema=../../schemas/templates-file.schema.yaml\n{}\n",
        )
        .expect("write templates yaml");
        fs::write(
            &objects_yaml,
            "# yaml-language-server: $schema=../../schemas/objects-file.schema.yaml\n- use: npc\n",
        )
        .expect("write objects yaml");

        let files = collect_schema_project_yml_files(temp.path());
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|path| path.ends_with("layers.yml")));
        assert!(files.iter().any(|path| path.ends_with("templates.yml")));
        assert!(files.iter().any(|path| path.ends_with("objects.yml")));
    }

    #[test]
    fn schema_scanner_includes_mod_local_overlay_schema_files() {
        let temp = tempdir().expect("temp dir");
        let scene_yaml = temp.path().join("intro.yml");
        let layers_yaml = temp.path().join("layers.yml");
        let templates_yaml = temp.path().join("templates.yml");
        let objects_yaml = temp.path().join("objects.yml");
        fs::write(
            &scene_yaml,
            "# yaml-language-server: $schema=../../schemas/scenes.yaml\nid: intro\ntitle: Intro\n",
        )
        .expect("write yaml");
        fs::write(
            &layers_yaml,
            "# yaml-language-server: $schema=../../schemas/layers.yaml\n- name: bg\n  sprites: []\n",
        )
        .expect("write yaml");
        fs::write(
            &templates_yaml,
            "# yaml-language-server: $schema=../../schemas/templates.yaml\n{}\n",
        )
        .expect("write yaml");
        fs::write(
            &objects_yaml,
            "# yaml-language-server: $schema=../../schemas/objects.yaml\n- use: npc\n",
        )
        .expect("write yaml");

        let files = collect_schema_project_yml_files(temp.path());
        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|path| path.ends_with("intro.yml")));
        assert!(files.iter().any(|path| path.ends_with("layers.yml")));
        assert!(files.iter().any(|path| path.ends_with("templates.yml")));
        assert!(files.iter().any(|path| path.ends_with("objects.yml")));
    }

    #[test]
    fn game_yaml_scanner_includes_objects_directory() {
        let temp = tempdir().expect("temp dir");
        let objects_dir = temp.path().join("objects");
        fs::create_dir_all(&objects_dir).expect("create objects dir");
        fs::write(objects_dir.join("suzan.yml"), "name: suzan\n").expect("write object");

        let files = collect_game_yaml_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("objects/suzan.yml"));
    }

    #[test]
    fn game_yaml_scanner_excludes_scene_partial_directories() {
        let temp = tempdir().expect("temp dir");
        let layers_dir = temp.path().join("scenes/intro/layers");
        fs::create_dir_all(&layers_dir).expect("create layers dir");
        fs::write(
            temp.path().join("scenes/intro/scene.yml"),
            "id: intro\ntitle: Intro\n",
        )
        .expect("write scene root");
        fs::write(layers_dir.join("base.yml"), "- name: bg\n  sprites: []\n")
            .expect("write layer partial");

        let files = collect_game_yaml_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("scenes/intro/scene.yml"));
    }

    #[test]
    fn scene_entry_scanner_only_returns_scene_roots() {
        let temp = tempdir().expect("temp dir");
        let layers_dir = temp.path().join("scenes/intro/layers");
        fs::create_dir_all(&layers_dir).expect("create layers dir");
        fs::write(
            temp.path().join("scenes/intro.yml"),
            "id: intro-flat\ntitle: Intro\n",
        )
        .expect("write flat scene");
        fs::write(
            temp.path().join("scenes/intro/scene.yml"),
            "id: intro\ntitle: Intro\n",
        )
        .expect("write scene root");
        fs::write(layers_dir.join("base.yml"), "- name: bg\n  sprites: []\n")
            .expect("write layer partial");

        let files = collect_scene_entry_files(temp.path());
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|path| path.ends_with("scenes/intro.yml")));
        assert!(files
            .iter()
            .any(|path| path.ends_with("scenes/intro/scene.yml")));
    }

    #[test]
    fn repo_schema_headers_resolve_to_existing_files() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let yaml_files = collect_game_yaml_files(&repo_root);
        let schema_files = collect_schema_project_yml_files(&repo_root);

        for path in yaml_files.iter().chain(schema_files.iter()) {
            let yaml_path = Path::new(path);
            let Some(schema_ref) = extract_schema_ref(yaml_path) else {
                continue;
            };
            let Some(schema_path) = resolve_schema_ref_path(&repo_root, yaml_path, &schema_ref)
            else {
                continue;
            };
            assert!(
                schema_path.exists(),
                "schema ref {schema_ref} from {} resolved to missing {}",
                yaml_path.display(),
                schema_path.display()
            );
        }
    }

    #[test]
    fn repo_schema_files_parse_and_refs_resolve() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .expect("repo root");
        let schema_files = collect_repo_schema_files(&repo_root);

        assert!(
            !schema_files.is_empty(),
            "expected repository schema files to exist"
        );

        for schema_path in schema_files {
            let raw = fs::read_to_string(&schema_path)
                .unwrap_or_else(|err| panic!("failed reading {}: {err}", schema_path.display()));
            let doc: Value = serde_yaml::from_str(&raw)
                .unwrap_or_else(|err| panic!("failed parsing {}: {err}", schema_path.display()));
            assert_schema_refs_resolve(&repo_root, &schema_path, &doc, &doc);
        }
    }

    fn collect_repo_schema_files(repo_root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let schema_root = repo_root.join("schemas");
        walk_schema_files(&schema_root, &mut files);
        files.sort();
        files
    }

    fn walk_schema_files(path: &Path, out: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(path) {
            Ok(v) => v,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_dir() {
                walk_schema_files(&file_path, out);
                continue;
            }
            if file_path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                out.push(file_path);
            }
        }
    }

    fn assert_schema_refs_resolve(
        repo_root: &Path,
        schema_path: &Path,
        root_doc: &Value,
        value: &Value,
    ) {
        match value {
            Value::Mapping(map) => {
                for (key, child) in map {
                    if key.as_str() == Some("$ref") {
                        let ref_value = child.as_str().unwrap_or_else(|| {
                            panic!("$ref in {} must be a string", schema_path.display())
                        });
                        assert_ref_resolves(repo_root, schema_path, root_doc, ref_value);
                    } else {
                        assert_schema_refs_resolve(repo_root, schema_path, root_doc, child);
                    }
                }
            }
            Value::Sequence(seq) => {
                for child in seq {
                    assert_schema_refs_resolve(repo_root, schema_path, root_doc, child);
                }
            }
            _ => {}
        }
    }

    fn assert_ref_resolves(
        repo_root: &Path,
        schema_path: &Path,
        root_doc: &Value,
        ref_value: &str,
    ) {
        let (path_part, pointer_part) = ref_value
            .split_once('#')
            .map_or((ref_value, None), |(p, f)| (p, Some(format!("#{f}"))));

        if path_part.is_empty() {
            let fragment = pointer_part.as_deref().unwrap_or("#");
            assert!(
                schema_pointer_exists(root_doc, fragment),
                "local $ref {ref_value} in {} points to missing fragment",
                schema_path.display()
            );
            return;
        }

        if ref_value.starts_with('#') {
            assert!(
                schema_pointer_exists(root_doc, ref_value),
                "local $ref {ref_value} in {} points to missing fragment",
                schema_path.display()
            );
            return;
        }

        let target_path =
            if let Some(relative) = path_part.strip_prefix("https://shell-quest.local/") {
                normalize_ref_path(&repo_root.join(relative))
            } else if let Some(relative) = path_part.strip_prefix("http://shell-quest.local/") {
                normalize_ref_path(&repo_root.join(relative))
            } else if path_part.contains("://") {
                return;
            } else {
                normalize_ref_path(&schema_path.parent().unwrap_or(repo_root).join(path_part))
            };

        assert!(
            target_path.exists(),
            "$ref {ref_value} in {} resolved to missing file {}",
            schema_path.display(),
            target_path.display()
        );

        if let Some(fragment) = pointer_part.as_deref() {
            let raw = fs::read_to_string(&target_path).unwrap_or_else(|err| {
                panic!("failed reading ref target {}: {err}", target_path.display())
            });
            let target_doc: Value = serde_yaml::from_str(&raw).unwrap_or_else(|err| {
                panic!("failed parsing ref target {}: {err}", target_path.display())
            });
            assert!(
                schema_pointer_exists(&target_doc, fragment),
                "$ref {ref_value} in {} points to missing fragment in {}",
                schema_path.display(),
                target_path.display()
            );
        }
    }

    fn schema_pointer_exists(root: &Value, pointer: &str) -> bool {
        if pointer == "#" {
            return true;
        }
        let Some(pointer) = pointer.strip_prefix("#/") else {
            return false;
        };

        let mut current = root;
        for raw_segment in pointer.split('/') {
            let segment = raw_segment.replace("~1", "/").replace("~0", "~");
            match current {
                Value::Mapping(map) => {
                    let key = Value::String(segment);
                    let Some(next) = map.get(&key) else {
                        return false;
                    };
                    current = next;
                }
                Value::Sequence(seq) => {
                    let Ok(index) = segment.parse::<usize>() else {
                        return false;
                    };
                    let Some(next) = seq.get(index) else {
                        return false;
                    };
                    current = next;
                }
                _ => return false,
            }
        }

        true
    }

    fn normalize_ref_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }
        normalized
    }
}
