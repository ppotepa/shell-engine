# Performance Optimization Tracker

Ranked by estimated impact. **Risk**: low/medium/high (chance of regression). **Gains**: micro/small/medium/large/huge.

---

## Critical — Terminal I/O & Buffer Output

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 1 | `engine/src/buffer.rs` | `render_to_stdout` | Full-buffer flush every frame even when most cells unchanged. RLE batching helps but still emits unchanged regions. | medium | **huge** | Implement dirty-rect tracking: only emit cells that changed since last frame. Keep a "previous frame" buffer and diff. |
| 2 | `engine/src/buffer.rs` | `blit_from` | Per-cell copy loop with bounds checking on every cell. Hot path called per-layer per-frame. | low | **large** | Use `unsafe` memcpy for contiguous row spans when source/dest are aligned. Skip rows entirely if source row is all-transparent. |
| 3 | `engine/src/buffer.rs` | `Cell` struct | Each Cell stores `Option<Color>` for fg/bg plus `char` — 24+ bytes per cell. Large buffers waste cache. | medium | **large** | Pack Cell into 8-12 bytes: use u32 for packed RGBA fg/bg, u32 for char. Halves memory bandwidth for blit/diff. |
| 4 | `engine/src/buffer.rs` | `fill()` | Clears entire buffer every frame even if about to be fully overwritten. | low | **medium** | Track "fully covered" layers; skip fill when subsequent blits will overwrite all cells. |
| 5 | `engine/src/systems/compositor/layer_compositor.rs` | `LAYER_SCRATCH` | Thread-local scratch buffer allocated once but `.fill()` called every layer every frame. | low | **medium** | Only clear cells that were actually written (track dirty region per layer). |

## Critical — 3D Rendering Pipeline

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 6 | `engine/src/systems/compositor/obj_render.rs` | face sorting | `sort_by` with `partial_cmp` on face centroids every frame for animated objects. O(n log n) per frame. | low | **large** | Use radix sort or bucket sort on quantized depth. For static objects, cache sorted order. |
| 7 | `engine/src/systems/compositor/obj_render.rs` | vertex projection | Each vertex multiplied by MVP matrix individually. No SIMD. | medium | **large** | Use `glam` or manual SIMD for 4×4 matrix × vec4 batch transforms. Process 4 vertices at once with SSE. |
| 8 | `engine/src/systems/compositor/obj_render.rs` | face shading | Per-face normal dot-product with N lights. Branchy code with per-light colour blending. | low | **medium** | Pre-sort lights by type, vectorize dot products. For static lights, cache shading per face normal. |
| 9 | `engine/src/systems/compositor/obj_render.rs` | depth buffer | Per-pixel f32 depth buffer cleared every frame. | low | **medium** | Use u16 quantized depth — halves memory, faster clears, better cache utilization. |
| 10 | `engine/src/systems/compositor/obj_render.rs` | triangle rasterization | Scanline rasterizer with per-pixel branch for depth test + color blend. | medium | **medium** | Use fixed-point arithmetic for edge walking. Batch pixels in tile groups for better cache locality. |
| 11 | `engine/src/systems/compositor/obj_render.rs` | OBJ parsing | `load_obj` called every render for non-prerendered sprites. Parses text OBJ each time. | low | **huge** | Cache parsed OBJ meshes in a global `HashMap<String, Arc<Mesh>>`. Parse once, reuse forever. |
| 12 | `engine/src/systems/compositor/obj_render.rs` | backface culling | Done after projection instead of before. Wastes projection work on invisible faces. | low | **medium** | Cull in model space before projection using camera-relative normal test. |

## Critical — Post-Processing Effects

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 13 | `engine/src/postfx/crt.rs` | CRT effect | Per-pixel `sin()`/`cos()` for scanline simulation. Transcendental functions are expensive. | low | **large** | Use lookup table (256-entry sin/cos LUT). Scanline pattern repeats — compute one period and tile. |
| 14 | `engine/src/postfx/crt.rs` | barrel distortion | Per-pixel coordinate remapping with sqrt and division. | low | **medium** | Pre-compute distortion map once on resize. Apply as index lookup per frame. |
| 15 | `engine/src/postfx/glow.rs` | glow/bloom | Blur pass is O(radius²) per pixel — uses box kernel. | low | **large** | Separable two-pass blur (horizontal then vertical): O(radius) per pixel. For large radii, use downscale→blur→upscale. |
| 16 | `engine/src/postfx/burn_in.rs` | burn-in blend | Clones entire buffer for alpha blending with previous frame. | low | **medium** | Blend in-place using the previous frame reference. Avoid the clone. |
| 17 | `engine/src/postfx/mod.rs` | postfx pipeline | Each effect receives and returns a `Buffer` — multiple full-buffer copies per frame. | medium | **large** | Chain effects in-place on a single mutable buffer. Only allocate scratch when effect truly needs double-buffering. |
| 18 | `engine/src/effects/blur.rs` | gaussian blur | Naive O(w×h×r²) implementation. Called potentially multiple times per frame. | low | **large** | Two-pass separable Gaussian. Use integer approximation (stack blur / box blur cascade). |

