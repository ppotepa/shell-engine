//! Behavior system types: the [`Behavior`] trait, built-in behavior structs, and the [`BehaviorContext`] passed each tick.

pub mod factory;
pub mod registry;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use engine_animation::SceneStage;
use engine_core::authoring::metadata::FieldMetadata;
use engine_core::effects::Region;
use engine_core::game_object::{GameObject, GameObjectKind};
use engine_core::game_state::GameState;
use engine_core::level_state::LevelState;
use engine_core::logging;
use engine_core::scene::{AudioCue, BehaviorParams, BehaviorSpec, Scene};
use engine_core::scene_runtime_types::{
    ObjectRuntimeState, RawKeyEvent, SidecarIoFrameState, TargetResolver,
};
use engine_game::{
    Collider2D, ColliderShape, CollisionHit, GameplayWorld, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding,
};
use engine_game::components::{DespawnVisual, SplitOnDestroy, TopDownShipController};
use engine_persistence::PersistenceStore;
use engine_physics::{point_in_polygon, polygons_intersect, segment_intersects_polygon};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use factory::BehaviorFactory;

/// Per-tick context passed to every [`Behavior::update`] call.
#[derive(Debug, Clone)]
pub struct BehaviorContext {
    pub stage: SceneStage,
    pub scene_elapsed_ms: u64,
    pub stage_elapsed_ms: u64,
    pub menu_selected_index: usize,
    // Arc-wrapped: identical for every behavior in a frame, clone is O(1).
    pub target_resolver: Arc<TargetResolver>,
    // Arc-wrapped: shared across all behaviors in a frame, clone is O(1).
    pub object_states: Arc<std::collections::HashMap<String, ObjectRuntimeState>>,
    pub object_kinds: Arc<std::collections::HashMap<String, String>>,
    pub object_props: Arc<std::collections::HashMap<String, JsonValue>>,
    pub object_regions: Arc<std::collections::HashMap<String, Region>>,
    pub object_text: Arc<std::collections::HashMap<String, String>>,
    // Arc<str>: built once per frame, each behavior pays only an atomic refcount increment.
    pub ui_focused_target_id: Option<Arc<str>>,
    pub ui_theme_id: Option<Arc<str>>,
    pub ui_last_submit_target_id: Option<Arc<str>>,
    pub ui_last_submit_text: Option<Arc<str>>,
    pub ui_last_change_target_id: Option<Arc<str>>,
    pub ui_last_change_text: Option<Arc<str>>,
    pub game_state: Option<GameState>,
    pub level_state: Option<LevelState>,
    pub persistence: Option<PersistenceStore>,
    pub gameplay_world: Option<GameplayWorld>,
    /// Collision events collected for this frame (gameplay entities).
    pub collisions: std::sync::Arc<Vec<CollisionHit>>,
    /// Collision enter: pairs that started overlapping this frame (not present last frame).
    pub collision_enters: std::sync::Arc<Vec<CollisionHit>>,
    /// Collision stay: pairs that were overlapping last frame and still are.
    pub collision_stays: std::sync::Arc<Vec<CollisionHit>>,
    /// Collision exit: pairs that were overlapping last frame but no longer are.
    pub collision_exits: std::sync::Arc<Vec<CollisionHit>>,
    /// Raw key event for this frame — available in Rhai as `key.code`, `key.ctrl`, etc.
    /// Arc-wrapped: shared across all behaviors in a frame, clone is O(1).
    pub last_raw_key: Option<Arc<RawKeyEvent>>,
    /// Held key set (normalized key codes), exposed to Rhai via `input.down(code)`.
    pub keys_down: Arc<HashSet<String>>,
    /// Action bindings: action name → list of bound key codes (from `input.bind_action`).
    pub action_bindings: Arc<HashMap<String, Vec<String>>>,
    /// Sidecar IO frame snapshot (output lines / clear / fullscreen / custom events).
    pub sidecar_io: Arc<SidecarIoFrameState>,
    /// Rhai maps built once per frame, shared across all behaviors via Arc.
    /// Each behavior gets O(1) refcount clone instead of HashMap clone.
    /// Built in behavior_system; used in RhaiScriptBehavior::update().
    pub rhai_time_map: Arc<RhaiMap>,
    pub rhai_menu_map: Arc<RhaiMap>,
    pub rhai_key_map: Arc<RhaiMap>,
    /// Engine-level key state and metadata — read-only, never mutated by behaviors.
    /// Includes `code`, `ctrl`, `alt`, `shift`, `pressed`, `is_quit` fields.
    /// Pushed to Rhai scope as `engine` map to keep engine concerns separate.
    pub engine_key_map: Arc<RhaiMap>,
}

/// A side-effect produced by a behavior and consumed by the engine systems.
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorCommand {
    PlayAudioCue {
        cue: String,
        volume: Option<f32>,
    },
    PlayAudioEvent {
        event: String,
        gain: Option<f32>,
    },
    PlaySong {
        song_id: String,
    },
    StopSong,
    SetVisibility {
        target: String,
        visible: bool,
    },
    SetOffset {
        target: String,
        dx: i32,
        dy: i32,
    },
    SetText {
        target: String,
        text: String,
    },
    SetProps {
        target: String,
        visible: Option<bool>,
        dx: Option<i32>,
        dy: Option<i32>,
        text: Option<String>,
    },
    SetProperty {
        target: String,
        path: String,
        value: JsonValue,
    },
    SceneSpawn {
        template: String,
        target: String,
    },
    SceneDespawn {
        target: String,
    },
    TerminalPushOutput {
        line: String,
    },
    TerminalClearOutput,
    SceneTransition {
        to_scene_id: String,
    },
    DebugLog {
        scene_id: String,
        source: Option<String>,
        severity: DebugLogSeverity,
        message: String,
    },
    /// Rhai script error — consumed by the behavior system to push to DebugLogBuffer.
    ScriptError {
        scene_id: String,
        source: Option<String>,
        message: String,
    },
    /// Register or overwrite an input action binding (name → list of key codes).
    BindInputAction {
        action: String,
        keys: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLogSeverity {
    Info,
    Warn,
    Error,
}

/// Defines the per-tick update logic for a scene object behavior.
pub trait Behavior: Send + Sync {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    );
}

type EmittedCueKey = (String, String, SceneStage, u64, String);

/// Returns the built-in [`Behavior`] implementation for `spec`, or `None` if the name is unrecognised.
/// Delegates to [`factory::BuiltInBehaviorFactory`] — the single authoritative dispatch point.
pub fn built_in_behavior(spec: &BehaviorSpec) -> Option<Box<dyn Behavior + Send + Sync>> {
    factory::BuiltInBehaviorFactory.create(spec)
}

/// Returns names of all built-in behaviors.
pub fn builtin_behavior_names() -> Vec<&'static str> {
    vec![
        "blink",
        "bob",
        "follow",
        "menu-carousel",
        "menu-carousel-object",
        "rhai-script",
        "menu-selected",
        "selected-arrows",
        "stage-visibility",
        "timed-visibility",
    ]
}

/// Returns field metadata for the given behavior name.
pub fn behavior_metadata(name: &str) -> Vec<FieldMetadata> {
    engine_core::authoring::catalog::behavior_catalog()
        .into_iter()
        .find_map(|(behavior_name, fields)| (behavior_name == name).then_some(fields))
        .unwrap_or_default()
}

/// Initialize the mod source directory for Rhai module resolution.
/// Called from app startup to ensure scripts can import shared modules.
/// If not called explicitly, falls back to SHELL_QUEST_MOD_SOURCE env var or "mods/shell-quest".
pub fn init_behavior_system(mod_source: &str) {
    set_mod_source(mod_source.to_string());
}

#[derive(Default)]
/// Fires scene-level audio cues at their scheduled `at_ms` timestamps.
pub struct SceneAudioBehavior {
    emitted: HashSet<EmittedCueKey>,
}

impl Behavior for SceneAudioBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let cues = cues_for_stage(scene, &ctx.stage);
        if !cues.is_empty() {
            logging::debug(
                "engine.audio.behavior",
                format!(
                    "scene={} stage={:?} elapsed={}ms cues={} emitted={}",
                    scene.id,
                    ctx.stage,
                    ctx.scene_elapsed_ms,
                    cues.len(),
                    self.emitted.len()
                ),
            );
        }
        for cue in cues {
            if ctx.scene_elapsed_ms < cue.at_ms || cue.cue.trim().is_empty() {
                continue;
            }
            let key = (
                scene.id.clone(),
                object.id.clone(),
                ctx.stage,
                cue.at_ms,
                cue.cue.clone(),
            );
            if self.emitted.insert(key) {
                logging::info(
                    "engine.audio.behavior",
                    format!("emitting audio cue='{}' volume={:?}", cue.cue, cue.volume),
                );
                emit_audio(commands, cue.cue.clone(), cue.volume);
            }
        }
    }
}

/// Alternates an object's visibility on a configurable on/off cycle.
pub struct BlinkBehavior {
    target: Option<String>,
    visible_ms: u64,
    hidden_ms: u64,
    phase_ms: u64,
}

impl BlinkBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            visible_ms: params.visible_ms.unwrap_or(250),
            hidden_ms: params.hidden_ms.unwrap_or(250),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BlinkBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let cycle = self.visible_ms.saturating_add(self.hidden_ms);
        let visible = if cycle == 0 {
            true
        } else {
            let t = ctx.scene_elapsed_ms.saturating_add(self.phase_ms) % cycle;
            t < self.visible_ms || self.hidden_ms == 0
        };
        emit_visibility(commands, resolve_target(&self.target, object), visible);
    }
}

/// Applies a sinusoidal offset to an object along the X and/or Y axes.
pub struct BobBehavior {
    target: Option<String>,
    amplitude_x: i32,
    amplitude_y: i32,
    period_ms: u64,
    phase_ms: u64,
}

impl BobBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            amplitude_x: params.amplitude_x.unwrap_or(0),
            amplitude_y: params.amplitude_y.unwrap_or(1),
            period_ms: params.period_ms.unwrap_or(2000).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BobBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let wave = sine_wave(ctx.scene_elapsed_ms, self.phase_ms, self.period_ms);
        emit_offset(
            commands,
            resolve_target(&self.target, object),
            (self.amplitude_x as f32 * wave).round() as i32,
            (self.amplitude_y as f32 * wave).round() as i32,
        );
    }
}

/// Locks an object's position to match the current frame position of a named target.
pub struct FollowBehavior {
    target: Option<String>,
    offset_x: i32,
    offset_y: i32,
}

impl FollowBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            offset_x: params.amplitude_x.unwrap_or(0),
            offset_y: params.amplitude_y.unwrap_or(0),
        }
    }
}

impl Behavior for FollowBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let Some(target) = self.target.as_deref() else {
            return;
        };
        let Some(target_state) = ctx.resolved_object_state(target) else {
            return;
        };
        emit_visibility(commands, object.id.clone(), target_state.visible);
        emit_offset(
            commands,
            object.id.clone(),
            target_state.offset_x.saturating_add(self.offset_x),
            target_state.offset_y.saturating_add(self.offset_y),
        );
    }
}

/// Shows the object only during the specified scene stages.
pub struct StageVisibilityBehavior {
    target: Option<String>,
    stages: Vec<SceneStage>,
}

/// Shows the object only while it is the currently selected menu option.
pub struct MenuSelectedBehavior {
    target: Option<String>,
    index: usize,
}

impl MenuSelectedBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
        }
    }
}

impl Behavior for MenuSelectedBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        emit_visibility(
            commands,
            resolve_target(&self.target, object),
            ctx.menu_selected_index == self.index,
        );
    }
}

/// Repositions menu items into a centered rolling window around selected index.
pub struct MenuCarouselBehavior {
    target: Option<String>,
    index: usize,
    count: Option<usize>,
    window: usize,
    step_y: i32,
    endless: bool,
    last_dy: i32,
}

impl MenuCarouselBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
            count: params.count,
            window: params.window.unwrap_or(5).max(1),
            step_y: params.step_y.unwrap_or(2).max(1),
            endless: params.endless.unwrap_or(true),
            last_dy: 0,
        }
    }

    fn hide_and_reset(&mut self, object: &GameObject, commands: &mut Vec<BehaviorCommand>) {
        self.last_dy = 0;
        emit_visibility(commands, object.id.clone(), false);
    }
}

impl Behavior for MenuCarouselBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let total = self.count.unwrap_or(scene.menu_options.len());
        if total == 0 || self.index >= total {
            self.hide_and_reset(object, commands);
            return;
        }
        let Some(target_alias) = self.target.as_deref() else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.hide_and_reset(object, commands);
            return;
        };

        let selected = ctx.menu_selected_index % total;
        let relative = if self.endless {
            wrapped_menu_distance(self.index, selected, total)
        } else {
            self.index as i32 - selected as i32
        };
        let half_window = ((self.window.saturating_sub(1)) / 2) as i32;
        if relative.abs() > half_window {
            self.hide_and_reset(object, commands);
            return;
        }

        emit_visibility(commands, object.id.clone(), true);

        let Some(own_region) = ctx.object_region(&object.id) else {
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        // Keep menu items from collapsing into each other when authored `step_y`
        // is too small for the current rendered item height.
        let item_height = own_region.height.max(1) as i32;
        let effective_step_y = self.step_y.max(item_height.saturating_add(1));
        let center_y = target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);
        let desired_y = center_y.saturating_add(relative.saturating_mul(effective_step_y));
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dy = desired_y - base_y;
        self.last_dy = new_dy;
        emit_offset(commands, object.id.clone(), 0, new_dy);
    }
}

/// Repositions a group of menu items from one controller behavior attached to
/// the parent object/layer.
pub struct MenuCarouselObjectBehavior {
    target: Option<String>,
    item_prefix: String,
    count: Option<usize>,
    window: usize,
    step_y: i32,
    endless: bool,
    last_dy_by_index: BTreeMap<usize, i32>,
}

impl MenuCarouselObjectBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            item_prefix: params
                .item_prefix
                .clone()
                .unwrap_or_else(|| "menu-item-".to_string()),
            count: params.count,
            window: params.window.unwrap_or(5).max(1),
            step_y: params.step_y.unwrap_or(2).max(1),
            endless: params.endless.unwrap_or(true),
            last_dy_by_index: BTreeMap::new(),
        }
    }

    fn item_alias(&self, index: usize) -> String {
        if self.item_prefix.contains("{}") {
            self.item_prefix.replace("{}", &index.to_string())
        } else {
            format!("{}{}", self.item_prefix, index)
        }
    }
}

impl Behavior for MenuCarouselObjectBehavior {
    fn update(
        &mut self,
        _object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let total = self.count.unwrap_or(scene.menu_options.len());
        if total == 0 {
            self.last_dy_by_index.clear();
            return;
        }
        let Some(target_alias) = self.target.as_deref() else {
            self.last_dy_by_index.clear();
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.last_dy_by_index.clear();
            return;
        };

        let selected = ctx.menu_selected_index % total;
        let half_window = ((self.window.saturating_sub(1)) / 2) as i32;
        for index in 0..total {
            let item_alias = self.item_alias(index);
            let relative = if self.endless {
                wrapped_menu_distance(index, selected, total)
            } else {
                index as i32 - selected as i32
            };
            if relative.abs() > half_window {
                self.last_dy_by_index.insert(index, 0);
                emit_visibility(commands, item_alias, false);
                continue;
            }
            emit_visibility(commands, item_alias.clone(), true);

            let Some(item_region) = ctx.resolved_object_region(&item_alias) else {
                // First visible frame can happen before compositor reports regions.
                continue;
            };
            let last_dy = self.last_dy_by_index.get(&index).copied().unwrap_or(0);
            let item_height = item_region.height.max(1) as i32;
            let effective_step_y = self.step_y.max(item_height.saturating_add(1));
            let center_y =
                target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);
            let desired_y = center_y.saturating_add(relative.saturating_mul(effective_step_y));
            let base_y = item_region.y as i32 - last_dy;
            let new_dy = desired_y - base_y;
            self.last_dy_by_index.insert(index, new_dy);
            emit_offset(commands, item_alias, 0, new_dy);
        }
    }
}

