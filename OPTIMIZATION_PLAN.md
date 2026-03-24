## Pipeline Optimization Roadmap — 20 Ranked --optimize-* Flags

**Goal:** Gradually introduce optimizations behind CLI flags to avoid regression.
**Pipeline:** simulate -> composite -> postfx -> present -> flush_to_terminal

---

### #1 — --optimize-buffered-stdout
**Impact: 5/5 | Gain: 30-60% renderer_us**
**Files:** engine/src/systems/renderer.rs

Steps:
1. In TerminalRenderer struct (line 13), change stdout: io::Stdout to stdout: io::BufWriter<io::Stdout>
2. In TerminalRenderer::new() (line 25), wrap: let stdout = io::BufWriter::with_capacity(65536, io::stdout())
3. In flush_batched() (line 529), change param from &mut io::Stdout to &mut io::BufWriter<io::Stdout>
4. The queue!() calls (lines 556-597) already write via Write trait — BufWriter collects them; single flush at line 599
5. clear_black() / reset_console() / shutdown() use self.stdout — unchanged; crossterm macros work on any Write
6. Add --optimize-buffered-stdout flag to app/src/main.rs (clap bool arg)
7. Pass through EngineConfig -> add field pub optimize_buffered_stdout: bool
8. In engine/src/lib.rs run(), read flag: when OFF use raw io::Stdout; when ON use BufWriter
9. Add pub optimize_buffered_stdout: bool to PipelineFlags, default false
10. Test: cargo test -p engine then cargo run -p app -- --optimize-buffered-stdout --dev
11. Verify in debug overlay: renderer_us should drop significantly

Why: io::Stdout locks/unlocks mutex on EVERY write(). ~500 queue!() calls/frame = 500 lock cycles. BufWriter batches into one flush.

---

### #2 — --optimize-color-state
**Impact: 4/5 | Gain: 20-40% less ANSI bytes**
**Files:** engine/src/systems/renderer.rs

Steps:
1. In flush_batched(), add before loop: let mut active_fg = style::Color::Reset; let mut active_bg = style::Color::Reset;
2. Extract helper fn emit_run() — skip SetForegroundColor when rfg == active_fg; skip SetBackgroundColor when rbg == active_bg; emit MoveTo only when cursor not at correct pos; queue Print(run); update cursor position
3. Replace all 4 emit sites (lines 555-570, 583-597) with calls to emit_run()
4. Add pub optimize_color_state: bool to PipelineFlags, default false
5. When flag OFF: always emit all commands (current); when ON: skip redundant
6. Test: all scenes, verify no color glitches

Why: Consecutive runs often share fg or bg. Each SetForegroundColor(Rgb) = ~20 bytes ANSI. Skip saves 30-50% of color commands.

---

### #3 — --optimize-write-buffer
**Impact: 4/5 | Gain: 1 syscall/frame**
**Files:** engine/src/systems/renderer.rs

Steps:
1. Add thread-local: static ANSI_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(65536));
2. In flush_batched(), borrow: let mut ansi = ANSI_BUF.borrow_mut(); ansi.clear();
3. Replace all queue!(stdout, ...) with queue!(&mut *ansi, ...)
4. After loop: stdout.write_all(&ansi)?; stdout.flush()?;
5. Entire frame ANSI goes into contiguous Vec<u8>, then one write_all syscall
6. Alternative to #1. Can combine: Vec -> BufWriter -> Stdout
7. Add pub optimize_write_buffer: bool to PipelineFlags, default false
8. Test: visual correctness across all scenes

Why: Each queue!(stdout, ...) triggers write(). Vec<u8> absorbs all writes as memcpy, then single write_all + flush.

---

### #4 — --optimize-layer-scratch
**Impact: 4/5 | Gain: 2x fewer buffer fills**
**Files:** engine/src/systems/compositor/layer_compositor.rs, effect_applicator.rs

Steps:
1. Add helper: fn layer_has_effects(layer, stage, step_idx) -> bool
2. In composite_layers() loop (line 37), before LAYER_SCRATCH: let needs_scratch = layer_has_effects(...)
3. When needs_scratch == false: call render_sprites() with buffer directly (scene buffer). Skip apply_layer_effects and blit_from.
4. When needs_scratch == true: keep current scratch path
5. Add pub optimize_layer_scratch: bool to PipelineFlags, default false
6. Guard: transparent sprites need scratch to avoid overwriting lower layers
7. Test: scenes with multiple overlapping layers, verify Z-order

