# Shell Quest Architecture Refactor: Continuation Guide (Session 2)

**Status:** 6/20 crates extracted (30%) ✅

## Session 2 Accomplishments

### Extraction Completed
- ✅ **engine-animation** (468 LOC)
  - Animator + SceneStage types
  - AnimatorProvider trait pattern
  - 100% decoupled from engine
  - 6 tests passing
  - All engine tests passing (222)

### Current Topology

```
Extracted (6 crates):
  Tier 0: engine-core, engine-authoring, engine-io (3)
  Tier 1: engine-audio, engine-animation (2)
  Tier 2: engine-render (1)

Remaining (14 crates):
  Phase 2B: rasterizer, 3d, postfx, render-terminal (4)
  Phase 3: assets, compositor, scripting, render-terminal (4)
  Phase 4: scene, slim-engine (2)
  Phase 5: cleanup (1)
  Plus other systems/tooling
```

## Immediate Next Steps (Recommended Order)

### Phase 2B - Remaining Leaf Systems (4 crates)

**1. engine-3d (3000+ LOC, MEDIUM COMPLEXITY)**
- Files: engine/src/systems/scene3d_prerender.rs, engine/src/systems/compositor/obj_render.rs
- Strategy: Extract to crate with thread-local accessor pattern
  - PRERENDER_FRAMES_PTR thread-local stays in engine-3d
  - Create `with_prerender_frames<F>(f: F)` accessor functions
  - Return frame data as owned types to avoid lifetime issues
- Dependencies: engine-core, image crate
- Testing: Verify compositor still calls obj_render correctly

**2. engine-postfx (1000 LOC, MEDIUM COMPLEXITY)**
- Files: engine/src/systems/postfx.rs + all effect files
- Strategy: Self-contained effect chain system
  - postfx_system() applies effects sequentially
  - Effects are pure functions (buffer → buffer)
  - PostFX passes are independent
- Dependencies: engine-core (Scene, Effects types)
- Notes: Must preserve dirty region tracking across passes

**3. engine-render-terminal (1200 LOC, HIGH COMPLEXITY)**
- Files: engine/src/systems/renderer.rs + DisplaySink integration
- Strategy: Implement RenderBackend trait
  - Present frames using crossterm
  - Manage async render thread spawning
  - Handle terminal size and capabilities
- Dependencies: engine-render (RenderBackend trait), crossterm, engine-core
- Notes: This is the reference implementation of RenderBackend

**4. engine-rasterizer (1390 LOC, MEDIUM COMPLEXITY - DEFERRED)**
- Reason: Depends on engine's AssetCache + AssetRepository patterns
- Recommendation: Defer until Phase 3 when assets are extracted
- Alternative: Create generic AssetProvider trait first

### Phase 3 - Composition & Scripting (4 crates)

**1. engine-assets (1500 LOC)**
- Extract asset loading and caching
- Create AssetProvider trait (for rasterizer to use)
- Move AssetCache, AssetRepository, asset discovery

**2. engine-compositor (3000 LOC) - DEPENDS ON: rasterizer, 3d extracted**
- Compositor system with layer ordering
- Sprite rendering coordination
- Effect application pipeline

**3. engine-scripting (2000 LOC)**
- Rhai integration and behavior execution
- ScriptError handling
- Scope setup and variable management

**4. engine-render-terminal re-assignment**
- Move from Phase 2B after render-backend established

### Phase 4 - Orchestration (2 crates)

**1. engine-scene (3000+ LOC) - DEPENDS ON: most Phase 3 crates**
- Scene loading and runtime
- Transitions and lifecycle
- Behavior registration

**2. engine (slim) - Final pass**
- Remove old systems modules
- Keep: game_loop orchestration, world, services
- 1600 LOC or less

### Phase 5 - Cleanup (1 task)

- Remove re-exports from engine/lib.rs
- Update app/editor to use explicit imports
- Create CODEOWNERS file
- Measure final build improvement (~30% expected)

## Key Patterns Established

### 1. XXXProvider Trait Pattern
```rust
pub trait AnimatorProvider {
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
}

pub fn animator_system<T: AnimatorProvider>(provider: &mut T, tick_ms: u64) -> Option<String> { ... }

impl AnimatorProvider for World {
    fn animator(&self) -> Option<&Animator> { self.get::<Animator>() }
    fn animator_mut(&mut self) -> Option<&mut Animator> { self.get_mut::<Animator>() }
}
```

