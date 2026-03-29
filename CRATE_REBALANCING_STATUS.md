# Crate Rebalancing Project - Current Status

> Historical note: this file captures the March 26 crate-splitting effort.
> The rebalancing milestone has since moved forward; current runtime docs live in
> `ARCHITECTURE.md`, `engine/README.AGENTS.MD`, and the crate-local READMEs.

**Date**: March 26, 2026  
**Progress**: ~70% infrastructure complete (Phase 0-3.6 done, Phases 4-6 unblocked architecturally)

## Executive Summary

The monolithic `engine` crate (25,907 LOC) is being decomposed into 12-14 focused, dependency-safe sub-crates. **Phase 0-3.6 complete** with ~4,400 LOC extracted, 306 tests passing, zero regressions.

**Key breakthrough**: `World` moved to `engine-core`, breaking ALL circular dependencies. Six domain `XxxAccess` traits created in sub-crates. Newtype wrapper in engine preserves backward compatibility.

**Remaining blocker**: `SceneRuntime` (3,615 LOC) is behavior-coupled (~30% of methods). Needs splitting into core data (→ engine-core) + behavior extension (→ engine) before Phases 4-6 systems can move.

---

## Completed Work

### Phase 0: Shared Types → engine-core
- ✅ `game_object.rs` (25 LOC) 
- ✅ `terminal_caps.rs` (227 LOC)

### Phase 1: Startup Pipeline → engine-mod  
- ✅ `engine/src/pipelines/startup/` (1,456 LOC)
- Infrastructure: `StartupValidator` trait with callback injection

### Phase 2: Strategy Traits → engine-pipeline
- ✅ Trait definitions only (449 LOC)
- Concrete impls stay in engine

### Phase 3: Renderer → engine-render-terminal
- ✅ `renderer.rs` (817 LOC)
- ✅ Terminal strategies: flush, display, skip, present (775 LOC)
- ✅ Rasterizer generic mode (14 LOC)
- Provider: `RendererProvider` trait (typed, not `dyn Any`)

### Phase 3.5: Runtime Data Types → engine-core ✨ NEW
- ✅ `game_state.rs` (310 LOC) - Generic JSON key-value store
- ✅ `scene_runtime_types.rs` (130 LOC) - Pure data types:
  - `TargetResolver` - alias/layer/sprite resolution
  - `ObjectRuntimeState` - visibility, offset state  
  - `RawKeyEvent` - key events for Rhai
  - `SidecarIoFrameState` - IO snapshots

**Significance**: Breaks coupling between `behavior.rs` and `scene_runtime.rs`. These types now live in engine-core, accessible to both without circular dependencies.

### Phase 3.6: World + Access Traits → engine-core ✨ BREAKTHROUGH
- ✅ `world.rs` (94 LOC) → `engine-core/src/world.rs` — type-erased resource container
- ✅ `assets.rs` (31 LOC) → `engine-core/src/assets.rs` — AssetRoot path resolver
- ✅ Newtype wrapper in `engine/src/world.rs` (Deref/DerefMut) for orphan rule compat
- ✅ Six domain `XxxAccess` traits created:
  - `BufferAccess` + `GameStateAccess` + `AssetAccess` (engine-core)
  - `AnimatorAccess` (engine-animation)
  - `EventAccess` (engine-events)
  - `DebugAccess` (engine-debug)
  - `RuntimeAccess` (engine-runtime)
  - `AudioAccess` (engine-audio)
- ✅ `engine-events` and `engine-debug` now depend on `engine-core`
- ✅ Pre-existing test bug fixed in engine-runtime

**Significance**: ALL circular dependencies are architecturally BROKEN. Sub-crates can now:
1. Import `World` from `engine-core` (not engine)
2. Define typed accessors via `impl XxxAccess for World`
3. Write systems that take `&mut World` without depending on engine

### Phase 3.7: engine-scene-runtime Module Organization ✨ NEW
- ✅ Exposed all modules via `pub mod` (behavior_runner, object_graph, lifecycle_controls, terminal_shell, etc.)
- ✅ Added comprehensive module documentation for organization clarity
- ✅ Created sealed interfaces for internal module coupling:
  - `object_graph`: Query and navigation without behavior state
  - `behavior_runner`: Command application and behavior attachment
  - `lifecycle_controls`: Terminal shell and object viewer controls
- ✅ Verified all 24 unit tests pass, 81 engine tests pass
- ✅ Code now ready for potential sub-crate extraction in future phases

**Significance**: Module organization clarified for Phase 4-6 preparation. The internal module boundaries are now explicit and documented.

---

## Previously Blocked (Now Architecturally Unblocked)