Why: Scratch path = fill(WxH) + render + blit(WxH) = 3 passes. Direct = render only = 1 pass.

---

### #5 — --optimize-halfblock-pack
**Impact: 4/5 | Gain: 80-95% on idle**
**Files:** engine/src/systems/compositor/mod.rs, engine-core/src/buffer.rs

Steps:
1. Flag exists: PipelineFlags::experimental_dirty_halfblock (pipeline_flags.rs:42)
2. Add to Buffer: pub fn dirty_bounds() -> (u16,u16,u16,u16)
3. Add: pub fn has_dirty() -> bool
4. In composite_scene_halfblock(), after composite_scene(): read virtual_buf.dirty_bounds()
5. Convert virtual dirty to output coords
6. Change pack_halfblock_buffer() to accept optional dirty range
7. When flag ON + dirty Some: iterate only dirty rows/cols
8. When flag OFF: current full-scan (safe)
9. Handle generation mismatch: pass dirty=None for full repack
10. Test: halfblock idle (zero pack), animation (partial), transition (full)

Why: On idle, dirty region empty -> skip entire pack. Currently always O(WxH).

---

### #6 — --optimize-scene-effects-ref
**Impact: 3/5 | Gain: eliminate Vec<Effect> clone/frame**
**Files:** engine/src/systems/compositor/mod.rs

Steps:
1. Line 99: scene_effects = current_step.map(|s| s.effects.clone())
2. Use raw pointer (same pattern as layers_ptr at line 46)
3. After borrow scope, recover reference: unsafe { &*effects_ptr } or &[] if null
4. SAFETY: same as layers_ptr — Scene not mutated until scene_runtime_mut() at function end
5. No flag needed — zero behavioral change
6. Test: cargo test -p engine, visual verification

Why: Avoids cloning Vec<Effect> containing Strings every frame. Data lives in Scene which outlives function.

---

### #7 — --optimize-postfx-swap
**Impact: 4/5 | Gain: halve memcpy in postfx**
**Files:** engine/src/systems/postfx.rs, engine-core/src/buffer.rs

Steps:
1. Add to Buffer: pub fn copy_back_from(&mut self, src: &Buffer) — only copies back buffer
2. Line 125 (skip-frame): replace buffer.clone_from(cached) with buffer.copy_back_from(cached)
3. Line 188 (cache output): reuse allocation where possible
4. Add pub optimize_postfx_swap: bool to PipelineFlags, default false
5. When OFF: keep clone/clone_from; when ON: use copy_back_from
6. Test: scenes with postfx (CRT, burn-in), verify identical output

Why: clone/clone_from copies both back+front. Cache only needs back buffer. Saves 50% memcpy.

---

### #8 — --optimize-postfx-passes
**Impact: 3/5 | Gain: 50% less memcpy per pass**
**Files:** engine/src/systems/postfx/pass_*.rs (6 files)

Steps per pass — replace dst.clone_from(src) + modify with single-pass read-src-write-dst:
1. pass_underlay.rs: iterate all pixels; if src non-transparent write src else write underlay color
2. pass_crt_distort.rs: iterate dst pixels; compute distorted src coords; sample src; write dst
3. pass_scan_glitch.rs: iterate dst rows; if glitch read src at offset; else read src at same pos
4. pass_crt.rs: single pass: read src, apply CRT scanline+phosphor+curvature, write dst
5. pass_burn_in.rs: refactor each stage to single-pass
6. Add pub optimize_postfx_passes: bool to PipelineFlags
7. Test each pass individually

Why: clone_from writes every pixel, then pass overwrites subset. Single-pass reads+writes once.

---

### #9 — --optimize-spritesheet-view
**Impact: 3/5 | Gain: zero-copy frame selection**
**Files:** engine/src/systems/compositor/image_render.rs, engine/src/image_loader.rs

