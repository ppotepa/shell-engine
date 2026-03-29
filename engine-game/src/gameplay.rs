//! Shared gameplay world state for dynamic gameplay entities.
//!
//! This crate intentionally keeps the data model generic. Engine systems and
//! Rhai scripts can use it to spawn, query, mutate, and despawn gameplay
//! entities without binding the runtime to one specific game.

use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::components::{Collider2D, Lifetime, PhysicsBody2D, Transform2D, VisualBinding};

/// Snapshot of a spawned gameplay entity.
#[derive(Clone, Debug, PartialEq)]
pub struct GameplayEntity {
    pub id: u64,
    pub kind: String,
    pub tags: BTreeSet<String>,
    pub data: JsonValue,
}

#[derive(Clone, Debug, Default)]
struct GameplayStore {
    next_id: u64,
    entities: BTreeMap<u64, GameplayEntity>,
    transforms: BTreeMap<u64, Transform2D>,
    physics: BTreeMap<u64, PhysicsBody2D>,
    colliders: BTreeMap<u64, Collider2D>,
    lifetimes: BTreeMap<u64, Lifetime>,
    visuals: BTreeMap<u64, VisualBinding>,
}

/// Thread-safe gameplay entity store.
///
/// The store is generic on purpose:
/// - `kind` is a lightweight gameplay classification.
/// - `tags` are optional role labels.
/// - `data` carries all gameplay-specific state.
#[derive(Clone, Debug)]
pub struct GameplayWorld {
    store: Arc<Mutex<GameplayStore>>,
}

impl GameplayWorld {
    /// Creates an empty gameplay world.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(GameplayStore::default())),
        }
    }

    /// Removes all gameplay entities and resets the id counter.
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.lock() {
            *store = GameplayStore::default();
        }
    }

    /// Returns the number of active entities.
    pub fn count(&self) -> usize {
        let Ok(store) = self.store.lock() else {
            return 0;
        };
        store.entities.len()
    }

    /// Spawns a new entity with the given kind and payload.
    ///
    /// If `payload` is an object with a top-level `tags: [...]` array, those
    /// tags are extracted into the entity tag set and removed from the stored
    /// payload.
    pub fn spawn(&self, kind: &str, payload: JsonValue) -> Option<u64> {
        let kind = kind.trim();
        if kind.is_empty() {
            return None;
        }

        let mut store = self.store.lock().ok()?;
        store.next_id = store.next_id.wrapping_add(1);
        if store.next_id == 0 {
            store.next_id = 1;
        }
        let id = store.next_id;
        let (tags, data) = split_payload(payload);
        store.entities.insert(
            id,
            GameplayEntity {
                id,
                kind: kind.to_string(),
                tags,
                data,
            },
        );
        Some(id)
    }

    /// Removes an entity by id.
    pub fn despawn(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let removed = store.entities.remove(&id).is_some();
        store.transforms.remove(&id);
        store.physics.remove(&id);
        store.colliders.remove(&id);
        store.lifetimes.remove(&id);
        store.visuals.remove(&id);
        removed
    }

    /// Returns `true` if the entity exists.
    pub fn exists(&self, id: u64) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store.entities.contains_key(&id)
    }

    /// Returns the kind of an entity.
    pub fn kind_of(&self, id: u64) -> Option<String> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).map(|entity| entity.kind.clone())
    }

    /// Returns the tags of an entity.
    pub fn tags(&self, id: u64) -> Vec<String> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .get(&id)
            .map(|entity| entity.tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns the ids of all entities, ordered by creation order.
    pub fn ids(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.entities.keys().copied().collect()
    }

    /// Returns the ids of all entities with the given kind.
    pub fn query_kind(&self, kind: &str) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .iter()
            .filter(|(_, entity)| entity.kind == kind)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the number of entities with the given kind.
    pub fn count_kind(&self, kind: &str) -> usize {
        self.query_kind(kind).len()
    }

    /// Returns the first entity id with the given kind, if any.
    pub fn first_kind(&self, kind: &str) -> Option<u64> {
        self.query_kind(kind).into_iter().next()
    }

    /// Returns the ids of all entities containing the given tag.
    pub fn query_tag(&self, tag: &str) -> Vec<u64> {
        let tag = tag.trim();
        if tag.is_empty() {
            return Vec::new();
        }
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .iter()
            .filter(|(_, entity)| entity.tags.contains(tag))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the number of entities containing the given tag.
    pub fn count_tag(&self, tag: &str) -> usize {
        self.query_tag(tag).len()
    }

    /// Returns the first entity id containing the given tag, if any.
    pub fn first_tag(&self, tag: &str) -> Option<u64> {
        self.query_tag(tag).into_iter().next()
    }

    /// Returns a clone of an entity snapshot.
    pub fn get_entity(&self, id: u64) -> Option<GameplayEntity> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).cloned()
    }

    /// Returns the entire data JSON blob of an entity, or None if the entity doesn't exist.
    pub fn data(&self, id: u64) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).map(|entity| entity.data.clone())
    }

    /// Bulk writes multiple properties into an entity using a map of key-value pairs.
    /// Each key is treated as a JSON pointer path (prefixed with /).
    pub fn set_many(&self, id: u64, map: &std::collections::BTreeMap<String, JsonValue>) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        for (key, value) in map {
            if !set_path(&mut entity.data, &format!("/{}", key), value.clone()) {
                return false;
            }
        }
        true
    }

    // --- Component accessors -------------------------------------------------

    pub fn set_transform(&self, id: u64, xf: Transform2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.transforms.insert(id, xf);
        true
    }

    pub fn transform(&self, id: u64) -> Option<Transform2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.transforms.get(&id).copied()
    }

    pub fn set_physics(&self, id: u64, body: PhysicsBody2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.physics.insert(id, body);
        true
    }

    pub fn physics(&self, id: u64) -> Option<PhysicsBody2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.physics.get(&id).copied()
    }

    pub fn set_collider(&self, id: u64, collider: Collider2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.colliders.insert(id, collider);
        true
    }

    pub fn collider(&self, id: u64) -> Option<Collider2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.colliders.get(&id).cloned()
    }

    pub fn set_lifetime(&self, id: u64, lifetime: Lifetime) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.lifetimes.insert(id, lifetime);
        true
    }

    pub fn lifetime(&self, id: u64) -> Option<Lifetime> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.lifetimes.get(&id).copied()
    }

    pub fn set_visual(&self, id: u64, binding: VisualBinding) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.visuals.insert(id, binding);
        true
    }

    pub fn visual(&self, id: u64) -> Option<VisualBinding> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.visuals.get(&id).cloned()
    }

    pub fn add_visual(&self, id: u64, visual_id: String) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store
            .visuals
            .entry(id)
            .or_default()
            .additional_visuals
            .push(visual_id);
        true
    }

    pub fn ids_with_physics(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.physics.keys().copied().collect()
    }

    pub fn ids_with_lifetime(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.lifetimes.keys().copied().collect()
    }

    pub fn ids_with_colliders(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.colliders.keys().copied().collect()
    }

    pub fn ids_with_visual_binding(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.visuals.keys().copied().collect()
    }

    pub fn remove_lifetime(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.lifetimes.remove(&id);
        }
    }

    /// Reads a value from an entity payload using JSON pointer notation.
    pub fn get(&self, id: u64, path: &str) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let entity = store.entities.get(&id)?;
        get_path(&entity.data, path)
    }

    /// Writes a value into an entity payload using JSON pointer notation.
    pub fn set(&self, id: u64, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        set_path(&mut entity.data, path, value)
    }

    /// Checks if a value exists at `path` in the entity payload.
    pub fn has(&self, id: u64, path: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get(&id) else {
            return false;
        };
        get_path(&entity.data, path).is_some()
    }

    /// Removes a value at `path` in the entity payload.
    pub fn remove(&self, id: u64, path: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        remove_path(&mut entity.data, path)
    }

    /// Pushes a value into an array at `path` in the entity payload.
    pub fn push(&self, id: u64, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        push_path(&mut entity.data, path, value)
    }
}