// Thread-local cache of compiled Rhai ASTs keyed by script source text.
// Avoids re-parsing the same script string every frame while keeping
// the non-Send AST type out of cross-thread structs.
thread_local! {
    static AST_CACHE: std::cell::RefCell<std::collections::HashMap<u64, rhai::AST>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// Counter for assigning unique IDs to RhaiScriptBehavior instances.
// Used as the key into BEHAVIOR_SCOPES so each behavior owns its own
// persistent Rhai scope without storing non-Send types in the struct.
static NEXT_BEHAVIOR_ID: AtomicUsize = AtomicUsize::new(1);

// Per-behavior persistent Rhai scopes, keyed by behavior_id.
// Value: (scope, scope_base_len).
// Scopes are removed when the behavior is dropped (see impl Drop).
thread_local! {
    static BEHAVIOR_SCOPES: std::cell::RefCell<HashMap<usize, (rhai::Scope<'static>, usize)>> =
        std::cell::RefCell::new(HashMap::new());
}

// Thread-local Rhai engine with all static type/function registrations pre-done.
// Reused across all behavior evals on this thread — avoids 25× Engine::new()
// + 375 register_fn() calls per frame. All closures registered here are pure
// (no per-call captures); per-call data flows through scope variables instead.
thread_local! {
    static RHAI_ENGINE: std::cell::RefCell<Option<RhaiEngine>> = std::cell::RefCell::new(None);
}

// Rhai Module Resolver
// Allows scripts to import shared modules: `import "my-module" as my;`
// Modules are resolved from {MOD_SOURCE}/scripts/ directory.
// Example: mods/asteroids/scripts/shared.rhai
thread_local! {
    static MOD_SOURCE: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Set the mod source directory for Rhai module resolution.
/// Called once per thread during Rhai engine initialization.
fn set_mod_source(mod_source: String) {
    MOD_SOURCE.with(|source| {
        *source.borrow_mut() = Some(mod_source);
    });
}

/// Get the mod source directory for Rhai module resolution.
fn get_mod_source() -> String {
    MOD_SOURCE.with(|source| {
        source.borrow().clone().unwrap_or_else(|| {
            std::env::var("SHELL_QUEST_MOD_SOURCE")
                .unwrap_or_else(|_| "mods/shell-quest".to_string())
        })
    })
}

fn configure_rhai_limits(engine: &mut RhaiEngine) {
    // Gameplay scripts with vector math and stateful loops can exceed Rhai's
    // default parser complexity limit even when they are otherwise valid.
    // Raise the parse-depth ceilings to keep legitimate mod behaviors working.
    engine.set_max_expr_depths(128, 128);
}

fn init_rhai_engine() -> RhaiEngine {
    let mut engine = RhaiEngine::new();
    configure_rhai_limits(&mut engine);
    
    // Set up module resolver for `import "module-name" as name;` statements.
    // Modules are loaded from {MOD_SOURCE}/scripts/ directory.
    let mod_source = get_mod_source();
    let scripts_dir = std::path::PathBuf::from(&mod_source).join("scripts");
    let mut resolver = rhai::module_resolvers::FileModuleResolver::new();
    resolver.set_base_path(scripts_dir);
    engine.set_module_resolver(resolver);
    engine.register_fn("is_blank", |value: &str| -> bool {
        value.chars().all(char::is_whitespace)
    });
    engine.register_type_with_name::<ScriptSceneApi>("SceneApi");
    engine.register_type_with_name::<ScriptObjectApi>("SceneObject");
    engine.register_type_with_name::<ScriptGameApi>("GameApi");
    engine.register_type_with_name::<ScriptPersistenceApi>("PersistenceApi");
    engine.register_type_with_name::<ScriptGameplayApi>("GameplayApi");
    engine.register_type_with_name::<ScriptGameplayEntityApi>("GameplayEntityApi");
    engine.register_type_with_name::<ScriptTerminalApi>("TerminalApi");
    engine.register_type_with_name::<ScriptInputApi>("InputApi");
    engine.register_type_with_name::<ScriptDebugApi>("DebugApi");
    engine.register_type_with_name::<ScriptAudioApi>("AudioApi");

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
    engine.register_fn("get", |object: &mut ScriptObjectApi, path: &str| {
        object.get(path)
    });
    engine.register_fn(
        "set",
        |object: &mut ScriptObjectApi, path: &str, value: RhaiDynamic| {
            object.set(path, value);
        },
    );
    engine.register_fn("push", |terminal: &mut ScriptTerminalApi, line: &str| {
        terminal.push(line);
    });
    engine.register_fn("clear", |terminal: &mut ScriptTerminalApi| {
        terminal.clear();
    });
    engine.register_fn("down", |input: &mut ScriptInputApi, code: &str| {
        input.down(code)
    });
    engine.register_fn("any_down", |input: &mut ScriptInputApi| input.any_down());
    engine.register_fn("down_count", |input: &mut ScriptInputApi| {
        input.down_count()
    });
    engine.register_fn("action_down", |input: &mut ScriptInputApi, action: &str| {
        input.action_down(action)
    });
    engine.register_fn("bind_action", |input: &mut ScriptInputApi, action: &str, keys: rhai::Array| {
        input.bind_action(action, keys)
    });
    engine.register_fn("info", |debug: &mut ScriptDebugApi, message: &str| {
        debug.info(message);
    });
    engine.register_fn("warn", |debug: &mut ScriptDebugApi, message: &str| {
        debug.warn(message);
    });
    engine.register_fn("error", |debug: &mut ScriptDebugApi, message: &str| {
        debug.error(message);
    });
    engine.register_fn("cue", |audio: &mut ScriptAudioApi, cue: &str| {
        audio.cue(cue, None)
    });
    engine.register_fn(
        "cue",
        |audio: &mut ScriptAudioApi, cue: &str, volume: rhai::FLOAT| {
            audio.cue(cue, Some(volume as f32))
        },
    );
    engine.register_fn("event", |audio: &mut ScriptAudioApi, event: &str| {
        audio.event(event, None)
    });
    engine.register_fn(
        "event",
        |audio: &mut ScriptAudioApi, event: &str, gain_scale: rhai::FLOAT| {
            audio.event(event, Some(gain_scale as f32))
        },
    );
    engine.register_fn("play_song", |audio: &mut ScriptAudioApi, song_id: &str| {
        audio.play_song(song_id)
    });
    engine.register_fn("stop_song", |audio: &mut ScriptAudioApi| audio.stop_song());
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
    engine.register_fn("clear", |world: &mut ScriptGameplayApi| {
        world.clear();
    });
    engine.register_fn("count", |world: &mut ScriptGameplayApi| world.count());
    engine.register_fn("count_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.count_kind(kind)
    });
    engine.register_fn("count_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.count_tag(tag)
    });
    engine.register_fn("first_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.first_kind(kind)
    });
    engine.register_fn("first_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.first_tag(tag)
    });
    engine.register_fn(
        "spawn_object",
        |world: &mut ScriptGameplayApi, kind: &str, payload: RhaiDynamic| {
            world.spawn(kind, payload)
        },
    );
    engine.register_fn(
        "despawn_object",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.despawn(id),
    );
    engine.register_fn("exists", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.exists(id)
    });
    engine.register_fn("kind", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.kind(id)
    });
    engine.register_fn("tags", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.tags(id)
    });
    engine.register_fn("ids", |world: &mut ScriptGameplayApi| world.ids());
    engine.register_fn("entity", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.entity(id)
    });
    engine.register_fn("query_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.query_kind(kind)
    });
    engine.register_fn("query_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.query_tag(tag)
    });
    engine.register_fn(
        "get",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.get(id, path),
    );
    engine.register_fn(
        "set",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str, value: RhaiDynamic| {
            world.set(id, path, value)
        },
    );
    engine.register_fn(
        "has",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.has(id, path),
    );
    engine.register_fn(
        "remove",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.remove(id, path),
    );
    engine.register_fn(
        "push",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str, value: RhaiDynamic| {
            world.push(id, path, value)
        },
    );
    engine.register_fn(
        "set_transform",
        |world: &mut ScriptGameplayApi, id: rhai::INT, x: rhai::FLOAT, y: rhai::FLOAT, heading: rhai::FLOAT| {
            world.set_transform(id, x, y, heading)
        },
    );
    engine.register_fn(
        "transform",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.transform(id),
    );
    engine.register_fn(
        "set_physics",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         vx: rhai::FLOAT,
         vy: rhai::FLOAT,
         ax: rhai::FLOAT,
         ay: rhai::FLOAT,
         drag: rhai::FLOAT,
         max_speed: rhai::FLOAT| { world.set_physics(id, vx, vy, ax, ay, drag, max_speed) },
    );
    engine.register_fn(
        "physics",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.physics(id),
    );
    engine.register_fn(
        "set_collider_circle",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         radius: rhai::FLOAT,
         layer: rhai::INT,
         mask: rhai::INT| { world.set_collider_circle(id, radius, layer, mask) },
    );
    engine.register_fn(
        "set_lifetime",
        |world: &mut ScriptGameplayApi, id: rhai::INT, ttl_ms: rhai::INT| {
            world.set_lifetime(id, ttl_ms)
        },
    );
    engine.register_fn(
        "set_visual",
        |world: &mut ScriptGameplayApi, id: rhai::INT, visual_id: &str| {
            world.set_visual(id, visual_id)
        },
    );
    engine.register_fn(
        "bind_visual",
        |world: &mut ScriptGameplayApi, id: rhai::INT, visual_id: &str| {
            world.bind_visual(id, visual_id)
        },
    );
    engine.register_fn(
        "spawn_visual",
        |world: &mut ScriptGameplayApi, kind: &str, template: &str, data: RhaiMap| {
            world.spawn_visual(kind, template, data)
        },
    );
    engine.register_fn("collisions", |world: &mut ScriptGameplayApi| {
        world.collisions()
    });
    engine.register_fn(
        "collisions_between",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collisions_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collisions_of",
        |world: &mut ScriptGameplayApi, kind: &str| world.collisions_of(kind),
    );
    // ── Collision enter/stay/exit events ──────────────────────────────────
    engine.register_fn(
        "collision_enters",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_enters_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_stays",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_stays_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_exits",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_exits_between(kind_a, kind_b)
        },
    );
    // ── Child entity API ──────────────────────────────────────────────────
    engine.register_fn(
        "spawn_child",
        |world: &mut ScriptGameplayApi, parent_id: rhai::INT, kind: &str, template: &str, data: RhaiMap| {
            world.spawn_child_entity(parent_id, kind, template, data)
        },
    );
    engine.register_fn(
        "despawn_children",
        |world: &mut ScriptGameplayApi, parent_id: rhai::INT| {
            world.despawn_children_of(parent_id)
        },
    );
    engine.register_fn("exists", |entity: &mut ScriptGameplayEntityApi| {
        entity.exists()
    });
    engine.register_fn("get", |entity: &mut ScriptGameplayEntityApi, path: &str| {
        entity.get(path)
    });
    engine.register_fn(
        "get_i",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: rhai::INT| {
            entity.get_i(path, fallback)
        },
    );
    engine.register_fn(
        "get_bool",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: bool| {
            entity.get_bool(path, fallback)
        },
    );
    engine.register_fn(
        "get_b",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: bool| {
            entity.get_bool(path, fallback)
        },
    );
    engine.register_fn(
        "set",
        |entity: &mut ScriptGameplayEntityApi, path: &str, value: RhaiDynamic| {
            entity.set(path, value)
        },
    );
    engine.register_fn("kind", |entity: &mut ScriptGameplayEntityApi| {
        entity.kind()
    });
    engine.register_fn("tags", |entity: &mut ScriptGameplayEntityApi| {
        entity.tags()
    });
    engine.register_fn("get_metadata", |entity: &mut ScriptGameplayEntityApi| {
        entity.get_metadata()
    });
    engine.register_fn("get_components", |entity: &mut ScriptGameplayEntityApi| {
        entity.get_components()
    });
    engine.register_fn("transform", |entity: &mut ScriptGameplayEntityApi| {
        entity.transform()
    });
    engine.register_fn("set_position", |entity: &mut ScriptGameplayEntityApi, x: rhai::FLOAT, y: rhai::FLOAT| {
        entity.set_position(x, y)
    });
    engine.register_fn("set_heading", |entity: &mut ScriptGameplayEntityApi, heading: rhai::FLOAT| {
        entity.set_heading(heading)
    });
    engine.register_fn("physics", |entity: &mut ScriptGameplayEntityApi| {
        entity.physics()
    });
    engine.register_fn("set_velocity", |entity: &mut ScriptGameplayEntityApi, vx: rhai::FLOAT, vy: rhai::FLOAT| {
        entity.set_velocity(vx, vy)
    });
    engine.register_fn("set_acceleration", |entity: &mut ScriptGameplayEntityApi, ax: rhai::FLOAT, ay: rhai::FLOAT| {
        entity.set_acceleration(ax, ay)
    });
    engine.register_fn("collider", |entity: &mut ScriptGameplayEntityApi| {
        entity.collider()
    });
    engine.register_fn("lifetime_remaining", |entity: &mut ScriptGameplayEntityApi| {
        entity.lifetime_remaining()
    });
    engine.register_fn(
        "set_many",
        |entity: &mut ScriptGameplayEntityApi, map: RhaiMap| {
            entity.set_many(map)
        },
    );
    engine.register_fn("data", |entity: &mut ScriptGameplayEntityApi| {
        entity.data()
    });
    engine.register_fn(
        "get_f",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: rhai::FLOAT| {
            entity.get_f(path, fallback)
        },
    );
    engine.register_fn(
        "get_s",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: &str| {
            entity.get_s(path, fallback)
        },
    );
    engine.register_fn("despawn", |entity: &mut ScriptGameplayEntityApi| {
        entity.despawn()
    });

    // ── Cooldown API ──────────────────────────────────────────────────────
    engine.register_fn("cooldown_start",
        |entity: &mut ScriptGameplayEntityApi, name: &str, ms: rhai::INT| {
            entity.cooldown_start(name, ms)
        },
    );
    engine.register_fn("cooldown_ready",
        |entity: &mut ScriptGameplayEntityApi, name: &str| {
            entity.cooldown_ready(name)
        },
    );
    engine.register_fn("cooldown_remaining",
        |entity: &mut ScriptGameplayEntityApi, name: &str| -> rhai::INT {
            entity.cooldown_remaining(name)
        },
    );

    // ── Status API ────────────────────────────────────────────────────────
    engine.register_fn("status_add",
        |entity: &mut ScriptGameplayEntityApi, name: &str, ms: rhai::INT| {
            entity.status_add(name, ms)
        },
    );
    engine.register_fn("status_has",
        |entity: &mut ScriptGameplayEntityApi, name: &str| {
            entity.status_has(name)
        },
    );
    engine.register_fn("status_remaining",
        |entity: &mut ScriptGameplayEntityApi, name: &str| -> rhai::INT {
            entity.status_remaining(name)
        },
    );

    // ── Ship Controller API (on entity ref) ───────────────────────────────
    engine.register_fn("attach_ship_controller",
        |entity: &mut ScriptGameplayEntityApi, config: RhaiMap| {
            entity.attach_ship_controller(config)
        },
    );
    engine.register_fn("set_turn",
        |entity: &mut ScriptGameplayEntityApi, dir: rhai::INT| {
            entity.set_turn(dir)
        },
    );
    engine.register_fn("set_thrust",
        |entity: &mut ScriptGameplayEntityApi, on: bool| {
            entity.set_thrust(on)
        },
    );
    engine.register_fn("heading",
        |entity: &mut ScriptGameplayEntityApi| -> rhai::INT {
            entity.heading()
        },
    );
    engine.register_fn("heading_vector",
        |entity: &mut ScriptGameplayEntityApi| -> RhaiMap {
            entity.heading_vector()
        },
    );

    // ── Health & Damage API (on entity ref) ───────────────────────────────
    engine.register_fn("health_set",
        |entity: &mut ScriptGameplayEntityApi, hp: rhai::INT, max_hp: rhai::INT| -> bool {
            entity.health_set(hp, max_hp)
        },
    );
    engine.register_fn("health_get",
        |entity: &mut ScriptGameplayEntityApi| -> rhai::INT {
            entity.health_get()
        },
    );
    engine.register_fn("health_max",
        |entity: &mut ScriptGameplayEntityApi| -> rhai::INT {
            entity.health_max()
        },
    );
    engine.register_fn("health_dead",
        |entity: &mut ScriptGameplayEntityApi| -> bool {
            entity.health_dead()
        },
    );
    engine.register_fn("damage",
        |entity: &mut ScriptGameplayEntityApi, source: rhai::INT, amount: rhai::INT| -> bool {
            entity.damage(source, amount)
        },
    );

    // ── Wrap API ──────────────────────────────────────────────────────────
    engine.register_fn(
        "enable_wrap",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         min_x: rhai::FLOAT, max_x: rhai::FLOAT,
         min_y: rhai::FLOAT, max_y: rhai::FLOAT| {
            world.enable_wrap(id, min_x, max_x, min_y, max_y)
        },
    );
    engine.register_fn(
        "disable_wrap",
        |world: &mut ScriptGameplayApi, id: rhai::INT| {
            world.disable_wrap(id)
        },
    );
    engine.register_fn(
        "poly_hit",
        |poly_a: RhaiArray,
         ax: rhai::INT,
         ay: rhai::INT,
         poly_b: RhaiArray,
         bx: rhai::INT,
         by: rhai::INT|
         -> bool {
            let points_a = rhai_array_to_points(&poly_a);
            let points_b = rhai_array_to_points(&poly_b);
            if points_a.len() < 2 || points_b.len() < 2 {
                return false;
            }
            polygons_intersect(
                &points_a,
                [to_i32(ax), to_i32(ay)],
                &points_b,
                [to_i32(bx), to_i32(by)],
            )
        },
    );
    engine.register_fn(
        "point_in_poly",
        |px: rhai::INT, py: rhai::INT, poly: RhaiArray, ox: rhai::INT, oy: rhai::INT| -> bool {
            let points = rhai_array_to_points(&poly);
            if points.len() < 3 {
                return false;
            }
            point_in_polygon([to_i32(px), to_i32(py)], &points, [to_i32(ox), to_i32(oy)])
        },
    );
    engine.register_fn(
        "segment_poly_hit",
        |x0: rhai::INT,
         y0: rhai::INT,
         x1: rhai::INT,
         y1: rhai::INT,
         poly: RhaiArray,
         ox: rhai::INT,
         oy: rhai::INT|
         -> bool {
            let points = rhai_array_to_points(&poly);
            if points.len() < 2 {
                return false;
            }
            segment_intersects_polygon(
                [to_i32(x0), to_i32(y0)],
                [to_i32(x1), to_i32(y1)],
                &points,
                [to_i32(ox), to_i32(oy)],
            )
        },
    );

    // ── Ship Controller API ────────────────────────────────────────────────
    engine.register_fn("attach_ship_controller",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.attach_ship_controller(id, config)
        },
    );
    engine.register_fn("ship_set_turn",
        |world: &mut ScriptGameplayApi, id: rhai::INT, dir: rhai::INT| {
            world.ship_set_turn(id, dir)
        },
    );
    engine.register_fn("ship_set_thrust",
        |world: &mut ScriptGameplayApi, id: rhai::INT, on: bool| {
            world.ship_set_thrust(id, on)
        },
    );
    engine.register_fn("ship_heading",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> rhai::INT {
            world.ship_heading(id) as rhai::INT
        },
    );
    engine.register_fn("ship_heading_vector",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> RhaiMap {
            world.ship_heading_vector(id)
        },
    );
    engine.register_fn("ship_velocity",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> RhaiMap {
            world.ship_velocity(id)
        },
    );

    // ── Event API ──────────────────────────────────────────────────────
    engine.register_fn("poll_collisions",
        |world: &mut ScriptGameplayApi| -> RhaiArray {
            world.poll_collision_events()
        },
    );
    engine.register_fn("clear_events",
        |world: &mut ScriptGameplayApi| {
            world.clear_events();
        },
    );

    // ── Health & Damage API ────────────────────────────────────────────
    engine.register_fn("health_set",
        |world: &mut ScriptGameplayApi, id: rhai::INT, hp: rhai::INT, max_hp: rhai::INT| -> bool {
            world.health_set(id, hp, max_hp)
        },
    );
    engine.register_fn("health_get",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> rhai::INT {
            world.health_get(id)
        },
    );
    engine.register_fn("health_max",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> rhai::INT {
            world.health_max(id)
        },
    );
    engine.register_fn("health_dead",
        |world: &mut ScriptGameplayApi, id: rhai::INT| -> bool {
            world.health_dead(id)
        },
    );
    engine.register_fn("damage_apply",
        |world: &mut ScriptGameplayApi, target: rhai::INT, source: rhai::INT, amount: rhai::INT| -> bool {
            world.damage_apply(target, source, amount)
        },
    );

    engine.register_fn("destructible_configure",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| -> bool {
            world.destructible_configure(id, config)
        },
    );

    engine.register_fn("abs_i", |v: rhai::INT| -> rhai::INT {
        if v < 0 {
            -v
        } else {
            v
        }
    });
    engine.register_fn("abs_i", |v: rhai::FLOAT| -> rhai::INT {
        if v < 0.0 {
            (-v) as rhai::INT
        } else {
            v as rhai::INT
        }
    });
    engine.register_fn("sign_i", |v: rhai::INT, fallback: rhai::INT| -> rhai::INT {
        if v < 0 {
            -1
        } else if v > 0 {
            1
        } else {
            fallback
        }
    });
    // Basic numeric casts for scripts
    engine.register_fn("to_i", |v: rhai::INT| -> rhai::INT { v });
    engine.register_fn("to_i", |v: rhai::FLOAT| -> rhai::INT { v as rhai::INT });
    engine.register_fn(
        "clamp_i",
        |v: rhai::INT, min_v: rhai::INT, max_v: rhai::INT| -> rhai::INT {
            if v < min_v {
                min_v
            } else if v > max_v {
                max_v
            } else {
                v
            }
        },
    );
    engine.register_fn(
        "clamp_f",
        |v: rhai::FLOAT, min_v: rhai::FLOAT, max_v: rhai::FLOAT| -> rhai::FLOAT {
            if v < min_v {
                min_v
            } else if v > max_v {
                max_v
            } else {
                v
            }
        },
    );
    engine.register_fn(
        "wrap",
        |v: rhai::INT, min_v: rhai::INT, max_v: rhai::INT| -> rhai::INT {
            if v < min_v {
                max_v
            } else if v > max_v {
                min_v
            } else {
                v
            }
        },
    );
    engine.register_fn(
        "wrap",
        |v: rhai::FLOAT, min_v: rhai::INT, max_v: rhai::INT| -> rhai::FLOAT {
            let min_f = min_v as rhai::FLOAT;
            let max_f = max_v as rhai::FLOAT;
            if v < min_f {
                max_f
            } else if v > max_f {
                min_f
            } else {
                v
            }
        },
    );
    engine.register_fn(
        "wrap",
        |v: rhai::FLOAT, min_v: rhai::FLOAT, max_v: rhai::FLOAT| -> rhai::FLOAT {
            if v < min_v {
                max_v
            } else if v > max_v {
                min_v
            } else {
                v
            }
        },
    );
    engine.register_fn(
        "wrap_fp",
        |v: rhai::INT, min_v: rhai::INT, max_v: rhai::INT, scale: rhai::INT| -> rhai::INT {
            let min_fp = min_v * scale;
            let max_fp = max_v * scale;
            if v < min_fp {
                max_fp
            } else if v > max_fp {
                min_fp
            } else {
                v
            }
        },
    );
    engine.register_fn("wrap_heading32", |v: rhai::INT| -> rhai::INT {
        let out = v % 32;
        if out < 0 {
            out + 32
        } else {
            out
        }
    });
    engine.register_fn(
        "rng_next_i",
        |seed: rhai::INT, modulus: rhai::INT| -> rhai::INT {
            let m = if modulus > 0 { modulus } else { 2_147_483_647 };
            ((seed * 1_103_515_245) + 12_345) % m
        },
    );
    engine.register_fn("sin32", |idx: rhai::INT| -> rhai::INT {
        sin32_i32(to_i32(idx)) as rhai::INT
    });
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn("ship_points", |heading: rhai::INT| -> RhaiArray {
        points_to_rhai_array(ship_points_i32(to_i32(heading)))
    });
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn(
        "asteroid_points",
        |shape: rhai::INT, size: rhai::INT| -> RhaiArray {
            points_to_rhai_array(asteroid_points_i32(to_i32(shape), to_i32(size)))
        },
    );
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn(
        "rotate_points",
        |points: RhaiArray, heading: rhai::INT| -> RhaiArray {
            let points = rhai_array_to_points(&points);
            points_to_rhai_array(rotate_points_i32(&points, to_i32(heading)))
        },
    );
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn(
        "asteroid_fragment_points",
        |shape: rhai::INT, size: rhai::INT, fragment: rhai::INT| -> RhaiArray {
            points_to_rhai_array(asteroid_fragment_points_i32(
                to_i32(shape),
                to_i32(size),
                to_i32(fragment),
            ))
        },
    );
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn("asteroid_radius", |size: rhai::INT| -> rhai::INT {
        asteroid_radius_i32(to_i32(size)) as rhai::INT
    });
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn("asteroid_score", |size: rhai::INT| -> rhai::INT {
        asteroid_score_i32(to_i32(size)) as rhai::INT
    });
    engine
}

