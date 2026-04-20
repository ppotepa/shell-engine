//! Pure scene query/snapshot helpers used by the Rhai-facing scene API.

use std::collections::HashMap;

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
use serde_json::Value as JsonValue;

use crate::rhai::conversion::{json_to_rhai_dynamic, merge_rhai_maps};

pub(crate) fn object_state_to_rhai_map(state: &ObjectRuntimeState) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("visible".into(), state.visible.into());
    map.insert("offset_x".into(), (state.offset_x as rhai::INT).into());
    map.insert("offset_y".into(), (state.offset_y as rhai::INT).into());
    map
}

pub(crate) fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("x".into(), (region.x as rhai::INT).into());
    map.insert("y".into(), (region.y as rhai::INT).into());
    map.insert("width".into(), (region.width as rhai::INT).into());
    map.insert("height".into(), (region.height as rhai::INT).into());
    map
}

pub(crate) fn kind_capabilities(kind: Option<&str>) -> RhaiMap {
    let mut cap = RhaiMap::new();
    cap.insert("visible".into(), true.into());
    cap.insert("offset.x".into(), true.into());
    cap.insert("offset.y".into(), true.into());
    cap.insert("position.x".into(), true.into());
    cap.insert("position.y".into(), true.into());

    if let Some(kind) = kind {
        match kind {
            "text" => {
                cap.insert("text.content".into(), true.into());
                cap.insert("text.font".into(), true.into());
                cap.insert("style.fg".into(), true.into());
                cap.insert("style.bg".into(), true.into());
            }
            "obj" => {
                cap.insert("obj.scale".into(), true.into());
                cap.insert("obj.yaw".into(), true.into());
                cap.insert("obj.pitch".into(), true.into());
                cap.insert("obj.roll".into(), true.into());
                cap.insert("obj.orbit_speed".into(), true.into());
                cap.insert("obj.surface_mode".into(), true.into());
                cap.insert("obj.world.x".into(), true.into());
                cap.insert("obj.world.y".into(), true.into());
                cap.insert("obj.world.z".into(), true.into());
                cap.insert("obj.cam.wx".into(), true.into());
                cap.insert("obj.cam.wy".into(), true.into());
                cap.insert("obj.cam.wz".into(), true.into());
                cap.insert("obj.view.rx".into(), true.into());
                cap.insert("obj.view.ry".into(), true.into());
                cap.insert("obj.view.rz".into(), true.into());
                cap.insert("obj.view.ux".into(), true.into());
                cap.insert("obj.view.uy".into(), true.into());
                cap.insert("obj.view.uz".into(), true.into());
                cap.insert("obj.view.fx".into(), true.into());
                cap.insert("obj.view.fy".into(), true.into());
                cap.insert("obj.view.fz".into(), true.into());
            }
            _ => {}
        }
    }

    cap
}

pub(crate) fn runtime_object_name(object_id: &str) -> String {
    object_id
        .rsplit('/')
        .next()
        .unwrap_or(object_id)
        .to_string()
}

fn scene_object_aliases(target_resolver: &TargetResolver, object_id: &str) -> Vec<String> {
    let mut aliases: Vec<String> = target_resolver
        .aliases_snapshot()
        .into_iter()
        .filter_map(|(alias, resolved)| (resolved == object_id).then_some(alias))
        .collect();
    aliases.push(runtime_object_name(object_id));
    aliases.sort();
    aliases.dedup();
    aliases
}

