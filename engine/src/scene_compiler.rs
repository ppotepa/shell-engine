use crate::scene::{Scene, SceneDocument};
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
        if let Some(args) = instance_map
            .get(Value::String("with".to_string()))
            .and_then(Value::as_mapping)
        {
            substitute_args(&mut object_value, args);
        }
        let Some(object_map) = object_value.as_mapping_mut() else {
            continue;
        };

        if let Some(object_layers) = object_map
            .get(Value::String("layers".to_string()))
            .and_then(Value::as_sequence)
        {
            for layer in object_layers {
                scene_layers.push(layer.clone());
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
        scene_layers.push(Value::Mapping(layer));
    }
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
}
