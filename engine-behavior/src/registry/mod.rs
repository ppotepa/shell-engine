//! Registry of mod-defined named behaviors loaded from `behaviors/*.yml` at engine startup.

pub mod provider;

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

pub use provider::BehaviorProvider;

/// World resource: holds mod-defined named behaviors loaded from `<mod_source>/behaviors/*.yml`.
#[derive(Default, Clone)]
pub struct ModBehaviorRegistry {
    behaviors: HashMap<String, ModBehavior>,
}

/// A single mod-defined behavior: a name and a Rhai script body.
#[derive(Clone)]
pub struct ModBehavior {
    pub name: String,
    pub script: String,
    pub src: Option<String>,
}

impl ModBehaviorRegistry {
    /// Returns the behavior registered under `name`, if any.
    pub fn get(&self, name: &str) -> Option<&ModBehavior> {
        self.behaviors.get(name)
    }

    /// Inserts a behavior into the registry (keyed by `b.name`).
    pub fn insert(&mut self, b: ModBehavior) {
        self.behaviors.insert(b.name.clone(), b);
    }

    /// Returns `true` if no behaviors have been registered.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct RawModBehavior {
    kind: String,
    name: String,
    #[serde(default)]
    script: Option<String>,
    #[serde(default)]
    src: Option<String>,
}

fn normalize_mod_path(mod_source: &Path, path: &Path) -> String {
    let display_path = path.strip_prefix(mod_source).unwrap_or(path);
    let mut parts = Vec::new();
    for component in display_path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = parts.pop();
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    let normalized = parts.join("/");
    if normalized.starts_with('/') {
        normalized
    } else {
        format!("/{normalized}")
    }
}

fn resolve_behavior_src_path(mod_source: &Path, behavior_path: &Path, script_ref: &str) -> PathBuf {
    if script_ref.starts_with('/') {
        return mod_source.join(script_ref.trim_start_matches('/'));
    }
    if script_ref.starts_with("./") || script_ref.starts_with("../") {
        let base = behavior_path.parent().unwrap_or(mod_source);
        return base.join(script_ref);
    }
    mod_source.join("scripts").join(script_ref)
}

fn parse_behavior_yml(path: &Path, src: &str, mod_source: &Path) -> Result<ModBehavior, String> {
    let raw: RawModBehavior =
        serde_yaml::from_str(src).map_err(|err| format!("yaml parse failed: {err}"))?;
    if raw.kind != "behavior" {
        return Err(format!("expected kind=behavior, got `{}`", raw.kind));
    }

    match (raw.script, raw.src) {
        (Some(script), None) => Ok(ModBehavior {
            name: raw.name,
            script,
            src: Some(normalize_mod_path(mod_source, path)),
        }),
        (None, Some(script_ref)) => {
            let resolved = resolve_behavior_src_path(mod_source, path, script_ref.trim());
            let script = std::fs::read_to_string(&resolved)
                .map_err(|err| format!("failed to read script {}: {err}", resolved.display()))?;
            Ok(ModBehavior {
                name: raw.name,
                script,
                src: Some(normalize_mod_path(mod_source, &resolved)),
            })
        }
        (Some(_), Some(_)) => Err("expected exactly one of `script` or `src`".to_string()),
        (None, None) => Err("missing required `script` or `src` field".to_string()),
    }
}

/// Scans `<mod_source>/behaviors/*.yml`, parses each file, and returns a populated registry
/// alongside any load errors encountered.
pub fn load_mod_behaviors_with_errors(mod_source: &Path) -> (ModBehaviorRegistry, Vec<String>) {
    let mut registry = ModBehaviorRegistry::default();
    let mut errors = Vec::new();
    let behaviors_dir = mod_source.join("behaviors");
    let read_dir = match std::fs::read_dir(&behaviors_dir) {
        Ok(d) => d,
        Err(_) => {
            // Directory simply doesn't exist — this is normal for mods with no behaviors.
            return (registry, errors);
        }
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let is_yml = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "yml" || e == "yaml")
            .unwrap_or(false);
        if !is_yml {
            continue;
        }
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(err) => {
                errors.push(format!("failed to read {}: {err}", path.display()));
                continue;
            }
        };
        match parse_behavior_yml(&path, &src, mod_source) {
            Ok(behavior) => {
                engine_core::logging::info(
                    "mod_behaviors",
                    format!("loaded mod behavior '{}'", behavior.name),
                );
                registry.insert(behavior);
            }
            Err(err) => {
                errors.push(format!("skipped {}: {err}", path.display()));
            }
        }
    }
    (registry, errors)
}

/// Scans `<mod_source>/behaviors/*.yml`, parses each file, and returns a populated registry.
pub fn load_mod_behaviors(mod_source: &Path) -> ModBehaviorRegistry {
    let (registry, errors) = load_mod_behaviors_with_errors(mod_source);
    for err in errors {
        engine_core::logging::warn("mod_behaviors", err);
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_behavior_yml_returns_none_for_wrong_kind() {
        let src = "kind: scene\nname: foo\nscript: |\n  let x = 1;\n";
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("foo.yml");
        assert!(parse_behavior_yml(&path, src, temp.path()).is_err());
    }

    #[test]
    fn parse_behavior_yml_returns_none_for_missing_name() {
        let src = "kind: behavior\nscript: |\n  let x = 1;\n";
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("foo.yml");
        assert!(parse_behavior_yml(&path, src, temp.path()).is_err());
    }

    #[test]
    fn parse_behavior_yml_returns_behavior_for_valid_doc() {
        let src = "kind: behavior\nname: my-anim\nscript: |\n  let x = 1;\n";
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("foo.yml");
        let b = parse_behavior_yml(&path, src, temp.path()).expect("should parse");
        assert_eq!(b.name, "my-anim");
        assert!(b.script.contains("let x = 1;"));
        assert_eq!(b.src.as_deref(), Some("/foo.yml"));
    }

    #[test]
    fn parse_behavior_yml_loads_external_rhai_script() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mod_dir = temp.path();
        let behavior_path = mod_dir.join("behaviors").join("test.yml");
        let script_path = mod_dir.join("behaviors").join("test.rhai");
        std::fs::create_dir_all(behavior_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&script_path, "let x = 1;").expect("write script");

        let src = "kind: behavior\nname: my-anim\nsrc: ./test.rhai\n";
        let b = parse_behavior_yml(&behavior_path, src, mod_dir).expect("should parse");
        assert_eq!(b.name, "my-anim");
        assert_eq!(b.script, "let x = 1;");
        assert_eq!(b.src.as_deref(), Some("/behaviors/test.rhai"));
    }

    #[test]
    fn registry_get_returns_inserted_behavior() {
        let mut reg = ModBehaviorRegistry::default();
        reg.insert(ModBehavior {
            name: "foo".into(),
            script: "1 + 1".into(),
            src: None,
        });
        assert!(reg.get("foo").is_some());
        assert!(reg.get("bar").is_none());
    }
}
