//! Shared YAML accessor utilities and expression-parsing helpers for scene normalization.

use serde_yaml::{Mapping, Number, Value};

/// Returns the first matching string value from `cfg` by trying each key.
pub(super) fn cfg_str<'a>(cfg: &'a Mapping, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| {
        cfg.get(Value::String((*k).to_string()))
            .and_then(Value::as_str)
    })
}

/// Returns the first matching u64 value from `cfg` by trying each key.
pub(super) fn cfg_u64(cfg: &Mapping, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|k| {
        cfg.get(Value::String((*k).to_string()))
            .and_then(Value::as_u64)
    })
}

/// Returns the first matching bool value from `cfg` by trying each key.
pub(super) fn cfg_bool(cfg: &Mapping, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|k| {
        cfg.get(Value::String((*k).to_string()))
            .and_then(Value::as_bool)
    })
}

/// Returns the first matching bool from `map` by trying each key name.
pub(super) fn map_get_bool(map: &Mapping, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|key| {
        map.get(Value::String((*key).to_string()))
            .and_then(Value::as_bool)
    })
}

/// Returns the first matching u64 from `map` by trying each key name.
/// Also accepts string values that parse as decimal integers.
pub(super) fn map_get_u64(map: &Mapping, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| {
        map.get(Value::String((*key).to_string()))
            .and_then(|value| {
                value
                    .as_u64()
                    .or_else(|| value.as_str().and_then(|raw| raw.parse::<u64>().ok()))
            })
    })
}

/// Returns the first matching string from `map` by trying each key name.
pub(super) fn map_get_str<'a>(map: &'a Mapping, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        map.get(Value::String((*key).to_string()))
            .and_then(Value::as_str)
    })
}

/// Renames field `from` to `to` if `to` is not already present.
///
/// Used for: bg→bg_colour, fg→fg_colour
pub(super) fn apply_alias(map: &mut Mapping, from: &str, to: &str) {
    let from_key = Value::String(from.to_string());
    let to_key = Value::String(to.to_string());
    if map.contains_key(&to_key) {
        return;
    }
    if let Some(value) = map.get(&from_key).cloned() {
        map.insert(to_key, value);
    }
}

/// Expands `at: anchor` shorthand into {align_x, align_y} pair.
///
/// Supported anchors: cc, ct, cb, lc, lt, lb, rc, rt, rb
/// Documented in: engine_core::authoring::catalog::sugar_catalog()
pub(super) fn apply_at_anchor(map: &mut Mapping) {
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

    map.entry(Value::String("align-x".to_string()))
        .or_insert_with(|| Value::String(ax.to_string()));
    map.entry(Value::String("align-y".to_string()))
        .or_insert_with(|| Value::String(ay.to_string()));
}

/// Returns true if the sprite map has `type: expected` (case-insensitive).
pub(super) fn is_sprite_type(map: &Mapping, expected: &str) -> bool {
    map.get(Value::String("type".to_string()))
        .and_then(Value::as_str)
        .map(|ty| ty.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

/// Merges parent and local defaults into a combined mapping.
pub(super) fn merge_defaults(parent: Option<&Mapping>, local: Option<&Mapping>) -> Option<Mapping> {
    match (parent, local) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(l)) => Some(l.clone()),
        (Some(p), Some(l)) => {
            let mut merged = p.clone();
            for (k, v) in l {
                merged.insert(k.clone(), v.clone());
            }
            Some(merged)
        }
    }
}

/// Copies default values from `defaults` into `map` for any key not already present.
pub(super) fn apply_defaults(map: &mut Mapping, defaults: Option<&Mapping>) {
    let Some(defaults) = defaults else {
        return;
    };
    for (k, v) in defaults {
        map.entry(k.clone()).or_insert_with(|| v.clone());
    }
}

/// Parses a duration value into milliseconds.
/// Accepts: integers, "Nms", "Ns", or plain decimal string.
pub(super) fn parse_duration_ms(value: &Value) -> Option<u64> {
    if let Some(ms) = value.as_u64() {
        return Some(ms);
    }
    if let Some(text) = value.as_str() {
        let trimmed = text.trim().to_ascii_lowercase();
        if let Some(ms) = trimmed.strip_suffix("ms") {
            return ms.trim().parse::<u64>().ok();
        }
        if let Some(sec) = trimmed.strip_suffix('s') {
            return sec
                .trim()
                .parse::<u64>()
                .ok()
                .map(|v| v.saturating_mul(1000));
        }
        return trimmed.parse::<u64>().ok();
    }
    None
}

