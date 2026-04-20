//! Root runtime-domain contracts and Rhai registration.

use rhai::{Array as RhaiArray, Engine as RhaiEngine};

/// Contract for object-registry lookup surfaces exposed under `runtime.scene.objects`.
///
/// `find` returns one resolved object handle/snapshot, while `all`, `by_tag`, and `by_name`
/// return Rhai arrays so implementations can surface either typed handles or plain maps.
/// These method names are part of the stable root runtime contract for script callers.
pub trait ObjectRegistryCoreApi<TObject>: Clone + 'static
where
    TObject: Clone + 'static,
{
    /// Resolve a single object by a stable user-facing target such as id or name.
    fn find(&mut self, target: &str) -> TObject;

    /// Return every discoverable object currently visible through the runtime scene graph.
    fn all(&mut self) -> RhaiArray;

    /// Return every discoverable object that advertises the requested tag.
    fn by_tag(&mut self, tag: &str) -> RhaiArray;

    /// Return every discoverable object whose authored/runtime name matches `name`.
    fn by_name(&mut self, name: &str) -> RhaiArray;
}

/// Scene-level runtime contract exposed under `runtime.scene`.
pub trait RuntimeSceneCoreApi<TRegistry>: Clone + 'static
where
    TRegistry: Clone + 'static,
{
    /// Return the stable object-registry entry point exposed to scripts as `runtime.scene.objects`.
    fn objects(&mut self) -> TRegistry;
}

/// World-level runtime contract exposed under `runtime.world`.
pub trait RuntimeWorldCoreApi: Clone + 'static {}

/// Services-level runtime contract exposed under `runtime.services`.
pub trait RuntimeServicesCoreApi: Clone + 'static {}

/// Stores-level runtime contract exposed under `runtime.stores`.
pub trait RuntimeStoresCoreApi: Clone + 'static {}

/// Root runtime contract that owns the `runtime.*` namespace entry points.
pub trait RuntimeCoreApi<TScene, TWorld, TServices, TStores>: Clone + 'static
where
    TScene: Clone + 'static,
    TWorld: Clone + 'static,
    TServices: Clone + 'static,
    TStores: Clone + 'static,
{
    /// Return the scene-runtime entry point exposed to scripts as `runtime.scene`.
    fn scene(&mut self) -> TScene;

    /// Return the gameplay/runtime world entry point exposed to scripts as `runtime.world`.
    fn world(&mut self) -> TWorld;

    /// Return the services entry point exposed to scripts as `runtime.services`.
    fn services(&mut self) -> TServices;

    /// Return the store/snapshot entry point exposed to scripts as `runtime.stores`.
    fn stores(&mut self) -> TStores;
}

/// Simple typed runtime handle that delegates the four root runtime entry points.
#[derive(Clone)]
pub struct ScriptRuntimeApi<TScene, TWorld, TServices, TStores>
where
    TScene: Clone + 'static,
    TWorld: Clone + 'static,
    TServices: Clone + 'static,
    TStores: Clone + 'static,
{
    scene: TScene,
    world: TWorld,
    services: TServices,
    stores: TStores,
}

impl<TScene, TWorld, TServices, TStores> ScriptRuntimeApi<TScene, TWorld, TServices, TStores>
where
    TScene: Clone + 'static,
    TWorld: Clone + 'static,
    TServices: Clone + 'static,
    TStores: Clone + 'static,
{
    pub fn new(scene: TScene, world: TWorld, services: TServices, stores: TStores) -> Self {
        Self {
            scene,
            world,
            services,
            stores,
        }
    }
}

impl<TScene, TWorld, TServices, TStores> RuntimeCoreApi<TScene, TWorld, TServices, TStores>
    for ScriptRuntimeApi<TScene, TWorld, TServices, TStores>
where
    TScene: Clone + 'static,
    TWorld: Clone + 'static,
    TServices: Clone + 'static,
    TStores: Clone + 'static,
{
    fn scene(&mut self) -> TScene {
        self.scene.clone()
    }

    fn world(&mut self) -> TWorld {
        self.world.clone()
    }

    fn services(&mut self) -> TServices {
        self.services.clone()
    }

    fn stores(&mut self) -> TStores {
        self.stores.clone()
    }
}

