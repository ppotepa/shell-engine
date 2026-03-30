use crate::document::LogicKind;
use serde::de::Error as _;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct SceneLogicSpec {
    #[serde(default, rename = "type", alias = "kind")]
    pub(super) kind: LogicKind,
    #[serde(default)]
    pub(super) behavior: Option<String>,
    #[serde(default)]
    pub(super) src: Option<String>,
    #[serde(default)]
    pub(super) params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct SceneScriptController {
    #[serde(default)]
    pub(super) behavior: Option<String>,
    #[serde(default)]
    pub(super) params: BTreeMap<String, Value>,
}

pub(super) fn expand_scene_logic<F>(
    root: &mut Value,
    scene_source_path: &str,
    asset_loader: &mut F,
) -> Result<(), serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return Ok(());
    };
    let logic_value = scene_map.get(Value::String("logic".to_string())).cloned();
    if logic_value.is_none() {
        // Explicit `logic:` block is required for scene logic wiring.
        return Ok(());
    }
    let logic = serde_yaml::from_value::<SceneLogicSpec>(
        logic_value.expect("logic_value checked above"),
    )
    .map_err(|err| serde_yaml::Error::custom(format!("failed to parse logic block: {err}")))?;

    match logic.kind {
        LogicKind::Native => {
            if let Some(behavior_name) = logic.behavior.as_deref() {
                attach_scene_behavior(scene_map, behavior_name, &logic.params);
            }
        }
        LogicKind::Script => {
            let Some(src) = logic
                .src
                .as_deref()
                .map(str::trim)
                .filter(|src| !src.is_empty())
            else {
                return Err(serde_yaml::Error::custom(
                    "logic.kind=script requires explicit logic.src",
                ));
            };
            let path = resolve_script_ref_path(scene_source_path, src);
            let Some(raw_script) = asset_loader(&path) else {
                return Err(serde_yaml::Error::custom(format!(
                    "logic.src '{}' resolved to '{}', but file was not found",
                    src, path
                )));
            };
            let mut merged_params = logic.params.clone();
            if is_rhai_path(&path) {
                merged_params.insert("src".to_string(), Value::String(path));
                merged_params.insert("script".to_string(), Value::String(raw_script));
                let behavior_name = logic
                    .behavior
                    .clone()
                    .unwrap_or_else(|| "rhai-script".to_string());
                attach_scene_behavior(scene_map, &behavior_name, &merged_params);
                return Ok(());
            }
            let controller =
                serde_yaml::from_str::<SceneScriptController>(&raw_script).map_err(|err| {
                    serde_yaml::Error::custom(format!(
                        "failed to parse logic.src '{}': {err}",
                        path
                    ))
                })?;
            for (k, v) in controller.params {
                merged_params.insert(k, v);
            }
            let Some(behavior_name) = logic.behavior.clone().or(controller.behavior) else {
                return Err(serde_yaml::Error::custom(format!(
                    "logic.src '{}' must declare a behavior or be paired with logic.behavior",
                    path
                )));
            };
            attach_scene_behavior(scene_map, &behavior_name, &merged_params);
        }
        LogicKind::Graph => {
            return Err(serde_yaml::Error::custom(
                "logic.kind=graph is experimental and not supported by runtime compiler",
            ));
        }
    }
    Ok(())
}

pub(super) fn expand_scene_stages_ref<F>(
    root: &mut Value,
    scene_source_path: &str,
    asset_loader: &mut F,
) -> Result<(), serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return Ok(());
    };
    let Some(stages_ref) = scene_map
        .get(Value::String("stages-ref".to_string()))
        .or_else(|| scene_map.get(Value::String("stages_ref".to_string())))
        .and_then(Value::as_str)
        .map(str::trim)
    else {
        return Ok(());
    };
    if stages_ref.is_empty() {
        return Err(serde_yaml::Error::custom("stages-ref cannot be empty"));
    }
    let path = resolve_stages_ref_path(scene_source_path, stages_ref);
    let Some(raw_stages) = asset_loader(&path) else {
        return Err(serde_yaml::Error::custom(format!(
            "stages-ref '{}' resolved to '{}', but file was not found",
            stages_ref, path
        )));
    };
    let parsed = serde_yaml::from_str::<Value>(&raw_stages).map_err(|err| {
        serde_yaml::Error::custom(format!("failed to parse stages-ref '{}': {err}", path))
    })?;
    let referenced = extract_referenced_stages_map(&parsed).ok_or_else(|| {
        serde_yaml::Error::custom(format!(
            "stages-ref '{}' must resolve to a mapping (or mapping with top-level 'stages')",
            path
        ))
    })?;
    merge_scene_stages(scene_map, referenced);
    scene_map.remove(Value::String("stages-ref".to_string()));
    scene_map.remove(Value::String("stages_ref".to_string()));
    Ok(())
}