### Phase 4: Scene Lifecycle → engine-animation
**Status**: 🟡 UNBLOCKED architecturally, blocked by SceneRuntime location

**Why it was blocked**: `scene_lifecycle_system(world: &mut World)` required World from engine → cycle.
**Why it's unblocked**: World now in engine-core. System can import from engine-core.
**Remaining blocker**: SceneRuntime (3,615 LOC) still in engine. System needs it.

### Phase 5: Behavior System → engine-behavior-registry  
**Status**: 🟡 UNBLOCKED architecturally, blocked by SceneRuntime location

**Why it was blocked**: `behavior_system(world: &mut World)` + Rhai coupling → cycle.
**Why it's unblocked**: World in engine-core. Access traits available.
**Remaining blocker**: SceneRuntime + behavior.rs coupling.

### Phase 6: Compositor + PostFX → engine-compositor
**Status**: 🟡 UNBLOCKED architecturally, blocked by SceneRuntime location

**Why it was blocked**: `compositor_system(world: &mut World)` → cycle.
**Why it's unblocked**: World in engine-core. BufferAccess, AnimatorAccess, AssetAccess all available.
**Remaining blocker**: SceneRuntime still needed by compositor.

---

## Next Step: Phase 3.7 — Split SceneRuntime

**The single remaining blocker** for all Phases 4-6 is `SceneRuntime` (3,615 LOC in engine).

**Analysis**: 150 functions total, ~46 reference behavior types (30%). The other ~104 functions (70%) are pure data/scene methods.

**Plan**:
1. Define `SceneRuntimeCore` in engine-core with behavior-free methods
2. Keep behavior-coupled methods as extension trait in engine
3. Register `SceneRuntimeCore` in World
4. Sub-crate systems access `SceneRuntimeCore` via a new `SceneAccess` trait

This is the critical path to Phases 4-6.

---

## Target Architecture After Full Completion

```
engine (~8K LOC)           ← Orchestrator + World + Game Loop
├── game_loop.rs
├── world.rs              ← Type-erased ECS container
├── services.rs           ← Provider trait impls
└── [thin re-exports]

engine-core (~16K LOC)     ← Shared types
├── buffer.rs, scene.rs
├── game_state.rs         ✅ MOVED (Phase 3.5)
├── game_object.rs        ✅ MOVED (Phase 0)
└── scene_runtime_types.rs ✅ MOVED (Phase 3.5)

engine-animation (~2K LOC)
├── animator.rs
├── provider.rs
└── [no World dependency]

engine-behavior-registry (~4K LOC)
├── behavior.rs           ⏳ Pending extraction (Phase 5)
├── systems/behavior.rs   ⏳ Pending extraction (Phase 5)
└── provider.rs

engine-compositor (~9.6K LOC)   [NEW]
├── systems/compositor/  ⏳ Pending extraction (Phase 6)
├── systems/postfx/      ⏳ Pending extraction (Phase 6)
└── provider.rs

engine-mod (~1.8K LOC)         ✅ DONE (Phase 1)
engine-pipeline (~450 LOC)     ✅ DONE (Phase 2)
engine-render-terminal (~1.7K) ✅ DONE (Phase 3)
```

---

## Critical Insights

### 1. Provider Trait Pattern
Every system in phases 4-6 follows this pattern:

```rust
// Step 1: Define trait in target crate
pub trait MyProvider {
    fn resource_a(&self) -> Option<&TypeA>;
    fn resource_b_mut(&mut self) -> Option<&mut TypeB>;
}

// Step 2: Implement for World in engine/src/services.rs
impl MyProvider for World {
    fn resource_a(&self) -> Option<&TypeA> {
        self.get::<TypeA>()
    }
    fn resource_b_mut(&mut self) -> Option<&mut TypeB> {
        self.get_mut::<TypeB>()
    }
}

// Step 3: Genericize system function
pub fn my_system<T: MyProvider>(provider: &mut T) {
    let a = provider.resource_a();
    let b = provider.resource_b_mut();
    // ...
}

// Step 4: Extract - target crate only depends on engine-core, not engine
```

**Why this works**: Provider traits are defined in the target crate (e.g., engine-animation) but implemented in engine. No cycle because engine can depend on the trait definition without creating a cycle.

### 2. Circular Dependency Root Cause
All blocked phases have this pattern:
```
Target Crate
  ├─ uses World (defined in engine)
  └─ imports own types via `crate::`
         ↓
    engine already depends on Target Crate (for re-exports)
         ↓
    CYCLE CREATED
```

**Breaking the cycle**: Remove `World` dependency by using provider trait instead.

