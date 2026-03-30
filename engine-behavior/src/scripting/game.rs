//! Game domain APIs: ScriptGameApi, ScriptLevelApi, ScriptTimeApi, ScriptPersistenceApi.

use std::sync::{Arc, Mutex};

use engine_animation::SceneStage;
use engine_core::game_state::GameState;
use engine_core::level_state::LevelState;
use engine_persistence::PersistenceStore;
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine};
use serde_json::{Number as JsonNumber, Value as JsonValue};

use crate::{BehaviorCommand};
use crate::rhai_util::{json_to_rhai_dynamic, rhai_dynamic_to_json};

// ── ScriptGameApi ────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptGameApi {
    state: Option<GameState>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptGameApi {
    pub(crate) fn new(state: Option<GameState>, queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { state, queue }
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        self.state
            .as_ref()
            .and_then(|state| state.get(path))
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        state.set(path, value)
    }

    fn has(&mut self, path: &str) -> bool {
        self.state
            .as_ref()
            .map(|state| state.has(path))
            .unwrap_or(false)
    }

    fn remove(&mut self, path: &str) -> bool {
        self.state
            .as_ref()
            .map(|state| state.remove(path))
            .unwrap_or(false)
    }

    fn push(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        state.push(path, value)
    }

    fn jump(&mut self, to_scene_id: &str) -> bool {
        if to_scene_id.trim().is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::SceneTransition {
            to_scene_id: to_scene_id.to_string(),
        });
        true
    }

    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get(path)
            .try_cast::<String>()
            .unwrap_or_else(|| fallback.to_string())
    }

    fn get_b(&mut self, path: &str, fallback: bool) -> bool {
        self.get(path).try_cast::<bool>().unwrap_or(fallback)
    }

    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get(path).try_cast::<rhai::FLOAT>().unwrap_or(fallback)
    }
}

// ── ScriptLevelApi ───────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptLevelApi {
    state: Option<LevelState>,
}

impl ScriptLevelApi {
    pub(crate) fn new(state: Option<LevelState>) -> Self {
        Self { state }
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        self.state
            .as_ref()
            .and_then(|state| state.get(path))
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        state.set(path, value)
    }

    fn has(&mut self, path: &str) -> bool {
        self.state
            .as_ref()
            .map(|state| state.has(path))
            .unwrap_or(false)
    }

    fn remove(&mut self, path: &str) -> bool {
        self.state
            .as_ref()
            .map(|state| state.remove(path))
            .unwrap_or(false)
    }

    fn push(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        state.push(path, value)
    }

    fn select(&mut self, level_id: &str) -> bool {
        self.state
            .as_ref()
            .map(|state| state.select(level_id))
            .unwrap_or(false)
    }

    fn current(&mut self) -> String {
        self.state
            .as_ref()
            .and_then(LevelState::current_id)
            .unwrap_or_default()
    }

    fn ids(&mut self) -> RhaiArray {
        self.state
            .as_ref()
            .map(|state| state.ids().into_iter().map(Into::into).collect())
            .unwrap_or_default()
    }
}

// ── ScriptTimeApi ────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptTimeApi {
    scene_elapsed_ms: u64,
    stage_elapsed_ms: u64,
    stage: String,
    game_state: Option<GameState>,
}

impl ScriptTimeApi {
    pub(crate) fn new(
        scene_elapsed_ms: u64,
        stage_elapsed_ms: u64,
        stage: SceneStage,
        game_state: Option<GameState>,
    ) -> Self {
        Self {
            scene_elapsed_ms,
            stage_elapsed_ms,
            stage: match stage {
                SceneStage::OnEnter => "on_enter",
                SceneStage::OnIdle => "on_idle",
                SceneStage::OnLeave => "on_leave",
                SceneStage::Done => "done",
            }
            .to_string(),
            game_state,
        }
    }

    fn scene_elapsed_ms(&mut self) -> rhai::INT {
        self.scene_elapsed_ms as rhai::INT
    }

    fn stage_elapsed_ms(&mut self) -> rhai::INT {
        self.stage_elapsed_ms as rhai::INT
    }

    fn stage(&mut self) -> String {
        self.stage.clone()
    }

    fn contains(&mut self, path: &str) -> bool {
        matches!(path, "scene_elapsed_ms" | "stage_elapsed_ms" | "stage")
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        match path {
            "scene_elapsed_ms" => self.scene_elapsed_ms().into(),
            "stage_elapsed_ms" => self.stage_elapsed_ms().into(),
            "stage" => self.stage().into(),
            _ => ().into(),
        }
    }

    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    fn delta_ms(&mut self, max_ms: rhai::INT, state_path: &str) -> rhai::INT {
        let now = self.scene_elapsed_ms as rhai::INT;
        let max_ms = max_ms.max(0);
        let Some(state) = self.game_state.as_ref() else {
            return 0;
        };

        let last = state
            .get(state_path)
            .and_then(|value| value.as_i64())
            .map(|value| value as rhai::INT)
            .unwrap_or(now);

        let raw = now - last;
        let dt = raw.clamp(0, max_ms);
        let _ = state.set(state_path, JsonValue::Number(JsonNumber::from(now)));
        dt
    }
}

// ── ScriptPersistenceApi ──────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptPersistenceApi {
    store: Option<PersistenceStore>,
}