/// Processes `x` and `y` fields for `oscillate()` expressions and `rotation-y` for `animate()`.
pub(super) fn normalize_expression_fields(map: &mut Mapping) {
    normalize_oscillate_axis(map, "x", "x");
    normalize_oscillate_axis(map, "y", "y");
    normalize_obj_rotation_y(map);
}

/// Expands `field: oscillate(min, max, period)` into a center value plus a float animation entry.
pub(super) fn normalize_oscillate_axis(map: &mut Mapping, field: &str, axis: &str) {
    let Some(expr) = map
        .get(Value::String(field.to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .map(str::to_string)
    else {
        return;
    };
    let Some(args) = parse_call_args(&expr, "oscillate") else {
        return;
    };
    if args.len() < 3 {
        return;
    }
    let Some(min) = parse_number_token(&args[0]) else {
        return;
    };
    let Some(max) = parse_number_token(&args[1]) else {
        return;
    };
    let Some(period_ms) = parse_duration_token(&args[2]) else {
        return;
    };
    let center = ((min + max) / 2.0).round() as i64;
    let amplitude = ((max - min).abs() / 2.0).round().max(1.0) as u64;

    map.insert(
        Value::String(field.to_string()),
        Value::Number(Number::from(center)),
    );
    let animations = map
        .entry(Value::String("animations".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(seq) = animations.as_sequence_mut() else {
        return;
    };
    let mut params = Mapping::new();
    params.insert(
        Value::String("axis".to_string()),
        Value::String(axis.to_string()),
    );
    params.insert(
        Value::String("amplitude".to_string()),
        Value::Number(Number::from(amplitude)),
    );
    params.insert(
        Value::String("period_ms".to_string()),
        Value::Number(Number::from(period_ms)),
    );
    let mut anim = Mapping::new();
    anim.insert(
        Value::String("name".to_string()),
        Value::String("float".to_string()),
    );
    anim.insert(Value::String("params".to_string()), Value::Mapping(params));
    anim.insert(Value::String("looping".to_string()), Value::Bool(true));
    seq.push(Value::Mapping(anim));
}

/// Expands `rotation-y: animate(start_deg, end_deg, duration)` into a static angle + speed.
pub(super) fn normalize_obj_rotation_y(map: &mut Mapping) {
    let Some(expr) = map
        .get(Value::String("rotation-y".to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .map(str::to_string)
    else {
        return;
    };
    let Some(args) = parse_call_args(&expr, "animate") else {
        return;
    };
    if args.len() < 3 {
        return;
    }
    let Some(start_deg) = parse_number_token(&args[0]) else {
        return;
    };
    let Some(end_deg) = parse_number_token(&args[1]) else {
        return;
    };
    let Some(duration_ms) = parse_duration_token(&args[2]) else {
        return;
    };
    if duration_ms == 0 {
        return;
    }
    let speed = (end_deg - start_deg) / (duration_ms as f32 / 1000.0);
    map.insert(
        Value::String("rotation-y".to_string()),
        serde_yaml::to_value(start_deg).unwrap_or(Value::Null),
    );
    map.insert(
        Value::String("rotate-y-deg-per-sec".to_string()),
        serde_yaml::to_value(speed).unwrap_or(Value::Null),
    );
}

/// Parses `name(arg1, arg2, ...)` and returns the args as strings.
pub(super) fn parse_call_args(expr: &str, name: &str) -> Option<Vec<String>> {
    let open = format!("{name}(");
    let lower = expr.to_ascii_lowercase();
    if !lower.starts_with(&open) || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[open.len()..expr.len() - 1];
    let args = inner
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    Some(args)
}

/// Parses a numeric token, stripping a trailing "deg" suffix.
pub(super) fn parse_number_token(token: &str) -> Option<f32> {
    let trimmed = token.trim().to_ascii_lowercase();
    let no_unit = trimmed.strip_suffix("deg").unwrap_or(trimmed.as_str());
    no_unit.parse::<f32>().ok()
}

/// Parses a duration token using [`parse_duration_ms`].
pub(super) fn parse_duration_token(token: &str) -> Option<u64> {
    let v = Value::String(token.trim().to_string());
    parse_duration_ms(&v)
}