/// Register the root `runtime.*` namespace and the scene-object registry exposed under
/// `runtime.scene.objects.*`.
///
/// Scripts discover runtime-owned scene handles through `runtime.scene.objects`, then use the
/// stable `find`, `all`, `by_tag`, and `by_name` registry methods from that entry point.
/// The root getters registered here are `runtime.scene`, `runtime.world`,
/// `runtime.services`, and `runtime.stores`.
pub fn register_runtime_core_api<TRuntime, TScene, TRegistry, TObject, TWorld, TServices, TStores>(
    engine: &mut RhaiEngine,
) where
    TRuntime: RuntimeCoreApi<TScene, TWorld, TServices, TStores>,
    TScene: RuntimeSceneCoreApi<TRegistry>,
    TRegistry: ObjectRegistryCoreApi<TObject>,
    TObject: Clone + 'static,
    TWorld: RuntimeWorldCoreApi,
    TServices: RuntimeServicesCoreApi,
    TStores: RuntimeStoresCoreApi,
{
    engine.register_type_with_name::<TRuntime>("RuntimeApi");
    engine.register_type_with_name::<TScene>("RuntimeSceneApi");
    engine.register_type_with_name::<TRegistry>("ObjectRegistryApi");
    engine.register_type_with_name::<TWorld>("RuntimeWorldApi");
    engine.register_type_with_name::<TServices>("RuntimeServicesApi");
    engine.register_type_with_name::<TStores>("RuntimeStoresApi");

    engine.register_get("scene", |runtime: &mut TRuntime| runtime.scene());
    engine.register_get("world", |runtime: &mut TRuntime| runtime.world());
    engine.register_get("services", |runtime: &mut TRuntime| runtime.services());
    engine.register_get("stores", |runtime: &mut TRuntime| runtime.stores());

    engine.register_get("objects", |scene: &mut TScene| scene.objects());

    engine.register_fn("find", |registry: &mut TRegistry, target: &str| {
        registry.find(target)
    });
    engine.register_fn("all", |registry: &mut TRegistry| registry.all());
    engine.register_fn("by_tag", |registry: &mut TRegistry, tag: &str| {
        registry.by_tag(tag)
    });
    engine.register_fn("by_name", |registry: &mut TRegistry, name: &str| {
        registry.by_name(name)
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};

    use super::*;

    #[derive(Clone)]
    struct FakeRuntime {
        scene: FakeScene,
        world: FakeWorld,
        services: FakeServices,
        stores: FakeStores,
    }

    impl RuntimeCoreApi<FakeScene, FakeWorld, FakeServices, FakeStores> for FakeRuntime {
        fn scene(&mut self) -> FakeScene {
            self.scene.clone()
        }

        fn world(&mut self) -> FakeWorld {
            self.world.clone()
        }

        fn services(&mut self) -> FakeServices {
            self.services.clone()
        }

        fn stores(&mut self) -> FakeStores {
            self.stores.clone()
        }
    }

    #[derive(Clone)]
    struct FakeScene {
        label: String,
        objects: FakeObjectRegistry,
    }

    impl RuntimeSceneCoreApi<FakeObjectRegistry> for FakeScene {
        fn objects(&mut self) -> FakeObjectRegistry {
            self.objects.clone()
        }
    }

    #[derive(Clone)]
    struct FakeWorld {
        label: String,
    }

    impl RuntimeWorldCoreApi for FakeWorld {}

    #[derive(Clone)]
    struct FakeServices {
        label: String,
    }

    impl RuntimeServicesCoreApi for FakeServices {}

    #[derive(Clone)]
    struct FakeStores {
        label: String,
    }

    impl RuntimeStoresCoreApi for FakeStores {}

    #[derive(Clone)]
    struct FakeObjectRegistry {
        objects: Arc<Vec<RhaiMap>>,
    }

    impl ObjectRegistryCoreApi<RhaiMap> for FakeObjectRegistry {
        fn find(&mut self, target: &str) -> RhaiMap {
            self.objects
                .iter()
                .find(|entry| {
                    entry
                        .get("id")
                        .and_then(|value| value.clone().try_cast::<String>())
                        .map(|id| id == target)
                        .unwrap_or(false)
                        || entry
                            .get("name")
                            .and_then(|value| value.clone().try_cast::<String>())
                            .map(|name| name == target)
                            .unwrap_or(false)
                })
                .cloned()
                .unwrap_or_default()
        }

        fn all(&mut self) -> RhaiArray {
            self.objects
                .iter()
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }

        fn by_tag(&mut self, tag: &str) -> RhaiArray {
            self.objects
                .iter()
                .filter(|entry| {
                    entry
                        .get("tags")
                        .and_then(|value| value.clone().into_array().ok())
                        .map(|tags| {
                            tags.into_iter().any(|tag_value| {
                                tag_value
                                    .clone()
                                    .try_cast::<String>()
                                    .map(|entry_tag| entry_tag == tag)
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }

        fn by_name(&mut self, name: &str) -> RhaiArray {
            self.objects
                .iter()
                .filter(|entry| {
                    entry
                        .get("name")
                        .and_then(|value| value.clone().try_cast::<String>())
                        .map(|entry_name| entry_name == name)
                        .unwrap_or(false)
                })
                .cloned()
                .map(RhaiDynamic::from)
                .collect()
        }
    }

    fn fake_object(id: &str, name: &str, tags: &[&str]) -> RhaiMap {
        let mut object = RhaiMap::new();
        object.insert("id".into(), id.to_string().into());
        object.insert("name".into(), name.to_string().into());
        object.insert(
            "tags".into(),
            tags.iter()
                .map(|tag| RhaiDynamic::from((*tag).to_string()))
                .collect::<RhaiArray>()
                .into(),
        );
        object
    }

    fn build_fake_runtime() -> FakeRuntime {
        FakeRuntime {
            scene: FakeScene {
                label: "scene".to_string(),
                objects: FakeObjectRegistry {
                    objects: Arc::new(vec![
                        fake_object("hud-score", "Score", &["ui", "hud"]),
                        fake_object("hud-label", "Label", &["ui"]),
                        fake_object("ship", "Ship", &["vehicle"]),
                    ]),
                },
            },
            world: FakeWorld {
                label: "world".to_string(),
            },
            services: FakeServices {
                label: "services".to_string(),
            },
            stores: FakeStores {
                label: "stores".to_string(),
            },
        }
    }

    fn map_string(map: &RhaiMap, key: &str) -> String {
        map.get(key)
            .and_then(|value| value.clone().try_cast::<String>())
            .unwrap_or_default()
    }

    fn register_fake_runtime_api(engine: &mut RhaiEngine) {
        crate::rhai::register::register_runtime_core_api::<
            FakeRuntime,
            FakeScene,
            FakeObjectRegistry,
            RhaiMap,
            FakeWorld,
            FakeServices,
            FakeStores,
        >(engine);
    }

    #[test]
    fn runtime_registration_exposes_root_entry_points_and_object_registry_contract() {
        let mut engine = RhaiEngine::new();
        register_fake_runtime_api(&mut engine);
        engine.register_fn("label", |scene: &mut FakeScene| scene.label.clone());
        engine.register_fn("label", |world: &mut FakeWorld| world.label.clone());
        engine.register_fn("label", |services: &mut FakeServices| {
            services.label.clone()
        });
        engine.register_fn("label", |stores: &mut FakeStores| stores.label.clone());

        let mut scope = rhai::Scope::new();
        scope.push("runtime", build_fake_runtime());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let found = runtime.scene.objects.find("hud-score");
                    let all = runtime.scene.objects.all();
                    let by_tag = runtime.scene.objects.by_tag("ui");
                    let by_name = runtime.scene.objects.by_name("Ship");

                    #{
                        scene_label: runtime.scene.label(),
                        world_label: runtime.world.label(),
                        services_label: runtime.services.label(),
                        stores_label: runtime.stores.label(),
                        found_id: found["id"] ?? "",
                        all_count: all.len(),
                        by_tag_count: by_tag.len(),
                        by_name_id: (by_name[0] ?? #{} )["id"] ?? ""
                    }
                "#,
            )
            .expect("runtime root namespace should resolve");

        let scene_label = result
            .get("scene_label")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("scene_label");
        let world_label = result
            .get("world_label")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("world_label");
        let services_label = result
            .get("services_label")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("services_label");
        let stores_label = result
            .get("stores_label")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("stores_label");
        let found_id = result
            .get("found_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("found_id");
        let all_count = result
            .get("all_count")
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
            .expect("all_count");
        let by_tag_count = result
            .get("by_tag_count")
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
            .expect("by_tag_count");
        let by_name_id = result
            .get("by_name_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("by_name_id");

        assert_eq!(scene_label, "scene");
        assert_eq!(world_label, "world");
        assert_eq!(services_label, "services");
        assert_eq!(stores_label, "stores");
        assert_eq!(found_id, "hud-score");
        assert_eq!(all_count, 3);
        assert_eq!(by_tag_count, 2);
        assert_eq!(by_name_id, "ship");
    }

    #[test]
    fn runtime_registration_supports_runtime_root_object_graph_discovery() {
        let mut engine = RhaiEngine::new();
        register_fake_runtime_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("runtime", build_fake_runtime());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let registry = runtime.scene.objects;
                    let all = registry.all();
                    let ui = registry.by_tag("ui");
                    let ships = registry.by_name("Ship");
                    let score = runtime.scene.objects.find("Score");

                    #{
                        first_all_id: (all[0] ?? #{} )["id"] ?? "",
                        second_all_id: (all[1] ?? #{} )["id"] ?? "",
                        third_all_id: (all[2] ?? #{} )["id"] ?? "",
                        first_ui_id: (ui[0] ?? #{} )["id"] ?? "",
                        second_ui_id: (ui[1] ?? #{} )["id"] ?? "",
                        first_ship_id: (ships[0] ?? #{} )["id"] ?? "",
                        score_id: score["id"] ?? ""
                    }
                "#,
            )
            .expect("runtime root object graph discovery should resolve");

        let first_all_id = result
            .get("first_all_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("first_all_id");
        let second_all_id = result
            .get("second_all_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("second_all_id");
        let third_all_id = result
            .get("third_all_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("third_all_id");
        let first_ui_id = result
            .get("first_ui_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("first_ui_id");
        let second_ui_id = result
            .get("second_ui_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("second_ui_id");
        let first_ship_id = result
            .get("first_ship_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("first_ship_id");
        let score_id = result
            .get("score_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("score_id");

        assert_eq!(first_all_id, "hud-score");
        assert_eq!(second_all_id, "hud-label");
        assert_eq!(third_all_id, "ship");
        assert_eq!(first_ui_id, "hud-score");
        assert_eq!(second_ui_id, "hud-label");
        assert_eq!(first_ship_id, "ship");
        assert_eq!(score_id, "hud-score");
    }

    #[test]
    fn runtime_registration_keeps_registry_methods_scoped_to_runtime_scene_objects() {
        let mut engine = RhaiEngine::new();
        register_fake_runtime_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("runtime", build_fake_runtime());

        let root_find = engine.eval_with_scope::<RhaiDynamic>(&mut scope, "runtime.find(\"ship\")");
        let root_all = engine.eval_with_scope::<RhaiDynamic>(&mut scope, "runtime.all()");
        let scene_find =
            engine.eval_with_scope::<RhaiDynamic>(&mut scope, "runtime.scene.find(\"ship\")");

        assert!(root_find.is_err());
        assert!(root_all.is_err());
        assert!(scene_find.is_err());
    }

    #[test]
    fn runtime_registration_find_resolves_stable_targets_by_id_and_name() {
        let mut engine = RhaiEngine::new();
        register_fake_runtime_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("runtime", build_fake_runtime());

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let registry = runtime.scene.objects;
                    let by_id = registry.find("ship");
                    let by_name = registry.find("Score");

                    #{
                        by_id: by_id["id"] ?? "",
                        by_name: by_name["id"] ?? ""
                    }
                "#,
            )
            .expect("registry find should resolve both id and name targets");

        let by_id = result
            .get("by_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("by_id");
        let by_name = result
            .get("by_name")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("by_name");

        assert_eq!(by_id, "ship");
        assert_eq!(by_name, "hud-score");
    }

    #[test]
    fn script_runtime_api_wrapper_delegates_root_handles() {
        let mut runtime = ScriptRuntimeApi::new(
            FakeScene {
                label: "scene".to_string(),
                objects: FakeObjectRegistry {
                    objects: Arc::new(Vec::new()),
                },
            },
            FakeWorld {
                label: "world".to_string(),
            },
            FakeServices {
                label: "services".to_string(),
            },
            FakeStores {
                label: "stores".to_string(),
            },
        );

        assert_eq!(runtime.scene().label, "scene");
        assert_eq!(runtime.world().label, "world");
        assert_eq!(runtime.services().label, "services");
        assert_eq!(runtime.stores().label, "stores");
    }

    #[test]
    fn fake_registry_contract_methods_stay_consistent_for_direct_lookup() {
        let mut registry = build_fake_runtime().scene.objects;

        let found = registry.find("ship");
        let all = registry.all();
        let by_tag = registry.by_tag("vehicle");
        let by_name = registry.by_name("Score");

        assert_eq!(map_string(&found, "id"), "ship");
        assert_eq!(all.len(), 3);
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_name.len(), 1);
        assert_eq!(
            by_name[0]
                .clone()
                .try_cast::<RhaiMap>()
                .map(|map| map_string(&map, "id"))
                .unwrap_or_default(),
            "hud-score"
        );
    }
}
