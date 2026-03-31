//! Shared conversion helpers for scripting domain APIs.

use engine_core::effects::Region;
use engine_core::scene_runtime_types::ObjectRuntimeState;
use rhai::{Array as RhaiArray, Map as RhaiMap};

/// Convert a Region to a Rhai map with fields: x, y, w, h.
pub(crate) fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut out = RhaiMap::new();
    out.insert("x".into(), (region.x as rhai::INT).into());
    out.insert("y".into(), (region.y as rhai::INT).into());
    out.insert("w".into(), (region.width as rhai::INT).into());
    out.insert("h".into(), (region.height as rhai::INT).into());
    out
}

/// Convert ObjectRuntimeState to a Rhai map with fields: visible, offset_x, offset_y.
pub(crate) fn object_state_to_rhai_map(state: &ObjectRuntimeState) -> RhaiMap {
    let mut out = RhaiMap::new();
    out.insert("visible".into(), state.visible.into());
    out.insert("offset_x".into(), (state.offset_x as rhai::INT).into());
    out.insert("offset_y".into(), (state.offset_y as rhai::INT).into());
    out
}

/// Get the list of capabilities supported by a given object kind.
pub(crate) fn kind_capabilities(kind: Option<&str>) -> RhaiArray {
    let mut caps = vec![
        "visible".to_string(),
        "offset.x".to_string(),
        "offset.y".to_string(),
        "position.x".to_string(),
        "position.y".to_string(),
    ];
    if kind.is_some_and(|value| value == "text") {
        caps.push("text.content".to_string());
        caps.push("text.font".to_string());
        caps.push("style.fg".to_string());
        caps.push("style.bg".to_string());
    }
    if kind.is_some_and(|value| value == "obj") {
        caps.push("obj.scale".to_string());
        caps.push("obj.yaw".to_string());
        caps.push("obj.pitch".to_string());
        caps.push("obj.roll".to_string());
        caps.push("obj.orbit_speed".to_string());
        caps.push("obj.surface_mode".to_string());
    }
    caps.into_iter().map(Into::into).collect()
}