Steps:
1. Add ImageView struct: source ref + offset + dimensions
2. Add constructors: ImageView::full(img) and ImageView::sub(img, x, y, w, h)
3. Change select_spritesheet_frame() return to ImageView
4. Line 94 (cols=1): return ImageView::full(image) instead of image.clone()
5. Lines 117-123: return ImageView::sub(...) instead of Vec alloc
6. Update rasterize_image_* functions to accept &ImageView
7. No flag needed — pure refactor
8. Test: spritesheet animations, static images

Why: Current code clones entire image per sprite per frame. ImageView = zero-copy view.

---

### #10 — --optimize-quadblock-alloc
**Impact: 3/5 | Gain: kill thousands of heap allocs**
**Files:** engine/src/systems/compositor/image_render.rs

Steps:
1. Change average_rgb() signature: &Vec<[u8;4]> -> &[[u8;4]]
2. In rasterize_image_quadblock(): replace Vec::new() with stack array [0u8;4] x 4, count
3. In rasterize_image_braille(): replace Vec::new() with stack array [0u8;4] x 8, count
4. No flag needed — zero behavioral change
5. Test: quadblock/braille scenes, verify identical pixels

Why: Inner pixel loop allocs Vec per pixel (WxH times). Stack array = zero heap allocs.

---

### #11 — --optimize-object-states
**Impact: 3/5 | Gain: O(1) vs O(N) on idle**
**Files:** engine/src/scene_runtime.rs

Steps:
1. Add field: object_states_dirty: bool to SceneRuntime (initially true)
2. Set dirty=true in every state-mutating method
3. In object_states_snapshot(): if !dirty && cached -> return Arc::clone(cached)
4. Add pub optimize_object_states: bool to PipelineFlags
5. When OFF: always rebuild; when ON: dirty check
6. Test: behaviors that mutate states still update visuals

Why: On idle, no mutations -> O(1) Arc clone vs O(N) HashMap rebuild.

---

### #12 — --optimize-rhai-scope
**Impact: 2/5 | Gain: fewer allocs per behavior**
**Files:** engine/src/systems/behavior.rs

Steps:
1. Identify static scope vars: scene_id, object_id, scene_w, scene_h
2. Identify dynamic vars: elapsed_ms, delta_ms, state, visible, offset_x/y
3. Add per-behavior cache: BehaviorScopeCache { scope, base_len, scene_id }
4. First call: build full scope, record base_len; subsequent: scope.rewind(base_len) + push dynamic
5. Gate behind PipelineFlags::optimize_rhai_scope
6. Clear cache on scene transition
7. Test: Rhai behaviors reading all scope vars

Why: scope.push() for Maps/Arrays allocates. Static vars pushed once and reused.

---

### #13 — --optimize-virtual-present
**Impact: 3/5 | Gain: skip present on static frames**
**Files:** engine/src/systems/renderer.rs, engine-core/src/buffer.rs

Steps:
1. Flag exists: PipelineFlags::adaptive_virtual_present
2. Add to Buffer: write_count: u64 field, increment on set/fill/blit_from
3. In present_virtual_to_output(): if write_count unchanged -> early return
4. Test: static scenes skip present; dynamic scenes update correctly

Why: On idle, compositor writes same pixels -> same write_count -> skip O(viewport_area) present.

---

### #14 — --optimize-virtual-fit-lut
**Impact: 2/5 | Gain: ~2x faster virtual present (Fit mode)**
**Files:** engine/src/systems/renderer.rs

Steps:
1. Add struct FitLut with precomputed x_map/y_map
2. Cache as thread-local, rebuild on resize
3. Replace per-pixel sample_fit_source() with LUT lookup
4. Gate behind PipelineFlags
5. Test: Fit policy virtual buffer, verify pixel-perfect

Why: sample_fit_source does 2 muls + 2 divs per cell. LUT = 1 array index.

---

### #15 — --optimize-compositor-skip
**Impact: 4/5 | Gain: zero compositor cost on idle**
**Files:** engine/src/systems/compositor/mod.rs, game_loop.rs, scene_runtime.rs

Steps:
1. Add compositor_dirty: bool to SceneRuntime
2. Set dirty on: behavior mutation, animator stage/step change, resize, scene load
3. In compositor_system(): if !dirty && flag ON { return; }
4. Edge case: scenes with postfx -> always dirty
5. Gate behind PipelineFlags
6. Test: idle menus (expect skip), interactions (expect responsive)

