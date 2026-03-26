//! Registry of mod-defined named behaviors loaded from `behaviors/*.yml` at engine startup.

pub mod provider;

use std::collections::HashMap;

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

/// Parses a single behavior YAML document.
/// Expected fields: `kind: behavior`, `name: String`, `script: String`.
/// Returns `None` if the document is missing required fields or has wrong `kind`.
fn parse_behavior_yml(src: &str) -> Option<ModBehavior> {
    let value: serde_yaml::Value = serde_yaml::from_str(src).ok()?;
    let map = value.as_mapping()?;
    let kind = map.get("kind")?.as_str()?;
    if kind != "behavior" {
        return None;
    }
    let name = map.get("name")?.as_str()?.to_string();
    let script = map.get("script")?.as_str()?.to_string();
    Some(ModBehavior { name, script })
}

/// Scans `<mod_source>/behaviors/*.yml`, parses each file, and returns a populated registry.
pub fn load_mod_behaviors(mod_source: &std::path::Path) -> ModBehaviorRegistry {
    let mut registry = ModBehaviorRegistry::default();
    let behaviors_dir = mod_source.join("behaviors");
    let read_dir = match std::fs::read_dir(&behaviors_dir) {
        Ok(d) => d,
        Err(_) => {
            // Directory simply doesn't exist — this is normal for mods with no behaviors.
            return registry;
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
                engine_core::logging::warn(
                    "mod_behaviors",
                    format!("failed to read {}: {err}", path.display()),
                );
                continue;
            }
        };
        match parse_behavior_yml(&src) {
            Some(behavior) => {
                engine_core::logging::info(
                    "mod_behaviors",
                    format!("loaded mod behavior '{}'", behavior.name),
                );
                registry.insert(behavior);
            }
            None => {
                engine_core::logging::warn(
                    "mod_behaviors",
                    format!(
                        "skipped {}: missing or invalid kind/name/script",
                        path.display()
                    ),
                );
            }
        }
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_behavior_yml_returns_none_for_wrong_kind() {
        let src = "kind: scene\nname: foo\nscript: |\n  let x = 1;\n";
        assert!(parse_behavior_yml(src).is_none());
    }

    #[test]
    fn parse_behavior_yml_returns_none_for_missing_name() {
        let src = "kind: behavior\nscript: |\n  let x = 1;\n";
        assert!(parse_behavior_yml(src).is_none());
    }

    #[test]
    fn parse_behavior_yml_returns_behavior_for_valid_doc() {
        let src = "kind: behavior\nname: my-anim\nscript: |\n  let x = 1;\n";
        let b = parse_behavior_yml(src).expect("should parse");
        assert_eq!(b.name, "my-anim");
        assert!(b.script.contains("let x = 1;"));
    }

    #[test]
    fn registry_get_returns_inserted_behavior() {
        let mut reg = ModBehaviorRegistry::default();
        reg.insert(ModBehavior {
            name: "foo".into(),
            script: "1 + 1".into(),
        });
        assert!(reg.get("foo").is_some());
        assert!(reg.get("bar").is_none());
    }
}
