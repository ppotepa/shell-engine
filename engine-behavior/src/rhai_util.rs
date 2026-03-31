//! Rhai type conversions and utility functions.

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use engine_core::effects::Region;
use engine_core::scene::BehaviorParams;

pub(crate) fn json_to_rhai_dynamic(value: &JsonValue) -> RhaiDynamic {
    match value {
        JsonValue::Null => ().into(),
        JsonValue::Bool(value) => (*value).into(),
        JsonValue::Number(value) => {
            if let Some(int) = value.as_i64() {
                (int as rhai::INT).into()
            } else if let Some(float) = value.as_f64() {
                float.into()
            } else {
                ().into()
            }
        }
        JsonValue::String(value) => value.clone().into(),
        JsonValue::Array(values) => {
            let mut out = RhaiArray::new();
            for item in values {
                out.push(json_to_rhai_dynamic(item));
            }
            out.into()
        }
        JsonValue::Object(map) => {
            let mut out = RhaiMap::new();
            for (key, value) in map {
                out.insert(key.into(), json_to_rhai_dynamic(value));
            }
            out.into()
        }
    }
}

pub(crate) fn rhai_dynamic_to_json(value: &RhaiDynamic) -> Option<JsonValue> {
    if value.is_unit() {
        return Some(JsonValue::Null);
    }
    if let Some(value) = value.clone().try_cast::<bool>() {
        return Some(JsonValue::Bool(value));
    }
    if let Some(value) = value.clone().try_cast::<rhai::INT>() {
        return Some(JsonValue::Number(JsonNumber::from(value)));
    }
    if let Some(value) = value.clone().try_cast::<rhai::FLOAT>() {
        if let Some(number) = JsonNumber::from_f64(value) {
            return Some(JsonValue::Number(number));
        }
        return None;
    }
    if let Some(value) = value.clone().try_cast::<String>() {
        return Some(JsonValue::String(value));
    }
    if let Some(values) = value.clone().try_cast::<RhaiArray>() {
        let mut out = Vec::with_capacity(values.len());
        for item in values {
            out.push(rhai_dynamic_to_json(&item)?);
        }
        return Some(JsonValue::Array(out));
    }
    if let Some(map) = value.clone().try_cast::<RhaiMap>() {
        let mut out = JsonMap::new();
        for (key, item) in map {
            out.insert(key.into(), rhai_dynamic_to_json(&item)?);
        }
        return Some(JsonValue::Object(out));
    }
    None
}

pub(crate) fn map_get_path_dynamic(map: &RhaiMap, path: &str) -> Option<RhaiDynamic> {
    let mut segments = path.split('.').filter(|segment| !segment.is_empty());
    let first = segments.next()?;
    let mut current = map.get(first)?.clone();
    for segment in segments {
        let next_map = current.clone().try_cast::<RhaiMap>()?;
        current = next_map.get(segment)?.clone();
    }
    Some(current)
}

pub(crate) fn map_set_path_dynamic(map: &mut RhaiMap, path: &str, value: RhaiDynamic) -> bool {
    let segments = path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return false;
    }
    map_set_path_recursive(map, &segments, value);
    true
}

fn map_set_path_recursive(map: &mut RhaiMap, segments: &[&str], value: RhaiDynamic) {
    let key = segments[0];
    if segments.len() == 1 {
        map.insert(key.into(), value);
        return;
    }
    let mut child = map
        .get(key)
        .and_then(|current| current.clone().try_cast::<RhaiMap>())
        .unwrap_or_default();
    map_set_path_recursive(&mut child, &segments[1..], value);
    map.insert(key.into(), child.into());
}

pub(crate) fn merge_rhai_maps(base: &mut RhaiMap, patch: &RhaiMap) {
    for (key, value) in patch {
        if let Some(existing) = base.get_mut(key.as_str()) {
            if let (Some(existing_map), Some(patch_map)) = (
                existing.clone().try_cast::<RhaiMap>(),
                value.clone().try_cast::<RhaiMap>(),
            ) {
                let mut merged = existing_map;
                merge_rhai_maps(&mut merged, &patch_map);
                *existing = merged.into();
                continue;
            }
        }
        base.insert(key.clone(), value.clone());
    }
}

pub(crate) fn normalize_set_path(path: &str) -> String {
    path.trim()
        .strip_prefix("props.")
        .unwrap_or(path.trim())
        .to_string()
}

pub(crate) fn normalize_input_code(code: &str) -> String {
    if code == " " {
        return " ".to_string();
    }
    let trimmed = code.trim();
    if trimmed.len() == 1 {
        return trimmed.to_ascii_lowercase();
    }
    trimmed.to_string()
}

pub(crate) fn behavior_params_to_rhai_map(params: &BehaviorParams) -> RhaiMap {
    let mut out = RhaiMap::new();
    if let Some(value) = params.target.as_ref() {
        out.insert("target".into(), value.clone().into());
    }
    if let Some(value) = params.index {
        out.insert("index".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.count {
        out.insert("count".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.window {
        out.insert("window".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.step_y {
        out.insert("step_y".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.endless {
        out.insert("endless".into(), value.into());
    }
    if let Some(value) = params.item_prefix.as_ref() {
        out.insert("item_prefix".into(), value.clone().into());
    }
    if let Some(value) = params.src.as_ref() {
        out.insert("src".into(), value.clone().into());
    }
    if let Some(value) = params.dur {
        out.insert("dur".into(), (value as rhai::INT).into());
    }
    out
}

pub(crate) fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut out = RhaiMap::new();
    out.insert("x".into(), (region.x as rhai::INT).into());
    out.insert("y".into(), (region.y as rhai::INT).into());
    out.insert("w".into(), (region.width as rhai::INT).into());
    out.insert("h".into(), (region.height as rhai::INT).into());
    out
}
