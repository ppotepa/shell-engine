ARCH.MD — CRATE REBALANCING PLAN (REFINED)
==========================================

STATUS: REFINED FOR DEPENDENCY SAFETY — Ready for execution
CONTEXT: Shell Quest engine was split into 20 sub-crates, but extraction moved
         only type definitions, leaving ~30K LOC of systems/logic in engine/.
         Eight sub-crates are empty shells. This plan reorders extractions to
         avoid circular dependencies.

GOAL: Move implementation code from engine/ into sub-crates in dependency-safe
      order (bottom-up: low-dependency systems first, high-dependency last).


========================================================================
CURRENT STATE
========================================================================

  engine/                  105 files   ~30K LOC   (MONOLITH)
  engine-core/              57 files   13K LOC    (healthy)
  engine-authoring/         12 files    9K LOC    (healthy)
  engine-behavior-registry   1 file    137 LOC    (empty — needs behavior.rs)
  8 other thin crates       <200 LOC each         (mostly empty shells)


========================================================================
REVISED PHASE ORDER (Dependency-Safe Bottom-Up)
========================================================================

PHASE 1: Startup Pipeline → engine-mod         (NO deps on other systems)
PHASE 2: Strategy Traits → engine-pipeline     (TYPE DEFS only, no impls)
PHASE 3: Renderer → engine-render-terminal    (depends on Phase 2)
PHASE 4: Scene Lifecycle → engine-animation   (depends on Phase 3)
PHASE 5: Behavior System → engine-behavior-registry (depends on Phase 1-4)
PHASE 6: Compositor + PostFX → engine-compositor (depends on Phase 2-5)
PHASE 7: Cleanup                               (delete empty dirs)

KEY: Each phase is self-contained and verified before moving to next.


========================================================================
PHASE 1: Startup Pipeline → engine-mod (LOWEST DEPS)
========================================================================

WHY:  Startup validation has ZERO runtime dependencies. It runs once at init,
      validates mod assets/scenes/fonts/effects, and reports errors. No other
      system depends on it.

WHAT MOVES:
  engine/src/pipelines/startup/  (1,456 LOC) → engine-mod/src/startup/
    All validation checks and runner

RESULT:
  engine-mod: 159 → ~1,615 LOC
  engine/: shrinks by ~1,456 LOC

VERIFY: cargo test -p engine-mod && cargo test -p engine
        Run app, should see no validation startup errors (or existing ones)

------------------------------------------------------------------------
PHASE 2: Strategy Trait Definitions → engine-pipeline (LOW DEPS)
------------------------------------------------------------------------

WHY:  The strategy traits are INTERFACE DEFINITIONS with zero logic.
      Concrete impls stay in engine/ for now; only trait defs move.
      This enables other crates to implement strategies without depending
      on engine/ internals.

WHAT MOVES:
  engine/src/strategy/mod.rs → engine-pipeline/src/strategies.rs
    Trait DEFINITIONS ONLY:
      - DiffStrategy
      - LayerCompositor
      - HalfblockPacker
      - VirtualPresenter
      - TerminalFlusher
      - FrameSkipOracle
      - SceneCompositor
      - BehaviorFactory
      - PipelineStrategies struct
      - from_flags() constructor

  NOTE: Concrete impls (halfblock.rs, layer.rs, flush.rs, present.rs, skip.rs)
        stay in engine/ for now.

RESULT:
  engine-pipeline: 92 → ~670 LOC
  engine/: shrinks by ~200 LOC

VERIFY: cargo test -p engine-pipeline && cargo test -p engine

------------------------------------------------------------------------
PHASE 3: Renderer System → engine-render-terminal (MEDIUM DEPS)
------------------------------------------------------------------------

WHY:  Renderer (817 LOC) plus terminal-specific strategies (flush, present,
      display) are the natural home for engine-render-terminal.
      Generic FrameSkipOracle impl goes to engine-render.

WHAT MOVES:
  engine/src/systems/renderer.rs      (817 LOC) → engine-render-terminal/src/renderer.rs
  engine/src/strategy/flush.rs                  → engine-render-terminal/src/strategy/flush.rs
  engine/src/strategy/present.rs                → engine-render-terminal/src/strategy/present.rs
  engine/src/strategy/display.rs                → engine-render-terminal/src/strategy/display.rs
  engine/src/strategy/skip.rs         (~80 LOC) → engine-render/src/strategy/skip.rs

HOW:
  1. Define RendererProvider trait in engine-render-terminal:
     pub trait RendererProvider {
         fn buffer(&self) -> Option<&Buffer>;
         fn buffer_mut(&mut self) -> Option<&mut Buffer>;
         fn output_buffer(&self) -> Option<&Buffer>;
         fn runtime_settings(&self) -> Option<&RuntimeSettings>;
         fn debug_features(&self) -> Option<&DebugFeatures>;
         fn debug_log_mut(&mut self) -> Option<&mut DebugLogBuffer>;
     }

  2. Implement RendererProvider for World in engine/src/services.rs

  3. Make renderer.rs generic: fn renderer_system<T: RendererProvider>(t: &mut T)

  4. Move files to engine-render-terminal/src/

  5. Update engine/game_loop.rs to call engine_render_terminal::renderer_system(world)

