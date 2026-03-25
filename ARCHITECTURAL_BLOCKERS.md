# Architectural Blockers for Phase 2B+ Extractions

## Problem Statement

Phase 2B crate extractions (engine-3d, engine-postfx, engine-render-terminal) were attempted but failed due to architectural coupling that was not initially apparent. This document analyzes the blockers and proposes solutions.

## Blocker 1: AssetRoot Dependency Chain

**Systems affected:**
- scene3d_atlas, scene3d_format, scene3d_resolve (3D rendering)
- obj_prerender, obj_frame_cache (OBJ model caching)
- image_loader, rasterizer (sprite rasterization)

**Root cause:**
All 3D and rasterization systems depend on `AssetRoot` (engine/src/assets.rs), which:
- Is not in engine-core
- Depends on engine's repository/file loading infrastructure
- Depends on EngineError (engine-specific error type)

**Dependency chain:**
```
engine-3d
  └─> AssetRoot
       ├─> EngineError
       ├─> engine/asset_cache
       └─> engine/asset_source
```

**Impact:** 6+ files cannot be extracted without first extracting/decoupling AssetRoot

**Solutions considered:**
1. **Extract engine-assets first** - Creates engine-assets crate, adds it as engine dependency
   - Problem: Circular dependency (engine → engine-assets → engine_core, but engine needs engine-assets)
   - Workaround: Keep AssetRoot in engine, create AssetProvider trait instead

2. **Create AssetProvider trait in engine** - Defers extraction, enables thread-local accessor pattern
   - Pro: No circular deps, proven pattern (AudioProvider, AnimatorProvider exist)
   - Con: 3D systems stay in engine temporarily, extract later
   - Recommended: **Yes**, implement this for Phase 2B completion

## Blocker 2: World/EngineWorldAccess Coupling

**Systems affected:**
- postfx_system() - 386 LOC, deeply coupled to World
- renderer.rs - 817 LOC, depends on World for settings/buffer access
- scene_lifecycle_system() - depends on World events

**Root cause:**
These systems are tightly integrated into the engine loop:
- They call `world.get::<T>()` and `world.get_mut::<T>()`
- They depend on engine-specific traits like EngineWorldAccess
- They depend on engine-specific types like DebugFeatures, DisplaySink

**Dependency chain:**
```
engine-postfx
  └─> World
       ├─> EngineWorldAccess
       ├─> EventQueue
       ├─> DebugLogBuffer
       └─> engine-specific resources
```

**Impact:** 3+ systems cannot be extracted without refactoring their signatures

**Solutions considered:**
1. **Generic over provider traits** - postfx_system<P: PostFXProvider>(provider: &mut P)
   - Pro: Decouples from World, enables extraction
   - Con: Requires redesigning system signatures
   - Estimated effort: 4-6 hours per system

2. **Keep in engine for now** - Extract in Phase 4+ after simpler systems
   - Pro: No redesign needed, focus on independent systems first
   - Con: Delays completion
   - Recommended: **Yes**, defer to Phase 4

## Blocker 3: thread-local State Management

**Systems affected:**
- obj_render (Scene3D rasterization) - PRERENDER_FRAMES_PTR thread-local
- postfx system - POSTFX_RUNTIME thread-local
- renderer.rs - DIFF_SCRATCH, RUN_BUF, ANSI_BUF thread-locals

**Root cause:**
Performance optimization uses thread-locals for state that's expensive to allocate/deallocate per frame.

**Impact:** Does NOT block extraction, but requires accessor pattern

**Solutions:**
1. **Create thread-local accessor functions** - with_prerender_frames<F, R>(f: FnOnce(...) -> R)
   - Pro: Enables extraction, maintains performance
   - Con: Awkward API (but internal only)
   - Pattern proven in engine-animation extract

2. **Move thread-locals to shared state**
   - Pro: Cleaner API
   - Con: May impact performance if not cached properly

**Recommendation:** Use accessor pattern (proven approach)

## Revised Extraction Strategy