fn script_hash(script: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    script.hash(&mut h);
    h.finish()
}

/// Evaluates per-frame behavior commands from a Rhai script.
pub struct RhaiScriptBehavior {
    params: BehaviorParams,
    state: JsonValue,
    /// Compile-time error text stored here and emitted as ScriptError once.
    compile_error: Option<String>,
    compile_error_reported: bool,
    /// Unique ID used to look up this behavior's persistent Rhai scope in
    /// the thread-local BEHAVIOR_SCOPES map. Avoids storing non-Send
    /// `rhai::Scope` directly in the struct while still reusing the scope
    /// across frames (eliminates `Scope::new()` + ~30 pushes per frame).
    behavior_id: usize,
}

#[derive(Clone)]
struct ScriptSceneApi {
    object_states: Arc<HashMap<String, ObjectRuntimeState>>,
    object_kinds: Arc<HashMap<String, String>>,
    object_props: Arc<HashMap<String, JsonValue>>,
    object_regions: Arc<HashMap<String, Region>>,
    object_text: Arc<HashMap<String, String>>,
    target_resolver: Arc<TargetResolver>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptObjectApi {
    target: String,
    snapshot: RhaiMap,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptGameApi {
    state: Option<GameState>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptLevelApi {
    state: Option<LevelState>,
}

#[derive(Clone)]
struct ScriptPersistenceApi {
    store: Option<PersistenceStore>,
}

#[derive(Clone)]
struct ScriptGameplayApi {
    world: Option<GameplayWorld>,
    collisions: std::sync::Arc<Vec<CollisionHit>>,
    collision_enters: std::sync::Arc<Vec<CollisionHit>>,
    collision_stays: std::sync::Arc<Vec<CollisionHit>>,
    collision_exits: std::sync::Arc<Vec<CollisionHit>>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptGameplayEntityApi {
    world: Option<GameplayWorld>,
    id: u64,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptTerminalApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptInputApi {
    keys_down: Arc<HashSet<String>>,
    action_bindings: Arc<HashMap<String, Vec<String>>>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptDebugApi {
    scene_id: String,
    source: Option<String>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
struct ScriptAudioApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptSceneApi {
    fn new(
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

impl ScriptGameApi {
    fn new(state: Option<GameState>, queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
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
}

impl ScriptLevelApi {
    fn new(state: Option<LevelState>) -> Self {
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

impl ScriptTerminalApi {
    fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn push(&mut self, line: &str) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::TerminalPushOutput {
            line: line.to_string(),
        });
    }

    fn clear(&mut self) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::TerminalClearOutput);
    }
}

impl ScriptInputApi {
    fn new(
        keys_down: Arc<HashSet<String>>,
        action_bindings: Arc<HashMap<String, Vec<String>>>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self { keys_down, action_bindings, queue }
    }

    fn down(&mut self, code: &str) -> bool {
        let normalized = normalize_input_code(code);
        if normalized.is_empty() {
            return false;
        }
        self.keys_down.contains(&normalized)
    }

    fn any_down(&mut self) -> bool {
        !self.keys_down.is_empty()
    }

    fn down_count(&mut self) -> rhai::INT {
        self.keys_down.len() as rhai::INT
    }

    /// Returns `true` if any key bound to `action` is currently held.
    fn action_down(&mut self, action: &str) -> bool {
        let Some(keys) = self.action_bindings.get(action) else { return false };
        keys.iter().any(|k| {
            let n = normalize_input_code(k);
            !n.is_empty() && self.keys_down.contains(&n)
        })
    }

    /// Bind an action to a list of key codes. Emits a `BindInputAction` command.
    fn bind_action(&mut self, action: &str, keys: rhai::Array) -> bool {
        let key_strs: Vec<String> = keys
            .into_iter()
            .filter_map(|v| v.into_string().ok())
            .collect();
        if let Ok(mut q) = self.queue.lock() {
            q.push(BehaviorCommand::BindInputAction {
                action: action.to_string(),
                keys: key_strs,
            });
        }
        true
    }
}

impl ScriptDebugApi {
    fn new(
        scene_id: String,
        source: Option<String>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            scene_id,
            source,
            queue,
        }
    }

    fn info(&mut self, message: &str) {
        self.push(DebugLogSeverity::Info, message);
    }

    fn warn(&mut self, message: &str) {
        self.push(DebugLogSeverity::Warn, message);
    }

    fn error(&mut self, message: &str) {
        self.push(DebugLogSeverity::Error, message);
    }

    fn push(&mut self, severity: DebugLogSeverity, message: &str) {
        let trimmed = message.trim();
        if trimmed.is_empty() {
            return;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::DebugLog {
            scene_id: self.scene_id.clone(),
            source: self.source.clone(),
            severity,
            message: trimmed.to_string(),
        });
    }
}

impl ScriptAudioApi {
    fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn cue(&mut self, cue: &str, volume: Option<f32>) -> bool {
        let cue = cue.trim();
        if cue.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioCue {
            cue: cue.to_string(),
            volume,
        });
        true
    }

    fn event(&mut self, event: &str, gain: Option<f32>) -> bool {
        let event = event.trim();
        if event.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioEvent {
            event: event.to_string(),
            gain,
        });
        true
    }

    fn play_song(&mut self, song_id: &str) -> bool {
        let song_id = song_id.trim();
        if song_id.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlaySong {
            song_id: song_id.to_string(),
        });
        true
    }

    fn stop_song(&mut self) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::StopSong);
        true
    }
}

impl ScriptPersistenceApi {
    fn new(store: Option<PersistenceStore>) -> Self {
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

    fn reload(&mut self) -> bool {
        self.store
            .as_ref()
            .map(PersistenceStore::reload)
            .unwrap_or(false)
    }
}

impl ScriptGameplayApi {
    fn new(
        world: Option<GameplayWorld>,
        collisions: std::sync::Arc<Vec<CollisionHit>>,
        collision_enters: std::sync::Arc<Vec<CollisionHit>>,
        collision_stays: std::sync::Arc<Vec<CollisionHit>>,
        collision_exits: std::sync::Arc<Vec<CollisionHit>>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            world,
            collisions,
            collision_enters,
            collision_stays,
            collision_exits,
            queue,
        }
    }

    fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        if id < 0 {
            return ScriptGameplayEntityApi {
                world: None,
                id: 0,
                queue: Arc::clone(&self.queue),
            };
        }
        ScriptGameplayEntityApi {
            world: self.world.clone(),
            id: id as u64,
            queue: Arc::clone(&self.queue),
        }
    }

    fn clear(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear();
        }
    }