RESULT:
  engine-render: 118 → ~200 LOC
  engine-render-terminal: 80 → ~1,200 LOC
  engine/: shrinks by ~1,600 LOC

VERIFY: cargo test -p engine-render-terminal && cargo test -p engine

------------------------------------------------------------------------
PHASE 4: Scene Lifecycle → engine-animation (MODERATE DEPS)
------------------------------------------------------------------------

WHY:  Scene lifecycle (1,455 LOC) handles stage transitions triggered by user
      input/events. The animator (391 LOC already in engine-animation) handles
      transitions triggered by time. Together they complete the animation domain.
      Menu.rs (97 LOC) is only called from scene_lifecycle.

WHAT MOVES:
  engine/src/systems/scene_lifecycle.rs  (1,455 LOC) → engine-animation/src/lifecycle.rs
  engine/src/systems/menu.rs             (   97 LOC) → engine-animation/src/menu.rs

HOW:
  1. Define LifecycleProvider trait extending AnimatorProvider:
     pub trait LifecycleProvider: AnimatorProvider {
         fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
         fn events_mut(&mut self) -> Option<&mut EventQueue>;
         fn debug_features(&self) -> Option<&DebugFeatures>;
     }

  2. Implement for World in engine/src/services.rs

  3. Move lifecycle.rs + menu.rs to engine-animation/src/

  4. Update engine/game_loop.rs

RESULT:
  engine-animation: 391 → ~1,940 LOC
  engine/: shrinks by ~1,550 LOC

VERIFY: cargo test -p engine-animation && cargo test -p engine

------------------------------------------------------------------------
PHASE 5: Behavior System → engine-behavior-registry (HIGH DEPS)
------------------------------------------------------------------------

WHY:  Behavior system is the LARGEST remaining (3,507 LOC behavior.rs +
      602 LOC systems/behavior.rs). It's also the most coupled. By this phase,
      earlier extractions have created enough flexibility via provider traits.

WHAT MOVES:
  engine/src/behavior.rs           (3,507 LOC) → engine-behavior-registry/src/runtime.rs
  engine/src/systems/behavior.rs   (  602 LOC) → engine-behavior-registry/src/system.rs

HOW:
  1. Define BehaviorProvider trait (comprehensive):
     pub trait BehaviorProvider {
         fn scene(&self) -> Option<Scene>;
         fn animator(&self) -> Option<&Animator>;
         fn scene_runtime(&self) -> Option<&SceneRuntime>;
         fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
         fn get_game_state(&self) -> Option<GameState>;
         fn get_mod_behaviors(&self) -> Option<ModBehaviorRegistry>;
         fn get_debug_log_mut(&mut self) -> Option<&mut DebugLogBuffer>;
         fn dispatch_audio_command(&mut self, cmd: AudioCommand);
         fn dispatch_behavior_command(&mut self, cmd: BehaviorCommand);
     }

  2. Implement for World in engine/src/services.rs

  3. Move behavior.rs and systems/behavior.rs

  4. Update engine/game_loop.rs

RESULT:
  engine-behavior-registry: 137 → ~4,250 LOC
  engine/: shrinks by ~4,100 LOC
  (Single biggest extraction)

VERIFY: cargo test -p engine-behavior-registry && cargo test -p engine

------------------------------------------------------------------------
PHASE 6: Compositor + PostFX → engine-compositor (HIGHEST DEPS)
------------------------------------------------------------------------

WHY:  This is the LARGEST remaining system (~9.6K LOC) and has deep internal
      complexity (obj_render needs AssetRoot, text_render needs rasterizer,
      postfx reads compositor output). Keep it for last when all dependencies
      are cleanest.

WHAT MOVES:
  engine/src/systems/compositor/    (6,195 LOC) → engine-compositor/src/compositor/
  engine/src/systems/postfx.rs      (  386 LOC) → engine-compositor/src/postfx/mod.rs
  engine/src/systems/postfx/        (1,643 LOC) → engine-compositor/src/postfx/
  engine/src/rasterizer/            (1,390 LOC) → engine-compositor/src/rasterizer/
  engine/src/strategy/halfblock.rs            → engine-compositor/src/strategy/halfblock.rs
  engine/src/strategy/layer.rs                → engine-compositor/src/strategy/layer.rs
  engine/src/strategy/scene_compositor.rs     → engine-compositor/src/strategy/scene_compositor.rs

HOW:
  1. Create new crate: engine-compositor/

  2. Define CompositorProvider trait:
     pub trait CompositorProvider {
         fn scene_runtime(&self) -> Option<&SceneRuntime>;
         fn animator(&self) -> Option<&Animator>;
         fn buffer_mut(&mut self) -> Option<&mut Buffer>;
         fn asset_root(&self) -> Option<&AssetRoot>;
         fn runtime_settings(&self) -> Option<&RuntimeSettings>;
     }

  3. Implement for World in engine/src/services.rs

  4. Move ALL files (compositor, postfx, rasterizer, strategies together)

  5. Update engine/game_loop.rs

