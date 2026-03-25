# Phase 2+ Extraction Guide: Completing the 20-Crate Split

## Status: 5/20 crates done (25%)

**Completed:**
- engine-core (foundation, pre-existing)
- engine-authoring (foundation, pre-existing)
- engine-io (foundation, pre-existing)
- engine-audio (Phase 1 - 394 LOC, tests passing)
- engine-render (Phase 2 - traits only, tests passing)

**Ready to Extract:** (Low-Medium risk, proven pattern)

### Phase 2 Tier 1 - Leaf Systems (5 crates)

#### 1. engine-animation (468 LOC)
```bash
# Files: engine/src/systems/animator.rs
# Dependencies: engine-core, crate::events, crate::services, crate::world
# Issue: Tightly coupled to engine (World, EngineEvent, EngineWorldAccess)
# Strategy: Extract types/logic to standalone; World impl via trait
# Pattern: Same as AudioProvider - create AnimatorProvider trait

# Steps:
1. cargo new --lib engine-animation
2. Move animator.rs, extract SceneStage + Animator to standalone types
3. Create AnimatorProvider trait (abstract World dependency)
4. Impl AnimatorProvider for World in engine/services.rs
5. Update game_loop.rs to use engine_animation types
6. Move systems::animator tests to engine-animation/tests
7. cargo test -p engine-animation && cargo test -p engine
8. Commit: refactor(phase-2): Extract engine-animation
```

#### 2. engine-rasterizer (52 KB, 5 files)
```bash
# Files: engine/src/rasterizer/*, engine/src/render_policy.rs
# Dependencies: engine-core (only)
# Status: Pure algorithmic - cleanest extraction
# Risk: Low (self-contained math + caching)

# Steps:
1. cargo new --lib engine-rasterizer
2. Copy rasterizer/ and render_policy.rs
3. Fix imports: crate::* → engine_core::*
4. Create lib.rs with: pub mod rasterizer; pub use rasterizer::*;
5. Update engine/Cargo.toml: add engine-rasterizer dep
6. Update engine/lib.rs: pub use engine_rasterizer as rasterizer
7. Update compositor references: crate::rasterizer → engine_rasterizer::
8. cargo test -p engine-rasterizer && cargo test -p engine
9. Commit: refactor(phase-2): Extract engine-rasterizer
```

#### 3. engine-postfx (1000 LOC, 10+ files)
```bash
# Files: engine/src/systems/postfx/*
# Dependencies: engine-core, engine-render (traits, for RenderError?)
# Status: Pure effect chain system
# Risk: Low-Medium (complex but self-contained)

# Steps:
1. cargo new --lib engine-postfx
2. Copy systems/postfx/* to engine-postfx/src/postfx/
3. Fix imports: crate::buffer → engine_core::buffer
4. Update engine/Cargo.toml: add engine-postfx dep
5. Update engine/lib.rs: pub use engine_postfx as postfx
6. Update game_loop.rs: postfx::system() remains callable
7. cargo test -p engine-postfx && cargo test -p engine
8. Commit: refactor(phase-2): Extract engine-postfx
```

#### 4. engine-3d (3000+ LOC, 8+ files - THREAD-LOCALS!)
```bash
# Files: engine/src/gpu/*, engine/src/systems/compositor/obj_*,
#        engine/src/*scene3d*, engine/src/systems/prerender.rs, warmup.rs
# Dependencies: engine-core, engine-render (RenderError)
# Status: Complex with thread-local caches
# Risk: Medium (PRERENDER_FRAMES_PTR thread-local)

# Pattern: Keep thread-locals in place
# pub thread_local! { pub static PRERENDER_FRAMES_PTR: ... }
# pub fn with_prerender_frames<F>(f: F) where F: FnOnce(...) { ... }

# Steps:
1. cargo new --lib engine-3d
2. Copy gpu/, obj_render.rs, obj_loader.rs, scene3d_* to engine-3d/src/
3. Move prerender.rs, warmup.rs to engine-3d/src/
4. Fix all crate::* → engine_core::* (except thread-locals which stay)
5. Create pub fn with_prerender_frames() accessor
6. Update engine/Cargo.toml: add engine-3d dep
7. Update engine/lib.rs: pub use engine_3d as gpu; pub use engine_3d::{obj_prerender, ...}
8. Update compositor: engine_3d::with_prerender_frames(|cache| ...)
9. Add tests in engine-3d/tests/ or keep in engine
10. cargo test -p engine-3d && cargo test -p engine
11. Commit: refactor(phase-2): Extract engine-3d (watch thread-locals)
```