Why: On truly static frames, compositor output identical to previous.

---

### #16 — --optimize-postfx-skip
**Impact: 2/5 | Gain: eliminate overhead on no-postfx scenes**
**Files:** engine/src/systems/postfx.rs

Steps:
1. At top of postfx_system(): check if scene has no postfx -> early return
2. Skips: scene_id.clone(), fingerprint hash, thread-local borrow
3. No flag needed — safe early return
4. Test: no-postfx scenes run as before; postfx scenes still work

Why: Many scenes have no postfx. Current code does unnecessary work before discovering passes empty.

---

### #17 — --optimize-effect-region
**Impact: 2/5 | Gain: cache region lookups per step**
**Files:** engine/src/systems/compositor/mod.rs

Steps:
1. Cache effect regions per (step_idx, effect_count) key
2. Reuse cached regions when key matches
3. Invalidate on step change
4. Gate behind flag if desired
5. Test: targeted effects, verify correct region

Why: Effect regions static within a step. HashMap lookup eliminated on repeated frames.

---

### #18 — --optimize-cell-pack
**Impact: 2/5 | Gain: 30% less memory, better cache**
**Files:** engine-core/src/buffer.rs

Steps:
1. Measure current Cell size
2. Try #[repr(C)] and field reordering
3. For SoA: separate chars/colors arrays (major refactor)
4. Gate behind compile feature
5. Benchmark with criterion
6. Test: entire test suite

Why: SoA means diff scanning loads only color data. More cells per cache line.

---

### #19 — --optimize-glow-cache-evict
**Impact: 1/5 | Gain: bounded memory**
**Files:** engine/src/systems/compositor/sprite_renderer.rs

Steps:
1. Add LRU tracking to GLOW_CACHE
2. Cap at 128 entries
3. No flag needed — memory safety
4. Test: long sessions, verify bounded memory

Why: Unbounded HashMap grows forever.

---

### #20 — --optimize-object-region-strings
**Impact: 2/5 | Gain: eliminate String heap allocs/frame**
**Files:** layer_compositor.rs, render/common.rs, compositor/mod.rs

Steps:
1. Change HashMap<String, Region> to HashMap<&str, Region>
2. Propagate lifetime through compositor functions
3. Convert to owned only at SceneRuntime handoff
4. No flag needed — pure refactor
5. Test: hit-testing, effect targeting still correct

Why: to_string() per layer per frame = 10-50 heap allocs. Borrowed &str = zero alloc during compositing.

---

## Summary Table

| # | Flag | Impact | Category |
|---|------|--------|----------|
| 1 | --optimize-buffered-stdout | 5/5 | Terminal output |
| 2 | --optimize-color-state | 4/5 | Terminal output |
| 3 | --optimize-write-buffer | 4/5 | Terminal output |
| 4 | --optimize-layer-scratch | 4/5 | Compositor |
| 5 | --optimize-halfblock-pack | 4/5 | Compositor |
| 6 | --optimize-scene-effects-ref | 3/5 | Compositor |
| 7 | --optimize-postfx-swap | 4/5 | PostFX |
| 8 | --optimize-postfx-passes | 3/5 | PostFX |
| 9 | --optimize-spritesheet-view | 3/5 | Image render |
| 10 | --optimize-quadblock-alloc | 3/5 | Image render |
| 11 | --optimize-object-states | 3/5 | Simulation |
| 12 | --optimize-rhai-scope | 2/5 | Simulation |
| 13 | --optimize-virtual-present | 3/5 | Present |
| 14 | --optimize-virtual-fit-lut | 2/5 | Present |
| 15 | --optimize-compositor-skip | 4/5 | Compositor |
| 16 | --optimize-postfx-skip | 2/5 | PostFX |
| 17 | --optimize-effect-region | 2/5 | Compositor |
| 18 | --optimize-cell-pack | 2/5 | Buffer core |
| 19 | --optimize-glow-cache-evict | 1/5 | Memory |
| 20 | --optimize-object-region-strings | 2/5 | Compositor |

## Recommended Implementation Order
Safe starters (no flag needed): #16, #10, #6, #9
High-impact flagged: #1, #7, #2, #15, #5
Medium: #3, #4, #8, #11, #13
Advanced: #12, #14, #17, #18, #20, #19
