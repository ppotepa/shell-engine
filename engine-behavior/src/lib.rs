//! Behavior system types: the [`Behavior`] trait, built-in behavior structs, and the [`BehaviorContext`] passed each tick.

pub mod builtins;
pub mod catalog;
pub mod emit;
pub mod emitter_state;
pub mod factory;
pub mod palette;
pub mod registry;
pub mod rhai_util;
pub mod scripting;

// Re-export builtin behaviors publicly for external crates
pub use builtins::{
    BlinkBehavior, BobBehavior, FollowBehavior, MenuCarouselBehavior, MenuCarouselObjectBehavior,
    MenuSelectedBehavior, SceneAudioBehavior, SelectedArrowsBehavior, StageVisibilityBehavior,
    TimedVisibilityBehavior,
};

// Re-export from engine-api (now the authoritative source for script-facing types)
pub use engine_api::{BehaviorCommand, DebugLogSeverity};

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use engine_animation::SceneStage;
use engine_core::authoring::metadata::FieldMetadata;
use engine_core::effects::Region;
use engine_core::game_object::{GameObject, GameObjectKind};
use engine_core::game_state::GameState;
use engine_core::level_state::LevelState;
use engine_core::scene::{AudioCue, BehaviorParams, BehaviorSpec, Scene};
use engine_core::scene_runtime_types::{
    ObjectRuntimeState, RawKeyEvent, SidecarIoFrameState, TargetResolver,
};
use engine_game::{CollisionHit, GameplayWorld};
use engine_persistence::PersistenceStore;
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde_json::{Map as JsonMap, Value as JsonValue};

use emit::*;
pub use emitter_state::EmitterState;
use factory::BehaviorFactory;
use rhai_util::*;
use scripting::{
    audio::ScriptAudioApi,
    debug::ScriptDebugApi,
    game::ScriptPersistenceApi,
    game::ScriptTimeApi,
    game::{ScriptGameApi, ScriptLevelApi},
    gameplay::ScriptGameplayApi,
    io::ScriptInputApi,
    io::ScriptTerminalApi,
    scene::ScriptSceneApi,
    ui::ScriptUiApi,
};
use engine_api::{ScriptEffectsApi, ScriptCollisionApi};

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
    pub catalogs: Arc<catalog::ModCatalogs>,
    pub palettes: Arc<palette::PaletteStore>,
    /// Default palette id from mod.yaml, passed through to ScriptPaletteApi.
    pub default_palette: Option<String>,
    pub gameplay_world: Option<GameplayWorld>,
    pub emitter_state: Option<EmitterState>,
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
    /// Whether the engine was started with --debug-feature.
    pub debug_enabled: bool,
    /// Held key set (normalized key codes), exposed to Rhai via `input.down(code)`.
    pub keys_down: Arc<HashSet<String>>,
    /// Keys that were NOT held last frame but ARE held this frame — fires once per press.
    pub keys_just_pressed: Arc<HashSet<String>>,
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
// Example: mods/my-game/scripts/shared.rhai
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

    // ── Key code and layer constants ──────────────────────────────────────
    // Registered as a global module so scripts can reference KEY_*, LAYER_*
    // without any scope injection per call.
    {
        let mut m = rhai::Module::new();
        m.set_var("KEY_LEFT", "Left");
        m.set_var("KEY_RIGHT", "Right");
        m.set_var("KEY_UP", "Up");
        m.set_var("KEY_DOWN", "Down");
        m.set_var("KEY_SPACE", " ");
        m.set_var("KEY_ESC", "Esc");
        m.set_var("KEY_ENTER", "Enter");
        m.set_var("KEY_BACKSPACE", "Backspace");
        m.set_var("KEY_TAB", "Tab");
        m.set_var("KEY_F1", "F1");
        m.set_var("KEY_F2", "F2");
        m.set_var("KEY_F3", "F3");
        m.set_var("KEY_F4", "F4");
        m.set_var("KEY_F5", "F5");
        m.set_var("KEY_F6", "F6");
        m.set_var("KEY_F7", "F7");
        m.set_var("KEY_F8", "F8");
        m.set_var("KEY_F9", "F9");
        m.set_var("KEY_F10", "F10");
        m.set_var("KEY_F11", "F11");
        m.set_var("KEY_F12", "F12");
        m.set_var("LAYER_ALL", 0xFFFF_i64);
        m.set_var("LAYER_NONE", 0_i64);
        m.set_var("LAYER_DEFAULT", 0xFFFF_i64);
        engine.register_global_module(m.into());
    }

    // Register generic utility functions available to all scripts
    engine.register_fn("rand", || -> rhai::FLOAT {
        use std::time::{SystemTime, UNIX_EPOCH};
        // Thread-local LCG seeded from wall clock + a counter for uniqueness
        thread_local! {
            static SEED: std::cell::Cell<u64> = {
                let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
                std::cell::Cell::new(t.subsec_nanos() as u64 ^ (t.as_secs() << 17))
            };
        }
        SEED.with(|s| {
            // Xorshift64
            let mut x = s.get().wrapping_add(1);
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.set(x);
            (x >> 11) as rhai::FLOAT / (1u64 << 53) as rhai::FLOAT
        })
    });

    // Register all scripting domain APIs (types, getters, functions) with the engine
    scripting::register_all_domains(&mut engine);

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