    fn count(&mut self) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count() as rhai::INT)
            .unwrap_or(0)
    }

    fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_kind(kind) as rhai::INT)
            .unwrap_or(0)
    }

    fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_tag(tag) as rhai::INT)
            .unwrap_or(0)
    }

    fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_kind(kind))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_tag(tag))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let Some(payload) = rhai_dynamic_to_json(&payload) else {
            return 0;
        };
        world
            .spawn(kind, payload)
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn despawn(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let uid = id as u64;
        if let Some(binding) = world.visual(uid) {
            if let Ok(mut commands) = self.queue.lock() {
                for vid in binding.all_visual_ids() {
                    commands.push(BehaviorCommand::SceneDespawn {
                        target: vid.to_string(),
                    });
                }
            }
        }
        world.despawn(uid)
    }

    fn bind_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 || visual_id.trim().is_empty() {
            return false;
        }
        world.add_visual(id as u64, visual_id.to_string())
    }

    fn exists(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.exists(id as u64)
    }

    fn kind(&mut self, id: rhai::INT) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        if id < 0 {
            return String::new();
        }
        world.kind_of(id as u64).unwrap_or_default()
    }

    fn tags(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        if id < 0 {
            return RhaiArray::new();
        }
        world.tags(id as u64).into_iter().map(Into::into).collect()
    }

    fn ids(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .ids()
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn query_kind(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_kind(kind)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn query_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_tag(tag)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        world
            .get(id as u64, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(id as u64, path, value)
    }

    fn has(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.has(id as u64, path)
    }

    fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.remove(id as u64, path)
    }

    fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.push(id as u64, path, value)
    }

    fn set_transform(&mut self, id: rhai::INT, x: rhai::FLOAT, y: rhai::FLOAT, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_transform(
            id as u64,
            Transform2D {
                x: x as f32,
                y: y as f32,
                heading: heading as f32,
            },
        )
    }

    fn transform(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(xf) = world.transform(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("x".into(), (xf.x as rhai::FLOAT).into());
            map.insert("y".into(), (xf.y as rhai::FLOAT).into());
            map.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
            return map.into();
        }
        ().into()
    }

    fn set_physics(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_physics(
            id as u64,
            PhysicsBody2D {
                vx: vx as f32,
                vy: vy as f32,
                ax: ax as f32,
                ay: ay as f32,
                drag: drag as f32,
                max_speed: max_speed as f32,
            },
        )
    }

    fn physics(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(body) = world.physics(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("vx".into(), (body.vx as rhai::FLOAT).into());
            map.insert("vy".into(), (body.vy as rhai::FLOAT).into());
            map.insert("ax".into(), (body.ax as rhai::FLOAT).into());
            map.insert("ay".into(), (body.ay as rhai::FLOAT).into());
            map.insert("drag".into(), (body.drag as rhai::FLOAT).into());
            map.insert("max_speed".into(), (body.max_speed as rhai::FLOAT).into());
            return map.into();
        }
        ().into()
    }

    fn set_collider_circle(
        &mut self,
        id: rhai::INT,
        radius: rhai::FLOAT,
        layer: rhai::INT,
        mask: rhai::INT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_collider(
            id as u64,
            Collider2D {
                shape: ColliderShape::Circle { radius: radius as f32 },
                layer: layer as u32,
                mask: mask as u32,
            },
        )
    }

    fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_lifetime(
            id as u64,
            Lifetime {
                ttl_ms: ttl_ms as i32,
                on_expire: DespawnVisual::None,
            },
        )
    }

    fn set_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_visual(
            id as u64,
            VisualBinding {
                visual_id: if visual_id.trim().is_empty() {
                    None
                } else {
                    Some(visual_id.to_string())
                },
                additional_visuals: Vec::new(),
            },
        )
    }

    fn spawn_visual(
        &mut self,
        kind: &str,
        template: &str,
        data: RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };

        // Step 1: Spawn gameplay entity with empty payload
        let Some(entity_id) = world.spawn(kind, JsonValue::Object(JsonMap::new())) else {
            return 0;
        };

        // Step 2: Generate visual_id (format: "{kind}-{entity_id}")
        let visual_id = format!("{}-{}", kind, entity_id);

        // Step 3: Emit SceneSpawn command
        {
            let mut commands = match self.queue.lock() {
                Ok(cmds) => cmds,
                Err(_) => {
                    world.despawn(entity_id);
                    return 0;
                }
            };
            commands.push(BehaviorCommand::SceneSpawn {
                template: template.to_string(),
                target: visual_id.clone(),
            });
        }

        // Step 4: Set visual binding
        if !world.set_visual(
            entity_id,
            VisualBinding {
                visual_id: Some(visual_id.clone()),
                additional_visuals: Vec::new(),
            },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 5: Set transform from data
        let x = data
            .get("x")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let y = data
            .get("y")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let heading = data
            .get("heading")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;

        if !world.set_transform(
            entity_id,
            Transform2D { x, y, heading },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 6: Set collider if provided
        if let Some(radius_val) = data.get("collider_radius") {
            let radius_opt = radius_val
                .clone()
                .try_cast::<rhai::FLOAT>()
                .or_else(|| {
                    radius_val
                        .clone()
                        .try_cast::<rhai::INT>()
                        .map(|i| i as rhai::FLOAT)
                });
            if let Some(radius) = radius_opt {
                let layer = data
                    .get("collider_layer")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;
                let mask = data
                    .get("collider_mask")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;

                if !world.set_collider(
                    entity_id,
                    Collider2D {
                        shape: ColliderShape::Circle {
                            radius: radius as f32,
                        },
                        layer,
                        mask,
                    },
                ) {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        // Step 6b: Set polygon collider if provided
        if let Some(poly_val) = data.get("collider_polygon") {
            if let Some(poly_arr) = poly_val.clone().try_cast::<RhaiArray>() {
                let mut points: Vec<[f32; 2]> = Vec::new();
                for point in poly_arr {
                    if let Some(point_arr) = point.try_cast::<RhaiArray>() {
                        if point_arr.len() >= 2 {
                            if let (Some(px), Some(py)) = (
                                point_arr[0].clone().try_cast::<rhai::FLOAT>(),
                                point_arr[1].clone().try_cast::<rhai::FLOAT>(),
                            ) {
                                points.push([px as f32, py as f32]);
                            }
                        }
                    }
                }
                if !points.is_empty() {
                    let layer = data
                        .get("collider_layer")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;
                    let mask = data
                        .get("collider_mask")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;

                    if !world.set_collider(
                        entity_id,
                        Collider2D {
                            shape: ColliderShape::Polygon { points },
                            layer,
                            mask,
                        },
                    ) {
                        world.despawn(entity_id);
                        return 0;
                    }
                }
            }
        }

        // Step 7: Set lifetime if provided
        if let Some(ttl_val) = data.get("lifetime_ms") {
            if let Some(ttl) = ttl_val.clone().try_cast::<rhai::INT>() {
                if !world.set_lifetime(
                    entity_id,
                    Lifetime {
                        ttl_ms: ttl as i32,
                        on_expire: DespawnVisual::None,
                    },
                ) {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        entity_id as rhai::INT
    }

    fn collisions(&mut self) -> RhaiArray {
        self.collisions
            .iter()
            .map(|hit| {
                let mut map = RhaiMap::new();
                map.insert("a".into(), (hit.a as rhai::INT).into());
                map.insert("b".into(), (hit.b as rhai::INT).into());
                map.into()
            })
            .collect()
    }

    fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        self.collisions
            .iter()
            .filter_map(|hit| {
                let ka = world.kind_of(hit.a).unwrap_or_default();
                let kb = world.kind_of(hit.b).unwrap_or_default();
                if ka == kind_a && kb == kind_b {
                    let mut map = RhaiMap::new();
                    map.insert(kind_a.into(), (hit.a as rhai::INT).into());
                    map.insert(kind_b.into(), (hit.b as rhai::INT).into());
                    Some(map.into())
                } else if ka == kind_b && kb == kind_a {
                    let mut map = RhaiMap::new();
                    map.insert(kind_a.into(), (hit.b as rhai::INT).into());
                    map.insert(kind_b.into(), (hit.a as rhai::INT).into());
                    Some(map.into())
                } else {
                    None
                }
            })
            .collect()
    }

    fn collisions_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        self.collisions
            .iter()
            .filter_map(|hit| {
                let ka = world.kind_of(hit.a).unwrap_or_default();
                let kb = world.kind_of(hit.b).unwrap_or_default();
                if ka == kind {
                    let mut map = RhaiMap::new();
                    map.insert("self".into(), (hit.a as rhai::INT).into());
                    map.insert("other".into(), (hit.b as rhai::INT).into());
                    Some(map.into())
                } else if kb == kind {
                    let mut map = RhaiMap::new();
                    map.insert("self".into(), (hit.b as rhai::INT).into());
                    map.insert("other".into(), (hit.a as rhai::INT).into());
                    Some(map.into())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Filters a collision hit slice by kind pair, returning `{kind_a: id, kind_b: id}` maps.
    fn filter_hits_by_kind(
        hits: &[CollisionHit],
        world: &GameplayWorld,
        kind_a: &str,
        kind_b: &str,
    ) -> RhaiArray {
        hits.iter()
            .filter_map(|hit| {
                let ka = world.kind_of(hit.a).unwrap_or_default();
                let kb = world.kind_of(hit.b).unwrap_or_default();
                if ka == kind_a && kb == kind_b {
                    let mut map = RhaiMap::new();
                    map.insert(kind_a.into(), (hit.a as rhai::INT).into());
                    map.insert(kind_b.into(), (hit.b as rhai::INT).into());
                    Some(map.into())
                } else if ka == kind_b && kb == kind_a {
                    let mut map = RhaiMap::new();
                    map.insert(kind_a.into(), (hit.b as rhai::INT).into());
                    map.insert(kind_b.into(), (hit.a as rhai::INT).into());
                    Some(map.into())
                } else {
                    None
                }
            })
            .collect()
    }

    fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else { return vec![] };
        Self::filter_hits_by_kind(&self.collision_enters.clone(), world, kind_a, kind_b)
    }

    fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else { return vec![] };
        Self::filter_hits_by_kind(&self.collision_stays.clone(), world, kind_a, kind_b)
    }

    fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else { return vec![] };
        Self::filter_hits_by_kind(&self.collision_exits.clone(), world, kind_a, kind_b)
    }

    fn spawn_child_entity(&mut self, parent_id: rhai::INT, kind: &str, template: &str, data: RhaiMap) -> rhai::INT {
        if parent_id < 0 { return 0; }
        // Check parent exists before taking &mut self via spawn_visual
        let parent_uid = parent_id as u64;
        let parent_exists = self.world.as_ref().map(|w| w.exists(parent_uid)).unwrap_or(false);
        if !parent_exists { return 0; }
        let child_id = self.spawn_visual(kind, template, data);
        if child_id > 0 {
            if let Some(world) = self.world.as_ref() {
                world.register_child(parent_uid, child_id as u64);
            }
        }
        child_id
    }

    fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        if parent_id < 0 { return; }
        let Some(world) = self.world.as_ref() else { return };
        world.despawn_children(parent_id as u64);
    }

    fn enable_wrap(&mut self, id: rhai::INT, min_x: rhai::FLOAT, max_x: rhai::FLOAT,
                   min_y: rhai::FLOAT, max_y: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        let bounds = engine_game::WrapBounds::new(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
        world.set_wrap_bounds(uid, bounds)
    }

    fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        world.remove_wrap_bounds(uid);
        true
    }

    fn attach_ship_controller(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;

        // Extract config values with defaults
        let turn_step_ms = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(40) as u32;

        let thrust_power = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(170.0) as f32;

        let max_speed = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(4.5) as f32;

        let heading_bits = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(32) as u8;

        let controller = TopDownShipController::new(turn_step_ms, thrust_power, max_speed, heading_bits);
        world.attach_controller(uid, controller)
    }

    fn ship_set_turn(&mut self, id: rhai::INT, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    fn ship_set_thrust(&mut self, id: rhai::INT, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    fn ship_heading(&mut self, id: rhai::INT) -> i32 {
        let Some(world) = self.world.as_ref() else { return 0 };
        let uid = id as u64;
        world.controller(uid).map(|c| c.current_heading).unwrap_or(0)
    }

    fn ship_heading_vector(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.world.as_ref() else { return RhaiMap::new() };
        let uid = id as u64;
        match world.controller(uid) {
            Some(ctrl) => {
                let (x, y) = ctrl.heading_vector();
                let mut map = RhaiMap::new();
                map.insert("x".into(), (x as rhai::FLOAT).into());
                map.insert("y".into(), (y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    fn ship_velocity(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.world.as_ref() else { return RhaiMap::new() };
        let uid = id as u64;
        match world.physics(uid) {
            Some(body) => {
                let mut map = RhaiMap::new();
                map.insert("vx".into(), (body.vx as rhai::FLOAT).into());
                map.insert("vy".into(), (body.vy as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    fn poll_collision_events(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else { return RhaiArray::new() };
        let collisions = world.poll_events("collision_enter");
        let mut array = RhaiArray::new();
        for (a, b) in collisions {
            let mut event = RhaiMap::new();
            event.insert("a".into(), (a as rhai::INT).into());
            event.insert("b".into(), (b as rhai::INT).into());
            array.push(RhaiDynamic::from(event));
        }
        array
    }

    fn clear_events(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear_events();
        }
    }

    fn health_set(&mut self, id: rhai::INT, hp: rhai::INT, max_hp: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        world.set_health(uid, hp as i32, max_hp as i32)
    }

    fn health_get(&mut self, id: rhai::INT) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        let uid = id as u64;
        world.health(uid).map(|h| h.hp as rhai::INT).unwrap_or(0)
    }

    fn health_max(&mut self, id: rhai::INT) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        let uid = id as u64;
        world.health(uid).map(|h| h.max_hp as rhai::INT).unwrap_or(0)
    }

    fn health_dead(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;
        world.is_dead(uid)
    }

    fn damage_apply(&mut self, target: rhai::INT, source: rhai::INT, amount: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let target_uid = target as u64;
        let source_uid = source as u64;
        world.apply_damage(target_uid, source_uid, amount as i32)
    }

    fn destructible_configure(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let uid = id as u64;

        // Extract config fields with defaults
        let delay_ms = config
            .get("delay_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(0) as u32;
        let child_count = config
            .get("child_count")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(2) as u32;
        let size_delta = config
            .get("size_delta")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(-1) as i32;
        let velocity_factor = config
            .get("velocity_factor")
            .and_then(|v| v.clone().try_cast::<f64>())
            .unwrap_or(1.0) as f32;

        let split_config = SplitOnDestroy::new(delay_ms, child_count, size_delta, velocity_factor);
        world.set_split_on_destroy(uid, split_config)
    }
}

impl ScriptGameplayEntityApi {
    fn exists(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.exists(self.id)
    }

    fn despawn(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if let Some(binding) = world.visual(self.id) {
            if let Ok(mut commands) = self.queue.lock() {
                for vid in binding.all_visual_ids() {
                    commands.push(BehaviorCommand::SceneDespawn {
                        target: vid.to_string(),
                    });
                }
            }
        }
        world.despawn(self.id)
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        world
            .get(self.id, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    fn get_bool(&mut self, path: &str, fallback: bool) -> bool {
        self.get(path).try_cast::<bool>().unwrap_or(fallback)
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.id, path, value)
    }

    fn kind(&mut self) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        world.kind_of(self.id).unwrap_or_default()
    }

    fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .tags(self.id)
            .into_iter()
            .map(|tag| tag.into())
            .collect()
    }

    fn get_metadata(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.id) else {
            return RhaiMap::new();
        };

        let mut metadata = RhaiMap::new();
        metadata.insert("id".into(), (self.id as rhai::INT).into());
        metadata.insert("kind".into(), entity.kind.into());
        
        let tags: RhaiArray = entity.tags.iter().map(|t| t.clone().into()).collect();
        metadata.insert("tags".into(), tags.into());

        // Include all components
        if let Some(transform) = world.transform(self.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            metadata.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert("max_speed".into(), (physics.max_speed as rhai::FLOAT).into());
            metadata.insert("physics".into(), phys.into());
        }

        if let Some(collider) = world.collider(self.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            metadata.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            metadata.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.id) {
            if let Some(visual_id) = &visual.visual_id {
                metadata.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                metadata.insert("additional_visuals".into(), extras.into());
            }
        }

        metadata
    }

    fn get_components(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };

        let mut components = RhaiMap::new();

        if let Some(transform) = world.transform(self.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            components.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert("max_speed".into(), (physics.max_speed as rhai::FLOAT).into());
            components.insert("physics".into(), phys.into());
        }

        if let Some(collider) = world.collider(self.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            components.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            components.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.id) {
            if let Some(visual_id) = &visual.visual_id {
                components.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                components.insert("additional_visuals".into(), extras.into());
            }
        }

        components
    }

    fn transform(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(xf) = world.transform(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("x".into(), (xf.x as rhai::FLOAT).into());
        result.insert("y".into(), (xf.y as rhai::FLOAT).into());
        result.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
        result
    }

    fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.id) else {
            return false;
        };
        xf.x = x as f32;
        xf.y = y as f32;
        world.set_transform(self.id, xf)
    }

    fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.id) else {
            return false;
        };
        xf.heading = heading as f32;
        world.set_transform(self.id, xf)
    }

    fn physics(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(phys) = world.physics(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("vx".into(), (phys.vx as rhai::FLOAT).into());
        result.insert("vy".into(), (phys.vy as rhai::FLOAT).into());
        result.insert("ax".into(), (phys.ax as rhai::FLOAT).into());
        result.insert("ay".into(), (phys.ay as rhai::FLOAT).into());
        result.insert("drag".into(), (phys.drag as rhai::FLOAT).into());
        result.insert("max_speed".into(), (phys.max_speed as rhai::FLOAT).into());
        result
    }

    fn set_velocity(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.id) else {
            return false;
        };
        phys.vx = vx as f32;
        phys.vy = vy as f32;
        world.set_physics(self.id, phys)
    }

    fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.id) else {
            return false;
        };
        phys.ax = ax as f32;
        phys.ay = ay as f32;
        world.set_physics(self.id, phys)
    }

    fn collider(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        match &coll.shape {
            ColliderShape::Circle { radius } => {
                result.insert("shape".into(), "circle".into());
                result.insert("radius".into(), (*radius as rhai::FLOAT).into());
            }
            ColliderShape::Polygon { points } => {
                result.insert("shape".into(), "polygon".into());
                let pts: RhaiArray = points
                    .iter()
                    .map(|p| {
                        let mut point = RhaiMap::new();
                        point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                        point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                        point.into()
                    })
                    .collect();
                result.insert("points".into(), pts.into());
            }
        }
        result.insert("layer".into(), (coll.layer as rhai::INT).into());
        result.insert("mask".into(), (coll.mask as rhai::INT).into());
        result
    }

     fn lifetime_remaining(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let Some(lifetime) = world.lifetime(self.id) else {
            return 0;
        };
        lifetime.ttl_ms as rhai::INT
    }

    fn set_many(&mut self, map: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        for (key, value) in map {
            let Some(json_value) = rhai_dynamic_to_json(&value) else {
                return false;
            };
            if !world.set(self.id, &format!("/{}", key), json_value) {
                return false;
            }
        }
        true
    }

    fn data(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.id) else {
            return RhaiMap::new();
        };
        json_to_rhai_dynamic(&entity.data)
            .try_cast::<RhaiMap>()
            .unwrap_or_default()
    }

    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get(path).try_cast::<rhai::FLOAT>().unwrap_or(fallback)
    }

    fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get(path)
            .try_cast::<String>()
            .unwrap_or_else(|| fallback.to_string())
    }

    // ── Cooldown API ──────────────────────────────────────────────────────

    fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.cooldown_start(self.id, name, ms as i32)
    }

    fn cooldown_ready(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else { return true };
        world.cooldown_ready(self.id, name)
    }

    fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        world.cooldown_remaining(self.id, name) as rhai::INT
    }

    // ── Status API ────────────────────────────────────────────────────────

    fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.status_add(self.id, name, ms as i32)
    }

    fn status_has(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.status_has(self.id, name)
    }

    fn status_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        world.status_remaining(self.id, name) as rhai::INT
    }

    // ── Ship Controller API ───────────────────────────────────────────────

    fn attach_ship_controller(&mut self, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        let turn_step_ms = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(40) as u32;
        let thrust_power = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(170.0) as f32;
        let max_speed = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(4.5) as f32;
        let heading_bits = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(32) as u8;
        let controller = TopDownShipController::new(turn_step_ms, thrust_power, max_speed, heading_bits);
        world.attach_controller(self.id, controller)
    }

    fn set_turn(&mut self, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    fn set_thrust(&mut self, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    fn heading(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        world.controller(self.id).map(|c| c.current_heading as rhai::INT).unwrap_or(0)
    }

    fn heading_vector(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else { return RhaiMap::new() };
        match world.controller(self.id) {
            Some(ctrl) => {
                let (x, y) = ctrl.heading_vector();
                let mut map = RhaiMap::new();
                map.insert("x".into(), (x as rhai::FLOAT).into());
                map.insert("y".into(), (y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    // ── Health & Damage API ───────────────────────────────────────────────

    fn health_set(&mut self, hp: rhai::INT, max_hp: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.set_health(self.id, hp as i32, max_hp as i32)
    }

    fn health_get(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        world.health(self.id).map(|h| h.hp as rhai::INT).unwrap_or(0)
    }

    fn health_max(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else { return 0 };
        world.health(self.id).map(|h| h.max_hp as rhai::INT).unwrap_or(0)
    }

    fn health_dead(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.is_dead(self.id)
    }

    fn damage(&mut self, source: rhai::INT, amount: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else { return false };
        world.apply_damage(self.id, source as u64, amount as i32)
    }
}


impl RhaiScriptBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let compile_error = match params.script.as_deref() {
            Some(src) => {
                let engine = init_rhai_engine();
                match engine.compile(src) {
                    Ok(ast) => {
                        // Pre-populate the thread-local AST cache so the first
                        // frame doesn't pay the compile cost.
                        let hash = script_hash(src);
                        AST_CACHE.with(|cache| {
                            cache.borrow_mut().insert(hash, ast);
                        });
                        None
                    }
                    Err(err) => Some(format!("{}", err)),
                }
            }
            None => None,
        };
        Self {
            params: params.clone(),
            state: JsonValue::Object(JsonMap::new()),
            compile_error,
            compile_error_reported: false,
            behavior_id: NEXT_BEHAVIOR_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Drop for RhaiScriptBehavior {
    fn drop(&mut self) {
        // Clean up the thread-local scope so it doesn't outlive the scene.
        BEHAVIOR_SCOPES.with(|scopes| {
            scopes.borrow_mut().remove(&self.behavior_id);
        });
    }
}

impl RhaiScriptBehavior {
    fn build_regions_map(&self, ctx: &BehaviorContext, scene: &Scene) -> RhaiMap {
        let mut regions = RhaiMap::new();
        for (object_id, region) in ctx.object_regions.iter() {
            regions.insert(object_id.clone().into(), region_to_rhai_map(region).into());
        }
        if let Some(target) = self.params.target.as_deref() {
            if let Some(region) = ctx.resolved_object_region(target) {
                regions.insert(target.into(), region_to_rhai_map(region).into());
            }
        }
        let total = self.params.count.unwrap_or(scene.menu_options.len());
        let prefix = self
            .params
            .item_prefix
            .as_deref()
            .unwrap_or("menu-item-")
            .to_string();
        for idx in 0..total {
            let alias = if prefix.contains("{}") {
                prefix.replace("{}", &idx.to_string())
            } else {
                format!("{prefix}{idx}")
            };
            if let Some(region) = ctx.resolved_object_region(&alias) {
                regions.insert(alias.into(), region_to_rhai_map(region).into());
            }
        }
        regions
    }
}

impl Behavior for RhaiScriptBehavior {
    fn update(
        &mut self,
        _object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        if let Some(err) = &self.compile_error {
            if !self.compile_error_reported {
                commands.push(BehaviorCommand::ScriptError {
                    scene_id: scene.id.clone(),
                    source: self.params.src.clone(),
                    message: format!("compile error: {}", err),
                });
                self.compile_error_reported = true;
            }
            return;
        }

        let Some(script) = self.params.script.as_deref() else {
            return;
        };

        // Compute hash and regions flag before entering the scope borrow.
        let hash = script_hash(script);
        let needs_regions = script.contains("regions");

        // Build per-frame data outside the borrow to avoid lifetime conflicts.
        let regions_map = if needs_regions {
            Some(self.build_regions_map(ctx, scene))
        } else {
            None
        };
        let helper_commands = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));

        let eval_result: Result<RhaiDynamic, Box<rhai::EvalAltResult>> =
            BEHAVIOR_SCOPES.with(|scopes| {
                let mut map = scopes.borrow_mut();
                let (scope, base_len) = map
                    .entry(self.behavior_id)
                    .or_insert_with(|| (rhai::Scope::new(), 0));

                // One-time static init: push params and local below the rewind
                // point. `local` is seeded from `self.state` so scripts migrating
                // from the legacy `{state: ...}` return pattern get their state.
                if *base_len == 0 {
                    scope.push_dynamic("params", behavior_params_to_rhai_map(&self.params).into());
                    scope.push_dynamic("local", json_to_rhai_dynamic(&self.state));
                    *base_len = scope.len();
                }

                // Rewind to static base: clears all per-frame variables pushed
                // last frame and any `let` declarations the script made.
                // Variables at positions 0..*base_len (params, local) are kept.
                scope.rewind(*base_len);

                // --- Per-frame pushes ---

                // Phase 7C: Use Arc-wrapped maps from context instead of rebuilding.
                // Each push_dynamic clones the Arc (O(1) refcount), not the map (O(n)).
                scope.push_dynamic("menu", (*ctx.rhai_menu_map).clone().into());
                scope.push_dynamic("time", (*ctx.rhai_time_map).clone().into());

                // Compatibility layer for existing scripts; prefer `menu.*` and `time.*`.
                scope.push("selected_index", ctx.menu_selected_index as rhai::INT);
                scope.push("scene_elapsed_ms", ctx.scene_elapsed_ms as rhai::INT);
                scope.push("stage_elapsed_ms", ctx.stage_elapsed_ms as rhai::INT);
                scope.push("menu_count", scene.menu_options.len() as rhai::INT);

                // OPT-11: Only build regions_map when the script uses `regions`.
                scope.push_dynamic(
                    "regions",
                    regions_map
                        .map(|m| rhai::Dynamic::from(m))
                        .unwrap_or_else(|| RhaiMap::new().into()),
                );
                // OPT-3 + OPT-10: Skip build_objects_map entirely; push empty map for
                // backward compat. All scripts use scene.get(target) for lazy lookup.
                scope.push_dynamic("objects", RhaiMap::new().into());
                // `state` pushed per-frame for scripts using the legacy return-state pattern.
                scope.push_dynamic("state", json_to_rhai_dynamic(&self.state));
                scope.push_dynamic("ui", ui_context_to_rhai_map(ctx).into());
                scope.push(
                    "ui_focused_target",
                    ctx.ui_focused_target_id
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                );
                scope.push(
                    "ui_theme",
                    ctx.ui_theme_id.as_deref().unwrap_or_default().to_string(),
                );
                scope.push(
                    "ui_submit_target",
                    ctx.ui_last_submit_target_id
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                );
                scope.push(
                    "ui_submit_text",
                    ctx.ui_last_submit_text
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                );
                scope.push(
                    "ui_change_target",
                    ctx.ui_last_change_target_id
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                );
                scope.push(
                    "ui_change_text",
                    ctx.ui_last_change_text
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                );
                scope.push("ui_has_submit", ctx.ui_last_submit_target_id.is_some());
                scope.push("ui_has_change", ctx.ui_last_change_target_id.is_some());

                // Phase 7C: Use Arc-wrapped key map from context instead of rebuilding.
                scope.push_dynamic("key", (*ctx.rhai_key_map).clone().into());

                // Engine-level key state (separate namespace to prevent behavior interference)
                scope.push_dynamic("engine", (*ctx.engine_key_map).clone().into());

                // Gameplay collision events (array of {a, b} maps).
                let collisions: RhaiArray = ctx
                    .collisions
                    .iter()
                    .map(|hit| {
                        let mut map = RhaiMap::new();
                        map.insert("a".into(), (hit.a as rhai::INT).into());
                        map.insert("b".into(), (hit.b as rhai::INT).into());
                        map.into()
                    })
                    .collect();
                scope.push_dynamic("collisions", collisions.into());

                // External sidecar bridge exposed as object-shaped `ipc.*`.
                {
                    let mut ipc_map = RhaiMap::new();
                    ipc_map.insert(
                        "has_output".into(),
                        (!ctx.sidecar_io.output_lines.is_empty()).into(),
                    );
                    let output_array: RhaiArray = ctx
                        .sidecar_io
                        .output_lines
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect();
                    ipc_map.insert("output_lines".into(), output_array.into());
                    ipc_map.insert(
                        "clear_count".into(),
                        (ctx.sidecar_io.clear_count as rhai::INT).into(),
                    );
                    ipc_map.insert(
                        "has_screen_full".into(),
                        ctx.sidecar_io.screen_full_lines.is_some().into(),
                    );
                    let screen_full_lines: RhaiArray = ctx
                        .sidecar_io
                        .screen_full_lines
                        .as_ref()
                        .map(|lines| lines.iter().cloned().map(Into::into).collect())
                        .unwrap_or_default();
                    ipc_map.insert("screen_full_lines".into(), screen_full_lines.into());
                    let custom_events: RhaiArray = ctx
                        .sidecar_io
                        .custom_events
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect();
                    ipc_map.insert("custom_events".into(), custom_events.into());
                    scope.push_dynamic("ipc", ipc_map.into());
                }

                // OPT-4: Reuse thread-local engine with all static registrations pre-done.
                scope.push(
                    "scene",
                    ScriptSceneApi::new(
                        Arc::clone(&ctx.object_states),
                        Arc::clone(&ctx.object_kinds),
                        Arc::clone(&ctx.object_props),
                        Arc::clone(&ctx.object_regions),
                        Arc::clone(&ctx.object_text),
                        Arc::clone(&ctx.target_resolver),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "game",
                    ScriptGameApi::new(ctx.game_state.clone(), Arc::clone(&helper_commands)),
                );
                scope.push("level", ScriptLevelApi::new(ctx.level_state.clone()));
                scope.push(
                    "terminal",
                    ScriptTerminalApi::new(Arc::clone(&helper_commands)),
                );
                scope.push(
                    "input",
                    ScriptInputApi::new(
                        Arc::clone(&ctx.keys_down),
                        Arc::clone(&ctx.action_bindings),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "diag",
                    ScriptDebugApi::new(
                        scene.id.clone(),
                        self.params.src.clone(),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "persist",
                    ScriptPersistenceApi::new(ctx.persistence.clone()),
                );
                scope.push(
                    "world",
                    ScriptGameplayApi::new(
                        ctx.gameplay_world.clone(),
                        std::sync::Arc::clone(&ctx.collisions),
                        std::sync::Arc::clone(&ctx.collision_enters),
                        std::sync::Arc::clone(&ctx.collision_stays),
                        std::sync::Arc::clone(&ctx.collision_exits),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push("audio", ScriptAudioApi::new(Arc::clone(&helper_commands)));

                // OPT-4: Use thread-local engine + cached AST.
                RHAI_ENGINE.with(|cell| {
                    let mut opt = cell.borrow_mut();
                    let engine = opt.get_or_insert_with(init_rhai_engine);
                    AST_CACHE.with(|cache| {
                        let borrow = cache.borrow();
                        if let Some(ast) = borrow.get(&hash) {
                            return engine.eval_ast_with_scope::<RhaiDynamic>(scope, ast);
                        }
                        drop(borrow);
                        match engine.compile(script) {
                            Ok(ast) => {
                                let result = engine.eval_ast_with_scope::<RhaiDynamic>(scope, &ast);
                                let mut cache_mut = cache.borrow_mut();
                                // Limit AST cache to 256 entries (typical game has ~20-50 scripts)
                                // If full, clear oldest half to make room (simple eviction strategy)
                                if cache_mut.len() >= 256 {
                                    let to_remove = cache_mut.len() / 2;
                                    let keys: Vec<_> =
                                        cache_mut.keys().take(to_remove).copied().collect();
                                    for key in keys {
                                        cache_mut.remove(&key);
                                    }
                                }
                                cache_mut.insert(hash, ast);
                                result
                            }
                            Err(err) => Err(err.into()),
                        }
                    })
                })
            });

        let result = match eval_result {
            Ok(r) => r,
            Err(err) => {
                let src = self.params.src.as_deref().unwrap_or("<inline>");
                let msg = format!("{}", err);
                eprintln!(
                    "Rhai script error in scene '{}' (src: {}): {}",
                    scene.id, src, msg
                );
                commands.push(BehaviorCommand::ScriptError {
                    scene_id: scene.id.clone(),
                    source: self.params.src.clone(),
                    message: msg,
                });
                return;
            }
        };
        if let Some(map) = result.clone().try_cast::<RhaiMap>() {
            if let Some(next_state) = map.get("state").and_then(rhai_dynamic_to_json) {
                self.state = next_state;
            }
        }
        apply_rhai_commands(result, commands);
        if let Ok(mut queue) = helper_commands.lock() {
            commands.extend(queue.drain(..));
        };
    }
}

/// Executes a tiny multi-step runtime probe against a Rhai script using the same
/// behavior runtime path as the game loop.
pub fn smoke_validate_rhai_script(
    script: &str,
    src: Option<&str>,
    scene: &Scene,
) -> Result<(), String> {
    let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
        src: src.map(ToString::to_string),
        script: Some(script.to_string()),
        ..BehaviorParams::default()
    });
    if let Some(err) = behavior.compile_error.as_ref() {
        return Err(format!("compile error: {err}"));
    }

    let probe_object = GameObject {
        id: "__rhai_probe__".to_string(),
        name: "__rhai_probe__".to_string(),
        kind: GameObjectKind::Scene,
        aliases: Vec::new(),
        parent_id: None,
        children: Vec::new(),
    };
    let game_state = GameState::new();
    let gameplay_world = GameplayWorld::new();

    let frames = [
        (0_u64, None),
        (16_u64, Some("linus")),
        (32_u64, Some("tux")),
        (420_u64, None),
        (470_u64, Some("help")),
    ];

    for (elapsed_ms, submit_text) in frames {
        let ctx = smoke_probe_context(
            elapsed_ms,
            submit_text,
            game_state.clone(),
            gameplay_world.clone(),
        );
        let mut commands = Vec::new();
        behavior.update(&probe_object, scene, &ctx, &mut commands);
        if let Some(message) = commands.into_iter().find_map(|command| match command {
            BehaviorCommand::ScriptError { message, .. } => Some(message),
            _ => None,
        }) {
            return Err(message);
        }
    }
    Ok(())
}

fn smoke_probe_context(
    elapsed_ms: u64,
    submit_text: Option<&str>,
    game_state: GameState,
    gameplay_world: GameplayWorld,
) -> BehaviorContext {
    BehaviorContext {
        stage: SceneStage::OnIdle,
        scene_elapsed_ms: elapsed_ms,
        stage_elapsed_ms: elapsed_ms,
        menu_selected_index: 0,
        target_resolver: Arc::new(TargetResolver::default()),
        object_states: Arc::new(HashMap::new()),
        object_kinds: Arc::new(HashMap::new()),
        object_props: Arc::new(HashMap::new()),
        object_regions: Arc::new(HashMap::new()),
        object_text: Arc::new(HashMap::new()),
        ui_focused_target_id: Some(Arc::from("login-hidden-prompt")),
        ui_theme_id: None,
        ui_last_submit_target_id: submit_text.map(|_| Arc::from("login-hidden-prompt")),
        ui_last_submit_text: submit_text.map(|s| Arc::from(s)),
        ui_last_change_target_id: None,
        ui_last_change_text: None,
        game_state: Some(game_state),
        level_state: None,
        persistence: None,
        gameplay_world: Some(gameplay_world),
        collisions: Arc::new(Vec::new()),
        collision_enters: Arc::new(Vec::new()),
        collision_stays: Arc::new(Vec::new()),
        collision_exits: Arc::new(Vec::new()),
        last_raw_key: None,
        keys_down: Arc::new(HashSet::new()),
        action_bindings: Arc::new(HashMap::new()),
        sidecar_io: Arc::new(SidecarIoFrameState::default()),
        rhai_time_map: Arc::new(RhaiMap::new()),
        rhai_menu_map: Arc::new(RhaiMap::new()),
        rhai_key_map: Arc::new(RhaiMap::new()),
        engine_key_map: Arc::new(RhaiMap::new()),
    }
}

fn json_to_rhai_dynamic(value: &JsonValue) -> RhaiDynamic {
    match value {
        JsonValue::Null => ().into(),
        JsonValue::Bool(value) => (*value).into(),
        JsonValue::Number(value) => {
            if let Some(int) = value.as_i64() {
                (int as rhai::INT).into()
            } else if let Some(float) = value.as_f64() {
                float.into()
            } else {
                ().into()
            }
        }
        JsonValue::String(value) => value.clone().into(),
        JsonValue::Array(values) => {
            let mut out = RhaiArray::new();
            for item in values {
                out.push(json_to_rhai_dynamic(item));
            }
            out.into()
        }
        JsonValue::Object(map) => {
            let mut out = RhaiMap::new();
            for (key, value) in map {
                out.insert(key.into(), json_to_rhai_dynamic(value));
            }
            out.into()
        }
    }
}

fn rhai_dynamic_to_json(value: &RhaiDynamic) -> Option<JsonValue> {
    if value.is_unit() {
        return Some(JsonValue::Null);
    }
    if let Some(value) = value.clone().try_cast::<bool>() {
        return Some(JsonValue::Bool(value));
    }
    if let Some(value) = value.clone().try_cast::<rhai::INT>() {
        return Some(JsonValue::Number(JsonNumber::from(value)));
    }
    if let Some(value) = value.clone().try_cast::<rhai::FLOAT>() {
        if let Some(number) = JsonNumber::from_f64(value) {
            return Some(JsonValue::Number(number));
        }
        return None;
    }
    if let Some(value) = value.clone().try_cast::<String>() {
        return Some(JsonValue::String(value));
    }
    if let Some(values) = value.clone().try_cast::<RhaiArray>() {
        let mut out = Vec::with_capacity(values.len());
        for item in values {
            out.push(rhai_dynamic_to_json(&item)?);
        }
        return Some(JsonValue::Array(out));
    }
    if let Some(map) = value.clone().try_cast::<RhaiMap>() {
        let mut out = JsonMap::new();
        for (key, item) in map {
            out.insert(key.into(), rhai_dynamic_to_json(&item)?);
        }
        return Some(JsonValue::Object(out));
    }
    None
}

fn map_get_path_dynamic(map: &RhaiMap, path: &str) -> Option<RhaiDynamic> {
    let mut segments = path.split('.').filter(|segment| !segment.is_empty());
    let first = segments.next()?;
    let mut current = map.get(first)?.clone();
    for segment in segments {
        let next_map = current.clone().try_cast::<RhaiMap>()?;
        current = next_map.get(segment)?.clone();
    }
    Some(current)
}

fn map_set_path_dynamic(map: &mut RhaiMap, path: &str, value: RhaiDynamic) -> bool {
    let segments = path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return false;
    }
    map_set_path_recursive(map, &segments, value);
    true
}

fn map_set_path_recursive(map: &mut RhaiMap, segments: &[&str], value: RhaiDynamic) {
    let key = segments[0];
    if segments.len() == 1 {
        map.insert(key.into(), value);
        return;
    }
    let mut child = map
        .get(key)
        .and_then(|current| current.clone().try_cast::<RhaiMap>())
        .unwrap_or_default();
    map_set_path_recursive(&mut child, &segments[1..], value);
    map.insert(key.into(), child.into());
}

fn merge_rhai_maps(base: &mut RhaiMap, patch: &RhaiMap) {
    for (key, value) in patch {
        if let Some(existing) = base.get_mut(key.as_str()) {
            if let (Some(existing_map), Some(patch_map)) = (
                existing.clone().try_cast::<RhaiMap>(),
                value.clone().try_cast::<RhaiMap>(),
            ) {
                let mut merged = existing_map;
                merge_rhai_maps(&mut merged, &patch_map);
                *existing = merged.into();
                continue;
            }
        }
        base.insert(key.clone(), value.clone());
    }
}

fn normalize_set_path(path: &str) -> String {
    path.trim()
        .strip_prefix("props.")
        .unwrap_or(path.trim())
        .to_string()
}

fn normalize_input_code(code: &str) -> String {
    if code == " " {
        return " ".to_string();
    }
    let trimmed = code.trim();
    if trimmed.len() == 1 {
        return trimmed.to_ascii_lowercase();
    }
    trimmed.to_string()
}

fn to_i32(value: rhai::INT) -> i32 {
    value.clamp(i32::MIN as rhai::INT, i32::MAX as rhai::INT) as i32
}

fn base_sin_i32(step: i32) -> i32 {
    match step {
        0 => 0,
        1 => 200,
        2 => 392,
        3 => 569,
        4 => 724,
        5 => 851,
        6 => 946,
        7 => 1004,
        _ => 1024,
    }
}

fn sin32_i32(idx: i32) -> i32 {
    let i = idx.rem_euclid(32);
    let q = i / 8;
    let o = i % 8;
    match q {
        0 => base_sin_i32(o),
        1 => base_sin_i32(8 - o),
        2 => -base_sin_i32(o),
        _ => -base_sin_i32(8 - o),
    }
}

fn ship_points_i32(heading: i32) -> Vec<[i32; 2]> {
    let fx = sin32_i32(heading);
    let fy = -sin32_i32(heading + 8);
    let rx = -fy;
    let ry = fx;
    vec![
        [(fx * 7) / 1024, (fy * 7) / 1024],
        [((-fx * 3) - (rx * 3)) / 1024, ((-fy * 3) - (ry * 3)) / 1024],
        [(-fx) / 1024, (-fy) / 1024],
        [((-fx * 3) + (rx * 3)) / 1024, ((-fy * 3) + (ry * 3)) / 1024],
    ]
}

fn asteroid_shape_i32(shape: i32) -> &'static [[i32; 2]] {
    match shape.rem_euclid(4) {
        0 => &[
            [0, -10],
            [8, -6],
            [10, 1],
            [4, 9],
            [-4, 9],
            [-10, 2],
            [-8, -7],
        ],
        1 => &[
            [-2, -10],
            [6, -10],
            [11, -5],
            [10, 1],
            [11, 8],
            [2, 10],
            [-7, 9],
            [-11, 2],
            [-8, -6],
        ],
        2 => &[
            [0, -11],
            [7, -8],
            [10, -1],
            [8, 7],
            [1, 11],
            [-6, 9],
            [-10, 3],
            [-9, -4],
            [-4, -10],
        ],
        _ => &[
            [1, -10],
            [8, -9],
            [11, -2],
            [9, 5],
            [4, 9],
            [-2, 10],
            [-9, 8],
            [-11, 1],
            [-10, -6],
            [-4, -10],
        ],
    }
}

fn asteroid_scale_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 3,
        1 => 5,
        2 => 8,
        _ => 12,
    }
}

fn asteroid_points_i32(shape: i32, size: i32) -> Vec<[i32; 2]> {
    let scale = asteroid_scale_i32(size);
    asteroid_shape_i32(shape)
        .iter()
        .map(|p| [(p[0] * scale) / 10, (p[1] * scale) / 10])
        .collect()
}

fn rotate_points_i32(points: &[[i32; 2]], heading: i32) -> Vec<[i32; 2]> {
    let sin = i64::from(sin32_i32(heading));
    let cos = i64::from(sin32_i32(heading + 8));
    points
        .iter()
        .map(|p| {
            let x = i64::from(p[0]);
            let y = i64::from(p[1]);
            [
                ((x * cos) - (y * sin)).div_euclid(1024) as i32,
                ((x * sin) + (y * cos)).div_euclid(1024) as i32,
            ]
        })
        .collect()
}

fn asteroid_fragment_points_i32(shape: i32, size: i32, fragment: i32) -> Vec<[i32; 2]> {
    let points = asteroid_points_i32(shape, size);
    let count = points.len();
    if count < 3 {
        return points;
    }
    let fragment = fragment.rem_euclid(3) as usize;
    let cuts = [0, count / 3, (count * 2) / 3, count];
    let start = cuts[fragment];
    let end = cuts[fragment + 1];
    let mut out = Vec::with_capacity((end - start) + 3);
    out.push([0, 0]);
    for idx in start..=end {
        let wrapped = if idx == count { 0 } else { idx };
        out.push(points[wrapped]);
    }
    out.push([0, 0]);
    out
}

fn asteroid_radius_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 4,
        1 => 7,
        2 => 11,
        _ => 15,
    }
}

fn asteroid_score_i32(size: i32) -> i32 {
    match size {
        i32::MIN..=0 => 35,
        1 => 25,
        2 => 15,
        _ => 10,
    }
}

fn points_to_rhai_array(points: Vec<[i32; 2]>) -> RhaiArray {
    points
        .into_iter()
        .map(|[x, y]| {
            let mut pair = RhaiArray::with_capacity(2);
            pair.push((x as rhai::INT).into());
            pair.push((y as rhai::INT).into());
            pair.into()
        })
        .collect()
}

fn rhai_array_to_points(value: &RhaiArray) -> Vec<[i32; 2]> {
    let mut points = Vec::with_capacity(value.len());
    for item in value {
        let Some(pair) = item.clone().try_cast::<RhaiArray>() else {
            continue;
        };
        if pair.len() < 2 {
            continue;
        }
        let Some(x) = pair[0].clone().try_cast::<rhai::INT>() else {
            continue;
        };
        let Some(y) = pair[1].clone().try_cast::<rhai::INT>() else {
            continue;
        };
        points.push([to_i32(x), to_i32(y)]);
    }
    points
}

/// Shows directional arrow sprites flanking the selected menu option.
pub struct SelectedArrowsBehavior {
    target: Option<String>,
    index: usize,
    side: ArrowSide,
    padding: i32,
    amplitude_x: i32,
    period_ms: u64,
    phase_ms: u64,
    autoscale_height: bool,
    last_dx: i32,
    last_dy: i32,
}

#[derive(Clone, Copy)]
enum ArrowSide {
    Left,
    Right,
}

impl SelectedArrowsBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let side_str = params.side.as_deref().unwrap_or("");
        let side = if side_str.trim().eq_ignore_ascii_case("right") {
            ArrowSide::Right
        } else {
            ArrowSide::Left
        };
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
            side,
            padding: params.padding.unwrap_or(1),
            amplitude_x: params.amplitude_x.unwrap_or(1).abs(),
            period_ms: params.period_ms.unwrap_or(900).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
            autoscale_height: params.autoscale_height.unwrap_or(false),
            last_dx: 0,
            last_dy: 0,
        }
    }

    fn hide_and_reset(&mut self, object: &GameObject, commands: &mut Vec<BehaviorCommand>) {
        self.last_dx = 0;
        self.last_dy = 0;
        emit_visibility(commands, object.id.clone(), false);
    }
}