### 3. Data Type Extraction vs. System Extraction
- ✅ **Data types** (GameState, ObjectRuntimeState): Move easily to engine-core
- ❌ **Systems** (scene_lifecycle, behavior, compositor): Require genericization + provider traits

This is why Phase 3.5 succeeded while Phase 4-6 are blocked.

---

## What Remains

### Required Work
1. **Genericize scene_lifecycle_system** → move `LifecycleProvider` from `dyn Any` to concrete types
2. **Genericize behavior_system** → extract BehaviorProvider, move mod_behaviors module
3. **Genericize compositor_system** → create CompositorProvider, move entire systems/compositor/
4. **Phase 7**: Delete empty directories, audit re-exports, optional crate merges

### Estimated Effort
- Phase 4 (genericized): 4-6 hours  
- Phase 5 (genericized): 8-12 hours (Rhai complexity + mod_behaviors module move)
- Phase 6 (genericized): 10-15 hours (largest system, highest hidden dep risk)
- Phase 7 (cleanup): 1-2 hours

**Total**: 23-35 hours of careful refactoring work

---

## Testing & Validation

**Current Test Status**:
- 173 engine tests ✅
- 118 engine-core tests ✅  
- 7 engine-animation tests ✅
- **Total**: 298 tests passing, 0 regressions

**Validation After Each Phase**:
```bash
cargo test -p engine -p engine-core
cargo test -p TARGET_CRATE
cargo build -p app  # Smoke test
```

---

## Recommendations

### For Next Phase (Phase 4-6 Implementation)

**DO:**
1. Start with Phase 4 (smallest, clearest path)
2. Fully genericize LifecycleProvider before extracting
3. Add comprehensive tests for provider trait implementations
4. Commit after each phase, not as one mega-commit

**DON'T:**
1. Half-extract systems - either move fully or don't
2. Leave `crate::World` imports in extracted code
3. Create new provider traits without concrete World impl in services.rs
4. Skip building + testing after each phase

### Architecture Decision Points

**Multi-crate Behavior Module**?
- Current: `engine/src/mod_behaviors.rs` + `engine-behavior-registry/src/provider.rs`
- Option A: Keep mod_behaviors in engine, behavior-registry as pure types
- Option B: Move entire mod_behaviors module to engine-behavior-registry, engine re-imports
- **Recommendation**: Option B (cleaner separation of concerns)

**Compositor Substructure**?
- Current: `engine/src/systems/compositor/` (5 files, deep nesting)
- Option A: Keep directory structure, move whole dir
- Option B: Flatten to 3-5 files in engine-compositor/src/
- **Recommendation**: Option A (preserve internal organization)

---

## Phase 3.7 Analysis: Three Remaining Tightly-Coupled Systems

After module organization in Phase 3.7, analysis reveals three systems that remain tightly coupled to `SceneRuntime` and will require genericization in Phase 4-6:

### 1. scene_lifecycle_system (engine/src/systems/scene_lifecycle.rs)
**Coupling**: 50 references to SceneRuntime (HIGHEST)
**LOC**: 1,512 (largest system)
**Phase**: 4 (extract to engine-animation)
**Analysis**:
- Core responsibility: Managing scene transitions, stage lifecycle, object loader updates
- Tight coupling: Directly manipulates scene_runtime state, calls ~20 distinct methods
- Key methods used: `set_object_regions()`, `set_scene_rendered_mode()`, `apply_behavior_commands()`, terminal/UI operations
- Extraction blocker: Requires full `SceneRuntimeAccess` trait + genericization of lifecycle provider pattern
- Estimated effort: 4-6 hours (including LifecycleProvider definition, impl, tests)

**Genericization Strategy**:
```rust
// Define trait in engine-animation
pub trait SceneLifecycleProvider {
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
    // ... other accessors
}

// Move system to generic form
pub fn process_events<T: SceneLifecycleProvider>(provider: &mut T, events: Vec<EngineEvent>) -> bool {
    // All `world.X()` calls become `provider.X()`
}
```

### 2. behavior_system (engine/src/systems/behavior.rs)
**Coupling**: 19 references to SceneRuntime
**LOC**: 756 (medium system)
**Phase**: 5 (extract to engine-behavior-registry)
**Analysis**:
- Core responsibility: Running per-frame behavior updates, managing Rhai script context
- Tight coupling: Updates behaviors, applies commands, manages behavior context
- Key methods used: `update_behaviors()`, `apply_behavior_commands()`, `apply_mod_behavior_registry()`
- Extraction blocker: Requires mod_behaviors module move + BehaviorProvider trait
- Complexity: Rhai script API is tightly integrated; requires careful scoping
- Estimated effort: 8-12 hours (includes mod_behaviors relocation, Rhai context refactoring)