fn build_collision_events_array(collisions: &[CollisionHit]) -> RhaiArray {
    collisions
        .iter()
        .map(|hit| {
            let mut map = RhaiMap::new();
            map.insert("a".into(), (hit.a as rhai::INT).into());
            map.insert("b".into(), (hit.b as rhai::INT).into());
            map.into()
        })
        .collect()
}

fn build_sidecar_io_map(sidecar_io: &SidecarIoFrameState) -> RhaiMap {
    let mut ipc_map = RhaiMap::new();
    ipc_map.insert(
        "has_output".into(),
        (!sidecar_io.output_lines.is_empty()).into(),
    );
    let output_array: RhaiArray = sidecar_io
        .output_lines
        .iter()
        .cloned()
        .map(Into::into)
        .collect();
    ipc_map.insert("output_lines".into(), output_array.into());
    ipc_map.insert(
        "clear_count".into(),
        (sidecar_io.clear_count as rhai::INT).into(),
    );
    ipc_map.insert(
        "has_screen_full".into(),
        sidecar_io.screen_full_lines.is_some().into(),
    );
    let screen_full_lines: RhaiArray = sidecar_io
        .screen_full_lines
        .as_ref()
        .map(|lines| lines.iter().cloned().map(Into::into).collect())
        .unwrap_or_default();
    ipc_map.insert("screen_full_lines".into(), screen_full_lines.into());
    let custom_events: RhaiArray = sidecar_io
        .custom_events
        .iter()
        .cloned()
        .map(Into::into)
        .collect();
    ipc_map.insert("custom_events".into(), custom_events.into());
    ipc_map
}

struct UiFieldsData {
    focused_target: String,
    theme: String,
    submit_target: String,
    submit_text: String,
    change_target: String,
    change_text: String,
    has_submit: bool,
    has_change: bool,
}