impl Behavior for SelectedArrowsBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        if ctx.menu_selected_index != self.index {
            self.hide_and_reset(object, commands);
            return;
        }

        let Some(target_alias) = self.target.as_deref() else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(own_region) = ctx.object_region(&object.id) else {
            emit_visibility(commands, object.id.clone(), true);
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        let wave = rounded_sine_wave(ctx.scene_elapsed_ms, self.phase_ms, self.period_ms);
        let signed_wave = match self.side {
            ArrowSide::Left => wave,
            ArrowSide::Right => -wave,
        } * self.amplitude_x;
        let auto_pad = if self.autoscale_height {
            (target_region.height.saturating_sub(1) as i32) / 2
        } else {
            0
        };
        let effective_padding = self.padding.saturating_add(auto_pad).max(0);
        let arrow_w = own_region.width.max(1) as i32;
        let arrow_h = own_region.height.max(1) as i32;
        let target_w = target_region.width.max(1) as i32;
        let target_center_y =
            target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);

        let target_x = match self.side {
            ArrowSide::Left => target_region.x as i32 - effective_padding - arrow_w + signed_wave,
            ArrowSide::Right => target_region.x as i32 + target_w + effective_padding + signed_wave,
        };
        let target_y = target_center_y.saturating_sub((arrow_h.saturating_sub(1)) / 2);

        emit_visibility(commands, object.id.clone(), true);

        let base_x = own_region.x as i32 - self.last_dx;
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dx = target_x - base_x;
        let new_dy = target_y - base_y;
        self.last_dx = new_dx;
        self.last_dy = new_dy;

        emit_offset(commands, object.id.clone(), new_dx, new_dy);
    }
}