pub(super) fn extract_referenced_stages_map(value: &Value) -> Option<&Mapping> {
    let map = value.as_mapping()?;
    if let Some(stages) = map
        .get(Value::String("stages".to_string()))
        .and_then(Value::as_mapping)
    {
        return Some(stages);
    }
    Some(map)
}

pub(super) fn merge_scene_stages(scene_map: &mut Mapping, referenced: &Mapping) {
    let stages = scene_map
        .entry(Value::String("stages".to_string()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let Some(stages_map) = stages.as_mapping_mut() else {
        return;
    };
    for stage_key in ["on_enter", "on_idle", "on_leave"] {
        let key = Value::String(stage_key.to_string());
        let Some(referenced_stage) = referenced.get(&key) else {
            continue;
        };
        if !stages_map.contains_key(&key) {
            stages_map.insert(key, referenced_stage.clone());
            continue;
        }
        let Some(existing_stage_map) = stages_map.get_mut(&key).and_then(Value::as_mapping_mut)
        else {
            continue;
        };
        let Some(referenced_stage_map) = referenced_stage.as_mapping() else {
            continue;
        };
        for (k, v) in referenced_stage_map {
            if !existing_stage_map.contains_key(k) {
                existing_stage_map.insert(k.clone(), v.clone());
            }
        }
    }
}

pub(super) fn resolve_script_ref_path(scene_source_path: &str, script_ref: &str) -> String {
    if script_ref.starts_with('/') {
        return super::normalize_mod_path(script_ref);
    }
    if script_ref.starts_with("./") || script_ref.starts_with("../") {
        let scene_dir = super::parent_dir(scene_source_path);
        return super::normalize_mod_path(&format!("{scene_dir}/{script_ref}"));
    }
    super::normalize_mod_path(&format!("/scripts/{script_ref}"))
}

pub(super) fn is_rhai_path(path: &str) -> bool {
    path.ends_with(".rhai")
}

pub(super) fn resolve_stages_ref_path(scene_source_path: &str, reference: &str) -> String {
    if reference.starts_with('/') {
        return super::normalize_mod_path(reference);
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        let scene_dir = super::parent_dir(scene_source_path);
        return super::normalize_mod_path(&format!("{scene_dir}/{reference}"));
    }
    let trimmed = reference.trim_start_matches('/');
    let has_yaml_ext = trimmed.ends_with(".yml") || trimmed.ends_with(".yaml");
    if has_yaml_ext {
        if trimmed.starts_with("stages/") {
            return super::normalize_mod_path(&format!("/{trimmed}"));
        }
        return super::normalize_mod_path(&format!("/stages/{trimmed}"));
    }
    super::normalize_mod_path(&format!("/stages/{trimmed}.yml"))
}

pub(super) fn attach_scene_behavior(
    scene_map: &mut Mapping,
    behavior_name: &str,
    params: &BTreeMap<String, Value>,
) {
    let behavior_value = build_behavior_spec(behavior_name, params);
    let behaviors_entry = scene_map
        .entry(Value::String("behaviors".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(seq) = behaviors_entry.as_sequence_mut() else {
        return;
    };
    seq.push(behavior_value);
}

pub(super) fn attach_layer_behavior(
    layer_value: &mut Value,
    behavior_name: &str,
    params: &BTreeMap<String, Value>,
) {
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

pub(super) fn build_behavior_spec(
    behavior_name: &str,
    params: &BTreeMap<String, Value>,
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
        map.insert(
            Value::String("params".to_string()),
            Value::Mapping(params_map),
        );
    }
    Value::Mapping(map)
}
