//! Level-scoped gameplay state and level catalog.
//!
//! `LevelState` complements [`crate::game_state::GameState`]:
//! - `GameState` is global/session-scoped.
//! - `LevelState` is active-level scoped with a catalog of named level payloads.

use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
struct LevelStore {
    levels: BTreeMap<String, JsonValue>,
    active: Option<String>,
}

/// Thread-safe level catalog with active-level selection.
///
/// Each level is stored as arbitrary JSON and addressed by string id.
/// Read/write path operations target the currently selected level payload.
#[derive(Clone, Debug)]
pub struct LevelState {
    store: Arc<Mutex<LevelStore>>,
}

impl LevelState {
    /// Creates an empty level state with no registered levels.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(LevelStore::default())),
        }
    }

    /// Registers (or replaces) a level payload under `level_id`.
    ///
    /// If no active level is selected yet, the first successfully registered level
    /// becomes active.
    pub fn register_level(&self, level_id: &str, payload: JsonValue) -> bool {
        let id = level_id.trim();
        if id.is_empty() {
            return false;
        }
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.levels.insert(id.to_string(), payload);
        if store.active.is_none() {
            store.active = Some(id.to_string());
        }
        true
    }

    /// Returns sorted ids of all registered levels.
    pub fn ids(&self) -> Vec<String> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.levels.keys().cloned().collect()
    }

    /// Returns true when a level with `level_id` exists.
    pub fn has_level(&self, level_id: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store.levels.contains_key(level_id.trim())
    }

    /// Selects the active level by id.
    pub fn select(&self, level_id: &str) -> bool {
        let id = level_id.trim();
        if id.is_empty() {
            return false;
        }
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.levels.contains_key(id) {
            return false;
        }
        store.active = Some(id.to_string());
        true
    }

    /// Returns the currently selected level id.
    pub fn current_id(&self) -> Option<String> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.active.clone()
    }

    /// Returns the full payload of the active level.
    pub fn active_snapshot(&self) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let active = store.active.as_ref()?;
        store.levels.get(active).cloned()
    }

    /// Gets a value from the active level using JSON-pointer-like path.
    pub fn get(&self, path: &str) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let active = store.active.as_ref()?;
        let payload = store.levels.get(active)?;
        get_path(payload, path)
    }

    /// Sets a value in the active level using path syntax.
    pub fn set(&self, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(payload) = active_payload_mut(&mut store) else {
            return false;
        };
        set_path(payload, path, value)
    }

    /// Checks if a path exists in the active level payload.
    pub fn has(&self, path: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        let Some(payload) = active_payload(&store) else {
            return false;
        };
        get_path(payload, path).is_some()
    }

    /// Removes a value at `path` from the active level payload.
    pub fn remove(&self, path: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(payload) = active_payload_mut(&mut store) else {
            return false;
        };
        remove_path(payload, path)
    }

    /// Pushes a value to an array at `path` in the active level payload.
    pub fn push(&self, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(payload) = active_payload_mut(&mut store) else {
            return false;
        };
        push_path(payload, path, value)
    }
}

impl Default for LevelState {
    fn default() -> Self {
        Self::new()
    }
}

fn active_payload(store: &LevelStore) -> Option<&JsonValue> {
    let active = store.active.as_ref()?;
    store.levels.get(active)
}

fn active_payload_mut(store: &mut LevelStore) -> Option<&mut JsonValue> {
    let active = store.active.clone()?;
    store.levels.get_mut(&active)
}

fn get_path(payload: &JsonValue, path: &str) -> Option<JsonValue> {
    if path.is_empty() || path == "/" {
        return Some(payload.clone());
    }
    payload.pointer(path).cloned()
}

fn set_path(payload: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    if path.is_empty() || path == "/" {
        *payload = value;
        return true;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        if !current.is_object() {
            *current = json!({});
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = json!({});
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    obj.insert(key.to_string(), value);
    true
}

fn remove_path(payload: &mut JsonValue, path: &str) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    obj.remove(key).is_some()
}

fn push_path(payload: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        if !current.is_object() {
            *current = json!({});
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = json!({});
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    let entry = obj.entry(key.to_string()).or_insert_with(|| json!([]));
    if let Some(arr) = entry.as_array_mut() {
        arr.push(value);
    } else {
        let prev = entry.clone();
        *entry = json!([prev, value]);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::LevelState;
    use serde_json::json;

    #[test]
    fn registers_and_selects_levels() {
        let levels = LevelState::new();
        assert!(levels.register_level("a", json!({"difficulty":1})));
        assert!(levels.register_level("b", json!({"difficulty":2})));
        assert_eq!(levels.current_id().as_deref(), Some("a"));
        assert!(levels.select("b"));
        assert_eq!(levels.current_id().as_deref(), Some("b"));
    }

    #[test]
    fn set_and_get_paths_on_active_level() {
        let levels = LevelState::new();
        assert!(levels.register_level("a", json!({})));
        assert!(levels.set("/player/lives", json!(3)));
        assert_eq!(levels.get("/player/lives"), Some(json!(3)));
        assert!(levels.has("/player/lives"));
    }

    #[test]
    fn push_converts_scalar_to_array() {
        let levels = LevelState::new();
        assert!(levels.register_level("a", json!({"events":"boot"})));
        assert!(levels.push("/events", json!("spawn")));
        assert_eq!(levels.get("/events"), Some(json!(["boot", "spawn"])));
    }
}