impl ScriptPersistenceApi {
    pub(crate) fn new(store: Option<PersistenceStore>) -> Self {
        Self { store }
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        self.store
            .as_ref()
            .and_then(|store| store.get(path))
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(store) = self.store.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        store.set(path, value)
    }

    fn has(&mut self, path: &str) -> bool {
        self.store
            .as_ref()
            .map(|store| store.has(path))
            .unwrap_or(false)
    }

    fn remove(&mut self, path: &str) -> bool {
        self.store
            .as_ref()
            .map(|store| store.remove(path))
            .unwrap_or(false)
    }

    fn push(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(store) = self.store.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        store.push(path, value)
    }

    fn reload(&mut self) {
        if let Some(store) = self.store.as_ref() {
            store.reload();
        }
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptGameApi>("GameApi");
    engine.register_type_with_name::<ScriptLevelApi>("LevelApi");
    engine.register_type_with_name::<ScriptTimeApi>("TimeApi");
    engine.register_type_with_name::<ScriptPersistenceApi>("PersistenceApi");

    // Time API
    engine.register_get("scene_elapsed_ms", |time: &mut ScriptTimeApi| {
        time.scene_elapsed_ms()
    });
    engine.register_get("stage_elapsed_ms", |time: &mut ScriptTimeApi| {
        time.stage_elapsed_ms()
    });
    engine.register_get("stage", |time: &mut ScriptTimeApi| time.stage());
    engine.register_fn("contains", |time: &mut ScriptTimeApi, path: &str| {
        time.contains(path)
    });
    engine.register_fn("get", |time: &mut ScriptTimeApi, path: &str| time.get(path));
    engine.register_fn(
        "get_i",
        |time: &mut ScriptTimeApi, path: &str, fallback: rhai::INT| time.get_i(path, fallback),
    );
    engine.register_fn(
        "delta_ms",
        |time: &mut ScriptTimeApi, max_ms: rhai::INT, state_path: &str| {
            time.delta_ms(max_ms, state_path)
        },
    );

    // Game API
    engine.register_fn("get", |game: &mut ScriptGameApi, path: &str| game.get(path));
    engine.register_fn(
        "set",
        |game: &mut ScriptGameApi, path: &str, value: RhaiDynamic| game.set(path, value),
    );
    engine.register_fn("has", |game: &mut ScriptGameApi, path: &str| game.has(path));
    engine.register_fn("remove", |game: &mut ScriptGameApi, path: &str| {
        game.remove(path)
    });
    engine.register_fn(
        "push",
        |game: &mut ScriptGameApi, path: &str, value: RhaiDynamic| game.push(path, value),
    );
    engine.register_fn("jump", |game: &mut ScriptGameApi, scene_id: &str| {
        game.jump(scene_id)
    });
    engine.register_fn(
        "get_i",
        |game: &mut ScriptGameApi, path: &str, fallback: rhai::INT| game.get_i(path, fallback),
    );
    engine.register_fn(
        "get_s",
        |game: &mut ScriptGameApi, path: &str, fallback: &str| game.get_s(path, fallback),
    );
    engine.register_fn(
        "get_b",
        |game: &mut ScriptGameApi, path: &str, fallback: bool| game.get_b(path, fallback),
    );
    engine.register_fn(
        "get_f",
        |game: &mut ScriptGameApi, path: &str, fallback: rhai::FLOAT| game.get_f(path, fallback),
    );

    // Level API
    engine.register_fn("get", |level: &mut ScriptLevelApi, path: &str| {
        level.get(path)
    });
    engine.register_fn(
        "set",
        |level: &mut ScriptLevelApi, path: &str, value: RhaiDynamic| level.set(path, value),
    );
    engine.register_fn("has", |level: &mut ScriptLevelApi, path: &str| {
        level.has(path)
    });
    engine.register_fn("remove", |level: &mut ScriptLevelApi, path: &str| {
        level.remove(path)
    });
    engine.register_fn(
        "push",
        |level: &mut ScriptLevelApi, path: &str, value: RhaiDynamic| level.push(path, value),
    );
    engine.register_fn("select", |level: &mut ScriptLevelApi, level_id: &str| {
        level.select(level_id)
    });
    engine.register_fn("current", |level: &mut ScriptLevelApi| level.current());
    engine.register_fn("ids", |level: &mut ScriptLevelApi| level.ids());

    // Persistence API
    engine.register_fn("get", |persist: &mut ScriptPersistenceApi, path: &str| {
        persist.get(path)
    });
    engine.register_fn(
        "set",
        |persist: &mut ScriptPersistenceApi, path: &str, value: RhaiDynamic| {
            persist.set(path, value)
        },
    );
    engine.register_fn("has", |persist: &mut ScriptPersistenceApi, path: &str| {
        persist.has(path)
    });
    engine.register_fn(
        "remove",
        |persist: &mut ScriptPersistenceApi, path: &str| persist.remove(path),
    );
    engine.register_fn(
        "push",
        |persist: &mut ScriptPersistenceApi, path: &str, value: RhaiDynamic| {
            persist.push(path, value)
        },
    );
    engine.register_fn("reload", |persist: &mut ScriptPersistenceApi| {
        persist.reload()
    });
}