fn scene_object_tags(kind: Option<&str>, props: Option<&JsonValue>) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(kind) = kind.filter(|value| !value.trim().is_empty()) {
        tags.push(kind.to_string());
    }
    if let Some(extra_tags) = props
        .and_then(|value| value.as_object())
        .and_then(|map| map.get("tags"))
        .and_then(|value| value.as_array())
    {
        for tag in extra_tags {
            if let Some(tag) = tag.as_str().filter(|value| !value.trim().is_empty()) {
                tags.push(tag.to_string());
            }
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

pub(crate) fn build_object_entry(
    object_states: &HashMap<String, ObjectRuntimeState>,
    object_kinds: &HashMap<String, String>,
    object_props: &HashMap<String, JsonValue>,
    object_regions: &HashMap<String, Region>,
    object_text: &HashMap<String, String>,
    target_resolver: &TargetResolver,
    object_id: &str,
) -> RhaiMap {
    let Some(state) = object_states.get(object_id) else {
        return RhaiMap::new();
    };
    let kind = object_kinds
        .get(object_id)
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let name = runtime_object_name(object_id);
    let aliases = scene_object_aliases(target_resolver, object_id);
    let tags = scene_object_tags(Some(kind.as_str()), object_props.get(object_id));

    let mut entry = RhaiMap::new();
    entry.insert("id".into(), object_id.to_string().into());
    entry.insert("kind".into(), kind.clone().into());
    entry.insert("name".into(), name.into());
    entry.insert(
        "aliases".into(),
        aliases
            .into_iter()
            .map(RhaiDynamic::from)
            .collect::<RhaiArray>()
            .into(),
    );
    entry.insert(
        "tags".into(),
        tags.iter()
            .cloned()
            .map(RhaiDynamic::from)
            .collect::<RhaiArray>()
            .into(),
    );
    entry.insert("state".into(), object_state_to_rhai_map(state).into());
    if let Some(region) = object_regions.get(object_id) {
        entry.insert("region".into(), region_to_rhai_map(region).into());
    }
    if let Some(text) = object_text.get(object_id) {
        let mut text_map = RhaiMap::new();
        text_map.insert("content".into(), text.clone().into());
        entry.insert("text".into(), text_map.into());
    }
    let mut props = RhaiMap::new();
    props.insert("visible".into(), state.visible.into());
    let mut offset = RhaiMap::new();
    offset.insert("x".into(), (state.offset_x as rhai::INT).into());
    offset.insert("y".into(), (state.offset_y as rhai::INT).into());
    props.insert("offset".into(), offset.into());
    if let Some(text) = object_text.get(object_id) {
        let mut text_props = RhaiMap::new();
        text_props.insert("content".into(), text.clone().into());
        props.insert("text".into(), text_props.into());
    }
    if let Some(extra_props) = object_props.get(object_id) {
        if let Some(extra_map) = json_to_rhai_dynamic(extra_props).try_cast::<RhaiMap>() {
            merge_rhai_maps(&mut props, &extra_map);
        }
    }
    entry.insert("props".into(), props.into());
    entry.insert(
        "capabilities".into(),
        kind_capabilities(Some(kind.as_str())).into(),
    );
    entry
}

pub(crate) fn object_state_from_entry(entry: &RhaiMap) -> Option<ObjectRuntimeState> {
    entry
        .get("state")
        .and_then(|state| state.clone().try_cast::<RhaiMap>())
        .and_then(|state| {
            let visible = state.get("visible")?.clone().try_cast::<bool>()?;
            let offset_x = state
                .get("offset_x")?
                .clone()
                .try_cast::<rhai::INT>()
                .and_then(|value| i32::try_from(value).ok())?;
            let offset_y = state
                .get("offset_y")?
                .clone()
                .try_cast::<rhai::INT>()
                .and_then(|value| i32::try_from(value).ok())?;
            Some(ObjectRuntimeState {
                visible,
                offset_x,
                offset_y,
                ..ObjectRuntimeState::default()
            })
        })
}

pub(crate) fn object_matches_name(
    target_resolver: &TargetResolver,
    object_id: &str,
    requested_name: &str,
) -> bool {
    let requested_name = requested_name.trim();
    if requested_name.is_empty() {
        return false;
    }
    scene_object_aliases(target_resolver, object_id)
        .into_iter()
        .any(|name| name == requested_name)
}

pub(crate) fn object_matches_tag(
    object_kinds: &HashMap<String, String>,
    object_props: &HashMap<String, JsonValue>,
    object_id: &str,
    requested_tag: &str,
) -> bool {
    let requested_tag = requested_tag.trim();
    if requested_tag.is_empty() {
        return false;
    }
    scene_object_tags(
        object_kinds.get(object_id).map(String::as_str),
        object_props.get(object_id),
    )
    .into_iter()
    .any(|tag| tag == requested_tag)
}
