//! Shared gameplay world state for dynamic gameplay entities.
//!
//! This crate intentionally keeps the data model generic. Engine systems and
//! Rhai scripts can use it to spawn, query, mutate, and despawn gameplay
//! entities without binding the runtime to one specific game.

use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::components::{Collider2D, EntityTimers, GameplayEvent, Lifetime, PhysicsBody2D, TopDownShipController, Transform2D, VisualBinding, WrapBounds};


/// Snapshot of a spawned gameplay entity.
#[derive(Clone, Debug, PartialEq)]
pub struct GameplayEntity {
    pub id: u64,
    pub kind: String,
    pub tags: BTreeSet<String>,
    pub data: JsonValue,
}

#[derive(Clone, Debug)]
struct GameplayStore {
    next_id: u64,
    entities: BTreeMap<u64, GameplayEntity>,
    transforms: BTreeMap<u64, Transform2D>,
    physics: BTreeMap<u64, PhysicsBody2D>,
    colliders: BTreeMap<u64, Collider2D>,
    lifetimes: BTreeMap<u64, Lifetime>,
    visuals: BTreeMap<u64, VisualBinding>,
    timers: BTreeMap<u64, EntityTimers>,
    wrap_bounds: BTreeMap<u64, WrapBounds>,
    ship_controllers: BTreeMap<u64, TopDownShipController>,
    /// Parent → child entity IDs. Children are auto-despawned when parent despawns.
    children: BTreeMap<u64, Vec<u64>>,
    /// Gameplay events accumulated this frame (cleared each frame start).
    events: Vec<GameplayEvent>,
    rng_seed: u64,
    world_bounds: Option<WrapBounds>,
}

impl Default for GameplayStore {
    fn default() -> Self {
        Self {
            next_id: 0,
            entities: BTreeMap::new(),
            transforms: BTreeMap::new(),
            physics: BTreeMap::new(),
            colliders: BTreeMap::new(),
            lifetimes: BTreeMap::new(),
            visuals: BTreeMap::new(),
            timers: BTreeMap::new(),
            wrap_bounds: BTreeMap::new(),
            ship_controllers: BTreeMap::new(),
            children: BTreeMap::new(),
            events: Vec::new(),
            rng_seed: 1337,
            world_bounds: None,
        }
    }
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

    /// Removes an entity by id. Any children registered via `register_child` are
    /// also despawned recursively.
    pub fn despawn(&self, id: u64) -> bool {
        let (removed, child_ids) = {
            let Ok(mut store) = self.store.lock() else {
                return false;
            };
            let removed = store.entities.remove(&id).is_some();
            store.transforms.remove(&id);
            store.physics.remove(&id);
            store.colliders.remove(&id);
            store.lifetimes.remove(&id);
            store.visuals.remove(&id);
            store.timers.remove(&id);
            store.wrap_bounds.remove(&id);
            store.ship_controllers.remove(&id);
            let child_ids = store.children.remove(&id).unwrap_or_default();
            (removed, child_ids)
        };
        // Recursively despawn children after releasing the lock.
        for child_id in child_ids {
            self.despawn(child_id);
        }
        removed
    }

