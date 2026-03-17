//! Compilation helpers that turn authored scene YAML into the runtime `Scene`
//! model, including object expansion before typed deserialization.

use crate::scene::Scene;

/// Compiles authored scene YAML into a runtime [`Scene`] using the default
/// root path when resolving referenced object documents.
#[allow(dead_code)]
pub fn compile_scene_document_with_loader<F>(
    content: &str,
    object_loader: F,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    engine_authoring::compile::compile_scene_document_with_loader(content, object_loader)
}

/// Compiles authored scene YAML into a runtime [`Scene`].
///
/// # Purpose
///
/// This is the authored-scene entry point used by repositories after they have
/// assembled any scene package fragments. It expands `objects:` references,
/// merges authored overrides from `with:`, and then hands the normalized YAML to
/// [`SceneDocument`] for the final authored-to-runtime conversion.
///
/// `scene_source_path` is used to resolve relative object references inside a
/// scene package.
pub fn compile_scene_document_with_loader_and_source<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    engine_authoring::compile::compile_scene_document_with_loader_and_source(
        content,
        scene_source_path,
        object_loader,
    )
}

#[cfg(test)]
mod tests {
    use super::{compile_scene_document_with_loader, compile_scene_document_with_loader_and_source};

    #[test]
    fn compiles_legacy_scene_yaml_into_runtime_scene() {
        let raw = r#"
id: intro
title: Intro
bg_colour: black
layers: []
"#;
        let scene =
            compile_scene_document_with_loader(raw, |_path| None).expect("scene should compile");
        assert_eq!(scene.id, "intro");
        assert_eq!(scene.title, "Intro");
    }

    #[test]
    fn expands_object_instances_into_scene_layers() {
        let scene_raw = r#"
id: playground
title: Playground
layers: []
objects:
  - use: suzan
    id: monkey-a
    with:
      label: "MONKEY"
"#;
        let object_raw = r#"
name: suzan
exports:
  label: DEFAULT
sprites:
  - type: text
    content: "$label"
    at: cc
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/suzan.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "monkey-a");
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text { content, .. } => assert_eq!(content, "MONKEY"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn uses_object_exports_as_default_substitution_values() {
        let scene_raw = r#"
id: playground
title: Playground
layers: []
objects:
  - use: suzan
"#;
        let object_raw = r#"
name: suzan
exports:
  label: DEFAULT
sprites:
  - type: text
    content: "$label"
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/suzan.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text { content, .. } => assert_eq!(content, "DEFAULT"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn maps_object_native_logic_to_layer_behaviors() {
        let scene_raw = r#"
id: playground
title: Playground
layers: []
objects:
  - use: suzan
    id: monkey-a
"#;
        let object_raw = r#"
name: suzan
logic:
  type: native
  behavior: bob
  params:
    amplitude_y: 2
sprites:
  - type: text
    content: "M"
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/suzan.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].behaviors.len(), 1);
        assert_eq!(scene.layers[0].behaviors[0].name, "bob");
        assert_eq!(scene.layers[0].behaviors[0].params.amplitude_y, Some(2));
    }

    #[test]
    fn resolves_relative_object_refs_from_scene_package_path() {
        let scene_raw = r#"
id: intro
title: Intro
layers: []
objects:
  - use: ../shared/objects/banner.yml
next: null
"#;
        let object_raw = r#"
name: banner
sprites:
  - type: text
    content: SHARED
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/scenes/shared/objects/banner.yml" {
                    Some(object_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn ref_and_as_syntax_expands_same_as_use_and_id() {
        let scene_raw = r#"
id: playground
title: Playground
layers: []
objects:
  - ref: suzan
    as: monkey-b
    with:
      label: "MONKEY"
    state:
      alive: true
    tags:
      - enemy
"#;
        let object_raw = r#"
name: suzan
exports:
  label: DEFAULT
sprites:
  - type: text
    content: "$label"
    at: cc
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/suzan.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "monkey-b");
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text { content, .. } => assert_eq!(content, "MONKEY"),
            _ => panic!("expected text sprite"),
        }
    }
}
