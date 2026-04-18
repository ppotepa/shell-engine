use serde::de::Error as _;
use serde_yaml::{Mapping, Value};

pub(super) fn expand_scene_effect_presets(root: &mut Value) -> Result<(), serde_yaml::Error> {
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

pub(super) fn expand_scene_effect_presets_ref<F>(
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
    let has_alternate_alias =
        scene_map.contains_key(Value::String("effect_presets_ref".to_string()));
    if has_canonical && has_alternate_alias {
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

pub(super) fn resolve_scene_effect_presets(
    scene_map: &Mapping,
) -> Result<Option<Mapping>, serde_yaml::Error> {
    let has_canonical = scene_map.contains_key(Value::String("effect-presets".to_string()));
    let has_alternate_alias = scene_map.contains_key(Value::String("effect_presets".to_string()));
    if has_canonical && has_alternate_alias {
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

pub(super) fn expand_effect_presets_in_value(
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

pub(super) fn expand_single_effect_entry(
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

pub(super) fn resolve_effect_preset_name(
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

pub(super) fn merge_mapping_deep(dst: &mut Mapping, src: &Mapping) {
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

pub(super) fn extract_referenced_effect_presets_map(value: &Value) -> Option<&Mapping> {
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

pub(super) fn resolve_effect_presets_ref_path(scene_source_path: &str, reference: &str) -> String {
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
        if trimmed.starts_with("effects/") {
            return super::normalize_mod_path(&format!("/{trimmed}"));
        }
        return super::normalize_mod_path(&format!("/effects/{trimmed}"));
    }
    super::normalize_mod_path(&format!("/effects/{trimmed}.yml"))
}

pub(super) fn substitute_args(value: &mut Value, args: &Mapping) {
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