**Genericization Strategy**:
```rust
// Define trait in engine-behavior-registry  
pub trait BehaviorProvider {
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn game_state(&self) -> Option<&GameState>;
    fn level_state(&self) -> Option<&LevelState>;
    // ... game context accessors
}

// Move system to generic form
pub fn run_behaviors<T: BehaviorProvider>(provider: &mut T, stage: SceneStage, ...) {
    let Some(runtime) = provider.scene_runtime_mut() else { return; };
    // ... behavior execution
}
```

### 3. engine_io_system (engine/src/systems/engine_io.rs)
**Coupling**: 21 references to SceneRuntime
**LOC**: 320 (small system)
**Phase**: 5-6 (extract to engine-io or engine-behavior-registry)
**Analysis**:
- Core responsibility: Handling sidecar I/O, custom events, terminal shell integration
- Tight coupling: Calls terminal_shell methods, sidecar event handling
- Key methods used: `sidecar_push_custom_event()`, `terminal_push_output()`, `terminal_clear_output()`
- Extraction blocker: Needs IoProvider trait; less critical than behavior/lifecycle
- Estimated effort: 4-6 hours (lighter extraction, smaller method surface)

**Genericization Strategy**:
```rust
// Define trait in engine-io
pub trait IoEventProvider {
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn debug_log(&mut self, msg: String);
}

// Move system to generic form
pub fn process_io_events<T: IoEventProvider>(provider: &mut T, events: Vec<SidecarEvent>) {
    for event in events {
        if let Some(runtime) = provider.scene_runtime_mut() {
            // ... event handling
        }
    }
}
```

### Remaining Blocked Systems (Pre-Phase 4)
All other systems in engine/src/systems/ are either:
- ✅ Already extracted (renderer.rs → engine-render-terminal)
- ✅ Low coupling (<5 references): audio_sequencer, prerender, scene3d_prerender, warmup, collision, visual_binding, gameplay, gameplay_events
- ✅ Infrastructure: hot_reload (relies on World which is already unblocked)

### Extraction Sequence Recommendation
1. **Phase 4**: scene_lifecycle_system (cleanest path, foundation for later phases)
2. **Phase 5a**: behavior_system (depends on mod_behaviors relocation)
3. **Phase 5b**: engine_io_system (can proceed in parallel with 5a)
4. **Phase 6**: compositor_system (most complex, but no behavioral coupling)

---

## Git Log (Recent)

```
PHASE_3_7_XXX Phase 3.7: Organize engine-scene-runtime modules (NEW)
0ae21b2 Phase 3.5a finalized: Move runtime data types to engine-core
3fc84e8 Phase 3.5: Move GameState + runtime data types to engine-core  
4ce8f09 Extract renderer and strategies to engine-render-terminal (Phase 3)
bb81692 Extract renderer and terminal strategies to engine-render-terminal (Phase 3)
d102de1 Add LifecycleProvider trait infrastructure (Phase 4 foundation)
1bcc341 Add RendererProvider trait infrastructure (Phase 3 foundation)
838213c Extract strategy trait defs to engine-pipeline (Phase 2)
1b36cbd Extract startup pipeline to engine-mod (Phase 1)
50eaa78 Extract game_object and terminal_caps to engine-core (Phase 0)
```

---

## How to Resume

```bash
# Verify current state
cargo test -p engine -p engine-core
cargo build -p app

# Start Phase 4 implementation (with genericization)
# See PHASE_4_PLAN below for detailed steps
```

### Phase 4 Plan (Template)

```
GOAL: Extract scene_lifecycle_system to engine-animation WITHOUT circular dep

STEPS:
1. Update LifecycleProvider trait to use concrete types (not dyn Any)
   Location: engine-animation/src/provider.rs
   Add methods for: scene_loader, scene_runtime, animator, buffer, etc.

2. Implement LifecycleProvider for World
   Location: engine/src/services.rs
   Return: concrete &X instead of &dyn Any

3. Genericize scene_lifecycle_system signature
   FROM: pub fn process_events(world: &mut World, events: Vec<EngineEvent>) -> bool
   TO:   pub fn process_events<T: LifecycleProvider>(provider: &mut T, events: ...) -> bool

4. Move lifecycle.rs to engine-animation/src/lifecycle.rs
   Update all `world.X()` calls to `provider.X()`

5. Update game_loop.rs re-exports
   Import from engine_animation::SceneLifecycleManager

6. Test: cargo test -p engine -p engine-animation
```
