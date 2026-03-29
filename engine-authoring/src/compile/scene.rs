//! Compilation helpers that turn authored scene YAML into the runtime `Scene`
//! model, including object expansion before typed deserialization.

use super::cutscene::{expand_scene_cutscene_ref_with_filters, CutsceneFilterRegistry};
use crate::document::{LogicKind, ObjectDocument, SceneDocument};
use engine_core::scene::Scene;
use serde::de::Error as _;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

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
    compile_scene_document_with_loader_and_source(content, "/", object_loader)
}

/// Compiles authored scene YAML into a runtime [`Scene`].
///
/// # Purpose
///
/// This is the authored-scene entry point used by repositories after they have
/// assembled any scene package fragments. It resolves `layers[].ref` references,
/// expands `objects:` and `layer.objects` references, merges authored overrides
/// from `with:`, and then hands the normalized YAML to [`SceneDocument`] for the
/// final authored-to-runtime conversion.
///
/// `scene_source_path` is used to resolve relative object references inside a
/// scene package. Scene logic wiring is explicit: `logic.kind: script` requires
/// `logic.src`, and effect preset reuse resolves only from declared presets.
pub fn compile_scene_document_with_loader_and_source<F>(
    content: &str,
    scene_source_path: &str,
    object_loader: F,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let filters = CutsceneFilterRegistry::with_builtin_filters();
    compile_scene_document_with_loader_and_source_and_filters(
        content,
        scene_source_path,
        object_loader,
        &filters,
    )
}

/// Compiles authored scene YAML into a runtime [`Scene`] with an explicit
/// cutscene filter registry.
pub fn compile_scene_document_with_loader_and_source_and_filters<F>(
    content: &str,
    scene_source_path: &str,
    mut object_loader: F,
    cutscene_filters: &CutsceneFilterRegistry,
) -> Result<Scene, serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut raw = serde_yaml::from_str::<Value>(content)?;
    expand_scene_layer_refs(&mut raw, scene_source_path, &mut object_loader);
    expand_scene_stages_ref(&mut raw, scene_source_path, &mut object_loader)?;
    expand_scene_effect_presets_ref(&mut raw, scene_source_path, &mut object_loader)?;
    expand_scene_objects(&mut raw, scene_source_path, &mut object_loader);
    expand_layer_objects(&mut raw, scene_source_path, &mut object_loader);
    expand_scene_cutscene_ref_with_filters(
        &mut raw,
        scene_source_path,
        &mut object_loader,
        cutscene_filters,
    )?;
    expand_scene_effect_presets(&mut raw)?;
    expand_scene_logic(&mut raw, scene_source_path, &mut object_loader)?;
    let mut compiled_input = serde_yaml::to_string(&raw)?;
    if !compiled_input.ends_with('\n') {
        compiled_input.push('\n');
    }
    let document = serde_yaml::from_str::<SceneDocument>(&compiled_input)?;
    document.compile()
}

