//! Script-facing `world.objects` lookup/handle API.
//!
//! This surface stays intentionally small: `find(...)` resolves one live handle,
//! while `all()`, `by_tag(...)`, and `by_name(...)` return directly iterable Rhai
//! arrays of the same handle type.

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};

/// Live handle for a gameplay-world object exposed through `world.objects.*`.
///
/// `set` must always report write success explicitly. Returning `false` is the
/// contract for stale handles, unsupported Rhai values, or rejected paths.
pub trait GameplayWorldObjectCoreApi: Clone + 'static {
    fn exists(&mut self) -> bool;
    fn id(&mut self) -> rhai::INT;
    fn kind(&mut self) -> String;
    fn tags(&mut self) -> RhaiArray;
    fn inspect(&mut self) -> RhaiMap;
    fn get(&mut self, path: &str) -> RhaiDynamic;
    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool;
}

/// Focused lookup surface for discovering live world-object handles.
///
/// Implementations should keep lookup precedence and collection ordering
/// deterministic for a given world snapshot so script iteration stays stable.
pub trait GameplayWorldObjectsCoreApi<TObject>: Clone + 'static
where
    TObject: GameplayWorldObjectCoreApi,
{
    /// Resolve one object handle from a stable user-facing target.
    ///
    /// Implementations may support numeric ids, bound visuals, and authored names.
    fn find(&mut self, target: &str) -> TObject;

    /// Resolve one object handle from a numeric runtime id.
    fn find_id(&mut self, id: rhai::INT) -> TObject;

    /// Return every visible object handle in deterministic iteration order.
    fn all(&mut self) -> RhaiArray;

    /// Return every object handle that currently advertises `tag`.
    fn by_tag(&mut self, tag: &str) -> RhaiArray;

    /// Return every object handle whose authored/runtime name matches `name`.
    fn by_name(&mut self, name: &str) -> RhaiArray;
}