impl Default for GameplayWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn split_payload(payload: JsonValue) -> (BTreeSet<String>, JsonValue) {
    let mut tags = BTreeSet::new();
    let data = match payload {
        JsonValue::Object(mut map) => {
            if let Some(JsonValue::Array(values)) = map.remove("tags") {
                for value in values {
                    if let Some(tag) = value.as_str().map(str::trim).filter(|tag| !tag.is_empty()) {
                        tags.insert(tag.to_string());
                    }
                }
            }

            JsonValue::Object(map)
        }
        other => return (tags, other),
    };
    (tags, data)
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
            *current = JsonValue::Object(JsonMap::new());
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
        *current = JsonValue::Object(JsonMap::new());
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
            *current = JsonValue::Object(JsonMap::new());
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
        *current = JsonValue::Object(JsonMap::new());
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
    use super::GameplayWorld;
    use serde_json::json;

    #[test]
    fn spawns_queries_and_mutates_entities() {
        let world = GameplayWorld::new();
        let id = world
            .spawn(
                "bullet",
                json!({
                    "tags": ["projectile", "player"],
                    "x": 10,
                    "y": 20
                }),
            )
            .expect("spawn should return an id");
        assert!(world.exists(id));
        assert_eq!(world.kind_of(id).as_deref(), Some("bullet"));
        assert_eq!(world.query_kind("bullet"), vec![id]);
        assert_eq!(world.query_tag("player"), vec![id]);
        assert_eq!(world.get(id, "/x"), Some(json!(10)));
        assert!(world.set(id, "/velocity/x", json!(4)));
        assert_eq!(world.get(id, "/velocity/x"), Some(json!(4)));
        assert!(world.remove(id, "/velocity/x"));
        assert!(!world.has(id, "/velocity/x"));
        assert!(world.despawn(id));
        assert!(!world.exists(id));
    }

    #[test]
    fn clear_resets_world() {
        let world = GameplayWorld::new();
        assert!(world.spawn("asteroid", json!({"x": 1})).is_some());
        assert_eq!(world.count(), 1);
        world.clear();
        assert_eq!(world.count(), 0);
        assert!(world.ids().is_empty());
        assert_eq!(world.query_kind("asteroid"), Vec::<u64>::new());
    }
}