    /// Registers `child` as a child of `parent`. When `parent` is despawned,
    /// `child` is automatically despawned too.
    pub fn register_child(&self, parent: u64, child: u64) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        if !store.entities.contains_key(&parent) { return false; }
        store.children.entry(parent).or_default().push(child);
        true
    }

    /// Despawns all children registered under `parent` without despawning the parent itself.
    pub fn despawn_children(&self, parent: u64) {
        let child_ids = {
            let Ok(mut store) = self.store.lock() else { return };
            store.children.remove(&parent).unwrap_or_default()
        };
        for child_id in child_ids {
            self.despawn(child_id);
        }
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

    // ── Timers (cooldowns + statuses) ────────────────────────────────────

    /// Start or reset a named cooldown for `id`. Counts down to 0 (ready).
    pub fn cooldown_start(&self, id: u64, name: &str, ms: i32) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        if !store.entities.contains_key(&id) { return false; }
        store.timers.entry(id).or_default().cooldowns.insert(name.to_string(), ms.max(0));
        true
    }

    /// Returns `true` if the named cooldown has expired (or was never started).
    pub fn cooldown_ready(&self, id: u64, name: &str) -> bool {
        let Ok(store) = self.store.lock() else { return true };
        store.timers.get(&id)
            .and_then(|t| t.cooldowns.get(name))
            .map(|&ms| ms <= 0)
            .unwrap_or(true)
    }

    /// Returns remaining ms for a cooldown, or 0 if ready/absent.
    pub fn cooldown_remaining(&self, id: u64, name: &str) -> i32 {
        let Ok(store) = self.store.lock() else { return 0 };
        store.timers.get(&id)
            .and_then(|t| t.cooldowns.get(name))
            .copied()
            .unwrap_or(0)
            .max(0)
    }

    /// Add or refresh a named status effect for `id`. Active while remaining > 0.
    pub fn status_add(&self, id: u64, name: &str, ms: i32) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        if !store.entities.contains_key(&id) { return false; }
        store.timers.entry(id).or_default().statuses.insert(name.to_string(), ms.max(1));
        true
    }

    /// Returns `true` if the named status is active (remaining > 0).
    pub fn status_has(&self, id: u64, name: &str) -> bool {
        let Ok(store) = self.store.lock() else { return false };
        store.timers.get(&id)
            .and_then(|t| t.statuses.get(name))
            .map(|&ms| ms > 0)
            .unwrap_or(false)
    }

    /// Returns remaining ms for a status, or 0 if inactive/absent.
    pub fn status_remaining(&self, id: u64, name: &str) -> i32 {
        let Ok(store) = self.store.lock() else { return 0 };
        store.timers.get(&id)
            .and_then(|t| t.statuses.get(name))
            .copied()
            .unwrap_or(0)
            .max(0)
    }

    pub fn ids_with_timers(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else { return Vec::new() };
        store.timers.keys().copied().collect()
    }

    /// Tick all timers by `dt_ms`. Cooldowns clamp at 0; expired statuses are removed.
    pub fn tick_timers(&self, dt_ms: u64) {
        let dt = dt_ms as i32;
        let Ok(mut store) = self.store.lock() else { return };
        store.timers.retain(|_, timers| {
            for v in timers.cooldowns.values_mut() {
                *v = (*v - dt).max(0);
            }
            timers.statuses.retain(|_, v| {
                *v -= dt;
                *v > 0
            });
            // Keep the entry as long as there are any cooldowns (even at 0)
            !timers.cooldowns.is_empty() || !timers.statuses.is_empty()
        });
    }

    // ── WrapBounds ────────────────────────────────────────────────────────

    /// Enable toroidal position wrap for `id` within `bounds`.
    pub fn set_wrap_bounds(&self, id: u64, bounds: WrapBounds) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        if !store.entities.contains_key(&id) { return false; }
        store.wrap_bounds.insert(id, bounds);
        true
    }

    pub fn wrap_bounds_for(&self, id: u64) -> Option<WrapBounds> {
        let Ok(store) = self.store.lock() else { return None };
        store.wrap_bounds.get(&id).copied()
    }

    pub fn remove_wrap_bounds(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.wrap_bounds.remove(&id);
        }
    }

    pub fn ids_with_wrap(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else { return Vec::new() };
        store.wrap_bounds.keys().copied().collect()
    }

    /// Apply toroidal wrap for all entities with WrapBounds after physics integration.
    pub fn apply_wrap(&self) {
        let ids = self.ids_with_wrap();
        for id in ids {
            let Some(bounds) = self.wrap_bounds_for(id) else { continue };
            let Some(mut xf) = self.transform(id) else { continue };
            let nx = bounds.wrap_x(xf.x);
            let ny = bounds.wrap_y(xf.y);
            if nx != xf.x || ny != xf.y {
                xf.x = nx;
                xf.y = ny;
                let _ = self.set_transform(id, xf);
            }
        }
    }

    /// Attach a TopDownShipController to an entity.
    pub fn attach_controller(&self, id: u64, controller: TopDownShipController) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        if !store.entities.contains_key(&id) { return false; }
        store.ship_controllers.insert(id, controller);
        true
    }

    /// Retrieve a ship controller for an entity.
    pub fn controller(&self, id: u64) -> Option<TopDownShipController> {
        let Ok(store) = self.store.lock() else { return None };
        store.ship_controllers.get(&id).cloned()
    }

    /// Mutate a ship controller. Returns false if entity has no controller.
    pub fn with_controller<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut TopDownShipController),
    {
        let Ok(mut store) = self.store.lock() else { return false };
        if let Some(ctrl) = store.ship_controllers.get_mut(&id) {
            f(ctrl);
            true
        } else {
            false
        }
    }

    /// Get all entity IDs with controllers.
    pub fn ids_with_controller(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else { return Vec::new() };
        store.ship_controllers.keys().copied().collect()
    }

    /// Remove a ship controller from an entity.
    pub fn remove_controller(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.ship_controllers.remove(&id);
        }
    }

    /// Emit a gameplay event to be collected and polled by scripts.
    pub fn emit_event(&self, event: GameplayEvent) {
        if let Ok(mut store) = self.store.lock() {
            store.events.push(event);
        }
    }

    /// Get all events of a specific type without clearing the buffer.
    ///
    /// Note: This returns copies of events without clearing. Call clear_events() separately.
    pub fn poll_events(&self, event_type: &str) -> Vec<(u64, u64)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };

        let mut results = Vec::new();
        match event_type {
            "collision_enter" => {
                for event in &store.events {
                    let GameplayEvent::CollisionEnter { a, b } = event;
                    results.push((*a, *b));
                }
            }
            _ => {}
        }
        results
    }

    /// Clear all accumulated events (call at start of frame before polling).
    pub fn clear_events(&self) {
        if let Ok(mut store) = self.store.lock() {
            store.events.clear();
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

    // ── RNG ──────────────────────────────────────────────────────────────

    /// LCG random int in [min, max] inclusive. Advances internal seed.
    pub fn rand_i(&self, min: i32, max: i32) -> i32 {
        let Ok(mut store) = self.store.lock() else { return min };
        store.rng_seed = store.rng_seed.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fff_ffff;
        let range = (max - min).abs() as u64 + 1;
        min + (store.rng_seed % range) as i32
    }

    /// Reset the RNG seed.
    pub fn rand_seed(&self, seed: i64) {
        let Ok(mut store) = self.store.lock() else { return };
        store.rng_seed = seed as u64 & 0x7fff_ffff;
    }

    // ── World-level wrap bounds ───────────────────────────────────────────

    /// Store global world bounds (used by enable_wrap_bounds).
    pub fn set_world_bounds(&self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        let Ok(mut store) = self.store.lock() else { return };
        store.world_bounds = Some(WrapBounds { min_x, max_x, min_y, max_y });
    }

    /// Read global world bounds.
    pub fn world_bounds(&self) -> Option<WrapBounds> {
        let Ok(store) = self.store.lock() else { return None };
        store.world_bounds
    }

    /// Enable wrap on entity using stored world bounds. No-op if world bounds not set.
    pub fn enable_wrap_bounds(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        let Some(bounds) = store.world_bounds else { return false };
        if !store.entities.contains_key(&id) { return false }
        store.wrap_bounds.insert(id, bounds);
        true
    }

    // ── Entity tag mutation ───────────────────────────────────────────────

    /// Add a runtime tag to an entity. Returns false if entity does not exist.
    pub fn tag_add(&self, id: u64, tag: &str) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        let Some(entity) = store.entities.get_mut(&id) else { return false };
        entity.tags.insert(tag.to_string());
        true
    }

    /// Remove a runtime tag from an entity.
    pub fn tag_remove(&self, id: u64, tag: &str) -> bool {
        let Ok(mut store) = self.store.lock() else { return false };
        let Some(entity) = store.entities.get_mut(&id) else { return false };
        entity.tags.remove(tag)
    }

    /// Check if an entity has a specific runtime tag.
    pub fn tag_has(&self, id: u64, tag: &str) -> bool {
        let Ok(store) = self.store.lock() else { return false };
        store.entities.get(&id).map(|e| e.tags.contains(tag)).unwrap_or(false)
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