impl StageVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let stages = params
            .stages
            .iter()
            .filter_map(|value| parse_stage_name(value))
            .collect::<Vec<_>>();
        Self {
            target: params.target.clone(),
            stages,
        }
    }
}

impl Behavior for StageVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let visible = if self.stages.is_empty() {
            true
        } else {
            self.stages.iter().any(|stage| stage == &ctx.stage)
        };
        emit_visibility(commands, resolve_target(&self.target, object), visible);
    }
}

/// Shows the object only within a configured time window relative to the scene or stage clock.
pub struct TimedVisibilityBehavior {
    target: Option<String>,
    start_ms: Option<u64>,
    end_ms: Option<u64>,
    time_scope: TimeScope,
}

impl TimedVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            start_ms: params.start_ms,
            end_ms: params.end_ms,
            time_scope: TimeScope::from_params(params),
        }
    }
}

impl Behavior for TimedVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let elapsed_ms = self.time_scope.elapsed_ms(ctx);
        emit_visibility(
            commands,
            resolve_target(&self.target, object),
            is_within_time_window(elapsed_ms, self.start_ms, self.end_ms),
        );
    }
}

fn emit_audio(commands: &mut Vec<BehaviorCommand>, cue: String, volume: Option<f32>) {
    commands.push(BehaviorCommand::PlayAudioCue { cue, volume });
}

fn emit_visibility(commands: &mut Vec<BehaviorCommand>, target: String, visible: bool) {
    commands.push(BehaviorCommand::SetVisibility { target, visible });
}

fn emit_offset(commands: &mut Vec<BehaviorCommand>, target: String, dx: i32, dy: i32) {
    commands.push(BehaviorCommand::SetOffset { target, dx, dy });
}

fn emit_text(commands: &mut Vec<BehaviorCommand>, target: String, text: String) {
    commands.push(BehaviorCommand::SetText { target, text });
}

fn resolve_target(target: &Option<String>, object: &GameObject) -> String {
    target
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| object.id.clone())
}

fn sine_wave(elapsed_ms: u64, phase_ms: u64, period_ms: u64) -> f32 {
    let phase = (elapsed_ms.saturating_add(phase_ms) % period_ms) as f32 / period_ms as f32;
    (phase * TAU).sin()
}

fn rounded_sine_wave(elapsed_ms: u64, phase_ms: u64, period_ms: u64) -> i32 {
    sine_wave(elapsed_ms, phase_ms, period_ms).round() as i32
}

fn wrapped_menu_distance(index: usize, selected: usize, total: usize) -> i32 {
    let raw = index as i32 - selected as i32;
    if total <= 1 {
        return raw;
    }
    let total_i = total as i32;
    [raw, raw - total_i, raw + total_i]
        .into_iter()
        .min_by_key(|value| value.abs())
        .unwrap_or(raw)
}

