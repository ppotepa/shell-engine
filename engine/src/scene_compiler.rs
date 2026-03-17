use crate::scene::{LogicKind, ObjectDocument, Scene, SceneDocument};
use serde_yaml::{Mapping, Value};

pub fn compile_scene_document_with_loader<F>(
    content: &str,
    mut object_loader: F,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut raw = serde_yaml::from_str::<Value>(content)?;
    expand_scene_objects(&mut raw, &mut object_loader);
    let mut compiled_input = serde_yaml::to_string(&raw)?;
    if !compiled_input.ends_with('\n') {
        compiled_input.push('\n');
    }
    let document = serde_yaml::from_str::<SceneDocument>(&compiled_input)?;
    document.compile()
}

fn expand_scene_objects<F>(root: &mut Value, object_loader: &mut F)
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return;
    };
    let object_instances = scene_map
        .get(Value::String("objects".to_string()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    if object_instances.is_empty() {
        return;
    }

    let layers_value = scene_map
        .entry(Value::String("layers".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(scene_layers) = layers_value.as_sequence_mut() else {
        return;
    };

    for instance in object_instances {
        let Some(instance_map) = instance.as_mapping() else {
            continue;
        };
        let Some(use_name) = instance_map
            .get(Value::String("use".to_string()))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let path = if use_name.starts_with('/') {
            use_name.to_string()
        } else {
            format!("/objects/{use_name}.yml")
        };
        let Some(raw_object) = object_loader(&path) else {
            continue;
        };
        let Ok(mut object_value) = serde_yaml::from_str::<Value>(&raw_object) else {
            continue;
        };
        let mut merged_args = Mapping::new();
        if let Some(object_map) = object_value.as_mapping() {
            if let Some(exports) = object_map
                .get(Value::String("exports".to_string()))
                .and_then(Value::as_mapping)
            {
                for (k, v) in exports {
                    merged_args.insert(k.clone(), v.clone());
                }
            }
        }
        if let Some(args) = instance_map
            .get(Value::String("with".to_string()))
            .and_then(Value::as_mapping)
        {
            for (k, v) in args {
                merged_args.insert(k.clone(), v.clone());
            }
        }
        if !merged_args.is_empty() {
            substitute_args(&mut object_value, &merged_args);
        }
        let object_doc = serde_yaml::from_value::<ObjectDocument>(object_value.clone()).ok();
        let native_logic_behavior = object_doc
            .as_ref()
            .and_then(|doc| doc.logic.as_ref())
            .and_then(|logic| {
                if logic.kind == LogicKind::Native {
                    logic.behavior.as_deref()
                } else {
                    None
                }
            });
        let native_logic_params = object_doc
            .as_ref()
            .and_then(|doc| doc.logic.as_ref())
            .map(|logic| logic.params.clone())
            .unwrap_or_default();
        let Some(object_map) = object_value.as_mapping_mut() else {
            continue;
        };

        if let Some(object_layers) = object_map
            .get(Value::String("layers".to_string()))
            .and_then(Value::as_sequence)
        {
            for layer in object_layers {
                let mut layer_value = layer.clone();
                if let Some(behavior_name) = native_logic_behavior {
                    attach_layer_behavior(&mut layer_value, behavior_name, &native_logic_params);
                }
                scene_layers.push(layer_value);
            }
            continue;
        }

        let Some(object_sprites) = object_map
            .get(Value::String("sprites".to_string()))
            .and_then(Value::as_sequence)
        else {
            continue;
        };

        let mut layer = Mapping::new();
        let layer_name = instance_map
            .get(Value::String("id".to_string()))
            .and_then(Value::as_str)
            .or_else(|| object_map.get(Value::String("name".to_string())).and_then(Value::as_str))
            .unwrap_or(use_name);
        layer.insert(
            Value::String("name".to_string()),
            Value::String(layer_name.to_string()),
        );
        layer.insert(
            Value::String("sprites".to_string()),
            Value::Sequence(object_sprites.clone()),
        );
        if let Some(behavior_name) = native_logic_behavior {
            let mut behaviors = Vec::new();
            behaviors.push(build_behavior_spec(behavior_name, &native_logic_params));
            layer.insert(
                Value::String("behaviors".to_string()),
                Value::Sequence(behaviors),
            );
        }
        scene_layers.push(Value::Mapping(layer));
    }
}

fn attach_layer_behavior(layer_value: &mut Value, behavior_name: &str, params: &std::collections::BTreeMap<String, Value>) {
    let Some(layer_map) = layer_value.as_mapping_mut() else {
        return;
    };
    let behavior_value = build_behavior_spec(behavior_name, params);
    let behaviors_entry = layer_map
        .entry(Value::String("behaviors".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(seq) = behaviors_entry.as_sequence_mut() else {
        return;
    };
    seq.push(behavior_value);
}

fn build_behavior_spec(
    behavior_name: &str,
    params: &std::collections::BTreeMap<String, Value>,
) -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("name".to_string()),
        Value::String(behavior_name.to_string()),
    );
    if !params.is_empty() {
        let mut params_map = Mapping::new();
        for (k, v) in params {
            params_map.insert(Value::String(k.clone()), v.clone());
        }
        map.insert(Value::String("params".to_string()), Value::Mapping(params_map));
    }
    Value::Mapping(map)
}

fn substitute_args(value: &mut Value, args: &Mapping) {
    match value {
        Value::String(s) => {
            if let Some(key) = s.strip_prefix('$') {
                if let Some(replacement) = args.get(Value::String(key.to_string())) {
                    *value = replacement.clone();
                }
            }
        }
        Value::Sequence(seq) => {
            for entry in seq {
                substitute_args(entry, args);
            }
        }
        Value::Mapping(map) => {
            let keys: Vec<Value> = map.keys().cloned().collect();
            for key in keys {
                if let Some(v) = map.get_mut(&key) {
                    substitute_args(v, args);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::compile_scene_document_with_loader;

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
}
