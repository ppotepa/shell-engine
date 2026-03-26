# Crate Rebalancing Project - Current Status

**Date**: March 26, 2026  
**Progress**: ~60% complete (infrastructure + Phase 0-3.5 done, Phases 4-6 blocked by architecture)

## Executive Summary

The monolithic `engine` crate (26,387 LOC) is being decomposed into 12-14 focused, dependency-safe sub-crates. **Phase 0-3.5 complete** with 4,210 LOC extracted, 298 tests passing, zero regressions.

**Blocker**: Phases 4-6 cannot proceed without genericizing `scene_lifecycle_system`, `behavior_system`, and `compositor_system` over provider traits to avoid circular dependencies with `World` type.

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

---

## Failed Attempts (Documented)

### Phase 4: Scene Lifecycle → engine-animation
**Status**: ❌ BLOCKED - Circular dependency

**Why**: `SceneLifecycleManager::process_events(world: &mut World)` requires `engine::World` type.
- If `engine-animation/src/lifecycle.rs` uses `World`, engine-animation depends on engine
- But engine already depends on engine-animation (for re-exports)
- **Cycle created**

**Solution needed**: Genericize `scene_lifecycle_system<T: LifecycleProvider>(provider: &mut T)`

### Phase 5: Behavior System → engine-behavior-registry  
**Status**: ❌ BLOCKED - Circular dependency + Rhai coupling

**Why**: `behavior_system(world: &mut World)` + `behavior.rs` module imports `engine::mod_behaviors`
- Result: engine-behavior-registry → engine → engine-behavior-registry (cycle)
- Additionally: Rhai runtime initialization tightly couples to BehaviorContext which depends on World

**Solution needed**:
1. Move `mod_behaviors` module entirely to engine-behavior-registry
2. Genericize `behavior_system<T: BehaviorProvider>(provider: &mut T)`
3. Refactor BehaviorContext to accept provider trait instead of World

### Phase 6: Compositor + PostFX → engine-compositor
**Status**: ❌ BLOCKED - Circular dependency

**Why**: `compositor_system(world: &mut World)` uses World directly
- Largest extraction (9,600 LOC) with deepest World coupling
- Would create same circular dependency as Phase 4

**Solution needed**: Genericize `compositor_system<T: CompositorProvider>(provider: &mut T)`

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

## Git Log (Recent)

```
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