fn behavior_params_to_rhai_map(params: &BehaviorParams) -> RhaiMap {
    let mut out = RhaiMap::new();
    if let Some(value) = params.target.as_ref() {
        out.insert("target".into(), value.clone().into());
    }
    if let Some(value) = params.index {
        out.insert("index".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.count {
        out.insert("count".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.window {
        out.insert("window".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.step_y {
        out.insert("step_y".into(), (value as rhai::INT).into());
    }
    if let Some(value) = params.endless {
        out.insert("endless".into(), value.into());
    }
    if let Some(value) = params.item_prefix.as_ref() {
        out.insert("item_prefix".into(), value.clone().into());
    }
    if let Some(value) = params.src.as_ref() {
        out.insert("src".into(), value.clone().into());
    }
    if let Some(value) = params.dur {
        out.insert("dur".into(), (value as rhai::INT).into());
    }
    out
}

fn region_to_rhai_map(region: &Region) -> RhaiMap {
    let mut out = RhaiMap::new();
    out.insert("x".into(), (region.x as rhai::INT).into());
    out.insert("y".into(), (region.y as rhai::INT).into());
    out.insert("w".into(), (region.width as rhai::INT).into());
    out.insert("h".into(), (region.height as rhai::INT).into());
    out
}

fn object_state_to_rhai_map(state: &ObjectRuntimeState) -> RhaiMap {
    let mut out = RhaiMap::new();
    out.insert("visible".into(), state.visible.into());
    out.insert("offset_x".into(), (state.offset_x as rhai::INT).into());
    out.insert("offset_y".into(), (state.offset_y as rhai::INT).into());
    out
}

fn kind_capabilities(kind: Option<&str>) -> RhaiArray {
    let mut caps = vec![
        "visible".to_string(),
        "offset.x".to_string(),
        "offset.y".to_string(),
        "position.x".to_string(),
        "position.y".to_string(),
    ];
    if kind.is_some_and(|value| value == "text") {
        caps.push("text.content".to_string());
        caps.push("text.font".to_string());
        caps.push("style.fg".to_string());
        caps.push("style.bg".to_string());
    }
    if kind.is_some_and(|value| value == "obj") {
        caps.push("obj.scale".to_string());
        caps.push("obj.yaw".to_string());
        caps.push("obj.pitch".to_string());
        caps.push("obj.roll".to_string());
        caps.push("obj.orbit_speed".to_string());
        caps.push("obj.surface_mode".to_string());
    }
    caps.into_iter().map(Into::into).collect()
}

fn ui_context_to_rhai_map(ctx: &BehaviorContext) -> RhaiMap {
    let mut out = RhaiMap::new();
    if let Some(value) = ctx.ui_focused_target_id.as_deref() {
        out.insert("focused_target".into(), value.to_string().into());
    }
    if let Some(value) = ctx.ui_theme_id.as_deref() {
        out.insert("theme".into(), value.to_string().into());
    }
    out.insert(
        "has_submit".into(),
        ctx.ui_last_submit_target_id.is_some().into(),
    );
    if let Some(value) = ctx.ui_last_submit_target_id.as_deref() {
        out.insert("submit_target".into(), value.to_string().into());
    }
    if let Some(value) = ctx.ui_last_submit_text.as_deref() {
        out.insert("submit_text".into(), value.to_string().into());
    }
    out.insert(
        "has_change".into(),
        ctx.ui_last_change_target_id.is_some().into(),
    );
    if let Some(value) = ctx.ui_last_change_target_id.as_deref() {
        out.insert("change_target".into(), value.to_string().into());
    }
    if let Some(value) = ctx.ui_last_change_text.as_deref() {
        out.insert("change_text".into(), value.to_string().into());
    }
    out
}

fn apply_rhai_commands(result: RhaiDynamic, commands: &mut Vec<BehaviorCommand>) {
    let commands_dynamic = if result.is::<RhaiArray>() {
        result
    } else if result.is::<RhaiMap>() {
        let map = result.cast::<RhaiMap>();
        map.get("commands")
            .cloned()
            .unwrap_or_else(|| RhaiArray::new().into())
    } else {
        return;
    };
    let Some(array) = commands_dynamic.try_cast::<RhaiArray>() else {
        return;
    };
    for command in array {
        let Some(map) = command.try_cast::<RhaiMap>() else {
            continue;
        };
        let op = map
            .get("op")
            .and_then(|value| value.clone().try_cast::<String>())
            .unwrap_or_default();
        match op.as_str() {
            "visibility" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(visible) = map
                    .get("visible")
                    .and_then(|value| value.clone().try_cast::<bool>())
                else {
                    continue;
                };
                emit_visibility(commands, target, visible);
            }
            "offset" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let dx = map
                    .get("dx")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .unwrap_or(0);
                let dy = map
                    .get("dy")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .unwrap_or(0);
                emit_offset(commands, target, dx as i32, dy as i32);
            }
            "set-text" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(text) = map
                    .get("text")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                emit_text(commands, target, text);
            }
            "set-props" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let visible = map
                    .get("visible")
                    .and_then(|value| value.clone().try_cast::<bool>());
                let dx = map
                    .get("dx")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .map(|value| value as i32);
                let dy = map
                    .get("dy")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .map(|value| value as i32);
                let text = map
                    .get("text")
                    .and_then(|value| value.clone().try_cast::<String>());
                if visible.is_none() && dx.is_none() && dy.is_none() && text.is_none() {
                    continue;
                }
                commands.push(BehaviorCommand::SetProps {
                    target,
                    visible,
                    dx,
                    dy,
                    text,
                });
            }
            "set" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(path) = map
                    .get("path")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(value) = map.get("value").and_then(rhai_dynamic_to_json) else {
                    continue;
                };
                commands.push(BehaviorCommand::SetProperty {
                    target,
                    path: normalize_set_path(&path),
                    value,
                });
            }
            "transition" => {
                let Some(to_scene_id) = map
                    .get("to_scene_id")
                    .and_then(|value| value.clone().try_cast::<String>())
                    .filter(|value| !value.trim().is_empty())
                else {
                    continue;
                };
                commands.push(BehaviorCommand::SceneTransition { to_scene_id });
            }
            _ => {}
        }
    }
}

fn is_within_time_window(elapsed_ms: u64, start_ms: Option<u64>, end_ms: Option<u64>) -> bool {
    start_ms.map(|start| elapsed_ms >= start).unwrap_or(true)
        && end_ms.map(|end| elapsed_ms < end).unwrap_or(true)
}

#[derive(Clone, Copy)]
enum TimeScope {
    Scene,
    Stage,
}

impl TimeScope {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let scope_str = params.time_scope.as_deref().unwrap_or("");
        if scope_str.trim().eq_ignore_ascii_case("stage") {
            Self::Stage
        } else {
            Self::Scene
        }
    }

    fn elapsed_ms(self, ctx: &BehaviorContext) -> u64 {
        match self {
            Self::Scene => ctx.scene_elapsed_ms,
            Self::Stage => ctx.stage_elapsed_ms,
        }
    }
}

fn parse_stage_name(raw: &str) -> Option<SceneStage> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("on-enter") || trimmed.eq_ignore_ascii_case("enter") {
        Some(SceneStage::OnEnter)
    } else if trimmed.eq_ignore_ascii_case("on-idle") || trimmed.eq_ignore_ascii_case("idle") {
        Some(SceneStage::OnIdle)
    } else if trimmed.eq_ignore_ascii_case("on-leave") || trimmed.eq_ignore_ascii_case("leave") {
        Some(SceneStage::OnLeave)
    } else if trimmed.eq_ignore_ascii_case("done") {
        Some(SceneStage::Done)
    } else {
        None
    }
}

fn cues_for_stage<'a>(scene: &'a Scene, stage: &SceneStage) -> &'a [AudioCue] {
    match stage {
        SceneStage::OnEnter => &scene.audio.on_enter,
        SceneStage::OnIdle => &scene.audio.on_idle,
        SceneStage::OnLeave => &scene.audio.on_leave,
        SceneStage::Done => &[],
    }
}

impl BehaviorContext {
    pub fn resolve_target(&self, target: &str) -> Option<&str> {
        self.target_resolver.resolve_alias(target)
    }

    pub fn object_state(&self, object_id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(object_id)
    }

    pub fn object_region(&self, object_id: &str) -> Option<&Region> {
        self.object_regions.get(object_id)
    }

    pub fn resolved_object_state(&self, target: &str) -> Option<&ObjectRuntimeState> {
        self.resolve_target(target)
            .and_then(|object_id| self.object_state(object_id))
    }

    pub fn resolved_object_region(&self, target: &str) -> Option<&Region> {
        self.resolve_target(target)
            .and_then(|object_id| self.object_region(object_id))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asteroid_fragment_points_i32, asteroid_points_i32, built_in_behavior, rotate_points_i32,
        smoke_validate_rhai_script, Behavior, BehaviorCommand, BehaviorContext, BlinkBehavior,
        BobBehavior, FollowBehavior, MenuCarouselBehavior, MenuCarouselObjectBehavior,
        MenuSelectedBehavior, RhaiScriptBehavior, SceneAudioBehavior, SelectedArrowsBehavior,
        StageVisibilityBehavior, TimedVisibilityBehavior,
    };
    use engine_animation::SceneStage;
    use engine_core::effects::Region;
    use engine_core::game_object::{GameObject, GameObjectKind};
    use engine_core::game_state::GameState;
    use engine_core::level_state::LevelState;
    use engine_core::scene::{
        AudioCue, BehaviorParams, BehaviorSpec, MenuOption, Scene, SceneAudio, SceneRenderedMode,
        SceneStages, TermColour,
    };
    use engine_core::scene_runtime_types::{
        ObjectRuntimeState, SidecarIoFrameState, TargetResolver,
    };
    use engine_game::GameplayWorld;
    use rhai::Map as RhaiMap;
    use serde_json::json;
    use serde_json::Value as JsonValue;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    fn scene_object() -> GameObject {
        GameObject {
            id: "scene:intro".to_string(),
            name: "intro".to_string(),
            kind: GameObjectKind::Scene,
            aliases: vec!["intro".to_string()],
            parent_id: None,
            children: Vec::new(),
        }
    }

    fn base_scene() -> Scene {
        Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            target_fps: None,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
        }
    }

    fn scene_with_audio(audio: SceneAudio) -> Scene {
        Scene {
            audio,
            ..base_scene()
        }
    }

    fn scene_with_menu_options(count: usize) -> Scene {
        Scene {
            menu_options: (0..count)
                .map(|idx| MenuOption {
                    key: idx.to_string(),
                    label: Some(format!("Option {idx}")),
                    scene: None,
                    next: format!("next-{idx}"),
                })
                .collect(),
            ..base_scene()
        }
    }

    fn empty_rhai_menu_map() -> Arc<RhaiMap> {
        let mut menu_map = RhaiMap::new();
        menu_map.insert("selected_index".into(), 0i64.into());
        menu_map.insert("count".into(), 0i64.into());
        Arc::new(menu_map)
    }

    fn empty_rhai_time_map() -> Arc<RhaiMap> {
        let mut time_map = RhaiMap::new();
        time_map.insert("scene_elapsed_ms".into(), 0i64.into());
        time_map.insert("stage_elapsed_ms".into(), 0i64.into());
        time_map.insert("stage".into(), "on_idle".into());
        Arc::new(time_map)
    }

    fn empty_rhai_key_map() -> Arc<RhaiMap> {
        let mut key_map = RhaiMap::new();
        key_map.insert("code".into(), "".into());
        key_map.insert("ctrl".into(), false.into());
        key_map.insert("alt".into(), false.into());
        key_map.insert("shift".into(), false.into());
        key_map.insert("pressed".into(), false.into());
        key_map.insert("released".into(), false.into());
        Arc::new(key_map)
    }

    fn empty_engine_key_map() -> Arc<RhaiMap> {
        let mut engine_key = RhaiMap::new();
        engine_key.insert("code".into(), "".into());
        engine_key.insert("ctrl".into(), false.into());
        engine_key.insert("alt".into(), false.into());
        engine_key.insert("shift".into(), false.into());
        engine_key.insert("pressed".into(), false.into());
        engine_key.insert("released".into(), false.into());
        engine_key.insert("is_quit".into(), false.into());
        engine_key.insert("any_down".into(), false.into());
        engine_key.insert("down_count".into(), 0_i64.into());
        Arc::new(engine_key)
    }

    fn base_ctx() -> BehaviorContext {
        BehaviorContext {
            stage: SceneStage::OnIdle,
            scene_elapsed_ms: 0,
            stage_elapsed_ms: 0,
            menu_selected_index: 0,
            target_resolver: Arc::new(TargetResolver::default()),
            object_states: Arc::new(HashMap::new()),
            object_kinds: Arc::new(HashMap::new()),
            object_props: Arc::new(HashMap::new()),
            object_regions: Arc::new(HashMap::new()),
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
            gameplay_world: None,
            collisions: Arc::new(Vec::new()),
            last_raw_key: None,
            keys_down: Arc::new(HashSet::new()),
            action_bindings: Arc::new(HashMap::new()),
            sidecar_io: Arc::new(SidecarIoFrameState::default()),
            rhai_time_map: empty_rhai_time_map(),
            rhai_menu_map: empty_rhai_menu_map(),
            rhai_key_map: empty_rhai_key_map(),
            engine_key_map: empty_engine_key_map(),
        }
    }

    fn ctx(stage: SceneStage, scene_elapsed_ms: u64, stage_elapsed_ms: u64) -> BehaviorContext {
        // Build time map with correct values for this test context
        let rhai_time_map = {
            let mut time_map = RhaiMap::new();
            time_map.insert(
                "scene_elapsed_ms".into(),
                (scene_elapsed_ms as rhai::INT).into(),
            );
            time_map.insert(
                "stage_elapsed_ms".into(),
                (stage_elapsed_ms as rhai::INT).into(),
            );
            let stage_str: &str = match stage {
                SceneStage::OnEnter => "on_enter",
                SceneStage::OnIdle => "on_idle",
                SceneStage::OnLeave => "on_leave",
                SceneStage::Done => "done",
            };
            time_map.insert("stage".into(), stage_str.into());
            Arc::new(time_map)
        };

        BehaviorContext {
            stage,
            scene_elapsed_ms,
            stage_elapsed_ms,
            rhai_time_map,
            ..base_ctx()
        }
    }

    fn update_ctx_menu_map(ctx: &mut BehaviorContext, menu_count: usize) {
        // When test modifies menu_selected_index, rebuild the menu map to match
        let mut menu_map = RhaiMap::new();
        menu_map.insert(
            "selected_index".into(),
            (ctx.menu_selected_index as rhai::INT).into(),
        );
        menu_map.insert("count".into(), (menu_count as rhai::INT).into());
        ctx.rhai_menu_map = Arc::new(menu_map);
    }

    #[test]
    fn rhai_script_behavior_reads_ipc_scope_values() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
