//! Typed behavior-side `runtime.*` root built on top of `engine-api`.
//!
//! `runtime.scene` and `runtime.world` are first-class handles. `runtime.services`
//! and `runtime.stores` group the existing behavior-side APIs without changing
//! their underlying implementations.

use engine_api::{
    register_runtime_core_api, ObjectRegistryCoreApi, RuntimeSceneCoreApi, RuntimeServicesCoreApi,
    RuntimeStoresCoreApi, RuntimeWorldCoreApi, ScriptCollisionApi, ScriptEffectsApi,
    ScriptObjectApi, ScriptRuntimeApi,
};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine};

use super::{
    audio::ScriptAudioApi,
    debug::ScriptDebugApi,
    game::{ScriptGameApi, ScriptLevelApi, ScriptPersistenceApi},
    gameplay::ScriptGameplayApi,
    gui::ScriptGuiApi,
    io::ScriptInputApi,
    palette::ScriptPaletteApi,
    scene::ScriptSceneApi,
    ui::ScriptUiApi,
};

pub(crate) type BehaviorRuntimeApi = ScriptRuntimeApi<
    BehaviorRuntimeSceneApi,
    ScriptGameplayApi,
    BehaviorRuntimeServicesApi,
    BehaviorRuntimeStoresApi,
>;

#[derive(Clone)]
pub(crate) struct BehaviorRuntimeSceneApi {
    scene: ScriptSceneApi,
}

impl BehaviorRuntimeSceneApi {
    pub(crate) fn new(scene: ScriptSceneApi) -> Self {
        Self { scene }
    }
}

#[derive(Clone)]
pub(crate) struct BehaviorRuntimeSceneObjectsApi {
    objects: engine_api::scene::ScriptSceneObjectsApi,
}

impl BehaviorRuntimeSceneObjectsApi {
    fn new(scene: &mut ScriptSceneApi) -> Self {
        Self {
            objects: scene.objects(),
        }
    }

    fn filter_handles(&mut self, predicate: impl Fn(&mut ScriptObjectApi) -> bool) -> RhaiArray {
        self.objects
            .all()
            .into_iter()
            .filter_map(|value| {
                let mut object = value.try_cast::<ScriptObjectApi>()?;
                predicate(&mut object).then_some(RhaiDynamic::from(object))
            })
            .collect()
    }

    fn object_has_tag(object: &mut ScriptObjectApi, tag: &str) -> bool {
        object
            .get("tags")
            .into_array()
            .ok()
            .map(|tags| {
                tags.into_iter()
                    .filter_map(|value| value.try_cast::<String>())
                    .any(|candidate| candidate == tag)
            })
            .unwrap_or(false)
            || object
                .get("tag")
                .try_cast::<String>()
                .map(|candidate| candidate == tag)
                .unwrap_or(false)
    }

    fn object_name_matches(object: &mut ScriptObjectApi, name: &str) -> bool {
        object
            .get("name")
            .try_cast::<String>()
            .map(|candidate| candidate == name)
            .unwrap_or(false)
            || object
                .get("id")
                .try_cast::<String>()
                .map(|candidate| candidate == name)
                .unwrap_or(false)
    }
}

impl ObjectRegistryCoreApi<ScriptObjectApi> for BehaviorRuntimeSceneObjectsApi {
    fn find(&mut self, target: &str) -> ScriptObjectApi {
        self.objects.find(target)
    }

    fn all(&mut self) -> RhaiArray {
        self.objects.all()
    }

    fn by_tag(&mut self, tag: &str) -> RhaiArray {
        self.filter_handles(|object| Self::object_has_tag(object, tag))
    }

    fn by_name(&mut self, name: &str) -> RhaiArray {
        self.filter_handles(|object| Self::object_name_matches(object, name))
    }
}

impl RuntimeSceneCoreApi<BehaviorRuntimeSceneObjectsApi> for BehaviorRuntimeSceneApi {
    fn objects(&mut self) -> BehaviorRuntimeSceneObjectsApi {
        BehaviorRuntimeSceneObjectsApi::new(&mut self.scene)
    }
}

impl RuntimeWorldCoreApi for ScriptGameplayApi {}

#[derive(Clone)]
pub(crate) struct BehaviorRuntimeServicesApi {
    input: ScriptInputApi,
    gui: ScriptGuiApi,
    ui: ScriptUiApi,
    diag: ScriptDebugApi,
    audio: ScriptAudioApi,
    effects: ScriptEffectsApi,
    collision: ScriptCollisionApi,
    palette: ScriptPaletteApi,
}

