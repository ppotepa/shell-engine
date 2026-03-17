use super::model::Scene;
use super::template::expand_scene_templates;
use serde::Deserialize;
use serde_yaml::{Mapping, Number, Value};

/// Authored scene document.
/// This remains intentionally loose for now and acts as a compilation boundary
/// between YAML input and runtime `Scene`.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SceneDocument {
    pub raw: serde_yaml::Value,
}

impl SceneDocument {
    pub fn compile(self) -> Result<Scene, serde_yaml::Error> {
        let mut normalized = self.raw;
        normalize_scene_value(&mut normalized);
        serde_yaml::from_value(normalized)
    }
}

fn normalize_scene_value(root: &mut Value) {
    let Some(scene) = root.as_mapping_mut() else {
        return;
    };

    apply_alias(scene, "bg", "bg_colour");
    expand_scene_templates(scene);

    if let Some(stages) = scene.get_mut(Value::String("stages".to_string())) {
        normalize_stages(stages);
    }
    if let Some(layers) = scene.get_mut(Value::String("layers".to_string())) {
        normalize_layers(layers);
    }
    normalize_menu_options(scene);
}

fn normalize_stages(stages: &mut Value) {
    let Some(stages_map) = stages.as_mapping_mut() else {
        return;
    };
    for key in ["on_enter", "on_idle", "on_leave"] {
        if let Some(stage) = stages_map.get_mut(Value::String(key.to_string())) {
            normalize_stage(stage);
        }
    }
}

fn normalize_stage(stage: &mut Value) {
    let Some(stage_map) = stage.as_mapping_mut() else {
        return;
    };
    let Some(steps) = stage_map.get_mut(Value::String("steps".to_string())) else {
        return;
    };
    let Some(steps_seq) = steps.as_sequence_mut() else {
        return;
    };

    for step in steps_seq {
        let Some(step_map) = step.as_mapping_mut() else {
            continue;
        };
        let Some(pause) = step_map.remove(Value::String("pause".to_string())) else {
            continue;
        };
        if step_map.contains_key(Value::String("effects".to_string())) {
            continue;
        }
        let duration_ms = parse_duration_ms(&pause).unwrap_or(0);
        step_map.insert(
            Value::String("duration".to_string()),
            Value::Number(Number::from(duration_ms)),
        );
        step_map.insert(
            Value::String("effects".to_string()),
            Value::Sequence(Vec::new()),
        );
    }
}

fn normalize_layers(layers: &mut Value) {
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
        normalize_sprites(sprites);
    }
}

fn normalize_sprites(sprites: &mut Value) {
    let Some(sprite_seq) = sprites.as_sequence_mut() else {
        return;
    };
    for sprite in sprite_seq {
        let Some(sprite_map) = sprite.as_mapping_mut() else {
            continue;
        };
        apply_alias(sprite_map, "fg", "fg_colour");
        apply_alias(sprite_map, "bg", "bg_colour");
        apply_at_anchor(sprite_map);

        if matches!(
            sprite_map
                .get(Value::String("type".to_string()))
                .and_then(Value::as_str),
            Some("grid")
        ) {
            if let Some(children) = sprite_map.get_mut(Value::String("children".to_string())) {
                normalize_sprites(children);
            }
        }
    }
}

fn normalize_menu_options(scene: &mut Mapping) {
    for key in ["menu-options", "menu_options"] {
        let Some(options) = scene.get_mut(Value::String(key.to_string())) else {
            continue;
        };
        let Some(seq) = options.as_sequence_mut() else {
            continue;
        };
        for option in seq {
            let Some(option_map) = option.as_mapping_mut() else {
                continue;
            };
            let Some(to_value) = option_map.get(Value::String("to".to_string())).cloned() else {
                continue;
            };
            option_map
                .entry(Value::String("scene".to_string()))
                .or_insert_with(|| to_value.clone());
            option_map
                .entry(Value::String("next".to_string()))
                .or_insert(to_value);
        }
    }
}