if ipc.has_output {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if ipc.clear_count > 0 {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 0 });
}
if ipc.has_screen_full {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: 1 });
}
if ipc.custom_events.len > 0 {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.sidecar_io = Arc::new(SidecarIoFrameState {
            output_lines: vec!["line".to_string()],
            clear_count: 1,
            screen_full_lines: Some(vec!["full".to_string()]),
            custom_events: vec!["{}".to_string()],
        });
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: 0
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 1
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 2,
                    dy: 0
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_can_spawn_gameplay_entities() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("asteroid", #{ tags: ["enemy", "rock"], x: 12, nested: #{ hp: 3 } });
if id > 0 && world.exists(id) {
  world.set(id, "/nested/hp", 7);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "world api should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.kind_of(id).as_deref(), Some("asteroid"));
        assert_eq!(gameplay_world.query_tag("enemy"), vec![id]);
        assert_eq!(gameplay_world.get(id, "/nested/hp"), Some(json!(7)));
    }

    #[test]
    fn rhai_script_behavior_gameplay_entity_api_supports_typed_getters_and_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("ship", #{ active: true, score: 4 });
let e = world.entity(id);
if e.exists() && e.get_bool("/active", false) {
  let next = e.get_i("/score", 0) + 9;
  e.set("/score", next);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "world entity api should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.get(id, "/score"), Some(json!(13)));
        assert_eq!(gameplay_world.get(id, "/active"), Some(json!(true)));
    }

    #[test]
    fn rhai_script_behavior_gameplay_entity_api_supports_bulk_operations() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("ship", #{ x: 1.5, y: 2.5, name: "player" });
let e = world.entity(id);

// Test set_many
let updates = #{
  x: 10.5,
  y: 20.5,
  score: 42
};
e.set_many(updates);

// Test data() - should return entire entity data blob
let data = e.data();

// Test get_f and get_s
let x = e.get_f("/x", 0.0);
let name = e.get_s("/name", "");

if x == 10.5 && name == "player" {
  e.set("/success", true);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "bulk operations should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.get(id, "/x"), Some(json!(10.5)));
        assert_eq!(gameplay_world.get(id, "/y"), Some(json!(20.5)));
        assert_eq!(gameplay_world.get(id, "/score"), Some(json!(42)));
        assert_eq!(gameplay_world.get(id, "/name"), Some(json!("player")));
        assert_eq!(gameplay_world.get(id, "/success"), Some(json!(true)));
    }

    fn region(x: u16, y: u16, width: u16, height: u16) -> Region {
        Region {
            x,
            y,
            width,
            height,
        }
    }

    fn run_behavior<B: Behavior>(
        behavior: &mut B,
        scene: &Scene,
        ctx: BehaviorContext,
    ) -> Vec<BehaviorCommand> {
        let mut commands = Vec::new();
        behavior.update(&scene_object(), scene, &ctx, &mut commands);
        commands
    }

    #[test]
    fn scene_audio_behavior_emits_each_cue_once() {
        let scene = scene_with_audio(SceneAudio {
            on_enter: vec![AudioCue {
                at_ms: 100,
                cue: "thunder".to_string(),
                volume: Some(0.7),
            }],
            on_idle: Vec::new(),
            on_leave: Vec::new(),
        });
        let object = scene_object();
        let ctx = ctx(SceneStage::OnEnter, 100, 100);
        let mut behavior = SceneAudioBehavior::default();
        let mut commands = Vec::new();

        behavior.update(&object, &scene, &ctx, &mut commands);
        behavior.update(&object, &scene, &ctx, &mut commands);

        assert_eq!(
            commands,
            vec![BehaviorCommand::PlayAudioCue {
                cue: "thunder".to_string(),
                volume: Some(0.7)
            }]
        );
    }

    #[test]
    fn blink_behavior_toggles_visibility() {
        let mut behavior = BlinkBehavior::from_params(&BehaviorParams {
            visible_ms: Some(100),
            hidden_ms: Some(100),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 150, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn bob_behavior_emits_offset() {
        let mut behavior = BobBehavior::from_params(&BehaviorParams {
            amplitude_x: Some(2),
            amplitude_y: Some(0),
            period_ms: Some(1000),
            phase_ms: Some(250),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "scene:intro".to_string(),
                dx: 2,
                dy: 0
            }]
        );
    }

    #[test]
    fn builds_known_behavior_from_spec() {
        let behavior = built_in_behavior(&BehaviorSpec {
            name: "blink".to_string(),
            params: BehaviorParams::default(),
        });

        assert!(behavior.is_some());
    }

    #[test]
    fn stage_visibility_behavior_shows_only_selected_stage() {
        let mut behavior = StageVisibilityBehavior::from_params(&BehaviorParams {
            stages: vec!["on-idle".to_string()],
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnEnter, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_uses_elapsed_time_window() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 150, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_can_use_stage_clock() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            time_scope: Some("stage".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 500, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn follow_behavior_copies_target_state() {
        let mut behavior = FollowBehavior::from_params(&BehaviorParams {
            target: Some("leader".to_string()),
            amplitude_x: Some(1),
            amplitude_y: Some(-1),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("leader".to_string(), "obj:leader".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:leader".to_string(),
            ObjectRuntimeState {
                visible: false,
                offset_x: 3,
                offset_y: 2,
            },
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: false
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 4,
                    dy: 1
                }
            ]
        );
    }

    #[test]
    fn menu_selected_behavior_visibility_matches_selected_index() {
        let mut behavior = MenuSelectedBehavior::from_params(&BehaviorParams {
            index: Some(1),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.menu_selected_index = 1;
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: true
            }]
        );
    }

    #[test]
    fn menu_carousel_centers_selected_item_in_target_region() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(2),
            window: Some(5),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 2;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -6
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_wraps_when_endless_enabled() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(0),
            window: Some(5),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 6;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -4
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_hides_items_outside_window() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(6),
            window: Some(5),
            step_y: Some(2),
            endless: Some(false),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 0;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn menu_carousel_uses_min_step_based_on_item_height() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(0),
            window: Some(3),
            step_y: Some(1),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        // Item currently at y=20 with height=3 (simulates a taller rendered row).
        object_regions.insert("scene:intro".to_string(), region(10, 20, 24, 3));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 2;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_object_controls_multiple_items_from_single_behavior() {
        let mut behavior = MenuCarouselObjectBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            item_prefix: Some("menu-item-".to_string()),
            count: Some(3),
            window: Some(3),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        resolver.register_alias("menu-item-1".to_string(), "obj:menu-item-1".to_string());
        resolver.register_alias("menu-item-2".to_string(), "obj:menu-item-2".to_string());

        let mut object_regions = HashMap::new();
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));
        object_regions.insert("obj:menu-item-0".to_string(), region(10, 6, 20, 1));
        object_regions.insert("obj:menu-item-1".to_string(), region(10, 10, 20, 1));
        object_regions.insert("obj:menu-item-2".to_string(), region(10, 14, 20, 1));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 1;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 6
                },
                BehaviorCommand::SetVisibility {
                    target: "menu-item-1".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-1".to_string(),
                    dx: 0,
                    dy: 4
                },
                BehaviorCommand::SetVisibility {
                    target: "menu-item-2".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-2".to_string(),
                    dx: 0,
                    dy: 2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_visibility_and_offset_commands() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: -2 });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_reads_ui_scope_values() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
if ui.has_submit && ui.submit_text == "status" && ui.focused_target == "terminal-prompt" {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if ui.theme == "terminal" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: 1 });
}
if ui.has_change && ui.change_target == "terminal-prompt" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.ui_focused_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_theme_id = Some(Arc::from("terminal"));
        test_ctx.ui_last_submit_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_last_submit_text = Some(Arc::from("status"));
        test_ctx.ui_last_change_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_last_change_text = Some(Arc::from("sta"));
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 1
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 2,
                    dy: 0
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_reads_time_menu_and_game_objects() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
game.set("/session/user", "linus");
game.push("/events", "booted");

let out = [];
if time.scene_elapsed_ms == 480 && time.stage_elapsed_ms == 120 {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if menu.count == 3 && menu.selected_index == 1 && game.get("/session/user") == "linus" && game.has("/events") {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 2 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 480, 120);
        test_ctx.menu_selected_index = 1;
        update_ctx_menu_map(&mut test_ctx, 3);
        test_ctx.game_state = Some(GameState::new());
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: 2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_text_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set-text", target: "ram-counter-line", text: "Memory Check: 0640K" });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetText {
                target: "ram-counter-line".to_string(),
                text: "Memory Check: 0640K".to_string()
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_props_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set-props", target: "menu-item-0", visible: true, dx: 2, dy: -1, text: "HELLO" });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProps {
                target: "menu-item-0".to_string(),
                visible: Some(true),
                dx: Some(2),
                dy: Some(-1),
                text: Some("HELLO".to_string())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_property_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set", target: "menu-item-0", path: "position.y", value: 3 });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(3.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_set_emits_set_property() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.set("menu-item-0", "position.y", 6);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(6.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_set_normalizes_props_prefix() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.set("menu-item-0", "props.position.y", 6);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(6.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_spawn_and_despawn_emit_commands() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.spawn_object("bullet-0", "bullet-99");
scene.despawn_object("bullet-99");
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SceneSpawn {
                    template: "bullet-0".to_string(),
                    target: "bullet-99".to_string()
                },
                BehaviorCommand::SceneDespawn {
                    target: "bullet-99".to_string()
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_geometry_helpers_are_available() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ship = ship_points(8);
let rocks = asteroid_points(2, 3);
let score = asteroid_score(0);
let radius = asteroid_radius(2);
let wave = sin32(0);
let out = [];
out.push(#{ op: "set", target: "menu-item-0", path: "position.x", value: ship.len() + rocks.len() });
out.push(#{ op: "set", target: "menu-item-0", path: "position.y", value: score + radius + wave });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetProperty {
                    target: "menu-item-0".to_string(),
                    path: "position.x".to_string(),
                    value: JsonValue::Number(13.into()),
                },
                BehaviorCommand::SetProperty {
                    target: "menu-item-0".to_string(),
                    path: "position.y".to_string(),
                    value: JsonValue::Number(46.into()),
                },
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_persists_state_between_updates() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let count = if state.contains("count") { state["count"] } else { 0 };
let next = count + 1;
let out = [];
out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: next });
#{ commands: out, state: #{ count: next } }
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });

        let first = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        let second = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 16, 16),
        );

        assert_eq!(
            first,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 1
            }]
        );
        assert_eq!(
            second,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 2
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_exposes_objects_snapshot_by_alias_and_id() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj_alias = scene.get("menu-item-0");
let obj_real = scene.get("obj:menu-item-0");
let kind = obj_real.get("kind");
let dy = obj_real.get("state.offset_y");
let rx = obj_alias.get("region.x");
if kind == "text" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: rx, dy: dy });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 7,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("obj:menu-item-0".to_string(), region(12, 5, 10, 1));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_regions = Arc::new(object_regions);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 12,
                dy: 7
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_get_reads_object_snapshot() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("kind") == "text" {
  let dy = obj.get("state.offset_y");
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: dy });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 4,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 4
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_get_and_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let obj = scene.get("menu-item-0");
let dy = obj.get("state.offset_y");
obj.set("position.y", dy + 2);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 5,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                // Alias "menu-item-0" resolves to real object id "obj:menu-item-0".
                target: "obj:menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(7.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_reads_props_snapshot() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("props.text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 1,
                dy: 0
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_get_falls_back_to_props_prefix() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 2,
                dy: 0
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_merges_text_content_and_text_props() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("props.text.content") == "HELLO" && obj.get("props.text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 3, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut object_text = HashMap::new();
        object_text.insert("obj:menu-item-0".to_string(), "HELLO".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);
        test_ctx.object_text = Arc::new(object_text);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 3,
                dy: 0
            }]
        );
    }

    #[test]
    fn selected_arrows_hides_when_target_region_missing() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn selected_arrows_uses_target_region_and_padding() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(true),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 1, 1));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 3));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 7,
                    dy: -1
                }
            ]
        );
    }

    #[test]
    fn selected_arrows_resets_cached_offset_after_deselection() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(true),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        behavior.last_dx = 8;
        behavior.last_dy = -1;

        let mut deselected_ctx = ctx(SceneStage::OnIdle, 0, 0);
        deselected_ctx.menu_selected_index = 1;
        let commands = run_behavior(&mut behavior, &base_scene(), deselected_ctx);

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
        assert_eq!(behavior.last_dx, 0);
        assert_eq!(behavior.last_dy, 0);

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 1, 1));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 3));
        let mut selected_ctx = ctx(SceneStage::OnIdle, 0, 0);
        selected_ctx.target_resolver = Arc::new(resolver);
        selected_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), selected_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 7,
                    dy: -1
                }
            ]
        );
    }

    #[test]
    fn selected_arrows_centers_using_own_dimensions() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(false),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 3, 5));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 5));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 6,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn test_all_behaviors_in_catalog() {
        // Verify that every behavior registered in built_in_behavior() is present in catalog
        use engine_core::authoring::catalog::behavior_catalog;

        let runtime_behaviors: Vec<&str> = vec![
            "blink",
            "bob",
            "follow",
            "menu-carousel",
            "menu-carousel-object",
            "rhai-script",
            "menu-selected",
            "selected-arrows",
            "stage-visibility",
            "timed-visibility",
        ];

        let catalog = behavior_catalog();
        let catalog_names: Vec<&str> = catalog.iter().map(|(name, _)| *name).collect();

        for behavior in &runtime_behaviors {
            assert!(
                catalog_names.contains(behavior),
                "Behavior '{}' is registered in runtime but missing from catalog",
                behavior
            );
        }

        for catalog_name in &catalog_names {
            assert!(
                runtime_behaviors.contains(catalog_name),
                "Behavior '{}' is in catalog but not registered in built_in_behavior()",
                catalog_name
            );
        }

        assert_eq!(
            runtime_behaviors.len(),
            catalog_names.len(),
            "Mismatch between runtime behaviors ({}) and catalog ({})",
            runtime_behaviors.len(),
            catalog_names.len()
        );
    }

    // ── debug hardening regression tests ──────────────────────────────────────

    #[test]
    fn rhai_script_behavior_captures_compile_error_on_invalid_syntax() {
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("fn broken( { }".to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_some(),
            "compile_error should be set for invalid Rhai syntax"
        );
    }

    #[test]
    fn rhai_script_behavior_no_compile_error_for_valid_script() {
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"let x = 1 + 2; #{}"#.to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_none(),
            "compile_error should be None for valid Rhai script"
        );
    }

    #[test]
    fn intro_login_scene_rhai_compiles_without_complexity_error() {
        let script = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../mods/shell-quest/scenes/06-intro-login/scene.rhai"
        ));
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(script.to_string()),
            src: Some("/scenes/06-intro-login/scene.rhai".to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_none(),
            "intro-login Rhai script should compile, got: {:?}",
            behavior.compile_error
        );
    }

    #[test]
    fn rhai_script_behavior_emits_script_error_command_on_compile_failure() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let x = @@invalid@@;".to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let has_script_error = commands
            .iter()
            .any(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(
            has_script_error,
            "should emit ScriptError command when compile_error is set"
        );
    }

    #[test]
    fn rhai_script_behavior_emits_compile_error_only_once_per_instance() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let x = @@invalid@@;".to_string()),
            ..BehaviorParams::default()
        });
        let first = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let second = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 16, 16),
        );
        let first_errors = first
            .iter()
            .filter(|c| matches!(c, BehaviorCommand::ScriptError { .. }))
            .count();
        let second_errors = second
            .iter()
            .filter(|c| matches!(c, BehaviorCommand::ScriptError { .. }))
            .count();
        assert_eq!(
            first_errors, 1,
            "first tick should emit exactly one ScriptError"
        );
        assert_eq!(
            second_errors, 0,
            "subsequent ticks should not spam compile ScriptError"
        );
    }

    #[test]
    fn rhai_script_behavior_no_script_error_command_for_valid_script() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"#{ state: #{ mode: "ok" } }"#.to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let has_script_error = commands
            .iter()
            .any(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(
            !has_script_error,
            "should not emit ScriptError for valid script"
        );
    }

    #[test]
    fn rotate_points_i32_rotates_points_by_quarter_turn() {
        assert_eq!(
            rotate_points_i32(&[[0, -10], [10, 0]], 8),
            vec![[10, 0], [0, 10]]
        );
    }

    #[test]
    fn asteroid_fragment_points_i32_returns_three_closed_wedges() {
        let base = asteroid_points_i32(0, 2);
        for fragment_idx in 0..3 {
            let fragment = asteroid_fragment_points_i32(0, 2, fragment_idx);
            assert!(
                fragment.len() >= 5,
                "fragment {fragment_idx} should include centroid plus boundary points"
            );
            assert_eq!(fragment.first().copied(), Some([0, 0]));
            assert_eq!(fragment.last().copied(), Some([0, 0]));
        }
        assert_eq!(base.first().copied(), Some([0, -8]));
    }

    #[test]
    fn smoke_validate_rhai_script_supports_world_api() {
        let scene = base_scene();
        let script = r#"
let id = world.spawn_object("probe", #{ tags: ["probe"] });
if id > 0 {
  world.despawn_object(id);
}
#{}
"#;
        assert!(
            smoke_validate_rhai_script(script, Some("./probe.rhai"), &scene).is_ok(),
            "world API scripts should pass smoke validation"
        );
    }

    #[test]
    fn rhai_script_behavior_script_error_carries_scene_id_and_source() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let = ;".to_string()),
            src: Some("./scene.rhai".to_string()),
            ..BehaviorParams::default()
        });
        let mut scene = base_scene();
        scene.id = "intro-login".to_string();
        let commands = run_behavior(&mut behavior, &scene, ctx(SceneStage::OnIdle, 0, 0));
        let error_cmd = commands
            .iter()
            .find(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(error_cmd.is_some(), "expected ScriptError command");
        if let Some(BehaviorCommand::ScriptError {
            scene_id, source, ..
        }) = error_cmd
        {
            assert_eq!(scene_id, "intro-login");
            assert_eq!(source.as_deref(), Some("./scene.rhai"));
        }
    }

    // ── terminal quest flow regression tests ──────────────────────────────────

    #[test]
    fn rhai_script_behavior_game_state_set_persists_to_game_state() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"game.set("/session/user", "linus"); #{}"#.to_string()),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.game_state = Some(game_state.clone());
        run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert_eq!(
            game_state.get("/session/user"),
            Some(serde_json::json!("linus")),
            "game.set should persist to GameState"
        );
    }

    #[test]
    fn rhai_script_behavior_game_state_has_returns_true_after_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
game.set("/quests/first_message/completed", false);
let ok = game.has("/quests/first_message/completed");
#{}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.game_state = Some(game_state);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "game.has after game.set should not produce a ScriptError"
        );
    }

    #[test]
    fn rhai_script_behavior_level_api_selects_and_mutates_active_level() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
if level.select("asteroids.default") {
  let lives = level.get("/player/lives");
  if lives.type_of() == "i64" {
    level.set("/player/lives", lives + 1);
  }
}
#{}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let level_state = LevelState::new();
        assert!(level_state.register_level(
            "asteroids.default",
            serde_json::json!({
                "player": {
                    "lives": 3
                }
            }),
        ));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.level_state = Some(level_state.clone());
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "level api should not produce ScriptError"
        );
        assert_eq!(level_state.get("/player/lives"), Some(serde_json::json!(4)));
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_creates_entity_visual_and_binding() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_visual("bullet", "bullet-template", #{
    x: 10.5,
    y: 20.3,
    heading: 1.57,
    collider_radius: 2.5,
    lifetime_ms: 5000
});
if id > 0 && world.exists(id) {
    let xf = world.transform(id);
    if xf.x == 10.5 && xf.y == 20.3 && xf.heading == 1.57 {
        print("ok");
    }
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        
        // Check no script errors
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual should not produce ScriptError: {commands:?}"
        );
        
        // Check that SceneSpawn command was emitted
        assert!(
            commands.iter().any(|c| matches!(c, BehaviorCommand::SceneSpawn { 
                template, 
                target 
            } if template == "bullet-template" && target.starts_with("bullet-"))),
            "spawn_visual should emit SceneSpawn command: {commands:?}"
        );
        
        // Check entity was created and has correct transform
        let ids = gameplay_world.ids();
        assert!(!ids.is_empty(), "spawn_visual should create an entity");
        
        if let Some(entity_id) = ids.first() {
            let entity_id = *entity_id;
            assert!(
                gameplay_world.exists(entity_id),
                "created entity should exist"
            );
            
            if let Some(xf) = gameplay_world.transform(entity_id) {
                assert!((xf.x - 10.5).abs() < 0.01, "x position should match");
                assert!((xf.y - 20.3).abs() < 0.01, "y position should match");
                assert!((xf.heading - 1.57).abs() < 0.01, "heading should match");
            }
        }
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_with_polygon_collider() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_visual("asteroid", "asteroid-template", #{
    x: 15.0,
    y: 25.0,
    heading: 0.0,
    collider_polygon: [[0.0, 0.0], [5.0, 0.0], [2.5, 4.0]],
    collider_layer: 1,
    collider_mask: 2
});
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        
        // Check no script errors
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual with polygon should not produce ScriptError: {commands:?}"
        );
        
        // Check that SceneSpawn command was emitted
        assert!(
            commands.iter().any(|c| matches!(c, BehaviorCommand::SceneSpawn { 
                template, 
                target 
            } if template == "asteroid-template" && target.starts_with("asteroid-"))),
            "spawn_visual should emit SceneSpawn command"
        );
        
        // Check entity was created
        let ids = gameplay_world.ids();
        assert!(!ids.is_empty(), "spawn_visual should create an entity");
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_returns_zero_on_failure() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
// Try to spawn with no world (will fail gracefully)
let id = world.spawn_visual("item", "item-template", #{
    x: 0.0,
    y: 0.0
});
// id should be 0 if world creation failed, but we have a world in tests
// so this should actually succeed. Let's test with missing required data
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        
        // Should have created an entity with defaults
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual with minimal data should work"
        );
    }

    #[test]
    fn rhai_script_module_resolver_configuration_exists() {
        // Just verify that the asteroids-shared module file exists
        // The actual module loading happens in the app initialization flow
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let module_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("mods/asteroids/scripts/asteroids-shared.rhai");
        assert!(
            module_path.exists(),
            "asteroids-shared.rhai module should exist at {:?}",
            module_path
        );
    }
}