impl BehaviorRuntimeServicesApi {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        input: ScriptInputApi,
        gui: ScriptGuiApi,
        ui: ScriptUiApi,
        diag: ScriptDebugApi,
        audio: ScriptAudioApi,
        effects: ScriptEffectsApi,
        collision: ScriptCollisionApi,
        palette: ScriptPaletteApi,
    ) -> Self {
        Self {
            input,
            gui,
            ui,
            diag,
            audio,
            effects,
            collision,
            palette,
        }
    }

    fn input(&mut self) -> ScriptInputApi {
        self.input.clone()
    }

    fn gui(&mut self) -> ScriptGuiApi {
        self.gui.clone()
    }

    fn ui(&mut self) -> ScriptUiApi {
        self.ui.clone()
    }

    fn diag(&mut self) -> ScriptDebugApi {
        self.diag.clone()
    }

    fn audio(&mut self) -> ScriptAudioApi {
        self.audio.clone()
    }

    fn effects(&mut self) -> ScriptEffectsApi {
        self.effects.clone()
    }

    fn collision(&mut self) -> ScriptCollisionApi {
        self.collision.clone()
    }

    fn palette(&mut self) -> ScriptPaletteApi {
        self.palette.clone()
    }
}

impl RuntimeServicesCoreApi for BehaviorRuntimeServicesApi {}

#[derive(Clone)]
pub(crate) struct BehaviorRuntimeStoresApi {
    game: ScriptGameApi,
    level: ScriptLevelApi,
    persist: ScriptPersistenceApi,
}

impl BehaviorRuntimeStoresApi {
    pub(crate) fn new(
        game: ScriptGameApi,
        level: ScriptLevelApi,
        persist: ScriptPersistenceApi,
    ) -> Self {
        Self {
            game,
            level,
            persist,
        }
    }

    fn game(&mut self) -> ScriptGameApi {
        self.game.clone()
    }

    fn level(&mut self) -> ScriptLevelApi {
        self.level.clone()
    }

    fn persist(&mut self) -> ScriptPersistenceApi {
        self.persist.clone()
    }
}

impl RuntimeStoresCoreApi for BehaviorRuntimeStoresApi {}