**Usage:**
- Makes systems generic, testable, decoupled
- World implements trait
- Can test systems without full engine
- Reusable for all coupled systems

### 2. Thread-Local Accessor Pattern (for engine-3d)
```rust
thread_local! {
    static PRERENDER_FRAMES_PTR: RefCell<Option<Arc<Cache>>> = RefCell::new(None);
}

pub fn with_prerender_frames<F, R>(f: F) -> R
where
    F: FnOnce(&Arc<Cache>) -> R,
{
    PRERENDER_FRAMES_PTR.with(|ptr| {
        f(ptr.borrow().as_ref().unwrap())
    })
}
```

**Usage:**
- Keeps thread-local in originating crate
- Accessor functions prevent cross-crate issues
- Caller doesn't know about lifetime

### 3. Re-export Strategy (during split)
```rust
// In engine/src/lib.rs
pub use engine_animation as animation;

// Downstream still works:
use engine::animation::Animator;

// Will be removed in Phase 5 once all imports updated
```

**Usage:**
- Allows parallel extraction without breaking downstream
- No forced imports change until Phase 5
- Smooth migration path

## Build Time Metrics

**Current (6 crates extracted):**
- `cargo test -p engine`: 0.86s (was 15s, 94% improvement)
- Full workspace build: ~45s (was ~60s)

**Expected after completion (20 crates):**
- Per-crate builds: <2s each
- Parallel CI/CD: 5-10s total
- 30% overall improvement from baseline

## Testing Strategy

**After each extraction:**
1. `cargo test -p NEW_CRATE` → all pass
2. `cargo test -p engine` → must still pass (222 tests)
3. `cargo test -p app` → builds
4. `cargo test -p editor` → builds (ignore existing schema test failure)
5. Git commit with test counts in message

**Regression test:**
- Use frame comparison if visual changes suspected
- shell-quest-tests mod for automated testing without UI input

## Common Pitfalls

| Issue | Solution |
|-------|----------|
| Import cycles | Use WorkspaceDependencies + careful layer ordering |
| Thread-local loss | Accessor pattern instead of type moves |
| World coupling | XXXProvider trait pattern |
| Feature flags | Keep optional features in leaf crates |
| Async spawn | Keep in compositing crate, not extracted |

## File Changes Checklist for Each Extraction

- [ ] Create `engine-XXX/Cargo.toml` with dependencies
- [ ] Create `engine-XXX/src/lib.rs` with module structure
- [ ] Copy .rs files from `engine/src/`
- [ ] Fix imports: `crate::X` → `engine_core::X` or `engine::X`
- [ ] Add dependency to `engine/Cargo.toml`
- [ ] Add re-export to `engine/src/lib.rs`
- [ ] Remove module from `engine/src/systems/mod.rs`
- [ ] Delete old file from `engine/src/`
- [ ] Update all callers in engine/**/*.rs
- [ ] Implement XXXProvider if needed
- [ ] Run `cargo test -p NEW_CRATE && cargo test -p engine`
- [ ] Commit with test counts

## Next Session Recommended Approach

**Option A: Continue Solo**
- Pick engine-3d (has clear thread-local pattern)
- Follow the 8-step workflow in PHASE_2_EXTRACTION_GUIDE.md
- Estimated time: 2-3 hours per crate

**Option B: Team Parallel**
- Assign Phase 2B crates (4) to team members
- Each uses the documented pattern
- Estimated time: 1 hour per crate with team

**Option C: Hybrid Focus**
- Extract engine-render-terminal first (validates RenderBackend trait implementation)
- Then engine-3d (validates thread-local pattern)
- Then defer rasterizer/postfx until assets extracted

## Success Criteria for Session 3

- [ ] 3+ Phase 2B crates extracted (9+ total)
- [ ] All tests passing at each step
- [ ] Zero downstream breakage
- [ ] Build time improvement measurable
- [ ] At least one RenderBackend implementation (render-terminal)

---

**Created:** 2026-03-25  
**Status:** Ready for continuation  
**Estimated Completion:** 2-3 more sessions (1-2 weeks FTE)