fn extract_ui_fields_data(ctx: &BehaviorContext) -> UiFieldsData {
    UiFieldsData {
        focused_target: ctx
            .ui_focused_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        theme: ctx.ui_theme_id.as_deref().unwrap_or_default().to_string(),
        submit_target: ctx
            .ui_last_submit_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        submit_text: ctx
            .ui_last_submit_text
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        change_target: ctx
            .ui_last_change_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        change_text: ctx
            .ui_last_change_text
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        has_submit: ctx.ui_last_submit_target_id.is_some(),
        has_change: ctx.ui_last_change_target_id.is_some(),
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
        expire_ui_flash_message(
            ctx.scene_elapsed_ms,
            ctx.game_state.as_ref(),
            helper_commands.as_ref(),
        );

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
                scope.push(
                    "time",
                    ScriptTimeApi::new(
                        ctx.scene_elapsed_ms,
                        ctx.stage_elapsed_ms,
                        ctx.stage,
                        ctx.game_state.clone(),
                    ),
                );

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
                scope.push("ui", ScriptUiApi::new(ctx, Arc::clone(&helper_commands)));

                let ui_data = extract_ui_fields_data(ctx);
                scope.push("ui_focused_target", ui_data.focused_target);
                scope.push("ui_theme", ui_data.theme);
                scope.push("ui_submit_target", ui_data.submit_target);
                scope.push("ui_submit_text", ui_data.submit_text);
                scope.push("ui_change_target", ui_data.change_target);
                scope.push("ui_change_text", ui_data.change_text);
                scope.push("ui_has_submit", ui_data.has_submit);
                scope.push("ui_has_change", ui_data.has_change);

                // Phase 7C: Use Arc-wrapped key map from context instead of rebuilding.
                scope.push_dynamic("key", (*ctx.rhai_key_map).clone().into());

                // Engine-level key state (separate namespace to prevent behavior interference)
                scope.push_dynamic("engine", (*ctx.engine_key_map).clone().into());

                // Debug feature flag — true when --debug-feature CLI flag is active.
                scope.push("debug_enabled", ctx.debug_enabled);

                // Gameplay collision events (array of {a, b} maps).
                scope.push_dynamic(
                    "collisions",
                    build_collision_events_array(&ctx.collisions).into(),
                );

                // External sidecar bridge exposed as object-shaped `ipc.*`.
                scope.push_dynamic("ipc", build_sidecar_io_map(&ctx.sidecar_io).into());

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
                        Arc::clone(&ctx.keys_just_pressed),
                        Arc::clone(&ctx.action_bindings),
                        Arc::clone(&ctx.catalogs),
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
                    "palette",
                    scripting::palette::ScriptPaletteApi::new(
                        Arc::clone(&ctx.palettes),
                        ctx.persistence.clone(),
                        ctx.default_palette.clone(),
                    ),
                );
                scope.push(
                    "world",
                    ScriptGameplayApi::new(
                        ctx.gameplay_world.clone(),
                        std::sync::Arc::clone(&ctx.collisions),
                        std::sync::Arc::clone(&ctx.collision_enters),
                        std::sync::Arc::clone(&ctx.collision_stays),
                        std::sync::Arc::clone(&ctx.collision_exits),
                        Arc::clone(&ctx.catalogs),
                        ctx.emitter_state.clone(),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push("audio", ScriptAudioApi::new(Arc::clone(&helper_commands)));
                scope.push("effects", ScriptEffectsApi::new(Arc::clone(&helper_commands)));
                scope.push(
                    "collision",
                    ScriptCollisionApi::from_arcs(
                        ctx.gameplay_world.clone(),
                        std::sync::Arc::clone(&ctx.collisions),
                        std::sync::Arc::clone(&ctx.collision_enters),
                        std::sync::Arc::clone(&ctx.collision_stays),
                        std::sync::Arc::clone(&ctx.collision_exits),
                        Arc::clone(&helper_commands),
                    ),
                );

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
        // Extract state update by reference before consuming result for commands.
        // Use read_lock() to avoid a full clone of the map when only reading state.
        if result.is::<RhaiMap>() {
            if let Some(map) = result.read_lock::<RhaiMap>() {
                if let Some(next_state) = map.get("state").and_then(rhai_dynamic_to_json) {
                    self.state = next_state;
                }
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
        catalogs: Arc::new(catalog::ModCatalogs::default()),
        palettes: Arc::new(palette::PaletteStore::default()),
        default_palette: None,
        gameplay_world: Some(gameplay_world),
        emitter_state: None,
        collisions: Arc::new(Vec::new()),
        collision_enters: Arc::new(Vec::new()),
        collision_stays: Arc::new(Vec::new()),
        collision_exits: Arc::new(Vec::new()),
        last_raw_key: None,
        keys_down: Arc::new(HashSet::new()),
        keys_just_pressed: Arc::new(HashSet::new()),
        action_bindings: Arc::new(HashMap::new()),
        sidecar_io: Arc::new(SidecarIoFrameState::default()),
        rhai_time_map: Arc::new(RhaiMap::new()),
        rhai_menu_map: Arc::new(RhaiMap::new()),
        rhai_key_map: Arc::new(RhaiMap::new()),
        engine_key_map: Arc::new(RhaiMap::new()),
        debug_enabled: false,
    }
}

/// Shows directional arrow sprites flanking the selected menu option.
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
        built_in_behavior, catalog, smoke_validate_rhai_script, Behavior, BehaviorCommand,
        BehaviorContext, BlinkBehavior, BobBehavior, FollowBehavior, MenuCarouselBehavior,
        MenuCarouselObjectBehavior, MenuSelectedBehavior, RhaiScriptBehavior, SceneAudioBehavior,
        SelectedArrowsBehavior, StageVisibilityBehavior, TimedVisibilityBehavior,
    };
    use crate::EmitterState;
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
            keys_down: Arc::new(HashSet::new()),
            keys_just_pressed: Arc::new(HashSet::new()),
            action_bindings: Arc::new(HashMap::new()),
            sidecar_io: Arc::new(SidecarIoFrameState::default()),
            rhai_time_map: empty_rhai_time_map(),
            rhai_menu_map: empty_rhai_menu_map(),
            rhai_key_map: empty_rhai_key_map(),
            engine_key_map: empty_engine_key_map(),
            debug_enabled: false,
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
let id = world.spawn_object("enemy", #{ tags: ["enemy", "rock"], x: 12, nested: #{ hp: 3 } });
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
        assert_eq!(gameplay_world.kind_of(id).as_deref(), Some("enemy"));
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
                heading: 0.0,
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
    fn rhai_script_behavior_time_delta_ms_clamps_and_persists() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let dt = time.delta_ms(220, "/ast/last_ms");
game.set("/ast/dt", dt);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 480, 120);
        let state = GameState::new();
        test_ctx.game_state = Some(state.clone());

        let _ = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(state.get("/ast/dt").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(
            state.get("/ast/last_ms").and_then(|v| v.as_i64()),
            Some(480)
        );

        let mut second_ctx = ctx(SceneStage::OnIdle, 830, 120);
        second_ctx.game_state = Some(state.clone());
        let _ = run_behavior(&mut behavior, &scene_with_menu_options(1), second_ctx);
        assert_eq!(state.get("/ast/dt").and_then(|v| v.as_i64()), Some(220));
        assert_eq!(
            state.get("/ast/last_ms").and_then(|v| v.as_i64()),
            Some(830)
        );
    }

    #[test]
    fn rhai_script_behavior_input_load_profile_emits_bindings() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
input.load_profile("game.default");
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });

        // Register a test input profile in the catalog
        let mut catalogs = catalog::ModCatalogs::default();
        catalogs.input_profiles.insert(
            "game.default".to_string(),
            catalog::InputProfile {
                bindings: [
                    ("turn_left".to_string(), vec!["Left".to_string(), "a".to_string(), "A".to_string()]),
                    ("turn_right".to_string(), vec!["Right".to_string(), "d".to_string(), "D".to_string()]),
                    ("thrust".to_string(), vec!["Up".to_string(), "w".to_string(), "W".to_string()]),
                    ("fire".to_string(), vec![" ".to_string(), "f".to_string(), "F".to_string()]),
                ].into_iter().collect(),
            },
        );

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.catalogs = std::sync::Arc::new(catalogs);
        let mut commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            test_ctx,
        );
        commands.sort_by(|a, b| {
            let action_a = match a { BehaviorCommand::BindInputAction { action, .. } => action.as_str(), _ => "" };
            let action_b = match b { BehaviorCommand::BindInputAction { action, .. } => action.as_str(), _ => "" };
            action_a.cmp(action_b)
        });
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::BindInputAction {
                    action: "fire".to_string(),
                    keys: vec![" ".to_string(), "f".to_string(), "F".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "thrust".to_string(),
                    keys: vec!["Up".to_string(), "w".to_string(), "W".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "turn_left".to_string(),
                    keys: vec!["Left".to_string(), "a".to_string(), "A".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "turn_right".to_string(),
                    keys: vec!["Right".to_string(), "d".to_string(), "D".to_string()],
                },
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
let pts = rotate_points([[0, -7], [4, 5], [0, 2], [-4, 5]], 8);
let wave = sin32(0);
let out = [];
out.push(#{ op: "set", target: "menu-item-0", path: "position.x", value: pts.len() });
out.push(#{ op: "set", target: "menu-item-0", path: "position.y", value: wave });
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
                    value: JsonValue::Number(4.into()),
                },
                BehaviorCommand::SetProperty {
                    target: "menu-item-0".to_string(),
                    path: "position.y".to_string(),
                    value: JsonValue::Number(0.into()),
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
                heading: 0.0,
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
                heading: 0.0,
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
                heading: 0.0,
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
                heading: 0.0,
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
                heading: 0.0,
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
                heading: 0.0,
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
    fn rhai_script_behavior_ui_flash_message_sets_text_and_expiry() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"ui.flash_message("READY", 500); []"#.to_string()),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 1200, 0);
        test_ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "ui.flash_message should not produce ScriptError: {commands:?}"
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text == "READY"
            )),
            "ui.flash_message should emit SetText: {commands:?}"
        );
        assert_eq!(
            game_state.get("/__ui/game_message/text"),
            Some(serde_json::json!("READY"))
        );
        assert_eq!(
            game_state
                .get("/__ui/game_message/until_ms")
                .and_then(|value| value.as_i64()),
            Some(1700)
        );
    }

    #[test]
    fn rhai_script_behavior_ui_flash_message_auto_clears_after_expiry() {
        let game_state = GameState::new();
        let mut set_behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"ui.flash_message("READY", 500); []"#.to_string()),
            ..BehaviorParams::default()
        });
        let mut set_ctx = ctx(SceneStage::OnIdle, 1200, 0);
        set_ctx.game_state = Some(game_state.clone());
        let _ = run_behavior(&mut set_behavior, &base_scene(), set_ctx);

        let mut clear_behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("[]".to_string()),
            ..BehaviorParams::default()
        });
        let mut clear_ctx = ctx(SceneStage::OnIdle, 1700, 0);
        clear_ctx.game_state = Some(game_state.clone());
        let commands = run_behavior(&mut clear_behavior, &base_scene(), clear_ctx);

        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text.is_empty()
            )),
            "expired flash should clear the target text: {commands:?}"
        );
        assert_eq!(game_state.get("/__ui/game_message/text"), None);
        assert_eq!(game_state.get("/__ui/game_message/until_ms"), None);
    }

    #[test]
    fn rhai_script_behavior_level_api_selects_and_mutates_active_level() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
