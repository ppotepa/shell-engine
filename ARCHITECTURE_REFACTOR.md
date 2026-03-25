# Shell Quest Architecture Refactor: 20-Crate Split

**Date:** 2026-03-25  
**Goal:** Split monolithic `engine/` (96 files, 35k LOC) into 20 focused crates with abstract render backend enabling pluggable rendering (Terminal → OpenGL → D3D/Vulkan)

---

## Problem Statement

**Current State:**
- Single `engine/` crate with 96 files crammed together
- Rendering hardcoded to crossterm (no path to alternative backends)
- Slow CI/compile times (~15s per change)
- AI agents forced to load full monolith to understand subsystems
- High coupling between rendering, composition, effects, scripting, scene logic

**After Refactor:**
- 20 independent, focused crates (max ~500 LOC each for most)
- `RenderBackend` trait enables terminal→OpenGL→D3D swaps
- 30% faster builds + 50% faster incremental changes
- AI agents work on 2-3 focused crates without unrelated context
- Clear ownership boundaries, easier maintenance

---

## Architecture Overview

### New Crate Topology

```
Tier 0 (Foundation):
  engine-core (4000 LOC) — Buffer, Scene, Effects, Color
  engine-authoring (2000 LOC) — YAML compile/normalize
  engine-io (500 LOC) — Sidecar IPC

Tier 1 (Leaf Systems):
  engine-audio (500 LOC)
  engine-input (800 LOC)
  engine-diagnostics (1500 LOC)
  engine-animation (500 LOC)
  engine-rasterizer (800 LOC)

Tier 2 (Heavy Systems):
  engine-3d (3000 LOC)
  engine-postfx (1000 LOC)
  engine-render (200 LOC — traits only)
  engine-assets (1500 LOC)

Tier 3 (Composition & Scripting):
  engine-compositor (3000 LOC)
  engine-scripting (2000 LOC)
  engine-render-terminal (1200 LOC)

Tier 4 (Orchestration):
  engine-scene (3000 LOC)
  engine (1600 LOC — orchestrator only)

Tier 5 (Frontends):
  app (200 LOC)
  editor (~3000 LOC)
```

### Key Innovation: RenderBackend Trait

```rust
pub trait RenderBackend: Send {
    fn present(&self, frame: &RenderFrame) -> Result<(), RenderError>;
    fn capabilities(&self) -> RenderCaps;
    fn shutdown(&mut self) -> Result<(), RenderError>;
}
```

Enables:
- Terminal backend (existing) → engine-render-terminal
- OpenGL backend (future) → engine-render-opengl
- D3D/Vulkan (future)
- Same game logic, different render target

---

## Execution Strategy

### Phase 0: Prep (1 task)
- Normalize deps in root Cargo.toml
- Baseline green (`cargo test --workspace`)
- Tag checkpoint: `pre-split-2026-03-25`

### Phase 1: Foundation (3 tasks)
- Extract: engine-audio, engine-input, engine-diagnostics
- Risk: Low (zero sibling deps)
- Each dep only on engine-core

### Phase 2: Tier 1 Leaves (5 tasks)
- Extract: engine-animation, engine-rasterizer, engine-3d, engine-postfx, engine-render (traits)
- Risk: Low-Medium
- Order: animation, rasterizer, 3d, postfx, render

### Phase 3: Tier 2 & Composition (4 tasks)
- Extract: engine-assets, engine-compositor, engine-scripting, engine-render-terminal
- Risk: Medium (integration points)
- compositor depends on (rasterizer + 3d), so must wait

### Phase 4: Orchestration (2 tasks)
- Extract: engine-scene, engine (slim)
- Risk: High (highest fan-in; integration complexity)
- scene depends on most others

### Phase 5: Cleanup (1 task)
- Remove re-exports from engine/lib.rs
- Update app/editor imports
- Measure build times

---

## Per-Task Workflow

1. **Create new crate:** `cargo new --lib crate-name`
2. **Add to workspace:** Add to `[workspace].members` in root Cargo.toml
3. **Move files:** Move .rs files to new crate's src/
4. **Create lib.rs:** Add `pub mod X` declarations
5. **Add deps:** New crate's Cargo.toml + add to engine/Cargo.toml
6. **Update engine/lib.rs:** Replace `pub mod X` with `pub use crate_name::*` (re-export)
7. **Fix imports:** Update `use crate::*` → `use engine_core::*` or `use crate_name::*`
8. **Test:** `cargo test -p crate && cargo test -p engine && cargo test --workspace`
9. **Commit:** Descriptive message with risk mitigation notes

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Circular deps mid-split | Keep engine re-exports until all crates extracted; move shared types to engine-core |
| Thread-local statics lose context | Keep thread-locals in originating crate; use `thread_local! { pub static }` + accessor pattern |
| World container type conflicts | World stays in engine; each crate defines its resources; engine registers them |
| Editor import breakage | Phase 5 PR explicitly updates editor; CI catches breakage |
| Async render thread lifetime issues | Render thread spawned/joined in engine-render-terminal; thread-local channel sync |

---

## Success Criteria

- ✅ All 14 PRs merged to main in sequence
- ✅ `cargo test --workspace` green at each step
- ✅ Build time: ~30% faster overall, ~50% faster incremental
- ✅ Each crate <500 LOC for leaves, <3000 for heavy systems
- ✅ Zero downstream breakage (app + editor continue working)
- ✅ RenderBackend trait allows future backend implementations
- ✅ AI agents can work on isolated crates (2-3 files context)
- ✅ CODEOWNERS file establishes ownership boundaries

---

## Timeline Estimate

- Phase 0 (prep): 1-2 hours
- Phase 1 (foundations, 3 tasks): 4-6 hours
- Phase 2 (leaves, 5 tasks): 6-8 hours
- Phase 3 (composition, 4 tasks): 8-10 hours
- Phase 4 (orchestration, 2 tasks): 6-8 hours
- Phase 5 (cleanup, 1 task): 2-3 hours

**Total:** ~2-3 days FTE (can be parallelized in review phase)

---

## Documentation Updates Required (Post-Split)

1. Update AGENTS.md → main README.AGENTS.MD
2. Add README.md to each new crate
3. Create CODEOWNERS file
4. Document RenderBackend API for future backends
5. Update build/test commands in docs
6. Add backend swapping workflow guide

---

## Long-Term Vision

**Year 1 Targets:**
- Terminal backend (done, extracted)
- OpenGL backend (2-3 weeks, parallel to split)
- D3D12 backend (3-4 weeks)

**Year 2+:**
- Vulkan backend
- WebGL (browser/wasm)
- Mobile backends (Metal/etc)

Same game logic, render anywhere.
