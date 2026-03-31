//! Scene domain API: ScriptSceneApi for scene object management, ScriptObjectApi for individual object state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::Value as JsonValue;

use super::helpers::{kind_capabilities, object_state_to_rhai_map, region_to_rhai_map};
use crate::rhai_util::{
    json_to_rhai_dynamic, map_get_path_dynamic, map_set_path_dynamic, merge_rhai_maps,
    normalize_set_path, rhai_dynamic_to_json,
};
use crate::BehaviorCommand;

#[derive(Clone)]
pub(crate) struct ScriptSceneApi {
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
pub(crate) struct ScriptObjectApi {
    target: String,
    snapshot: RhaiMap,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptSceneApi {
    pub(crate) fn new(
        object_states: Arc<HashMap<String, ObjectRuntimeState>>,
        object_kinds: Arc<HashMap<String, String>>,
        object_props: Arc<HashMap<String, JsonValue>>,
        object_regions: Arc<HashMap<String, Region>>,
        object_text: Arc<HashMap<String, String>>,
        target_resolver: Arc<TargetResolver>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            object_states,
            object_kinds,
            object_props,
            object_regions,
            object_text,
            target_resolver,
            queue,
        }
    }

    /// Lazily build a single-object entry on demand instead of pre-building the
    /// entire 50+ object map. This is the critical hot-path optimization (OPT-3).
    fn get(&mut self, target: &str) -> ScriptObjectApi {
        // Resolve alias → real object id.
        let object_id = self.target_resolver.resolve_alias(target).unwrap_or(target);

        let snapshot = self.build_object_entry(object_id);
        ScriptObjectApi {
            target: object_id.to_string(),
            snapshot,
            queue: Arc::clone(&self.queue),
        }
    }

    fn build_object_entry(&self, object_id: &str) -> RhaiMap {
        let Some(state) = self.object_states.get(object_id) else {
            return RhaiMap::new();
        };
        let kind = self
            .object_kinds
            .get(object_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut entry = RhaiMap::new();
        entry.insert("id".into(), object_id.to_string().into());
        entry.insert("kind".into(), kind.clone().into());
        entry.insert("state".into(), object_state_to_rhai_map(state).into());
        if let Some(region) = self.object_regions.get(object_id) {
            entry.insert("region".into(), region_to_rhai_map(region).into());
        }
        if let Some(text) = self.object_text.get(object_id) {
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
        if let Some(text) = self.object_text.get(object_id) {
            let mut text_props = RhaiMap::new();
            text_props.insert("content".into(), text.clone().into());
            props.insert("text".into(), text_props.into());
        }
        if let Some(extra_props) = self.object_props.get(object_id) {
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

    fn set(&mut self, target: &str, path: &str, value: RhaiDynamic) {
        let normalized_path = normalize_set_path(path);
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        // Resolve alias for the target.
        let resolved = self
            .target_resolver
            .resolve_alias(target)
            .unwrap_or(target)
            .to_string();
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::SetProperty {
            target: resolved,
            path: normalized_path,
            value,
        });
    }

    fn spawn(&mut self, template: &str, target: &str) -> bool {
        if template.trim().is_empty() || target.trim().is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::SceneSpawn {
            template: template.to_string(),
            target: target.to_string(),
        });
        true
    }

    fn despawn(&mut self, target: &str) -> bool {
        if target.trim().is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::SceneDespawn {
            target: target.to_string(),
        });
        true
    }

    fn set_vector(&mut self, id: &str, points: RhaiDynamic, fg: &str, bg: &str) {
        self.set(id, "vector.points", points);
        self.set(id, "vector.fg", fg.to_string().into());
        self.set(id, "vector.bg", bg.to_string().into());
    }

    fn set_visible(&mut self, id: &str, visible: bool) {
        self.set(id, "props.visible", visible.into());
    }

    fn batch(&mut self, id: &str, props: RhaiMap) {
        for (key, value) in props {
            self.set(id, key.as_str(), value);
        }
    }
}

impl ScriptObjectApi {
    fn get(&mut self, path: &str) -> RhaiDynamic {
        map_get_path_dynamic(&self.snapshot, path)
            .or_else(|| map_get_path_dynamic(&self.snapshot, &format!("props.{path}")))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) {
        let normalized_path = normalize_set_path(path);
        if !map_set_path_dynamic(&mut self.snapshot, &normalized_path, value.clone()) {
            return;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::SetProperty {
            target: self.target.clone(),
            path: normalized_path,
            value,
        });
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptSceneApi>("SceneApi");
    engine.register_type_with_name::<ScriptObjectApi>("SceneObject");

    engine.register_fn("get", |scene: &mut ScriptSceneApi, target: &str| {
        scene.get(target)
    });
    engine.register_fn(
        "set",
        |scene: &mut ScriptSceneApi, target: &str, path: &str, value: RhaiDynamic| {
            scene.set(target, path, value);
        },
    );
    engine.register_fn(
        "spawn_object",
        |scene: &mut ScriptSceneApi, template: &str, target: &str| scene.spawn(template, target),
    );
    engine.register_fn(
        "despawn_object",
        |scene: &mut ScriptSceneApi, target: &str| scene.despawn(target),
    );
    engine.register_fn(
        "set_vector",
        |scene: &mut ScriptSceneApi, id: &str, points: RhaiDynamic, fg: &str, bg: &str| {
            scene.set_vector(id, points, fg, bg);
        },
    );
    engine.register_fn(
        "set_visible",
        |scene: &mut ScriptSceneApi, id: &str, visible: bool| {
            scene.set_visible(id, visible);
        },
    );
    engine.register_fn(
        "batch",
        |scene: &mut ScriptSceneApi, id: &str, props: RhaiMap| {
            scene.batch(id, props);
        },
    );

    engine.register_fn("get", |object: &mut ScriptObjectApi, path: &str| {
        object.get(path)
    });
    engine.register_fn(
        "set",
        |object: &mut ScriptObjectApi, path: &str, value: RhaiDynamic| {
            object.set(path, value);
        },
    );
}