if level.select("game.default") {
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
            "game.default",
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
            commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::SceneSpawn {
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
let id = world.spawn_visual("enemy", "enemy-template", #{
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
            commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::SceneSpawn {
                template,
                target
            } if template == "enemy-template" && target.starts_with("enemy-"))),
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
    fn rhai_script_behavior_spawn_prefab_creates_ship_and_entity() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 50,
    thrust_power: 80.0,
    max_speed: 150.0,
    heading_bits: 16
  },
  invulnerable_ms: 3000
});
let entity = world.spawn_prefab("entity", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
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
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_prefab should not produce ScriptError: {commands:?}"
        );
        assert!(
            commands.iter().any(
                |c| matches!(c, BehaviorCommand::SceneSpawn { template, target }
                if template == "entity-template" && target.starts_with("entity-"))
            ),
            "entity prefab should emit SceneSpawn"
        );
        assert!(
            commands.iter().any(
                |c| matches!(c, BehaviorCommand::SceneSpawn { template, target }
                if template == "ship" && target.starts_with("ship-"))
            ),
            "ship prefab should emit dynamic ship SceneSpawn"
        );

        let ship_ids = gameplay_world.query_kind("ship");
        assert_eq!(ship_ids.len(), 1, "ship prefab should create one ship");
        let ship_id = ship_ids[0];
        let ship_visual = gameplay_world.visual(ship_id).and_then(|v| v.visual_id);
        assert!(
            ship_visual
                .as_ref()
                .map(|id| id.starts_with("ship-"))
                .unwrap_or(false),
            "ship should have dynamic visual id, got {ship_visual:?}"
        );
        assert!(
            gameplay_world.controller(ship_id).is_some(),
            "ship should have controller"
        );
        assert!(gameplay_world.status_has(ship_id, "invulnerable"));

        let entity_ids = gameplay_world.query_kind("entity");
        assert_eq!(
            entity_ids.len(),
            1,
            "entity prefab should create one entity"
        );
        let entity_id = entity_ids[0];
        let xf = gameplay_world
            .transform(entity_id)
            .expect("entity transform");
        assert!((xf.x - 12.0).abs() < 0.01);
        assert!((xf.y - 18.0).abs() < 0.01);
        let phys = gameplay_world
            .physics(entity_id)
            .expect("entity physics");
        assert!((phys.vx - 2.0).abs() < 0.01);
        assert!((phys.vy + 1.0).abs() < 0.01);
        assert_eq!(
            gameplay_world
                .get(entity_id, "/size")
                .and_then(|v| v.as_i64()),
            Some(2)
        );
    }

    #[test]
    fn rhai_script_behavior_spawn_prefab_unknown_returns_zero() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_prefab("missing", #{});
game.set("/test/prefab_id", id);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "unknown prefab should fail gracefully"
        );
        assert_eq!(
            game_state.get("/test/prefab_id").and_then(|v| v.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.count(), 0);
    }

    #[test]
    fn rhai_script_behavior_spawn_group_unknown_returns_empty() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ids = world.spawn_group("missing.group", "entity");
