//! Scene-local sprite template expansion for authored YAML documents.

use serde_yaml::{Mapping, Value};

/// Expands scene-local sprite templates inside every authored layer sprite
/// list before typed deserialization.
pub fn expand_scene_templates(scene: &mut Mapping) {
    let templates = collect_templates(scene);
    if templates.is_empty() {
        return;
    }
    if let Some(layers) = scene.get_mut(Value::String("layers".to_string())) {
        expand_templates_in_layers(layers, &templates);
    }
}

fn collect_templates(scene: &Mapping) -> Vec<(String, Mapping)> {
    let Some(raw_templates) = scene.get(Value::String("templates".to_string())) else {
        return Vec::new();
    };
    let Some(template_map) = raw_templates.as_mapping() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(template_map.len());
    for (k, v) in template_map {
        let Some(name) = k.as_str() else {
            continue;
        };
        let Some(def_map) = v.as_mapping() else {
            continue;
        };
        let template_sprite = def_map
            .get(Value::String("sprite".to_string()))
            .and_then(Value::as_mapping)
            .cloned()
            .unwrap_or_else(|| def_map.clone());
        out.push((name.to_string(), template_sprite));
    }
    out
}

fn expand_templates_in_layers(layers: &mut Value, templates: &[(String, Mapping)]) {
    let Some(layer_seq) = layers.as_sequence_mut() else {
        return;
    };
    for layer in layer_seq {
        let Some(layer_map) = layer.as_mapping_mut() else {
            continue;
        };
        let Some(sprites) = layer_map.get_mut(Value::String("sprites".to_string())) else {
            continue;
        };
        expand_templates_in_sprites(sprites, templates);
    }
}

fn expand_templates_in_sprites(sprites: &mut Value, templates: &[(String, Mapping)]) {
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        return;
    };
    for sprite in sprite_seq {
        expand_template_in_sprite(sprite, templates);
    }
}

fn expand_template_in_sprite(sprite: &mut Value, templates: &[(String, Mapping)]) {
    let mut replaced = false;
    if let Some(sprite_map) = sprite.as_mapping() {
        if let Some(template_name) = sprite_map
            .get(Value::String("use".to_string()))
            .and_then(Value::as_str)
        {
            if let Some((_, template)) = templates.iter().find(|(name, _)| name == template_name) {
                let mut merged = template.clone();
                for (k, v) in sprite_map {
                    if k.as_str() == Some("use") || k.as_str() == Some("args") {
                        continue;
                    }
                    merged.insert(k.clone(), v.clone());
                }
                if let Some(args) = sprite_map
                    .get(Value::String("args".to_string()))
                    .and_then(Value::as_mapping)
                {
                    let mut expanded = Value::Mapping(merged.clone());
                    substitute_args(&mut expanded, args);
                    if let Some(updated) = expanded.as_mapping() {
                        merged = updated.clone();
                    }
                }
                *sprite = Value::Mapping(merged);
                replaced = true;
            }
        }
    }

    if !replaced && !sprite.is_mapping() {
        return;
    }
    if let Some(map) = sprite.as_mapping_mut() {
        if let Some(children) = map.get_mut(Value::String("children".to_string())) {
            expand_templates_in_sprites(children, templates);
        }
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