/// Register `world.objects.*` collection methods plus the live object handle surface.
///
/// The collection methods return Rhai arrays of live handles so scripts can write
/// either lookup-style expressions such as `world.objects.by_name("Ace")[0]` or
/// direct iteration such as `for object in world.objects.all() { ... }`.
pub fn register_world_objects_api<TObjects, TObject>(engine: &mut RhaiEngine)
where
    TObjects: GameplayWorldObjectsCoreApi<TObject>,
    TObject: GameplayWorldObjectCoreApi,
{
    engine.register_type_with_name::<TObjects>("WorldObjectsApi");
    engine.register_type_with_name::<TObject>("WorldObject");

    engine.register_fn("find", |objects: &mut TObjects, target: &str| {
        objects.find(target)
    });
    engine.register_fn("find", |objects: &mut TObjects, id: rhai::INT| {
        objects.find_id(id)
    });
    engine.register_fn("all", |objects: &mut TObjects| objects.all());
    engine.register_fn("by_tag", |objects: &mut TObjects, tag: &str| {
        objects.by_tag(tag)
    });
    engine.register_fn("by_name", |objects: &mut TObjects, name: &str| {
        objects.by_name(name)
    });

    engine.register_fn("exists", |object: &mut TObject| object.exists());
    engine.register_fn("id", |object: &mut TObject| object.id());
    engine.register_fn("kind", |object: &mut TObject| object.kind());
    engine.register_fn("tags", |object: &mut TObject| object.tags());
    engine.register_fn("inspect", |object: &mut TObject| object.inspect());
    engine.register_fn("get", |object: &mut TObject, path: &str| object.get(path));
    engine.register_fn(
        "set",
        |object: &mut TObject, path: &str, value: RhaiDynamic| object.set(path, value),
    );
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        register_world_objects_api, GameplayWorldObjectCoreApi, GameplayWorldObjectsCoreApi,
    };
    use rhai::{
        Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap,
        Scope as RhaiScope,
    };

    #[derive(Clone, Default)]
    struct StubWorldObject {
        id: i64,
        kind: String,
        name: String,
        tags: Vec<String>,
        visual_ids: Vec<String>,
        alive: Arc<Mutex<bool>>,
        data: Arc<Mutex<RhaiMap>>,
    }

    impl StubWorldObject {
        fn missing() -> Self {
            Self::default()
        }

        fn new(
            id: i64,
            kind: &str,
            name: &str,
            tags: &[&str],
            visual_ids: &[&str],
            hp: i64,
        ) -> Self {
            let mut data = RhaiMap::new();
            data.insert("hp".into(), hp.into());
            data.insert("name".into(), name.into());
            Self {
                id,
                kind: kind.to_string(),
                name: name.to_string(),
                tags: tags.iter().map(|tag| tag.to_string()).collect(),
                visual_ids: visual_ids
                    .iter()
                    .map(|visual_id| visual_id.to_string())
                    .collect(),
                alive: Arc::new(Mutex::new(true)),
                data: Arc::new(Mutex::new(data)),
            }
        }

        fn invalidate(&self) -> bool {
            let Ok(mut alive) = self.alive.lock() else {
                return false;
            };
            *alive = false;
            true
        }
    }

    impl GameplayWorldObjectCoreApi for StubWorldObject {
        fn exists(&mut self) -> bool {
            self.id > 0 && self.alive.lock().map(|alive| *alive).unwrap_or(false)
        }

        fn id(&mut self) -> rhai::INT {
            self.id
        }

        fn kind(&mut self) -> String {
            self.kind.clone()
        }

        fn tags(&mut self) -> RhaiArray {
            self.tags.iter().cloned().map(Into::into).collect()
        }

        fn inspect(&mut self) -> RhaiMap {
            let mut map = RhaiMap::new();
            map.insert("id".into(), self.id.into());
            map.insert("kind".into(), self.kind.clone().into());
            map.insert("name".into(), self.name.clone().into());
            map.insert(
                "tags".into(),
                self.tags
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect::<RhaiArray>()
                    .into(),
            );
            map
        }

        fn get(&mut self, path: &str) -> RhaiDynamic {
            self.data
                .lock()
                .ok()
                .and_then(|data| data.get(path).cloned())
                .unwrap_or_else(|| ().into())
        }

        fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
            if !self.exists() {
                return false;
            }
            let Ok(mut data) = self.data.lock() else {
                return false;
            };
            data.insert(path.into(), value);
            true
        }
    }

    #[derive(Clone, Default)]
    struct StubWorldObjects {
        ordered: Arc<Vec<StubWorldObject>>,
    }

    impl StubWorldObjects {
        fn new() -> Self {
            let first =
                StubWorldObject::new(2, "wingman", "Ace", &["player", "squad"], &["wingman-2"], 8);
            let second = StubWorldObject::new(1, "ship", "Ace", &["player"], &["ship-main"], 12);
            let third = StubWorldObject::new(3, "decoy", "ship-main", &["hazard"], &["decoy-3"], 4);
            let fourth = StubWorldObject::new(4, "probe", "1", &["probe"], &["probe-4"], 1);
            Self {
                ordered: Arc::new(vec![first, second, third, fourth]),
            }
        }

        fn invalidate(&mut self, id: rhai::INT) -> bool {
            self.ordered
                .iter()
                .find(|object| object.id == id)
                .map(StubWorldObject::invalidate)
                .unwrap_or(false)
        }
    }

    impl GameplayWorldObjectsCoreApi<StubWorldObject> for StubWorldObjects {
        fn find(&mut self, target: &str) -> StubWorldObject {
            let target = target.trim();
            if target.is_empty() {
                return StubWorldObject::missing();
            }
            target
                .parse::<i64>()
                .ok()
                .and_then(|id| self.ordered.iter().find(|object| object.id == id).cloned())
                .or_else(|| {
                    self.ordered
                        .iter()
                        .find(|object| {
                            object
                                .visual_ids
                                .iter()
                                .any(|visual_id| visual_id == target)
                        })
                        .cloned()
                })
                .or_else(|| {
                    self.ordered
                        .iter()
                        .find(|object| object.name == target)
                        .cloned()
                })
                .unwrap_or_else(StubWorldObject::missing)
        }

        fn find_id(&mut self, id: rhai::INT) -> StubWorldObject {
            self.ordered
                .iter()
                .find(|object| object.id == id)
                .cloned()
                .unwrap_or_else(StubWorldObject::missing)
        }

        fn all(&mut self) -> RhaiArray {
            self.ordered
                .iter()
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }

        fn by_tag(&mut self, tag: &str) -> RhaiArray {
            self.ordered
                .iter()
                .filter(|object| object.tags.iter().any(|object_tag| object_tag == tag))
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }

        fn by_name(&mut self, name: &str) -> RhaiArray {
            self.ordered
                .iter()
                .filter(|object| object.name == name)
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }
    }

    fn map_int(map: &RhaiMap, key: &str) -> Option<rhai::INT> {
        map.get(key)
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
    }

    fn map_bool(map: &RhaiMap, key: &str) -> Option<bool> {
        map.get(key)
            .and_then(|value| value.clone().try_cast::<bool>())
    }

    fn map_string(map: &RhaiMap, key: &str) -> Option<String> {
        map.get(key)
            .and_then(|value| value.clone().try_cast::<String>())
    }

    fn map_int_array(map: &RhaiMap, key: &str) -> Vec<rhai::INT> {
        map.get(key)
            .and_then(|value| value.clone().into_array().ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.try_cast::<rhai::INT>())
            .collect()
    }

    fn map_string_array(map: &RhaiMap, key: &str) -> Vec<String> {
        map.get(key)
            .and_then(|value| value.clone().into_array().ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.try_cast::<String>())
            .collect()
    }

    #[test]
    fn register_world_objects_api_exposes_deterministic_lookup_and_collections() {
        let mut engine = RhaiEngine::new();
        register_world_objects_api::<StubWorldObjects, StubWorldObject>(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("objects", StubWorldObjects::new());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let visual_match = objects.find("ship-main");
                    let numeric_match = objects.find("1");
                    let name_match = objects.find("Ace");
                    let all = objects.all();
                    let tagged = objects.by_tag("player");
                    let named = objects.by_name("Ace");
                    let all_ids = [];
                    for object in all {
                        all_ids.push(object.id());
                    }

                    #{
                        visual_match_id: visual_match.id(),
                        numeric_match_id: numeric_match.id(),
                        name_match_id: name_match.id(),
                        all_ids: all_ids,
                        tagged_ids: tagged.map(|object| object.id()),
                        named_ids: named.map(|object| object.id()),
                        inspect_name: visual_match.inspect()["name"]
                    }
                "#,
            )
            .expect("world objects API should evaluate");

        assert_eq!(map_int(&result, "visual_match_id"), Some(1));
        assert_eq!(map_int(&result, "numeric_match_id"), Some(1));
        assert_eq!(map_int(&result, "name_match_id"), Some(2));
        assert_eq!(map_int_array(&result, "all_ids"), vec![2, 1, 3, 4]);
        assert_eq!(map_int_array(&result, "tagged_ids"), vec![2, 1]);
        assert_eq!(map_int_array(&result, "named_ids"), vec![2, 1]);
        assert_eq!(map_string(&result, "inspect_name"), Some("Ace".to_string()));
    }

    #[test]
    fn register_world_objects_api_supports_iteration_and_explicit_set_results() {
        let mut engine = RhaiEngine::new();
        register_world_objects_api::<StubWorldObjects, StubWorldObject>(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("objects", StubWorldObjects::new());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let registry = objects;
                    let leader = registry.find("ship-main");
                    let squad_kinds = [];
                    for object in registry.by_tag("squad") {
                        squad_kinds.push(object.kind());
                    }
                    let set_ok = leader.set("hp", 99);
                    let set_missing = registry.find("missing").set("hp", 5);

                    #{
                        set_ok: set_ok,
                        hp: leader.get("hp"),
                        set_missing: set_missing,
                        squad_kinds: squad_kinds
                    }
                "#,
            )
            .expect("world objects iteration should evaluate");

        assert_eq!(map_bool(&result, "set_ok"), Some(true));
        assert_eq!(map_int(&result, "hp"), Some(99));
        assert_eq!(map_bool(&result, "set_missing"), Some(false));
        assert_eq!(
            map_string_array(&result, "squad_kinds"),
            vec!["wingman".to_string()]
        );
    }

    #[test]
    fn register_world_objects_api_returns_false_for_stale_handles() {
        let mut engine = RhaiEngine::new();
        register_world_objects_api::<StubWorldObjects, StubWorldObject>(&mut engine);
        engine.register_fn(
            "invalidate",
            |objects: &mut StubWorldObjects, id: rhai::INT| objects.invalidate(id),
        );

        let mut scope = RhaiScope::new();
        scope.push("objects", StubWorldObjects::new());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let found = objects.find("ship-main");
                    let invalidated = objects.invalidate(found.id());
                    let stale_exists = found.exists();
                    let stale_set = found.set("hp", 5);

                    #{
                        invalidated: invalidated,
                        stale_exists: stale_exists,
                        stale_set: stale_set
                    }
                "#,
            )
            .expect("stale world objects handle should still evaluate");

        assert_eq!(map_bool(&result, "invalidated"), Some(true));
        assert_eq!(map_bool(&result, "stale_exists"), Some(false));
        assert_eq!(map_bool(&result, "stale_set"), Some(false));
    }
}