RESULT:
  NEW engine-compositor: ~9,600 LOC
  engine/: shrinks by ~9,500 LOC
  (The monolith finally breaks!)

VERIFY: cargo test -p engine-compositor && cargo test -p engine

------------------------------------------------------------------------
PHASE 7: Cleanup
--------================================================================

DELETE empty directories (all files moved):
  - engine/src/strategy/
  - engine/src/pipelines/
  - engine/src/systems/compositor/
  - engine/src/systems/postfx/
  - engine/src/rasterizer/

AUDIT:
  - Verify engine/src/lib.rs re-exports all public modules
  - Verify all engine::Type imports still work (via re-exports)
  - Check engine/src/services.rs for unused code

OPTIONAL: Merge leaf crates
  - engine-error (51 LOC) + engine-events (44 LOC) + engine-frame (74 LOC)
    → engine-core (since they have no deps and used everywhere)
  - Update all Cargo.toml deps to use engine-core instead

RESULT:
  engine/: ~12K LOC (pure orchestration)
  All sub-crates: focused, clear responsibilities
  Compile time: 3-4 crates parallel instead of full monolith rebuild

VERIFY: cargo build -p engine && cargo test --all


========================================================================
EXPECTED FINAL STATE (After All Phases)
========================================================================

engine/                  ~40 files   ~12K LOC   (orchestrator)
  game_loop.rs           — main loop calling systems
  scene_runtime.rs       — runtime state
  scene_loader.rs        — scene loading
  world.rs               — type-erased ECS
  services.rs            — provider trait implementations
  lib.rs                 — re-exports

engine-compositor/       NEW        ~9.6K LOC  (rendering pipeline)
  compositor/            — layer compositing, sprite rendering
  postfx/                — post-processing effects
  rasterizer/            — text rasterization
  strategy/              — HalfblockPacker, LayerCompositor, SceneCompositor impls

engine-core/             ~57 files  ~13K LOC   (shared types, unchanged)

engine-authoring/        ~12 files  ~9K LOC    (YAML pipeline, unchanged)

engine-behavior-registry/ ~5 files  ~4.3K LOC  (Rhai runtime + behavior system)

engine-animation/         ~6 files  ~1.9K LOC  (Animator + Lifecycle + Menu)

engine-mod/              ~15 files  ~1.6K LOC  (Manifest + Startup Validation)

engine-render-terminal/   ~5 files  ~1.2K LOC  (Renderer + Terminal Strategies)

engine-pipeline/          ~3 files  ~670 LOC   (Strategy trait defs + flags)

engine-render/            ~1 file   ~200 LOC   (FrameSkipOracle impl)

Leaf crates (unchanged):
  engine-3d, engine-audio, engine-io, engine-capture, engine-debug,
  engine-game, engine-runtime, engine-terminal, engine-render-policy,
  engine-error, engine-events, engine-frame


========================================================================
EXECUTION CHECKLIST
========================================================================

For each phase:

  [ ] Read phase description and understand what moves
  [ ] Create/update Cargo.toml deps (add needed crates)
  [ ] Define provider trait in target crate
  [ ] Implement provider trait for World in engine/src/services.rs
  [ ] Copy files to target crate
  [ ] Update imports (crate:: → engine_xxx::, etc.)
  [ ] Update engine/game_loop.rs to call new system location
  [ ] Update engine/src/lib.rs re-exports
  [ ] cargo test -p TARGET_CRATE — should pass
  [ ] cargo test -p engine — all 204 tests should pass
  [ ] git add, git commit with "Extract SYSTEM → TARGET_CRATE"
  [ ] Move to next phase


========================================================================
PROVIDER TRAIT PATTERN (Standard Template)
========================================================================

For each extraction, follow this pattern to avoid circular deps:

1. IN TARGET CRATE, define trait:
   ```rust
   pub trait FooProvider {
       fn resource(&self) -> Option<&ResourceType>;
       fn resource_mut(&mut self) -> Option<&mut ResourceType>;
   }
   ```

2. IN engine/src/services.rs, implement for World:
   ```rust
   impl FooProvider for World {
       fn resource(&self) -> Option<&ResourceType> {
           self.get::<ResourceType>()
       }
       fn resource_mut(&mut self) -> Option<&mut ResourceType> {
           self.get_mut::<ResourceType>()
       }
   }
   ```

3. IN TARGET CRATE, make system generic:
   ```rust
   pub fn foo_system<T: FooProvider>(provider: &mut T) {
       // system logic, no World dependency
   }
   ```

4. IN engine/src/game_loop.rs, call via World:
   ```rust
   target_crate::foo_system(world);
   ```

Result: target_crate depends ONLY on engine-core and peer crates.
        Zero circular dependencies.
        World remains the ECS container (implements all provider traits).