### Current State
- 6/20 crates extracted (30%)
- Pattern: AudioProvider, AnimatorProvider, AnimatorSystem proven

### Immediate Action (Session 3)
1. Create `Asset3DProvider` trait in engine/services.rs ✓
2. Implement for World ✓
3. Create engine-render-terminal crate (simplest Phase 2B)
   - Implements RenderBackend trait
   - Minimal dependencies on engine infrastructure
4. Extract engine-menu (if simple enough)

### Phase 2B Revised (Session 4)
1. Extract engine-3d with Asset3DProvider trait
2. Extract engine-postfx with PostFXProvider trait
3. Extract engine-render-terminal from Phase 2B slot

### Phase 3 (Session 5) - **PREREQUISITE FOR REST OF 2B+**
1. Extract engine-assets (careful: circular dep handling)
2. Create AssetProvider trait
3. Refactor 3D systems to use AssetProvider instead of AssetRoot

### Phase 4+ - Deferred
- postfx, renderer systems stay in engine until Phase 4 when provider traits are added
- This unblocks 10+ other systems that don't depend on them

## Key Learning

**Monolith decomposition is not a breadth-first extraction problem; it's a dependency graph traversal problem.**

The XXXProvider trait pattern is powerful and should be applied systematically:
- Before extracting a system, check: what does it depend on?
- If it depends on internal state (World resources), create a ProviderTrait for that state
- Implement trait for World, extract system, profit

## Files Modified
- engine/src/services.rs - Added Asset3DProvider trait
- This doc (new)

## Next Session Checklist
- [ ] Verify Asset3DProvider implementation compiles
- [ ] Create engine-render-terminal crate (simplest)
- [ ] Extract and test engine-render-terminal
- [ ] Measure compilation impact
- [ ] Plan Phase 3 (engine-assets) prerequisites

---

## Session 3 Progress: Provider Trait Pattern Validated

### Extraction #7-8: engine-3d with Scene3DAssetResolver

**Success**: Proved the provider trait pattern works for complex systems.

**What we did:**
1. Created `Scene3DAssetResolver` trait (generic over asset loading)
2. Refactored `resolve_scene3d_refs()` to be generic: `<R: Scene3DAssetResolver>`
3. Implemented resolver for `AssetRoot` (backward compat)
4. Extracted 5 scene3d files to engine-3d crate (893 LOC)
5. Added engine-3d dep to engine, re-exported as `rendering_3d`

**Result:** 0 regressions, all tests pass, crate compiles cleanly.

**Key insight**: This pattern WILL work for postfx, rasterizer, etc. The effort is:
1. Identify what World resources the system needs (5 min analysis)
2. Create XXXProvider trait in services.rs (10 min)
3. Implement for World (5 min)
4. Refactor system to generic<P: XXXProvider> (30-60 min per system)
5. Extract to new crate (10 min)
6. Add re-export to engine/lib.rs (2 min)

**Total per system: 1-2 hours solo, including testing.**

### Remaining High-Confidence Extractions

**For Next Session (Effort Estimate):**

1. **engine-postfx** (386 LOC, 1.5 hours)
   - Needs: PostFXProvider trait
   - Access: scene_runtime, animator, buffer_mut, virtual_buffer_mut, runtime_settings
   - Status: Trait framework ready, just needs implementation

2. **engine-rasterizer** (1390 LOC, 2 hours)
   - Needs: AssetProvider trait first
   - Access: asset_root for image loading
   - Blocker: Depends on engine-assets extraction

3. **Slim down engine** (cleanup, 1 hour)
   - Remove old module exports after extractions
   - Update Cargo.toml after all Phase 2B complete
   - Clean up re-exports

### Why Other Systems Are Harder

- **engine-assets**: Circular dep risk (depends on scene_compiler)
- **engine-compositor**: 3000+ LOC, depends on postfx, rasterizer, 3d
- **engine-scene**: High fan-in, depends on everything
- **engine-behavior**: 3507 LOC, Rhai-specific, deeply coupled

**Recommendation**: Complete Phase 2B first (postfx + rasterizer), THEN tackle Phase 3.