#### 5. engine-render-terminal (1200 LOC - ASYNC RENDER THREAD!)
```bash
# Files: engine/src/systems/renderer.rs (TerminalRenderer, HalfblockPacker, etc)
# Dependencies: engine-core, engine-render (RenderBackend trait impl)
# Status: Complex with async display sink
# Risk: Medium (async thread lifetime, channel drain)

# Key: TerminalRenderer implements RenderBackend trait from engine-render

# Steps:
1. cargo new --lib engine-render-terminal
2. Copy systems/renderer.rs → engine-render-terminal/src/renderer.rs
3. Extract TerminalRenderer struct to new file
4. Extract HalfblockPacker, AnsiBatchFlusher, AsyncDisplaySink to separate modules
5. Update Cargo.toml: deps = [engine-core, engine-render, crossterm, ...]
6. Impl RenderBackend for TerminalRenderer
7. Fix all imports: crate::* → engine_core::* or engine_render::*
8. Update engine/Cargo.toml: add engine-render-terminal dep
9. Update game_loop.rs: renderer_system() stays; TerminalRenderer creation moves to app/
10. Update app/main.rs: create TerminalRenderer there
11. cargo test -p engine-render-terminal && cargo test -p engine
12. Commit: refactor(phase-2): Extract engine-render-terminal (implements RenderBackend)
```

### Phase 3 - Composition & Heavy Systems (4 crates)

These depend on Tier 2 systems above, so extract only after Phase 2 complete.

#### 6. engine-assets (1500 LOC)
```
Deps: engine-core, engine-authoring
Extract: assets.rs, asset_cache.rs, image_loader.rs, repositories.rs, mod_loader.rs
Pattern: Pure loader abstraction - low risk
```

#### 7. engine-compositor (3000 LOC - HEAVIEST)
```
Deps: engine-core, engine-rasterizer (from Phase 2), engine-3d (from Phase 2)
Extract: systems/compositor/*, layer_compositor.rs, sprite_renderer.rs, layout/*
Pattern: Dispatch to rasterizer/3d as black boxes
Risk: Medium (depends on 2 Tier 2 crates + complex layout logic)
```

#### 8. engine-scripting (2000 LOC)
```
Deps: engine-core
Extract: behavior.rs, mod_behaviors.rs, strategy/behavior_factory.rs
Pattern: Pure Rhai integration - low risk
Create: BehaviorContext, BehaviorCommand enum, Behavior trait
```

### Phase 4 - Orchestration (2 crates)

#### 9. engine-scene (3000 LOC)
```
Deps: ALL Tier 1/2/3 crates above
Extract: scene_runtime.rs, scene_loader.rs, scene_compiler.rs,
         game_state.rs, game_object.rs, systems/scene_lifecycle.rs
Risk: High (fan-in from all subsystems)
```

#### 10. engine (orchestrator, now 1600 LOC)
```
Deps: ALL engine-* crates
Result: Slimmed engine only holds game_loop.rs, world.rs, services.rs,
        pipeline_flags.rs, runtime_settings.rs, splash.rs, startup pipelines
```

### Phase 5 - Cleanup & Frontends (1 crate + docs)

#### 11. Final cleanup
```
Remove: pub use <subcrate>::* re-exports from engine/lib.rs
Update: app/Cargo.toml, editor/Cargo.toml to import needed crates directly
Create: CODEOWNERS file
Update: Build documentation
Measure: Build time improvement (target: 30% faster)
```

---

## Extraction Pattern (Proven with audio + render)

1. **Create crate**: `cargo new --lib engine-<name>`
2. **Set Cargo.toml**: workspace deps, path to engine-core/other
3. **Copy files**: move .rs files to new crate's src/
4. **Fix imports**:
   - Replace `use crate::*` with `use engine_core::*` (or `use crate_name::*`)
   - Keep path dependencies (e.g., `engine_core::buffer::Buffer`)
5. **Create lib.rs**: Declare modules, re-export public API
6. **Handle state**:
   - If coupled to World: Create XXXProvider trait, impl for World
   - If thread-local: Keep in crate, create accessor functions
7. **Update engine/**:
   - Add to Cargo.toml deps
   - In lib.rs: `pub use engine_<name> as <short_name>;`
   - Remove old module decls from systems/mod.rs or lib.rs
8. **Update game_loop.rs**: Call new crate functions
9. **Test**: `cargo test -p engine-<name> && cargo test -p engine`
10. **Commit**: Describe what moved, risk level, test results

---

## Timeline Estimate (With Parallel Reviews)

- Phase 2 (5 crates): ~8-10 hours serial, ~4-6 hours with parallel reviews
- Phase 3 (4 crates): ~8-12 hours serial, ~5-7 hours with parallel reviews
- Phase 4 (2 crates): ~6-8 hours serial
- Phase 5 (cleanup): ~2 hours

**Total: ~3-4 weeks FTE with team reviews, or ~1 week FTE solo**

---

## Success Metrics

✅ All 20 crates in workspace  
✅ cargo test --workspace green at each phase  
✅ Each crate <500 LOC (leaves), <3000 LOC (systems)  
✅ Build time: 30% faster overall  
✅ No breakage to app/editor  
✅ RenderBackend trait tested with multiple implementations (Terminal + future OpenGL stub)  

