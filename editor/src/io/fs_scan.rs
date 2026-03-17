use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProjectValidation {
    pub valid: bool,
    pub code: &'static str,
    pub message: String,
}

pub fn collect_files(root: &Path, rel: &str, ext: &str) -> Vec<String> {
    let base = root.join(rel);
    let mut out = Vec::new();
    walk(&base, ext, &mut out);
    out.sort();
    out
}

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
    raw.lines().take(3).any(|line| {
        line.contains("$schema=")
            && (line.contains("schemas/scene.schema.yaml")
                || line.contains("schemas/scene-file.schema.yaml")
                || line.contains("schemas/layers-file.schema.yaml")
                || line.contains("schemas/sprites-file.schema.yaml")
                || line.contains("schemas/templates-file.schema.yaml")
                || line.contains("schemas/objects-file.schema.yaml")
                || line.contains("schemas/effect-file.schema.yaml")
                || line.contains("schemas/object.schema.yaml")
                || line.contains("schemas/mod.schema.yaml")
                || line.contains("schemas/font-manifest.schema.yaml")
                || line.contains("shell-quest.local/schemas/scene.schema.yaml")
                || line.contains("shell-quest.local/schemas/scene-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/layers-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/sprites-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/templates-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/objects-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/effect-file.schema.yaml")
                || line.contains("shell-quest.local/schemas/object.schema.yaml")
                || line.contains("shell-quest.local/schemas/mod.schema.yaml")
                || line.contains("shell-quest.local/schemas/font-manifest.schema.yaml"))
    })
}

pub fn collect_game_yaml_files(mod_root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_game_yaml(mod_root, mod_root, &mut out);
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
    if rel_s.starts_with("scenes/") && (rel_s.ends_with(".yml") || rel_s.ends_with(".yaml")) {
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
    use super::{collect_game_yaml_files, collect_schema_project_yml_files};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn schema_scanner_includes_object_schema_files() {
        let temp = tempdir().expect("temp dir");
        let object_yaml = temp.path().join("npc.yml");
        fs::write(
            &object_yaml,
            "# yaml-language-server: $schema=../../schemas/object.schema.yaml\nname: npc\n",
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
    fn game_yaml_scanner_includes_objects_directory() {
        let temp = tempdir().expect("temp dir");
        let objects_dir = temp.path().join("objects");
        fs::create_dir_all(&objects_dir).expect("create objects dir");
        fs::write(objects_dir.join("suzan.yml"), "name: suzan\n").expect("write object");

        let files = collect_game_yaml_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("objects/suzan.yml"));
    }
}