## High — Text & Sprite Rendering

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 19 | `engine/src/systems/compositor/sprite_renderer.rs` | text sprites | Font glyph lookup per character per frame. String allocation for each text segment. | low | **medium** | Cache rendered text lines. Only re-render when text content or style changes. |
| 20 | `engine/src/systems/compositor/sprite_renderer.rs` | image sprites | Image decoded and scaled every frame for non-prerendered images. | low | **large** | Cache decoded+scaled image buffers keyed by (path, target_w, target_h). |
| 21 | `engine/src/systems/compositor/sprite_renderer.rs` | layout computation | Flex/Grid layout recalculated every frame even when nothing changed. | medium | **medium** | Cache layout results. Invalidate only when container size or children change. |
| 22 | `engine/src/font.rs` | glyph rendering | Per-character HashMap lookup for font glyphs. | low | **small** | Use a flat array indexed by char code for ASCII range (0-127). Fall back to HashMap for extended chars only. |
| 23 | `engine/src/text_render.rs` | text wrapping | Re-wraps text every frame. Allocates intermediate String/Vec per line. | low | **medium** | Cache wrap results keyed by (text, max_width). Use `SmallVec` for line segments. |

## High — Data Structures & Allocations

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 24 | `engine/src/systems/compositor/obj_render.rs` | per-frame Vec allocs | `Vec<Face>`, `Vec<ProjectedVertex>` allocated every render call. | low | **medium** | Use thread-local pooled Vecs (clear + reuse pattern like `OBJ_CANVAS`). |
| 25 | `engine/src/effects/*.rs` | effect apply functions | Many effects allocate `Vec<Cell>` or clone buffer regions per frame. | low | **medium** | Pre-allocate scratch buffers in thread-local storage. Reuse across frames. |
| 26 | `engine/src/scene_runtime.rs` | `HashMap<String, ObjectRuntimeState>` | String-keyed HashMap for object states. Hashing strings is slow for hot lookups. | medium | **medium** | Use interned string IDs (u32/u64) or `FxHashMap` for faster hashing. |
| 27 | `engine/src/world.rs` | `TypeId` lookups | `HashMap<TypeId, Box<dyn Any>>` for world resources. Boxing + downcasting per access. | medium | **small** | Use `FxHashMap` for TypeId keys (already random, don't need SipHash). Consider typed resource slots for hot resources. |
| 28 | `engine/src/systems/animator.rs` | animation state | Allocates interpolation results per animated property per frame. | low | **small** | Pre-allocate animation scratch. Use stack-allocated arrays for common cases (≤8 properties). |

## High — Scene & Asset Loading

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 29 | `engine/src/assets.rs` | zip archive | Archive reopened/re-parsed for each asset load. | low | **large** | Keep archive handle open. Cache directory listing. Use memory-mapped I/O for zip contents. |
| 30 | `engine/src/scene_loader.rs` | YAML parsing | Scene YAML parsed with serde_yaml on every scene load. No caching. | low | **medium** | Cache parsed Scene objects. For development, use file modification time to invalidate. |
| 31 | `engine-core/src/scene/sprite.rs` | Sprite enum | Large enum variant (Sprite::Obj has 40+ fields) — every Sprite match arm pays size cost. | medium | **small** | Box the large variants: `Obj(Box<ObjSprite>)`. Reduces enum size from ~300 bytes to ~8 bytes, improving cache for sprite iteration. |
| 32 | `engine/src/obj_loader.rs` | OBJ file parsing | Text-based OBJ parsing with per-line String allocation and split(). | low | **medium** | Use `memchr` for line splitting. Parse floats with `fast-float` crate. Pre-allocate vertex/face Vecs based on file size heuristic. |

## Medium — Scripting & Behaviors

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 33 | `engine/src/systems/behaviors/rhai_behavior.rs` | Rhai scope | Scope recreated from scratch every behavior tick. Copies all variables in. | low | **medium** | Persist Scope across ticks. Only update changed variables. |
| 34 | `engine/src/systems/behaviors/rhai_behavior.rs` | script compilation | Scripts recompiled on each scene load. AST not cached across scenes. | low | **medium** | Global AST cache keyed by script path. Scripts rarely change at runtime. |
| 35 | `engine/src/systems/behaviors/*.rs` | behavior dispatch | Linear scan through all behaviors each tick to find active ones. | low | **small** | Index behaviors by stage/step. Only iterate relevant behaviors per tick. |

## Medium — Rendering Modes

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 36 | `engine/src/systems/compositor/obj_render.rs` | halfblock renderer | Per-cell pair processing with branch for upper/lower half. | low | **medium** | Process in 2-row strips. Amortize bounds checking across strip. |
| 37 | `engine/src/systems/compositor/obj_render.rs` | braille renderer | 2×4 dot matrix per cell — 8 depth tests per character cell. | low | **small** | Batch depth tests. Use bitwise operations for dot pattern assembly instead of conditional branches. |
| 38 | `engine/src/systems/compositor/obj_render.rs` | quadblock renderer | Similar to braille but 2×2. Per-quadrant color blending. | low | **small** | Pre-compute quadrant masks. Use SIMD for 4-way color blend. |

## Medium — Frame Timing & Main Loop

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 39 | `engine/src/game_loop.rs` | frame pacing | `thread::sleep` for frame timing — OS scheduler granularity is 1-15ms. | low | **medium** | Use spin-wait for sub-ms precision when close to deadline. Hybrid: sleep for bulk, spin for last 1ms. |
| 40 | `engine/src/game_loop.rs` | system ordering | All systems run sequentially even when independent. | high | **medium** | Identify independent systems (e.g., audio vs rendering) and run in parallel. Requires careful dependency analysis. |
| 41 | `engine/src/game_loop.rs` | input processing | Input polled and processed every frame even when no input available. | low | **small** | Use `crossterm::event::poll(Duration::ZERO)` and skip input processing when no events pending. (May already be done.) |

## Medium — Effects Pipeline

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 42 | `engine/src/effects/chromatic_aberration.rs` | chromatic aberration | Per-pixel channel offset with bounds checking. Three passes (R, G, B). | low | **medium** | Single pass: read 3 offset positions, write combined pixel. Use unchecked indexing for interior pixels. |
| 43 | `engine/src/effects/noise.rs` | noise effect | `rand::thread_rng()` per pixel. RNG is relatively expensive per call. | low | **small** | Use a fast PRNG (xorshift64) seeded per-frame. Or pre-generate a noise texture and tile it. |
| 44 | `engine/src/effects/wave.rs` | wave distortion | Per-row sin() computation for horizontal wave. | low | **small** | Pre-compute sin LUT (one per frame, 256 entries). Index by row. |
| 45 | `engine/src/effects/pixelate.rs` | pixelation | Nested loops with redundant bounds checks per block. | low | **small** | Compute block boundaries once, iterate blocks without per-pixel bounds checks. |
| 46 | `engine/src/effects/fade.rs` | fade effect | Per-pixel color interpolation with f32 math. | low | **small** | Use integer lerp (0-255 range) with shift instead of float multiply. |

## Medium — Compositor Architecture

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 47 | `engine/src/systems/compositor/layer_compositor.rs` | layer compositing | Every layer blitted onto scene buffer regardless of overlap. No occlusion culling. | medium | **medium** | Track fully-opaque layers. Skip blitting layers completely hidden by opaque layers above. |
| 48 | `engine/src/systems/compositor/layer_compositor.rs` | layer effects | Effects applied per-layer even if layer hasn't changed. | medium | **medium** | Cache layer effect results. Only reapply when layer content or effect params change. |
| 49 | `engine/src/systems/compositor/sprite_renderer.rs` | sprite culling | All sprites rendered even if off-screen or fully occluded. | low | **medium** | Bounding-box test before rendering. Skip sprites entirely outside viewport. |

## Lower — Miscellaneous

| # | File | Location | Issue | Risk | Gains | Fix |
|---|------|----------|-------|------|-------|-----|
| 50 | `engine/src/scene_pipeline.rs` | preparation steps | Steps run sequentially. Some could be parallelized. | medium | **small** | Run independent preparation steps in parallel with rayon. |
| 51 | `engine/src/systems/prerender.rs` | prerender | Entire prerender happens synchronously during scene transition. | low | **small** | Already uses rayon. Consider progressive prerender (render visible first, others async). |
| 52 | `engine/src/render_policy.rs` | mode resolution | Called per-sprite per-frame. Small but adds up. | low | **micro** | Inline the function or make it `#[inline(always)]`. |
| 53 | `engine-core/src/logging.rs` | logging | String formatting via `format!()` even when log level is disabled. | low | **small** | Use lazy formatting: `log_if!(level, "msg {}", val)` macro that skips format! when level is off. |
| 54 | `engine/src/systems/compositor/obj_render.rs` | color conversion | `Color::from(TermColour)` called per light per face per frame. | low | **micro** | Cache converted colors at scene load time. |
| 55 | `engine/src/buffer.rs` | `set_cell` | Bounds check on every cell write. Hot path. | low | **small** | Provide `set_cell_unchecked` for internal use where bounds are guaranteed by caller. |
| 56 | `engine/src/scene3d_atlas.rs` | atlas lookup | `HashMap<(String, String), Buffer>` — string pair key hashed per lookup. | low | **small** | Use interned IDs or `FxHashMap`. Pre-resolve keys at scene load. |
| 57 | `engine/src/effects/region.rs` | region clipping | Per-pixel region membership test in some effects. | low | **small** | Pre-compute clipped iteration bounds. Iterate only within region. |
| 58 | `engine/src/systems/behaviors/mod.rs` | behavior matching | String comparison for behavior type dispatch. | low | **micro** | Use enum discriminant or pre-hashed comparison. |
| 59 | `engine/src/image_render.rs` | image scaling | Nearest-neighbor scaling recomputed per frame. | low | **medium** | Cache scaled image. Only rescale on size change. |
| 60 | `engine/src/buffer.rs` | `resize` | Drops and reallocates entire cell vec on resize. | low | **small** | If new size ≤ old capacity, reuse allocation and just update width/height. |
| 61 | `engine/src/systems/compositor/obj_render.rs` | normal calculation | Face normals recalculated every render from vertex positions. | low | **small** | Pre-compute and cache face normals in parsed mesh data. |
| 62 | `engine/src/postfx/mod.rs` | effect chain | Dynamic dispatch via trait objects for each postfx effect. | low | **micro** | Use enum dispatch instead of `dyn` trait. Avoids vtable indirection. |
| 63 | `engine/src/systems/compositor/sprite_renderer.rs` | Scene3D atlas lookup | HashMap lookup per Scene3D sprite per frame. | low | **small** | Resolve atlas index once at scene load, store index in sprite runtime state. |
| 64 | `engine/src/world.rs` | resource access pattern | `world.get::<T>()` does HashMap lookup + downcast per access. Multiple accesses per frame. | low | **small** | Cache frequently-accessed resources in typed fields on a "frame context" struct passed through the pipeline. |
| 65 | `engine/src/effects/typewriter.rs` | typewriter effect | Recalculates visible character count from elapsed time every frame. | low | **micro** | Cache last computed count and only recalculate if elapsed changed. |
| 66 | `engine/src/systems/compositor/obj_render.rs` | MVP matrix | Matrix rebuilt from angles every frame for static objects. | low | **small** | Cache MVP matrix when rotation params haven't changed. |
| 67 | `engine/src/buffer.rs` | stdout write | Uses `BufWriter` but flushes per frame. Crossterm command batching may not be optimal. | low | **medium** | Measure actual write syscall count. Consider `writev` / vectored I/O for fewer syscalls. |
| 68 | `engine/src/systems/compositor/obj_render.rs` | face clipping | Faces partially outside viewport still fully rasterized with per-pixel clip. | medium | **small** | Clip triangles to viewport before rasterization (Sutherland-Hodgman). Avoids wasted scanline work. |
| 69 | `engine/src/postfx/crt.rs` | color channel math | Per-pixel f32 multiply + clamp for RGB channels. | low | **small** | Use u8 fixed-point with lookup tables for brightness curves. |
| 70 | `engine-core/src/scene/model.rs` | Scene struct size | Scene carries all fields in a flat struct — large memcpy on scene transitions. | low | **small** | Box or Arc heavy fields (layers, stages). Cheaper to pass around. |

---

## Priority Tiers Summary

**Tier 1 — Do First (items 1-12):** Terminal I/O diffing, OBJ mesh caching, buffer optimization, SIMD projection, separable blur. These dominate frame time.

**Tier 2 — High Impact (items 13-32):** Postfx LUT-ification, text/image caching, asset caching, allocation reduction. Clear wins with low risk.

**Tier 3 — Medium Impact (items 33-49):** Scripting optimization, rendering mode tuning, compositor improvements. Good gains, some require architectural changes.

**Tier 4 — Polish (items 50-70):** Micro-optimizations, inlining, cache-friendly patterns. Small individual gains but compound nicely.
