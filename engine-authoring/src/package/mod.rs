//! Scene-package assembly helpers.
//!
//! This module owns the authored scene-package merge semantics independently
//! from filesystem or zip transport details. Callers provide the root manifest
//! plus already-loaded partial files, and this module applies the shared merge
//! order used by the authoring pipeline.

use serde_yaml::{Mapping, Value};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// YAML file content already loaded from a scene package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageYamlFile {
    /// Logical source path used in diagnostics.
    pub path: String,
    /// Raw YAML text for this package fragment.
    pub content: String,
}

impl PackageYamlFile {
    /// Creates a package fragment record.
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

/// Grouped authored partial files that belong to one scene package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScenePackagePartials {
    /// `layers/*.yml` sequence fragments appended into `layers:`.
    pub layers: Vec<PackageYamlFile>,
    /// `templates/*.yml` mapping fragments merged into `templates:`.
    pub templates: Vec<PackageYamlFile>,
    /// `objects/*.yml` sequence fragments appended into `objects:`.
    pub objects: Vec<PackageYamlFile>,
}

/// Error raised while assembling a scene package from authored YAML fragments.
#[derive(Debug)]
pub struct PackageError {
    path: String,
    source: serde_yaml::Error,
}

impl PackageError {
    fn invalid_yaml(path: impl Into<String>, source: serde_yaml::Error) -> Self {
        Self {
            path: path.into(),
            source,
        }
    }

    /// Consumes the error into its logical source path and YAML cause.
    pub fn into_parts(self) -> (String, serde_yaml::Error) {
        (self.path, self.source)
    }
}

impl Display for PackageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid scene package YAML in {}: {}",
            self.path, self.source
        )
    }
}

impl Error for PackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Assembles one authored scene package into a single YAML document.
///
/// Merge order is fixed and matches the current package contract:
/// root `scene.yml` -> `layers/` -> `templates/` -> `objects/`.
///
/// Sequence partials for `layers` and `objects` are root-optional package
/// inputs: if the root scene already defines the key explicitly, the authored
/// root wins and package partials are not appended automatically.
pub fn assemble_scene_package(
    root_content: &str,
    root_path: &str,
    partials: &ScenePackagePartials,
) -> Result<String, PackageError> {
    let mut root = parse_yaml_value(root_content, root_path)?;
    append_sequence_partials_if_absent(&mut root, "layers", &partials.layers)?;
    merge_mapping_partials(&mut root, "templates", &partials.templates)?;
    append_sequence_partials_if_absent(&mut root, "objects", &partials.objects)?;
    to_yaml_string(&root, root_path)
}

fn append_sequence_partials_if_absent(
    root: &mut Value,
    key: &str,
    partials: &[PackageYamlFile],
) -> Result<(), PackageError> {
    if root_has_key(root, key) {
        return Ok(());
    }
    let mut entries = Vec::new();
    for partial in partials {
        let value = parse_yaml_value(&partial.content, &partial.path)?;
        if let Some(seq) = value.as_sequence() {
            entries.extend(seq.iter().cloned());
        }
    }
    append_sequence_entries(root, key, entries);
    Ok(())
}

fn root_has_key(root: &Value, key: &str) -> bool {
    root.as_mapping()
        .is_some_and(|map| map.contains_key(Value::String(key.to_string())))
}

fn merge_mapping_partials(
    root: &mut Value,
    key: &str,
    partials: &[PackageYamlFile],
) -> Result<(), PackageError> {
    let mut entries = Mapping::new();
    for partial in partials {
        let value = parse_yaml_value(&partial.content, &partial.path)?;
        if let Some(map) = value.as_mapping() {
            for (k, v) in map {
                entries.insert(k.clone(), v.clone());
            }
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

fn parse_yaml_value(raw: &str, path: &str) -> Result<Value, PackageError> {
    serde_yaml::from_str(raw).map_err(|source| PackageError::invalid_yaml(path, source))
}

fn to_yaml_string(value: &Value, path: &str) -> Result<String, PackageError> {
    let mut out =
        serde_yaml::to_string(value).map_err(|source| PackageError::invalid_yaml(path, source))?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{assemble_scene_package, PackageYamlFile, ScenePackagePartials};

    #[test]
    fn assembles_scene_root_with_layers_templates_and_objects() {
        let root = r#"
id: intro
title: Intro
next: null
"#;
        let partials = ScenePackagePartials {
            layers: vec![PackageYamlFile::new(
                "/scenes/intro/layers/base.yml",
                r#"
- name: base
  sprites: []
"#,
            )],
            templates: vec![PackageYamlFile::new(
                "/scenes/intro/templates/common.yml",
                r#"
title:
  type: text
  content: HELLO
"#,
            )],
            objects: vec![PackageYamlFile::new(
                "/scenes/intro/objects/banner.yml",
                r#"
- use: banner
"#,
            )],
        };

        let assembled =
            assemble_scene_package(root, "/scenes/intro/scene.yml", &partials).expect("assemble");

        assert!(assembled.contains("layers:"));
        assert!(assembled.contains("templates:"));
        assert!(assembled.contains("objects:"));
    }

    #[test]
    fn reports_partial_path_for_invalid_yaml() {
        let partials = ScenePackagePartials {
            layers: vec![PackageYamlFile::new(
                "/scenes/intro/layers/base.yml",
                "layers: [",
            )],
            ..ScenePackagePartials::default()
        };

        let error = assemble_scene_package(
            "id: intro\ntitle: Intro\nnext: null\n",
            "/scenes/intro/scene.yml",
            &partials,
        )
        .expect_err("invalid partial");

        let (path, _) = error.into_parts();
        assert_eq!(path, "/scenes/intro/layers/base.yml");
    }

    #[test]
    fn explicit_root_layers_win_over_package_layer_partials() {
        let root = r#"
id: intro
title: Intro
layers:
  - ref: main
next: null
"#;
        let partials = ScenePackagePartials {
            layers: vec![PackageYamlFile::new(
                "/scenes/intro/layers/base.yml",
                r#"
- name: base
  sprites: []
"#,
            )],
            ..ScenePackagePartials::default()
        };

        let assembled =
            assemble_scene_package(root, "/scenes/intro/scene.yml", &partials).expect("assemble");

        assert!(assembled.contains("ref: main"));
        assert!(!assembled.contains("name: base"));
    }

    #[test]
    fn explicit_root_objects_win_over_package_object_partials() {
        let root = r#"
id: intro
title: Intro
objects:
  - ref: banner
next: null
"#;
        let partials = ScenePackagePartials {
            objects: vec![PackageYamlFile::new(
                "/scenes/intro/objects/banner.yml",
                r#"
- use: sample-banner
"#,
            )],
            ..ScenePackagePartials::default()
        };

        let assembled =
            assemble_scene_package(root, "/scenes/intro/scene.yml", &partials).expect("assemble");

        assert!(assembled.contains("ref: banner"));
        assert!(!assembled.contains("sample-banner"));
    }
}