pub(crate) fn build_runtime_api(
    scene: ScriptSceneApi,
    world: ScriptGameplayApi,
    services: BehaviorRuntimeServicesApi,
    stores: BehaviorRuntimeStoresApi,
) -> BehaviorRuntimeApi {
    ScriptRuntimeApi::new(BehaviorRuntimeSceneApi::new(scene), world, services, stores)
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    register_runtime_core_api::<
        BehaviorRuntimeApi,
        BehaviorRuntimeSceneApi,
        BehaviorRuntimeSceneObjectsApi,
        ScriptObjectApi,
        ScriptGameplayApi,
        BehaviorRuntimeServicesApi,
        BehaviorRuntimeStoresApi,
    >(engine);

    engine.register_get("input", |services: &mut BehaviorRuntimeServicesApi| {
        services.input()
    });
    engine.register_get("gui", |services: &mut BehaviorRuntimeServicesApi| {
        services.gui()
    });
    engine.register_get("ui", |services: &mut BehaviorRuntimeServicesApi| {
        services.ui()
    });
    engine.register_get("diag", |services: &mut BehaviorRuntimeServicesApi| {
        services.diag()
    });
    engine.register_get("audio", |services: &mut BehaviorRuntimeServicesApi| {
        services.audio()
    });
    engine.register_get("effects", |services: &mut BehaviorRuntimeServicesApi| {
        services.effects()
    });
    engine.register_get("collision", |services: &mut BehaviorRuntimeServicesApi| {
        services.collision()
    });
    engine.register_get("palette", |services: &mut BehaviorRuntimeServicesApi| {
        services.palette()
    });

    engine.register_get("game", |stores: &mut BehaviorRuntimeStoresApi| {
        stores.game()
    });
    engine.register_get("level", |stores: &mut BehaviorRuntimeStoresApi| {
        stores.level()
    });
    engine.register_get("persist", |stores: &mut BehaviorRuntimeStoresApi| {
        stores.persist()
    });
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    use engine_core::effects::Region;
    use engine_core::game_state::GameState;
    use engine_core::scene_runtime_types::{
        ObjectRuntimeState, SidecarIoFrameState, TargetResolver,
    };
    use engine_game::GameplayWorld;
    use rhai::{Engine as RhaiEngine, Map as RhaiMap, Scope as RhaiScope};
    use serde_json::json;

    use crate::{catalog, palette, scripting, BehaviorCommand, BehaviorContext};

    use super::*;

    fn test_ctx() -> BehaviorContext {
        let mut menu_map = RhaiMap::new();
        menu_map.insert("selected_index".into(), 0_i64.into());
        menu_map.insert("count".into(), 0_i64.into());

        let mut time_map = RhaiMap::new();
        time_map.insert("scene_elapsed_ms".into(), 0_i64.into());
        time_map.insert("stage_elapsed_ms".into(), 0_i64.into());
        time_map.insert("stage".into(), "on_idle".into());

        let mut key_map = RhaiMap::new();
        key_map.insert("code".into(), "".into());
        key_map.insert("ctrl".into(), false.into());
        key_map.insert("alt".into(), false.into());
        key_map.insert("shift".into(), false.into());
        key_map.insert("pressed".into(), false.into());
        key_map.insert("released".into(), false.into());

        let mut engine_key_map = key_map.clone();
        engine_key_map.insert("is_quit".into(), false.into());
        engine_key_map.insert("any_down".into(), false.into());
        engine_key_map.insert("down_count".into(), 0_i64.into());

        BehaviorContext {
            stage: engine_animation::SceneStage::OnIdle,
            scene_elapsed_ms: 0,
            stage_elapsed_ms: 0,
            menu_selected_index: 0,
            target_resolver: Arc::new(TargetResolver::default()),
            object_states: Arc::new(HashMap::new()),
            object_kinds: Arc::new(HashMap::new()),
            object_props: Arc::new(HashMap::new()),
            object_regions: Arc::new(HashMap::new()),
            layout_regions_stale: false,
            object_text: Arc::new(HashMap::new()),
            ui_focused_target_id: None,
            ui_theme_id: None,
            ui_last_submit_target_id: None,
            ui_last_submit_text: None,
            ui_last_change_target_id: None,
            ui_last_change_text: None,
            game_state: None,
            level_state: None,
            persistence: None,
            catalogs: Arc::new(catalog::ModCatalogs::test_catalogs()),
            palettes: Arc::new(palette::PaletteStore::default()),
            default_palette: None,
            gameplay_world: None,
            emitter_state: None,
            collisions: Arc::new(Vec::new()),
            collision_enters: Arc::new(Vec::new()),
            collision_stays: Arc::new(Vec::new()),
            collision_exits: Arc::new(Vec::new()),
            last_raw_key: None,
            debug_enabled: false,
            orbit_active: false,
            keys_down: Arc::new(HashSet::new()),
            keys_just_pressed: Arc::new(HashSet::new()),
            action_bindings: Arc::new(HashMap::new()),
            sidecar_io: Arc::new(SidecarIoFrameState::default()),
            rhai_time_map: Arc::new(time_map),
            rhai_menu_map: Arc::new(menu_map),
            rhai_key_map: Arc::new(key_map),
            engine_key_map: Arc::new(engine_key_map),
            frame_ms: 16,
            mouse_x: 0.0,
            mouse_y: 0.0,
            gui_state: None,
            spatial_meters_per_world_unit: None,
        }
    }

    fn build_scene_api(queue: &Arc<Mutex<Vec<BehaviorCommand>>>) -> ScriptSceneApi {
        let mut object_states = HashMap::new();
        object_states.insert(
            "runtime-score".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 2,
                offset_y: 3,
                ..ObjectRuntimeState::default()
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("runtime-score".to_string(), "text".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert(
            "runtime-score".to_string(),
            Region {
                x: 10,
                y: 20,
                width: 30,
                height: 4,
            },
        );
        let mut object_text = HashMap::new();
        object_text.insert("runtime-score".to_string(), "42".to_string());
        let mut resolver = TargetResolver::new("scene-root".to_string());
        resolver.register_alias("hud-score".to_string(), "runtime-score".to_string());

        ScriptSceneApi::new(
            Arc::new(object_states),
            Arc::new(object_kinds),
            Arc::new(HashMap::new()),
            Arc::new(object_regions),
            Arc::new(object_text),
            Arc::new(resolver),
            Arc::clone(queue),
        )
    }

    fn build_world_api(
        queue: &Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> (ScriptGameplayApi, GameplayWorld) {
        let world = GameplayWorld::new();
        let ship_id = world
            .spawn("ship", json!({ "name": "Ace", "tags": ["player"] }))
            .expect("ship");
        let _probe_id = world
            .spawn("probe", json!({ "name": "Beacon", "tags": ["utility"] }))
            .expect("probe");
        assert!(ship_id > 0);

        let ctx = test_ctx();
        let api = ScriptGameplayApi::new(
            Some(world.clone()),
            Arc::clone(&ctx.collisions),
            Arc::clone(&ctx.collision_enters),
            Arc::clone(&ctx.collision_stays),
            Arc::clone(&ctx.collision_exits),
            ctx.spatial_meters_per_world_unit,
            Arc::clone(&ctx.catalogs),
            ctx.emitter_state.clone(),
            Arc::clone(queue),
            Arc::clone(&ctx.palettes),
            ctx.persistence.clone(),
            ctx.default_palette.clone(),
        );
        (api, world)
    }

    fn build_services_api(
        ctx: &BehaviorContext,
        queue: &Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> BehaviorRuntimeServicesApi {
        BehaviorRuntimeServicesApi::new(
            super::super::io::ScriptInputApi::new(
                Arc::clone(&ctx.keys_down),
                Arc::clone(&ctx.keys_just_pressed),
                Arc::clone(&ctx.action_bindings),
                Arc::clone(&ctx.catalogs),
                Arc::clone(queue),
            ),
            super::super::gui::ScriptGuiApi::new(ctx, Arc::clone(queue)),
            super::super::ui::ScriptUiApi::new(ctx, Arc::clone(queue)),
            super::super::debug::ScriptDebugApi::new(
                "runtime-test".to_string(),
                Some("./runtime.rhai".to_string()),
                Arc::clone(queue),
            ),
            super::super::audio::ScriptAudioApi::new(Arc::clone(queue)),
            ScriptEffectsApi::new(Arc::clone(queue)),
            ScriptCollisionApi::from_arcs(
                ctx.gameplay_world.clone(),
                Arc::clone(&ctx.collisions),
                Arc::clone(&ctx.collision_enters),
                Arc::clone(&ctx.collision_stays),
                Arc::clone(&ctx.collision_exits),
                Arc::clone(queue),
            ),
            super::super::palette::ScriptPaletteApi::new(
                Arc::clone(&ctx.palettes),
                ctx.persistence.clone(),
                ctx.default_palette.clone(),
            ),
        )
    }

    fn build_stores_api(
        ctx: &BehaviorContext,
        queue: &Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> BehaviorRuntimeStoresApi {
        BehaviorRuntimeStoresApi::new(
            super::super::game::ScriptGameApi::new(ctx.game_state.clone(), Arc::clone(queue)),
            super::super::game::ScriptLevelApi::new(ctx.level_state.clone()),
            super::super::game::ScriptPersistenceApi::new(ctx.persistence.clone()),
        )
    }

    #[test]
    fn typed_runtime_root_exposes_scene_world_services_and_stores() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut ctx = test_ctx();
        ctx.game_state = Some(GameState::new());
        let scene_api = build_scene_api(&queue);
        let (world_api, world) = build_world_api(&queue);
        ctx.gameplay_world = Some(world);

        let services = build_services_api(&ctx, &queue);
        let stores = build_stores_api(&ctx, &queue);
        let runtime = build_runtime_api(scene_api.clone(), world_api.clone(), services, stores);

        let mut engine = RhaiEngine::new();
        scripting::register_all_domains(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("scene", scene_api);
        scope.push("world", world_api);
        scope.push("runtime", runtime);

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    runtime.scene.objects.find("hud-score").set("text.content", "108");

                    let named = runtime.world.objects.by_name("Ace");
                    let tagged = runtime.world.objects.by_tag("player");
                    let found = runtime.world.objects.find(named[0].id());

                    runtime.stores.game.set("/session/pilot", "Ada");

                    #{
                        scene_text: scene.object("hud-score").get("text.content"),
                        world_kind: found.kind(),
                        named_len: named.len(),
                        tagged_len: tagged.len(),
                        pilot: runtime.stores.game.get_s("/session/pilot", ""),
                        any_key: runtime.services.input.any_down()
                    }
                "#,
            )
            .expect("runtime root should resolve");

        let scene_text = result
            .get("scene_text")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("scene text");
        let world_kind = result
            .get("world_kind")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("world kind");
        let named_len = result
            .get("named_len")
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
            .expect("named len");
        let tagged_len = result
            .get("tagged_len")
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
            .expect("tagged len");
        let pilot = result
            .get("pilot")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("pilot");
        let any_key = result
            .get("any_key")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("any key");

        assert_eq!(scene_text, "108");
        assert_eq!(world_kind, "ship");
        assert_eq!(named_len, 1);
        assert_eq!(tagged_len, 1);
        assert_eq!(pilot, "Ada");
        assert!(!any_key);
    }
}
