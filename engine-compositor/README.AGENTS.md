# engine-compositor: Scene Composition, PostFX, & 3D Assets

## Overview
Extracted from `engine` (2,310 LOC total), this crate provides:
- **PostFX Passes** — CRT, burn-in, glow, scan glitch effects (~1,600 LOC)
- **Scene Compositor Types** — Strategy pattern for cell vs halfblock rendering
- **3D Asset Caching** — OBJ prerender frames, Scene3D atlas
- **CompositorAccess Trait** — Decoupling World dependency from compositor logic

## Key Types

### PostFX System
- **`postfx_system(world: &mut World)`** — Main entry point, applies all post-processing passes per frame
- **`PostFxContext`** — Frame-local state (dirty regions, frame count, previous output)
- **`CompiledPostFx`** — Compiled effect definitions into executable passes

### Scene Compositor Strategy
- **`SceneCompositor` trait** — Defines `composite()` method for rendering
- **`CellSceneCompositor`** — Renders at terminal cell resolution
- **`HalfblockSceneCompositor`** — Renders at 2× height, packs to halfblocks
- **`CompositeParams`** — Bundle of inputs (bg, layers, effects, camera, etc.)

### 3D Assets
- **`ObjPrerenderedFrames`** — Thread-local cache of precomputed OBJ frames
- **`Scene3DAtlas`** — Material + mesh index for loaded 3D objects

### Access Trait (Decoupling)
- **`CompositorAccess`** — Abstracts World resource access for compositor
  - `scene_runtime()`, `animator()`, `buffer_mut()`, `runtime_settings()`
  - Implemented for `World` in engine/services.rs
  - Enables future compositor logic to work with any type providing these resources

## PostFX Passes (8 Types)

1. **CRT** (441 LOC) — Cathode ray tube scanline effect with customizable shadow mask
2. **Burn-in** (287 LOC) — Phosphor burn-in simulation (ghosting on static content)
3. **Glow** (303 LOC) — Bloom effect with shadow mask filtering
4. **Ruby CRT** (115 LOC) — Variant CRT effect
5. **CRT Distort** (128 LOC) — CRT with barrel/pincushion distortion
6. **Scan Glitch** (84 LOC) — Random scan line glitches
7. **Underlay** (70 LOC) — Underlay effect (accent coloring)
8. **Registry** (215 LOC) — Compiles and caches effect pipelines

## Dependencies
- `engine-core` — Buffer, effects, world types
- `engine-animation` — SceneStage for timing
- `engine-runtime` — RuntimeSettings access
- `engine-scene-runtime` — SceneRuntime type
- `crossterm::style::Color` — Terminal colors

## Architecture: PostFX Pipeline

**Per-Frame Flow**:
1. `postfx_system(world)` called in game loop
2. Early return if scene has no effects (optimization)
3. For each effect in scene definition:
   - Resolve to compiled pass via registry cache
   - Swap scratch buffers (src/dst)
   - Apply pass logic (pixel/color transformations)
   - Update dirty region
4. Final output written back to main buffer

**Dirty Region Tracking**:
- Each pass marks affected cells
- Reduces compositor overhead by only flushing changed cells
- Region re-expanded after all effects complete

**Caching**:
- Compiled effect pipelines cached per scene_id (thread-local POSTFX_RUNTIME)
- Avoids recompiling effects each frame

## CompositorAccess Trait (Phase 4.6)

Enables decoupling compositor from `engine::World`:

```rust
pub trait CompositorAccess {
    fn scene_runtime(&self) -> Option<&dyn Any>;
    fn animator(&self) -> Option<&Animator>;
    fn buffer_mut(&self) -> Option<&mut Buffer>;
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
    // ... etc
}

impl CompositorAccess for World {
    fn scene_runtime(&self) -> Option<&dyn Any> {
        self.get::<SceneRuntime>().map(|sr| sr as &dyn Any)
    }
    // ...
}
```

**Use Case**: Future compositor refactors can be generic over `CompositorAccess` instead of hard-coded `World`.

## Testing
- 6 PostFX tests covering effect coalescence, CRT distortion, coverage aliasing
- Scene compositor types tested indirectly via compositor_system integration tests

## Performance Notes

**PostFX Bottleneck**: Buffer copies between passes. Optimization candidates:
- Async offload to render thread (future)
- Single-pass fusion for consecutive effects of same type
- Dirty region skipping within passes

**Current Status**: ~5ms per frame for typical effect chain (CRT + burn-in + glow) on 160×100 virtual buffer.

## Rendering Coupling Notes

**What Stayed in engine**:
- `compositor_system()` entry point (needs World, asset loading)
- Sprite rendering (depends on ModAssetSourceLoader)
- Image/text/OBJ rendering (depends on asset caching)
- Layout system (interdependent with all renderers)

**Why**: These systems depend on engine's asset loading infrastructure (repositories, ZIP loading, mod sources). Extracting would require first refactoring asset system separately.

**Recommendation**: Keep compositor rendering in engine; document as "rendering service boundary". Future Phase 7 can extract asset system + renderers together.

## Future Improvements
- Async render thread for PostFX
- GPU-accelerated effects (if terminal supports)
- Effect chaining syntax in YAML (e.g., `crt+glow` operator)
- Effect parameter animation over time
