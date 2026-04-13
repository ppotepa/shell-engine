//! Scene domain API: ScriptSceneApi for scene object management, ScriptObjectApi for individual object state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::Value as JsonValue;

use crate::rhai::conversion::{
    json_to_rhai_dynamic, map_get_path_dynamic, map_set_path_dynamic, merge_rhai_maps,
    normalize_set_path, rhai_dynamic_to_json,
};
use crate::BehaviorCommand;

/// Helpers for object state conversion (should ideally be shared or generic).
fn object_state_to_rhai_map(state: &ObjectRuntimeState) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("visible".into(), state.visible.into());
    map.insert("offset_x".into(), (state.offset_x as rhai::INT).into());
    map.insert("offset_y".into(), (state.offset_y as rhai::INT).into());
    map
}

fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut map = RhaiMap::new();
    map.insert("x".into(), (region.x as rhai::INT).into());
    map.insert("y".into(), (region.y as rhai::INT).into());
    map.insert("width".into(), (region.width as rhai::INT).into());
    map.insert("height".into(), (region.height as rhai::INT).into());
    map
}

fn kind_capabilities(kind: Option<&str>) -> RhaiMap {
    let mut cap = RhaiMap::new();
    // Add generic capabilities available to all kinds
    cap.insert("visible".into(), true.into());
    cap.insert("offset.x".into(), true.into());
    cap.insert("offset.y".into(), true.into());
    cap.insert("position.x".into(), true.into());
    cap.insert("position.y".into(), true.into());

    // Kind-specific capabilities
    if let Some(k) = kind {
        match k {
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
            }
            _ => {}
        }
    }
    cap
}

/// Script-facing API for scene management.
#[derive(Clone)]
pub struct ScriptSceneApi {
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

/// Script-facing API for individual scene objects.
#[derive(Clone)]
pub struct ScriptObjectApi {
    target: String,
    snapshot: RhaiMap,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptSceneApi {
    /// Create a new scene API with the given backing state and command queue.
    pub fn new(
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

    /// Get a single scene object API by target (alias or ID).
    pub fn get(&mut self, target: &str) -> ScriptObjectApi {
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

    /// Set a property on a scene object.
    pub fn set(&mut self, target: &str, path: &str, value: RhaiDynamic) {
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

    /// Set the same property on multiple scene objects in a single call.
    ///
    /// ```rhai
    /// scene.set_multi(["star-0", "star-1", ..., "star-19"], "style.fg", col);
    /// ```
    pub fn set_multi(&mut self, targets: RhaiDynamic, path: &str, value: RhaiDynamic) {
        let Ok(arr) = targets.into_array() else {
            return;
        };
        let normalized_path = normalize_set_path(path);
        let Some(json_value) = rhai_dynamic_to_json(&value) else {
            return;
        };
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        for t in arr {
            let Ok(target_str) = t.into_string() else {
                continue;
            };
            let resolved = self
                .target_resolver
                .resolve_alias(&target_str)
                .unwrap_or(&target_str)
                .to_string();
            queue.push(BehaviorCommand::SetProperty {
                target: resolved,
                path: normalized_path.clone(),
                value: json_value.clone(),
            });
        }
    }

    /// Spawn a scene object from a template.
    pub fn spawn(&mut self, template: &str, target: &str) -> bool {
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

    /// Despawn a scene object.
    pub fn despawn(&mut self, target: &str) -> bool {
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

    /// Set vector sprite points and colors.
    pub fn set_vector(&mut self, id: &str, points: RhaiDynamic, fg: &str, bg: &str) {
        self.set(id, "vector.points", points);
        self.set(id, "vector.fg", fg.to_string().into());
        self.set(id, "vector.bg", bg.to_string().into());
    }

    /// Set object visibility.
    pub fn set_visible(&mut self, id: &str, visible: bool) {
        self.set(id, "props.visible", visible.into());
    }

    /// Change the scene background color.
    pub fn set_bg(&mut self, color: &str) {
        if color.trim().is_empty() {
            return;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::SetSceneBg {
            color: color.to_string(),
        });
    }

    /// Batch set multiple properties on a scene object.
    pub fn batch(&mut self, id: &str, props: RhaiMap) {
        for (key, value) in props {
            self.set(id, key.as_str(), value);
        }
    }
}

impl ScriptObjectApi {
    /// Get a property from the object.
    pub fn get(&mut self, path: &str) -> RhaiDynamic {
        map_get_path_dynamic(&self.snapshot, path)
            .or_else(|| map_get_path_dynamic(&self.snapshot, &format!("props.{path}")))
            .unwrap_or_else(|| ().into())
    }

    /// Set a property on the object.
    pub fn set(&mut self, path: &str, value: RhaiDynamic) {
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

/// Register scene API into the Rhai engine.
pub fn register_scene_api(engine: &mut RhaiEngine) {
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
        "set_multi",
        |scene: &mut ScriptSceneApi, targets: RhaiDynamic, path: &str, value: RhaiDynamic| {
            scene.set_multi(targets, path, value);
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
    engine.register_fn("set_bg", |scene: &mut ScriptSceneApi, color: &str| {
        scene.set_bg(color);
    });
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