#[derive(Debug, Clone, Deserialize, Default)]
struct SceneLogicSpec {
    #[serde(default, rename = "type", alias = "kind")]
    kind: LogicKind,
    #[serde(default)]
    behavior: Option<String>,
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct SceneScriptController {
    #[serde(default)]
    behavior: Option<String>,
    #[serde(default)]
    params: BTreeMap<String, Value>,
}

fn expand_scene_logic<F>(
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

fn expand_scene_stages_ref<F>(
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

fn extract_referenced_stages_map(value: &Value) -> Option<&Mapping> {
    let map = value.as_mapping()?;
    if let Some(stages) = map
        .get(Value::String("stages".to_string()))
        .and_then(Value::as_mapping)
    {
        return Some(stages);
    }
    Some(map)
}

fn merge_scene_stages(scene_map: &mut Mapping, referenced: &Mapping) {
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

fn expand_scene_effect_presets(root: &mut Value) -> Result<(), serde_yaml::Error> {
    let presets = {
        let Some(scene_map) = root.as_mapping() else {
            return Ok(());
        };
        resolve_scene_effect_presets(scene_map)?
    };
    let Some(presets) = presets else {
        return Ok(());
    };
    expand_effect_presets_in_value(root, &presets)?;
    if let Some(scene_map) = root.as_mapping_mut() {
        scene_map.remove(Value::String("effect-presets".to_string()));
        scene_map.remove(Value::String("effect_presets".to_string()));
    }
    Ok(())
}

fn expand_scene_effect_presets_ref<F>(
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

    let has_canonical = scene_map.contains_key(Value::String("effect-presets-ref".to_string()));
    let has_legacy = scene_map.contains_key(Value::String("effect_presets_ref".to_string()));
    if has_canonical && has_legacy {
        return Err(serde_yaml::Error::custom(
            "scene defines both 'effect-presets-ref' and 'effect_presets_ref'; use only one",
        ));
    }

    let Some(reference) = scene_map
        .get(Value::String("effect-presets-ref".to_string()))
        .or_else(|| scene_map.get(Value::String("effect_presets_ref".to_string())))
        .and_then(Value::as_str)
        .map(str::trim)
    else {
        return Ok(());
    };

    if reference.is_empty() {
        return Err(serde_yaml::Error::custom(
            "effect-presets-ref cannot be empty",
        ));
    }

    let path = resolve_effect_presets_ref_path(scene_source_path, reference);
    let Some(raw_presets) = asset_loader(&path) else {
        return Err(serde_yaml::Error::custom(format!(
            "effect-presets-ref '{}' resolved to '{}', but file was not found",
            reference, path
        )));
    };
    let parsed = serde_yaml::from_str::<Value>(&raw_presets).map_err(|err| {
        serde_yaml::Error::custom(format!(
            "failed to parse effect-presets-ref '{}': {err}",
            path
        ))
    })?;
    let mut merged = extract_referenced_effect_presets_map(&parsed)
        .ok_or_else(|| {
            serde_yaml::Error::custom(format!(
                "effect-presets-ref '{}' must resolve to a mapping (or mapping with top-level 'effect-presets')",
                path
            ))
        })?
        .clone();
    if let Some(local) = resolve_scene_effect_presets(scene_map)? {
        merge_mapping_deep(&mut merged, &local);
    }
    scene_map.insert(
        Value::String("effect-presets".to_string()),
        Value::Mapping(merged),
    );
    scene_map.remove(Value::String("effect_presets".to_string()));
    scene_map.remove(Value::String("effect-presets-ref".to_string()));
    scene_map.remove(Value::String("effect_presets_ref".to_string()));
    Ok(())
}

fn resolve_scene_effect_presets(scene_map: &Mapping) -> Result<Option<Mapping>, serde_yaml::Error> {
    let has_canonical = scene_map.contains_key(Value::String("effect-presets".to_string()));
    let has_legacy = scene_map.contains_key(Value::String("effect_presets".to_string()));
    if has_canonical && has_legacy {
        return Err(serde_yaml::Error::custom(
            "scene defines both 'effect-presets' and 'effect_presets'; use only one",
        ));
    }
    let Some(raw_presets) = scene_map
        .get(Value::String("effect-presets".to_string()))
        .or_else(|| scene_map.get(Value::String("effect_presets".to_string())))
    else {
        return Ok(None);
    };
    let Some(presets) = raw_presets.as_mapping() else {
        return Err(serde_yaml::Error::custom(
            "scene effect presets must be a mapping",
        ));
    };
    Ok(Some(presets.clone()))
}

fn expand_effect_presets_in_value(
    value: &mut Value,
    presets: &Mapping,
) -> Result<(), serde_yaml::Error> {
    match value {
        Value::Mapping(map) => {
            if let Some(effects) = map
                .get_mut(Value::String("effects".to_string()))
                .and_then(Value::as_sequence_mut)
            {
                for effect in effects {
                    expand_single_effect_entry(effect, presets)?;
                }
            }
            if let Some(postfx) = map
                .get_mut(Value::String("postfx".to_string()))
                .and_then(Value::as_sequence_mut)
            {
                for effect in postfx {
                    expand_single_effect_entry(effect, presets)?;
                }
            }
            for child in map.values_mut() {
                expand_effect_presets_in_value(child, presets)?;
            }
        }
        Value::Sequence(seq) => {
            for child in seq {
                expand_effect_presets_in_value(child, presets)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn expand_single_effect_entry(
    effect: &mut Value,
    presets: &Mapping,
) -> Result<(), serde_yaml::Error> {
    let Some(effect_map) = effect.as_mapping() else {
        return Ok(());
    };
    let Some((_, preset_name)) = resolve_effect_preset_name(effect_map)? else {
        return Ok(());
    };
    let Some(base) = presets
        .get(Value::String(preset_name.clone()))
        .and_then(Value::as_mapping)
        .cloned()
    else {
        return Err(serde_yaml::Error::custom(format!(
            "effect preset '{}' was not found",
            preset_name
        )));
    };

    let mut merged = base;
    if let Some(overrides) = effect_map
        .get(Value::String("overrides".to_string()))
        .and_then(Value::as_mapping)
    {
        merge_mapping_deep(&mut merged, overrides);
    }
    for (k, v) in effect_map {
        let key = k.as_str().unwrap_or_default();
        if key == "use" || key == "preset" || key == "ref" || key == "overrides" {
            continue;
        }
        merged.insert(k.clone(), v.clone());
    }
    *effect = Value::Mapping(merged);
    Ok(())
}

fn resolve_effect_preset_name(
    effect_map: &Mapping,
) -> Result<Option<(&'static str, String)>, serde_yaml::Error> {
    let mut selected: Option<(&'static str, String)> = None;
    for alias in ["use", "preset", "ref"] {
        let Some(raw_value) = effect_map.get(Value::String(alias.to_string())) else {
            continue;
        };
        let Some(raw_name) = raw_value.as_str().map(str::trim) else {
            return Err(serde_yaml::Error::custom(format!(
                "effect preset alias '{}' must be a string",
                alias
            )));
        };
        if raw_name.is_empty() {
            return Err(serde_yaml::Error::custom(format!(
                "effect preset alias '{}' cannot be empty",
                alias
            )));
        }
        match selected.as_ref() {
            Some((selected_alias, selected_name)) if selected_name != raw_name => {
                return Err(serde_yaml::Error::custom(format!(
                    "conflicting effect preset aliases: '{}: {}' and '{}: {}'",
                    selected_alias, selected_name, alias, raw_name
                )));
            }
            None => selected = Some((alias, raw_name.to_string())),
            _ => {}
        }
    }
    Ok(selected)
}

fn merge_mapping_deep(dst: &mut Mapping, src: &Mapping) {
    for (k, v) in src {
        match (dst.get_mut(k), v) {
            (Some(Value::Mapping(dst_map)), Value::Mapping(src_map)) => {
                merge_mapping_deep(dst_map, src_map);
            }
            _ => {
                dst.insert(k.clone(), v.clone());
            }
        }
    }
}

fn expand_scene_layer_refs<F>(root: &mut Value, scene_source_path: &str, asset_loader: &mut F)
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return;
    };
    let Some(layers) = scene_map
        .get(Value::String("layers".to_string()))
        .and_then(Value::as_sequence)
        .cloned()
    else {
        return;
    };

    let mut expanded_layers = Vec::new();
    let mut active_layer_ref_stack = Vec::new();
    for layer_entry in layers {
        resolve_layer_entry_refs(
            layer_entry,
            scene_source_path,
            asset_loader,
            &mut active_layer_ref_stack,
            &mut expanded_layers,
        );
    }

    scene_map.insert(
        Value::String("layers".to_string()),
        Value::Sequence(expanded_layers),
    );
}

fn resolve_layer_entry_refs<F>(
    layer_entry: Value,
    source_path: &str,
    asset_loader: &mut F,
    active_layer_ref_stack: &mut Vec<String>,
    out: &mut Vec<Value>,
) where
    F: FnMut(&str) -> Option<String>,
{
    let Some(layer_map) = layer_entry.as_mapping() else {
        out.push(layer_entry);
        return;
    };
    let Some(layer_ref) = layer_map
        .get(Value::String("ref".to_string()))
        .or_else(|| layer_map.get(Value::String("use".to_string())))
        .and_then(Value::as_str)
    else {
        out.push(layer_entry);
        return;
    };

    let path = resolve_layer_ref_path(source_path, layer_ref);
    if active_layer_ref_stack.contains(&path) {
        out.push(layer_entry);
        return;
    }
    let Some(raw_layer) = asset_loader(&path) else {
        out.push(layer_entry);
        return;
    };
    let Ok(mut loaded_layer_value) = serde_yaml::from_str::<Value>(&raw_layer) else {
        out.push(layer_entry);
        return;
    };

    if let Some(args) = layer_map
        .get(Value::String("with".to_string()))
        .and_then(Value::as_mapping)
    {
        substitute_args(&mut loaded_layer_value, args);
    }

    let mut loaded_entries = layer_entries_from_value(loaded_layer_value);
    for loaded_entry in &mut loaded_entries {
        apply_layer_ref_overrides(loaded_entry, layer_map);
    }

    active_layer_ref_stack.push(path.clone());
    for loaded_entry in loaded_entries {
        resolve_layer_entry_refs(
            loaded_entry,
            &path,
            asset_loader,
            active_layer_ref_stack,
            out,
        );
    }
    let _ = active_layer_ref_stack.pop();
}

fn layer_entries_from_value(value: Value) -> Vec<Value> {
    match value {
        Value::Sequence(seq) => seq,
        other => vec![other],
    }
}

fn apply_layer_ref_overrides(layer_entry: &mut Value, layer_ref_instance: &Mapping) {
    let Some(layer_map) = layer_entry.as_mapping_mut() else {
        return;
    };

    if let Some(name) = layer_ref_instance
        .get(Value::String("as".to_string()))
        .or_else(|| layer_ref_instance.get(Value::String("id".to_string())))
        .and_then(Value::as_str)
    {
        layer_map.insert(
            Value::String("name".to_string()),
            Value::String(name.to_string()),
        );
    }

    for key in [
        "z_index",
        "visible",
        "ui",
        "stages",
        "behaviors",
        "sprites",
        "objects",
    ] {
        if let Some(value) = layer_ref_instance.get(Value::String(key.to_string())) {
            layer_map.insert(Value::String(key.to_string()), value.clone());
        }
    }
}

fn expand_scene_objects<F>(root: &mut Value, scene_source_path: &str, object_loader: &mut F)
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return;
    };
    let object_instances_raw = scene_map
        .get(Value::String("objects".to_string()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    if object_instances_raw.is_empty() {
        return;
    }
    let object_instances = expand_object_instances(&object_instances_raw);

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
        let Some(mut loaded) = load_object_instance(instance_map, scene_source_path, object_loader)
        else {
            continue;
        };
        let Some(object_map) = loaded.object_value.as_mapping_mut() else {
            continue;
        };

        if let Some(object_layers) = object_map
            .get(Value::String("layers".to_string()))
            .and_then(Value::as_sequence)
        {
            for layer in object_layers {
                let mut layer_value = layer.clone();
                if let Some(behavior_name) = loaded.native_logic_behavior.as_deref() {
                    attach_layer_behavior(
                        &mut layer_value,
                        behavior_name,
                        &loaded.native_logic_params,
                    );
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
            .get(Value::String("as".to_string()))
            .or_else(|| instance_map.get(Value::String("id".to_string())))
            .and_then(Value::as_str)
            .or_else(|| {
                object_map
                    .get(Value::String("name".to_string()))
                    .and_then(Value::as_str)
            })
            .unwrap_or(loaded.instance_ref.as_str());
        layer.insert(
            Value::String("name".to_string()),
            Value::String(layer_name.to_string()),
        );
        layer.insert(
            Value::String("sprites".to_string()),
            Value::Sequence(object_sprites.clone()),
        );
        if let Some(behavior_name) = loaded.native_logic_behavior.as_deref() {
            let mut behaviors = Vec::new();
            behaviors.push(build_behavior_spec(
                behavior_name,
                &loaded.native_logic_params,
            ));
            layer.insert(
                Value::String("behaviors".to_string()),
                Value::Sequence(behaviors),
            );
        }
        scene_layers.push(Value::Mapping(layer));
    }
}

fn expand_layer_objects<F>(root: &mut Value, scene_source_path: &str, object_loader: &mut F)
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return;
    };
    let Some(scene_layers) = scene_map
        .get_mut(Value::String("layers".to_string()))
        .and_then(Value::as_sequence_mut)
    else {
        return;
    };

    for layer in scene_layers {
        let Some(layer_map) = layer.as_mapping_mut() else {
            continue;
        };
        let object_instances_raw = layer_map
            .get(Value::String("objects".to_string()))
            .and_then(Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if object_instances_raw.is_empty() {
            continue;
        }
        let object_instances = expand_object_instances(&object_instances_raw);

        for instance in object_instances {
            let Some(instance_map) = instance.as_mapping() else {
                continue;
            };
            let Some(mut loaded) =
                load_object_instance(instance_map, scene_source_path, object_loader)
            else {
                continue;
            };
            let Some(object_map) = loaded.object_value.as_mapping_mut() else {
                continue;
            };

            if let Some(object_layers) = object_map
                .get(Value::String("layers".to_string()))
                .and_then(Value::as_sequence)
            {
                for object_layer in object_layers {
                    let Some(obj_layer_map) = object_layer.as_mapping() else {
                        continue;
                    };
                    if let Some(sprites) = obj_layer_map
                        .get(Value::String("sprites".to_string()))
                        .and_then(Value::as_sequence)
                    {
                        append_sequence_items(layer_map, "sprites", sprites.to_vec());
                    }
                    if let Some(behaviors) = obj_layer_map
                        .get(Value::String("behaviors".to_string()))
                        .and_then(Value::as_sequence)
                    {
                        append_sequence_items(layer_map, "behaviors", behaviors.to_vec());
                    }
                }
            }

            if let Some(object_sprites) = object_map
                .get(Value::String("sprites".to_string()))
                .and_then(Value::as_sequence)
            {
                append_sequence_items(layer_map, "sprites", object_sprites.to_vec());
            }

            if let Some(behavior_name) = loaded.native_logic_behavior.as_deref() {
                append_sequence_items(
                    layer_map,
                    "behaviors",
                    vec![build_behavior_spec(
                        behavior_name,
                        &loaded.native_logic_params,
                    )],
                );
            }
        }

        layer_map.remove(Value::String("objects".to_string()));
    }
}

fn expand_object_instances(instances: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for instance in instances {
        let Some(instance_map) = instance.as_mapping() else {
            out.push(instance.clone());
            continue;
        };

        let Some(repeat_map) = instance_map
            .get(Value::String("repeat".to_string()))
            .and_then(Value::as_mapping)
        else {
            out.push(instance.clone());
            continue;
        };

        let Some(count) = repeat_map
            .get(Value::String("count".to_string()))
            .and_then(Value::as_i64)
        else {
            continue;
        };
        if count <= 0 || count > 4096 {
            continue;
        }

        let (ref_key, ref_template) = if let Some(v) = repeat_map
            .get(Value::String("ref".to_string()))
            .and_then(Value::as_str)
        {
            ("ref", v)
        } else if let Some(v) = repeat_map
            .get(Value::String("use".to_string()))
            .and_then(Value::as_str)
        {
            ("use", v)
        } else {
            continue;
        };

        let alias_template = repeat_map
            .get(Value::String("as".to_string()))
            .or_else(|| repeat_map.get(Value::String("id".to_string())))
            .and_then(Value::as_str)
            .map(str::to_string);
        let with_template = repeat_map
            .get(Value::String("with".to_string()))
            .cloned()
            .unwrap_or_else(|| Value::Mapping(Mapping::new()));

        for idx in 0..count {
            let mut expanded = Mapping::new();
            expanded.insert(
                Value::String(ref_key.to_string()),
                Value::String(render_repeat_token(ref_template, idx)),
            );
            if let Some(alias_template) = alias_template.as_deref() {
                expanded.insert(
                    Value::String("as".to_string()),
                    Value::String(render_repeat_token(alias_template, idx)),
                );
            } else {
                let fallback_alias = format!("{}-{idx}", object_ref_stem(ref_template));
                expanded.insert(
                    Value::String("as".to_string()),
                    Value::String(fallback_alias),
                );
            }
            let mut with_value = with_template.clone();
            substitute_repeat_token(&mut with_value, idx);
            if let Some(with_map) = with_value.as_mapping() {
                if !with_map.is_empty() {
                    expanded.insert(Value::String("with".to_string()), with_value);
                }
            }
            out.push(Value::Mapping(expanded));
        }
    }
    out
}

fn render_repeat_token(template: &str, idx: i64) -> String {
    template.replace("{i}", &idx.to_string())
}

fn substitute_repeat_token(value: &mut Value, idx: i64) {
    match value {
        Value::String(s) => {
            *s = render_repeat_token(s, idx);
        }
        Value::Sequence(seq) => {
            for entry in seq {
                substitute_repeat_token(entry, idx);
            }
        }
        Value::Mapping(map) => {
            let keys: Vec<Value> = map.keys().cloned().collect();
            for key in keys {
                if let Some(v) = map.get_mut(&key) {
                    substitute_repeat_token(v, idx);
                }
            }
        }
        _ => {}
    }
}

fn object_ref_stem(reference: &str) -> String {
    let name = reference.rsplit('/').next().unwrap_or(reference).trim();
    let no_ext = name
        .strip_suffix(".yml")
        .or_else(|| name.strip_suffix(".yaml"))
        .unwrap_or(name);
    if no_ext.is_empty() {
        "object".to_string()
    } else {
        no_ext.to_string()
    }
}

fn append_sequence_items(layer_map: &mut Mapping, key: &str, mut items: Vec<Value>) {
    let key_value = Value::String(key.to_string());
    let entry = layer_map
        .entry(key_value.clone())
        .or_insert_with(|| Value::Sequence(Vec::new()));
    if let Some(seq) = entry.as_sequence_mut() {
        seq.append(&mut items);
    } else {
        layer_map.insert(key_value, Value::Sequence(items));
    }
}

struct LoadedObjectInstance {
    instance_ref: String,
    object_value: Value,
    native_logic_behavior: Option<String>,
    native_logic_params: BTreeMap<String, Value>,
}

fn load_object_instance<F>(
    instance_map: &Mapping,
    scene_source_path: &str,
    object_loader: &mut F,
) -> Option<LoadedObjectInstance>
where
    F: FnMut(&str) -> Option<String>,
{
    let use_name = instance_map
        .get(Value::String("ref".to_string()))
        .or_else(|| instance_map.get(Value::String("use".to_string())))
        .and_then(Value::as_str)?;
    let path = resolve_object_ref_path(scene_source_path, use_name);
    let raw_object = object_loader(&path)?;
    let mut object_value = serde_yaml::from_str::<Value>(&raw_object).ok()?;

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
                logic.behavior.clone()
            } else {
                None
            }
        });
    let native_logic_params = object_doc
        .as_ref()
        .and_then(|doc| doc.logic.as_ref())
        .map(|logic| logic.params.clone())
        .unwrap_or_default();

    Some(LoadedObjectInstance {
        instance_ref: use_name.to_string(),
        object_value,
        native_logic_behavior,
        native_logic_params,
    })
}

fn resolve_object_ref_path(scene_source_path: &str, use_name: &str) -> String {
    if use_name.starts_with('/') {
        return normalize_mod_path(use_name);
    }
    if use_name.starts_with("./") || use_name.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{use_name}"));
    }
    format!("/objects/{use_name}.yml")
}

fn resolve_layer_ref_path(scene_source_path: &str, use_name: &str) -> String {
    if use_name.starts_with('/') {
        return normalize_mod_path(use_name);
    }
    if use_name.starts_with("./") || use_name.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{use_name}"));
    }

    let trimmed = use_name.trim_start_matches('/');
    let has_yaml_ext = trimmed.ends_with(".yml") || trimmed.ends_with(".yaml");
    let scene_dir = parent_dir(scene_source_path);
    let source_is_layer_dir = scene_dir.ends_with("/layers");

    if has_yaml_ext {
        if trimmed.starts_with("scenes/") {
            return normalize_mod_path(&format!("/{trimmed}"));
        }
        if trimmed.starts_with("layers/") {
            if source_is_layer_dir {
                let rel = trimmed.trim_start_matches("layers/");
                return normalize_mod_path(&format!("{scene_dir}/{rel}"));
            }
            return normalize_mod_path(&format!("{scene_dir}/{trimmed}"));
        }
        if trimmed.contains('/') {
            return normalize_mod_path(&format!("/scenes/{trimmed}"));
        }
        if source_is_layer_dir {
            return normalize_mod_path(&format!("{scene_dir}/{trimmed}"));
        }
        return normalize_mod_path(&format!("{scene_dir}/layers/{trimmed}"));
    }

    if trimmed.contains('/') {
        return normalize_mod_path(&format!("/scenes/{trimmed}.yml"));
    }
    if source_is_layer_dir {
        return normalize_mod_path(&format!("{scene_dir}/{trimmed}.yml"));
    }
    normalize_mod_path(&format!("{scene_dir}/layers/{trimmed}.yml"))
}

fn resolve_script_ref_path(scene_source_path: &str, script_ref: &str) -> String {
    if script_ref.starts_with('/') {
        return normalize_mod_path(script_ref);
    }
    if script_ref.starts_with("./") || script_ref.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{script_ref}"));
    }
    normalize_mod_path(&format!("/scripts/{script_ref}"))
}

fn is_rhai_path(path: &str) -> bool {
    path.ends_with(".rhai")
}

fn resolve_stages_ref_path(scene_source_path: &str, reference: &str) -> String {
    if reference.starts_with('/') {
        return normalize_mod_path(reference);
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{reference}"));
    }
    let trimmed = reference.trim_start_matches('/');
    let has_yaml_ext = trimmed.ends_with(".yml") || trimmed.ends_with(".yaml");
    if has_yaml_ext {
        if trimmed.starts_with("stages/") {
            return normalize_mod_path(&format!("/{trimmed}"));
        }
        return normalize_mod_path(&format!("/stages/{trimmed}"));
    }
    normalize_mod_path(&format!("/stages/{trimmed}.yml"))
}

fn extract_referenced_effect_presets_map(value: &Value) -> Option<&Mapping> {
    let map = value.as_mapping()?;
    if let Some(presets) = map
        .get(Value::String("effect-presets".to_string()))
        .and_then(Value::as_mapping)
    {
        return Some(presets);
    }
    if let Some(presets) = map
        .get(Value::String("effect_presets".to_string()))
        .and_then(Value::as_mapping)
    {
        return Some(presets);
    }
    Some(map)
}

fn resolve_effect_presets_ref_path(scene_source_path: &str, reference: &str) -> String {
    if reference.starts_with('/') {
        return normalize_mod_path(reference);
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{reference}"));
    }
    let trimmed = reference.trim_start_matches('/');
    let has_yaml_ext = trimmed.ends_with(".yml") || trimmed.ends_with(".yaml");
    if has_yaml_ext {
        if trimmed.starts_with("effects/") {
            return normalize_mod_path(&format!("/{trimmed}"));
        }
        return normalize_mod_path(&format!("/effects/{trimmed}"));
    }
    normalize_mod_path(&format!("/effects/{trimmed}.yml"))
}

fn parent_dir(path: &str) -> String {
    let normalized = normalize_mod_path(path);
    match normalized.rsplit_once('/') {
        Some(("", _)) | None => "/".to_string(),
        Some((dir, _)) => dir.to_string(),
    }
}

fn normalize_mod_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            let _ = parts.pop();
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn attach_scene_behavior(
    scene_map: &mut Mapping,
    behavior_name: &str,
    params: &std::collections::BTreeMap<String, Value>,
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

fn attach_layer_behavior(
    layer_value: &mut Value,
    behavior_name: &str,
    params: &std::collections::BTreeMap<String, Value>,
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
        map.insert(
            Value::String("params".to_string()),
            Value::Mapping(params_map),
        );
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
    use super::{
        compile_scene_document_with_loader, compile_scene_document_with_loader_and_source,
        compile_scene_document_with_loader_and_source_and_filters,
    };
    use crate::compile::{CutsceneCompileFilter, CutsceneCompileFrame, CutsceneFilterRegistry};
    use engine_core::scene::Sprite;

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
            Sprite::Text { content, .. } => assert_eq!(content, "MONKEY"),
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
            Sprite::Text { content, .. } => assert_eq!(content, "DEFAULT"),
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
    fn maps_scene_native_logic_to_scene_behaviors() {
        let scene_raw = r#"
id: playground
title: Playground
logic:
  type: native
  behavior: bob
  params:
    amplitude_y: 3
layers: []
next: null
"#;
        let scene =
            compile_scene_document_with_loader(scene_raw, |_path| None).expect("scene compile");
        assert_eq!(scene.behaviors.len(), 1);
        assert_eq!(scene.behaviors[0].name, "bob");
        assert_eq!(scene.behaviors[0].params.amplitude_y, Some(3));
    }

    #[test]
    fn rejects_graph_logic_kind_as_experimental() {
        let scene_raw = r#"
id: playground
title: Playground
logic:
  type: graph
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader(scene_raw, |_path| None)
            .expect_err("graph logic should be rejected");
        assert!(
            err.to_string().contains("logic.kind=graph"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_malformed_logic_block_with_direct_error() {
        let scene_raw = r#"
id: playground
title: Playground
logic: 1
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader(scene_raw, |_path| None)
            .expect_err("malformed logic block should be rejected");
        assert!(
            err.to_string().contains("failed to parse logic block"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn maps_scene_script_logic_from_explicit_src() {
        let scene_raw = r#"
id: menu
title: Menu
logic:
  type: script
  src: ./menu.logic.yml
layers: []
next: null
"#;
        let script_raw = r#"
behavior: menu-carousel-object
params:
  target: menu-grid
  item_prefix: menu-item-
  count: 3
  window: 3
  step_y: 2
  endless: true
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/menu/scene.yml",
            |path| {
                if path == "/scenes/menu/menu.logic.yml" {
                    Some(script_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.behaviors.len(), 1);
        assert_eq!(scene.behaviors[0].name, "menu-carousel-object");
        assert_eq!(scene.behaviors[0].params.count, Some(3));
        assert_eq!(scene.behaviors[0].params.window, Some(3));
    }

    #[test]
    fn rejects_script_logic_without_explicit_src() {
        let scene_raw = r#"
id: menu
title: Menu
logic:
  type: script
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/menu/scene.yml",
            |path| {
                if path == "/scenes/menu/menu.logic.yml" {
                    Some(
                        r#"
behavior: blink
params:
  visible_ms: 400
"#
                        .to_string(),
                    )
                } else {
                    None
                }
            },
        )
        .expect_err("script logic without src should be rejected");
        assert!(
            err.to_string()
                .contains("logic.kind=script requires explicit logic.src"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn does_not_auto_detect_scene_logic_file_without_explicit_logic_block() {
        let scene_raw = r#"
id: menu
title: Menu
layers: []
next: null
"#;
        let script_raw = r#"
behavior: blink
params:
  visible_ms: 400
  hidden_ms: 200
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/mainmenu/scene.yml",
            |path| {
                if path == "/scenes/mainmenu/mainmenu.logic.yml" {
                    Some(script_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.behaviors.len(), 0);
    }

    #[test]
    fn maps_scene_script_logic_from_explicit_rhai_src() {
        let scene_raw = r#"
id: menu
title: Menu
logic:
  type: script
  src: ./menu.rhai
layers: []
next: null
"#;
        let script_raw = r#"
let out = [];
out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
out
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/menu/scene.yml",
            |path| {
                if path == "/scenes/menu/menu.rhai" {
                    Some(script_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.behaviors.len(), 1);
        assert_eq!(scene.behaviors[0].name, "rhai-script");
        assert_eq!(
            scene.behaviors[0].params.src.as_deref(),
            Some("/scenes/menu/menu.rhai")
        );
        assert!(scene.behaviors[0].params.script.is_some());
    }

    #[test]
    fn does_not_auto_detect_rhai_logic_file_without_explicit_logic_block() {
        let scene_raw = r#"
id: menu
title: Menu
layers: []
next: null
"#;
        let script_raw = r#"
let out = [];
out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
out
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/mainmenu/scene.yml",
            |path| {
                if path == "/scenes/mainmenu/mainmenu.rhai" {
                    Some(script_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.behaviors.len(), 0);
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
    fn expands_effect_preset_use_in_scene_stages() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.flash:
    name: whiteout
    duration: 120
    target_kind: scene
stages:
  on_enter:
    steps:
      - effects:
          - use: fx.flash
layers: []
next: null
"#;
        let scene =
            compile_scene_document_with_loader(scene_raw, |_path| None).expect("scene compile");
        let effects = &scene.stages.on_enter.steps[0].effects;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].name, "whiteout");
        assert_eq!(effects[0].duration, 120);
    }

    #[test]
    fn rejects_conflicting_effect_preset_root_aliases() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.flash:
    name: whiteout
effect_presets:
  fx.flash:
    name: whiteout
stages:
  on_enter:
    steps:
      - effects:
          - use: fx.flash
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader(scene_raw, |_path| None)
            .expect_err("conflicting effect preset roots should be rejected");
        assert!(
            err.to_string()
                .contains("defines both 'effect-presets' and 'effect_presets'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_conflicting_effect_preset_aliases() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.flash:
    name: whiteout
    duration: 120
stages:
  on_enter:
    steps:
      - effects:
          - use: fx.flash
            preset: fx.other
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader(scene_raw, |_path| None)
            .expect_err("conflicting effect preset aliases should be rejected");
        assert!(
            err.to_string()
                .contains("conflicting effect preset aliases"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_missing_effect_preset_reference() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.flash:
    name: whiteout
stages:
  on_enter:
    steps:
      - effects:
          - use: fx.missing
layers: []
next: null
"#;
        let err = compile_scene_document_with_loader(scene_raw, |_path| None)
            .expect_err("missing effect preset should be rejected");
        assert!(
            err.to_string()
                .contains("effect preset 'fx.missing' was not found"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn effect_preset_overrides_merge_nested_params() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.light:
    name: shine
    duration: 300
    target_kind: sprite_bitmap
    params:
      intensity: 1.0
      width: 2.0
stages:
  on_enter:
    steps:
      - effects:
          - use: fx.light
            overrides:
              params:
                intensity: 1.4
layers: []
next: null
"#;
        let scene =
            compile_scene_document_with_loader(scene_raw, |_path| None).expect("scene compile");
        let effect = &scene.stages.on_enter.steps[0].effects[0];
        assert_eq!(effect.name, "shine");
        assert_eq!(effect.duration, 300);
        assert_eq!(effect.params.intensity, Some(1.4));
        assert_eq!(effect.params.width, Some(2.0));
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
            Sprite::Text { content, .. } => assert_eq!(content, "MONKEY"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn expands_scene_object_repeat_entries() {
        let scene_raw = r#"
id: playground
title: Playground
layers: []
objects:
  - repeat:
      count: 3
      ref: bullet
      as: bullet-{i}
      with:
        id: bullet-{i}
        content: "B{i}"
"#;
        let object_raw = r#"
name: bullet
exports:
  id: default-id
  content: default
sprites:
  - type: text
    id: "$id"
    content: "$content"
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/bullet.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 3);
        assert_eq!(scene.layers[0].name, "bullet-0");
        assert_eq!(scene.layers[1].name, "bullet-1");
        assert_eq!(scene.layers[2].name, "bullet-2");
        match &scene.layers[2].sprites[0] {
            Sprite::Text { id, content, .. } => {
                assert_eq!(id.as_deref(), Some("bullet-2"));
                assert_eq!(content, "B2");
            }
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn expands_layer_object_repeat_entries() {
        let scene_raw = r#"
id: playground
title: Playground
layers:
  - name: game
    objects:
      - repeat:
          count: 2
          ref: marker
          as: marker-{i}
          with:
            label: "M{i}"
"#;
        let object_raw = r#"
name: marker
sprites:
  - type: text
    content: "$label"
"#;
        let scene = compile_scene_document_with_loader(scene_raw, |path| {
            if path == "/objects/marker.yml" {
                Some(object_raw.to_string())
            } else {
                None
            }
        })
        .expect("scene compile");
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "game");
        assert_eq!(scene.layers[0].sprites.len(), 2);
        match &scene.layers[0].sprites[0] {
            Sprite::Text { content, .. } => assert_eq!(content, "M0"),
            _ => panic!("expected text sprite"),
        }
        match &scene.layers[0].sprites[1] {
            Sprite::Text { content, .. } => assert_eq!(content, "M1"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn effect_presets_ref_merges_referenced_and_local_presets() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets-ref: /effects/shared-crt.yml
effect-presets:
  fx.local:
    name: crt-ruby
    duration: 240
    params:
      intensity: 0.25
postfx:
  - use: fx.shared
  - use: fx.local
layers: []
next: null
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| match path {
                "/effects/shared-crt.yml" => Some(
                    r#"
effect-presets:
  fx.shared:
    name: terminal-crt
    duration: 900
    params:
      intensity: 0.8
"#
                    .to_string(),
                ),
                _ => None,
            },
        )
        .expect("scene compile");
        assert_eq!(scene.postfx.len(), 2);
        assert_eq!(scene.postfx[0].name, "terminal-crt");
        assert_eq!(scene.postfx[0].duration, 900);
        assert_eq!(scene.postfx[1].name, "crt-ruby");
        assert_eq!(scene.postfx[1].duration, 240);
    }

    #[test]
    fn effect_presets_expand_inside_postfx() {
        let scene_raw = r#"
id: intro
title: Intro
effect-presets:
  fx.crt:
    name: terminal-crt
    duration: 1200
    params:
      intensity: 0.9
postfx:
  - use: fx.crt
    overrides:
      params:
        intensity: 1.1
layers: []
next: null
"#;
        let scene =
            compile_scene_document_with_loader(scene_raw, |_path| None).expect("scene compile");
        assert_eq!(scene.postfx.len(), 1);
        assert_eq!(scene.postfx[0].name, "terminal-crt");
        assert_eq!(scene.postfx[0].duration, 1200);
        assert_eq!(scene.postfx[0].params.intensity, Some(1.1));
    }

    #[test]
    fn expands_scene_layer_ref_with_with_args_and_overrides() {
        let scene_raw = r#"
id: intro
title: Intro
layers:
  - ref: background
    as: bg-main
    z_index: 7
    ui: true
    with:
      label: HELLO
next: null
"#;
        let layer_raw = r#"
- name: background
  sprites:
    - type: text
      content: "$label"
"#;

        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/scenes/intro/layers/background.yml" {
                    Some(layer_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");

        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "bg-main");
        assert_eq!(scene.layers[0].z_index, 7);
        assert!(scene.layers[0].ui);
        match &scene.layers[0].sprites[0] {
            Sprite::Text { content, .. } => assert_eq!(content, "HELLO"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn expands_layer_local_objects_into_layer_sprites() {
        let scene_raw = r#"
id: intro
title: Intro
layers:
  - name: base
    objects:
      - ref: suzan
        with:
          label: LAYER
next: null
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

        assert_eq!(scene.layers.len(), 1);
        match &scene.layers[0].sprites[0] {
            Sprite::Text { content, .. } => assert_eq!(content, "LAYER"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn resolves_nested_layer_refs_recursively() {
        let scene_raw = r#"
id: intro
title: Intro
layers:
  - ref: base
next: null
"#;
        let base_layer_raw = r#"
- ref: nested
"#;
        let nested_layer_raw = r#"
name: nested
sprites:
  - type: text
    content: "OK"
"#;

        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| match path {
                "/scenes/intro/layers/base.yml" => Some(base_layer_raw.to_string()),
                "/scenes/intro/layers/nested.yml" => Some(nested_layer_raw.to_string()),
                _ => None,
            },
        )
        .expect("scene compile");

        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "nested");
        match &scene.layers[0].sprites[0] {
            Sprite::Text { content, .. } => assert_eq!(content, "OK"),
            _ => panic!("expected text sprite"),
        }
    }

    #[test]
    fn layer_ref_cycles_do_not_recurse_forever() {
        let scene_raw = r#"
id: intro
title: Intro
layers:
  - ref: a
next: null
"#;
        let a_layer_raw = r#"
- ref: b
"#;
        let b_layer_raw = r#"
- ref: a
"#;

        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| match path {
                "/scenes/intro/layers/a.yml" => Some(a_layer_raw.to_string()),
                "/scenes/intro/layers/b.yml" => Some(b_layer_raw.to_string()),
                _ => None,
            },
        )
        .expect("scene compile");

        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn expands_stages_ref_into_scene_stages() {
        let scene_raw = r#"
id: intro
title: Intro
stages-ref: cinematic-fade
layers: []
next: null
"#;
        let stages_raw = r#"
on_enter:
  steps:
    - pause: 300ms
on_idle:
  trigger: any-key
  steps:
    - pause: 1ms
on_leave:
  steps:
    - effects:
        - name: fade-out
          duration: 220
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/stages/cinematic-fade.yml" {
                    Some(stages_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");

        assert_eq!(scene.stages.on_enter.steps.len(), 1);
        assert_eq!(scene.stages.on_enter.steps[0].duration, Some(300));
        assert_eq!(scene.stages.on_idle.steps.len(), 1);
        assert_eq!(scene.stages.on_leave.steps.len(), 1);
    }

    #[test]
    fn stages_ref_merges_with_local_stage_overrides() {
        let scene_raw = r#"
id: intro
title: Intro
stages-ref: cinematic-fade
stages:
  on_idle:
    trigger: timeout
    steps:
      - pause: 5s
layers: []
next: null
"#;
        let stages_raw = r#"
on_enter:
  steps:
    - pause: 300ms
on_idle:
  trigger: any-key
  looping: true
  steps:
    - pause: 1ms
on_leave:
  steps:
    - effects:
        - name: fade-out
          duration: 220
"#;
        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/stages/cinematic-fade.yml" {
                    Some(stages_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");
        assert_eq!(scene.stages.on_enter.steps[0].duration, Some(300));
        assert!(matches!(
            scene.stages.on_idle.trigger,
            engine_core::scene::StageTrigger::Timeout
        ));
        assert_eq!(scene.stages.on_idle.steps[0].duration, Some(5000));
        assert_eq!(scene.stages.on_leave.steps.len(), 1);
    }

    #[test]
    fn expands_cutscene_ref_into_timed_image_sprites() {
        let scene_raw = r#"
id: intro
title: Intro
layers: []
cutscene-ref: intro-sequence
next: null
"#;
        let cutscene_raw = r#"
layer-name: intro-cutscene
defaults:
  at: cc
  width: 30
  height: 12
frames:
  - source: /assets/images/intro/1.png
    delay-ms: 100
  - source: /assets/images/intro/2.png
    delay-ms: 200
"#;

        let scene = compile_scene_document_with_loader_and_source(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/cutscenes/intro-sequence.yml" {
                    Some(cutscene_raw.to_string())
                } else {
                    None
                }
            },
        )
        .expect("scene compile");

        assert!(scene.cutscene);
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].name, "intro-cutscene");
        assert_eq!(scene.layers[0].sprites.len(), 2);

        match &scene.layers[0].sprites[0] {
            Sprite::Image {
                source,
                width,
                height,
                appear_at_ms,
                disappear_at_ms,
                ..
            } => {
                assert_eq!(source, "/assets/images/intro/1.png");
                assert_eq!(*width, Some(30));
                assert_eq!(*height, Some(12));
                assert_eq!(*appear_at_ms, Some(0));
                assert_eq!(*disappear_at_ms, Some(100));
            }
            _ => panic!("expected image sprite"),
        }
        match &scene.layers[0].sprites[1] {
            Sprite::Image {
                source,
                appear_at_ms,
                disappear_at_ms,
                ..
            } => {
                assert_eq!(source, "/assets/images/intro/2.png");
                assert_eq!(*appear_at_ms, Some(100));
                assert_eq!(*disappear_at_ms, Some(300));
            }
            _ => panic!("expected image sprite"),
        }
    }

    struct AddDelayFilter;

    impl CutsceneCompileFilter for AddDelayFilter {
        fn name(&self) -> &'static str {
            "add-delay"
        }

        fn apply(
            &self,
            frame: &mut CutsceneCompileFrame,
            params: &serde_yaml::Mapping,
        ) -> Result<(), serde_yaml::Error> {
            let add_ms = params
                .get(serde_yaml::Value::String("ms".to_string()))
                .and_then(serde_yaml::Value::as_u64)
                .unwrap_or(0);
            frame.delay_ms = frame.delay_ms.saturating_add(add_ms);
            Ok(())
        }
    }

    #[test]
    fn applies_custom_cutscene_filter_registry() {
        let scene_raw = r#"
id: intro
title: Intro
layers: []
cutscene-ref: intro-sequence
next: null
"#;
        let cutscene_raw = r#"
filters:
  - name: add-delay
    params:
      ms: 5
frames:
  - source: /assets/images/intro/1.png
    delay-ms: 10
  - source: /assets/images/intro/2.png
    delay-ms: 20
"#;
        let mut filters = CutsceneFilterRegistry::default();
        filters.register(AddDelayFilter);

        let scene = compile_scene_document_with_loader_and_source_and_filters(
            scene_raw,
            "/scenes/intro/scene.yml",
            |path| {
                if path == "/cutscenes/intro-sequence.yml" {
                    Some(cutscene_raw.to_string())
                } else {
                    None
                }
            },
            &filters,
        )
        .expect("scene compile");

        match &scene.layers[0].sprites[0] {
            Sprite::Image {
                disappear_at_ms, ..
            } => assert_eq!(*disappear_at_ms, Some(15)),
            _ => panic!("expected image sprite"),
        }
        match &scene.layers[0].sprites[1] {
            Sprite::Image {
                appear_at_ms,
                disappear_at_ms,
                ..
            } => {
                assert_eq!(*appear_at_ms, Some(15));
                assert_eq!(*disappear_at_ms, Some(40));
            }
            _ => panic!("expected image sprite"),
        }
    }

    #[test]
    fn audio_cues_survive_scene_compilation() {
        let raw = r#"
id: audio-test
title: Audio Test
bg: black
layers: []
audio:
  on_enter:
    - { cue: daisy-bell, at_ms: 500, volume: 0.9 }
"#;
        let scene = compile_scene_document_with_loader_and_source(raw, "test/scene.yml", |_| None)
            .expect("should compile");
        assert_eq!(scene.audio.on_enter.len(), 1);
        assert_eq!(scene.audio.on_enter[0].cue, "daisy-bell");
        assert_eq!(scene.audio.on_enter[0].at_ms, 500);
        assert_eq!(scene.audio.on_enter[0].volume, Some(0.9));
    }
}