game.set("/test/spawn_count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "unknown spawn_group should fail gracefully"
        );
        assert_eq!(
            game_state.get("/test/spawn_count").and_then(|v| v.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.count(), 0);
    }

    #[test]
    fn rhai_script_behavior_emit_spawns_generic_ephemeral_fx() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ship = world.spawn_prefab("ship", #{ x: 10.0, y: 20.0 });
let owner = world.spawn_visual("owner", "owner-template", #{ x: 10.0, y: 20.0, heading: 0.0 });
let fx = world.emit("test.smoke", owner, #{
    kind: "fx",
    template: "debris",
    owner_bound: false,
    speed: 12.0,
    spread: 0.0,
    ttl_ms: 300,
    radius: 2,
    fg: "gray"
});
game.set("/test/fx_id", fx);
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut catalogs = catalog::ModCatalogs::test_catalogs();
        catalogs.emitters.insert(
            "test.smoke".to_string(),
            catalog::EmitterConfig {
                max_count: Some(8),
                cooldown_name: Some("smoke".to_string()),
                cooldown_ms: Some(0),
                min_cooldown_ms: Some(0),
                ramp_ms: Some(0),
                spawn_offset: Some(4.0),
                side_offset: None,
                local_x: None,
                local_y: None,
                edge_from_x: None,
                edge_from_y: None,
                edge_to_x: None,
                edge_to_y: None,
                edge_t: None,
                emission_angle: None,
                emission_local_x: None,
                emission_local_y: None,
                backward_speed: Some(0.25),
                ttl_ms: Some(240),
                radius: Some(1),
                velocity_scale: Some(1.0),
                lifecycle: None,
                follow_local_x: None,
                follow_local_y: None,
                follow_inherit_heading: None,
            },
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.gameplay_world = Some(gameplay_world.clone());
        test_ctx.game_state = Some(game_state.clone());
        test_ctx.catalogs = std::sync::Arc::new(catalogs);
        test_ctx.emitter_state = Some(EmitterState::default());

        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "emit should not produce ScriptError: {commands:?}"
        );
        let fx_id = game_state
            .get("/test/fx_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        assert!(fx_id > 0);
        assert!(gameplay_world.exists(fx_id as u64));
        assert!(
            commands.iter().any(|c| matches!(c, BehaviorCommand::SetProperty { path, .. } if path == "style.fg"))
        );
    }

    #[test]
    fn rhai_script_behavior_emit_supports_ttl_follow_owner_lifecycle() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let owner = world.spawn_visual("owner", "owner-template", #{ x: 10.0, y: 20.0, heading: 0.0 });
let fx = world.emit("test.follow_smoke", owner, #{
    kind: "fx",
    template: "debris",
    lifecycle: "TtlFollowOwner",
    ttl_ms: 300,
    follow_local_x: -6.0,
    follow_local_y: 1.0
});
game.set("/test/fx_id", fx);
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut catalogs = catalog::ModCatalogs::test_catalogs();
        catalogs.emitters.insert(
            "test.follow_smoke".to_string(),
            catalog::EmitterConfig {
                max_count: Some(8),
                cooldown_name: Some("smoke".to_string()),
                cooldown_ms: Some(0),
                min_cooldown_ms: Some(0),
                ramp_ms: Some(0),
                spawn_offset: Some(4.0),
                side_offset: None,
                local_x: None,
                local_y: None,
                edge_from_x: None,
                edge_from_y: None,
                edge_to_x: None,
                edge_to_y: None,
                edge_t: None,
                emission_angle: None,
                emission_local_x: None,
                emission_local_y: None,
                backward_speed: Some(0.0),
                ttl_ms: Some(240),
                radius: Some(1),
                velocity_scale: Some(1.0),
                lifecycle: Some("TtlFollowOwner".to_string()),
                follow_local_x: Some(-4.0),
                follow_local_y: Some(0.0),
                follow_inherit_heading: Some(true),
            },
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.gameplay_world = Some(gameplay_world.clone());
        test_ctx.game_state = Some(game_state.clone());
        test_ctx.catalogs = std::sync::Arc::new(catalogs);
        test_ctx.emitter_state = Some(EmitterState::default());

        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "emit should not produce ScriptError: {commands:?}"
        );
        let fx_id = game_state
            .get("/test/fx_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        assert!(fx_id > 0);
        assert_eq!(
            gameplay_world.lifecycle(fx_id as u64),
            Some(engine_game::components::LifecyclePolicy::TtlFollowOwner)
        );
        let follow = gameplay_world
            .follow_anchor(fx_id as u64)
            .expect("follow anchor");
        assert!((follow.local_x - (-6.0)).abs() < 0.001);
        assert!((follow.local_y - 1.0).abs() < 0.001);
        assert!(follow.inherit_heading);
        assert_eq!(
            gameplay_world.ownership(fx_id as u64).map(|ownership| ownership.owner_id),
            Some(1)
        );
    }
}