fn apply_alias(map: &mut Mapping, from: &str, to: &str) {
    let from_key = Value::String(from.to_string());
    let to_key = Value::String(to.to_string());
    if map.contains_key(&to_key) {
        return;
    }
    if let Some(value) = map.get(&from_key).cloned() {
        map.insert(to_key, value);
    }
}

fn apply_at_anchor(map: &mut Mapping) {
    let Some(anchor) = map
        .get(Value::String("at".to_string()))
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
    else {
        return;
    };
    let (ax, ay) = match anchor.as_str() {
        "cc" => ("center", "center"),
        "ct" => ("center", "top"),
        "cb" => ("center", "bottom"),
        "lc" => ("left", "center"),
        "rc" => ("right", "center"),
        "lt" => ("left", "top"),
        "lb" => ("left", "bottom"),
        "rt" => ("right", "top"),
        "rb" => ("right", "bottom"),
        _ => return,
    };

    map.entry(Value::String("align_x".to_string()))
        .or_insert_with(|| Value::String(ax.to_string()));
    map.entry(Value::String("align_y".to_string()))
        .or_insert_with(|| Value::String(ay.to_string()));
}

fn parse_duration_ms(value: &Value) -> Option<u64> {
    if let Some(ms) = value.as_u64() {
        return Some(ms);
    }
    if let Some(text) = value.as_str() {
        let trimmed = text.trim().to_ascii_lowercase();
        if let Some(ms) = trimmed.strip_suffix("ms") {
            return ms.trim().parse::<u64>().ok();
        }
        if let Some(sec) = trimmed.strip_suffix('s') {
            return sec.trim().parse::<u64>().ok().map(|v| v.saturating_mul(1000));
        }
        return trimmed.parse::<u64>().ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::SceneDocument;

    #[test]
    fn compiles_scene_with_aliases_and_pause_shorthand() {
        let raw = r#"
id: menu
title: Menu
bg: black
stages:
  on_enter:
    steps:
      - pause: 2s
layers:
  - sprites:
      - type: text
        content: START
        at: cc
        fg: white
menu-options:
  - key: "1"
    to: next-scene
"#;

        let doc: SceneDocument = serde_yaml::from_str(raw).expect("document");
        let scene = doc.compile().expect("scene");
        assert!(scene.bg_colour.is_some());
        assert_eq!(scene.stages.on_enter.steps[0].duration, Some(2000));
        assert!(scene.stages.on_enter.steps[0].effects.is_empty());
        assert_eq!(scene.menu_options[0].scene.as_deref(), Some("next-scene"));
        assert_eq!(scene.menu_options[0].next, "next-scene");
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text {
                align_x,
                align_y,
                fg_colour,
                ..
            } => {
                assert!(matches!(
                    align_x,
                    Some(crate::scene::HorizontalAlign::Center)
                ));
                assert!(matches!(
                    align_y,
                    Some(crate::scene::VerticalAlign::Center)
                ));
                assert!(fg_colour.is_some());
            }
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn expands_scene_templates_with_args() {
        let raw = r#"
id: menu
title: Menu
templates:
  menu-item:
    type: text
    content: "$label"
    at: cc
layers:
  - sprites:
      - use: menu-item
        args:
          label: START
        y: 2
"#;

        let doc: SceneDocument = serde_yaml::from_str(raw).expect("document");
        let scene = doc.compile().expect("scene");
        match &scene.layers[0].sprites[0] {
            crate::scene::Sprite::Text {
                content,
                y,
                align_x,
                align_y,
                ..
            } => {
                assert_eq!(content, "START");
                assert_eq!(*y, 2);
                assert!(matches!(
                    align_x,
                    Some(crate::scene::HorizontalAlign::Center)
                ));
                assert!(matches!(
                    align_y,
                    Some(crate::scene::VerticalAlign::Center)
                ));
            }
            _ => panic!("expected text sprite"),
        }
    }
}
