# Optimization, Strategy & Benchmark — Progress Report

> Generated from codebase state as of commit `dd60a1d` + uncommitted `--opt`/buffer-metrics work.

---

## File Tree

```
shell-quest/
├── app/
│   └── src/
│       └── main.rs                          # CLI flags: --opt, --opt-comp/present/diff, --bench <SECS>
│
├── engine-core/
│   └── src/
│       ├── buffer.rs                        # Double-buffer: dirty tracking, diff, write_count, last_diff_count
│       ├── effects/
│       │   └── builtin/
│       │       ├── fade.rs                  # FadeOut glitch fix (transparent at p≥0.999)
│       │       └── whiteout.rs              # Whiteout glitch fix (skip transparent cells)
│       └── strategy/
│           ├── mod.rs                       # re-exports: DiffStrategy, ModEffectFactory
│           ├── diff.rs                      # trait DiffStrategy → FullScanDiff, DirtyRegionDiff
│           └── effect_factory.rs            # trait ModEffectFactory (mod-defined effects hook)
│
├── engine/
│   └── src/
│       ├── bench.rs                         # Benchmark infra: FrameSample, BenchmarkState, BenchResults, report writer
│       ├── game_loop.rs                     # Per-frame timing for 11 systems + buffer metrics → BenchmarkState
│       ├── lib.rs                           # EngineConfig (bench_secs, opt_*), BenchmarkState registration
│       ├── pipeline_flags.rs                # PipelineFlags struct (read at startup → constructs strategies)
│       ├── systems/
│       │   ├── renderer.rs                  # Strategy dispatch (diff, flush, present), diff count tracking
│       │   └── scene_lifecycle.rs           # begin_leave fix (stage_elapsed_ms = 0)
│       └── strategy/
│           ├── mod.rs                       # PipelineStrategies container, from_flags(), default_safe()
│           ├── layer.rs                     # trait LayerCompositor → ScratchLayerCompositor, DirectLayerCompositor
│           ├── halfblock.rs                 # trait HalfblockPacker → FullScanPacker, DirtyRegionPacker
│           ├── present.rs                   # trait VirtualPresenter → AlwaysPresenter, HashSkipPresenter
│           ├── flush.rs                     # trait TerminalFlusher → AnsiBatchFlusher, NaiveFlusher
│           ├── scene_compositor.rs          # trait SceneCompositor → CellSceneCompositor, HalfblockSceneCompositor
│           └── behavior_factory.rs          # trait BehaviorFactory → BuiltInBehaviorFactory
│
├── reports/
│   └── benchmark/                           # (gitignored) timestamped .txt benchmark reports
│       ├── 20260324-175833.txt
│       └── 20260324-175852.txt
│
└── .gitignore                               # includes reports/benchmark/
```

---

## Strategy Pattern — Traits & Implementations

| # | Trait | Crate | File | Implementations | CLI Gate |
|---|-------|-------|------|-----------------|----------|
| 1 | `DiffStrategy` | engine-core | `strategy/diff.rs` | **FullScanDiff** (safe), **DirtyRegionDiff** | `--opt-diff` |
| 2 | `LayerCompositor` | engine | `strategy/layer.rs` | **ScratchLayerCompositor** (safe), **DirectLayerCompositor** | `--opt-comp` |
| 3 | `HalfblockPacker` | engine | `strategy/halfblock.rs` | **FullScanPacker** (safe), **DirtyRegionPacker** | `--opt-comp` |
| 4 | `VirtualPresenter` | engine | `strategy/present.rs` | **AlwaysPresenter** (safe), **HashSkipPresenter** | `--opt-present` |
| 5 | `TerminalFlusher` | engine | `strategy/flush.rs` | **AnsiBatchFlusher** (default), **NaiveFlusher** (debug) | — |
| 6 | `SceneCompositor` | engine | `strategy/scene_compositor.rs` | **CellSceneCompositor**, **HalfblockSceneCompositor** | auto per scene |
| 7 | `BehaviorFactory` | engine | `strategy/behavior_factory.rs` | **BuiltInBehaviorFactory** (10 built-in behaviors) | — |
| 8 | `ModEffectFactory` | engine-core | `strategy/effect_factory.rs` | *(mod-defined, future)* | — |

### PipelineStrategies Container

```
struct PipelineStrategies {
    diff:      Box<dyn DiffStrategy>,       // FullScanDiff | DirtyRegionDiff
    layer:     Box<dyn LayerCompositor>,     // ScratchLayerCompositor | DirectLayerCompositor
    halfblock: Box<dyn HalfblockPacker>,     // FullScanPacker | DirtyRegionPacker
    present:   Box<dyn VirtualPresenter>,    // AlwaysPresenter | HashSkipPresenter
    flush:     Box<dyn TerminalFlusher>,     // AnsiBatchFlusher | NaiveFlusher
}
```

Registered once in `World` at startup. Systems call trait methods — zero boolean branching in hot loops.

**Current state:** `from_flags()` always returns `default_safe()` — all optimizations disabled while we re-enable them one-by-one with benchmark verification.

---

## CLI Flags

| Flag | Default | Purpose | Status |
|------|---------|---------|--------|
| `--opt-comp` | off | Skip scratch buffer for effectless layers + dirty-region halfblock packing | **disabled** |
| `--opt-present` | off | Hash-based frame skip when virtual buffer unchanged | **disabled** (known bug: skips fill) |
| `--opt-diff` | off | DirtyRegionDiff instead of FullScanDiff (~90% diff savings) | **disabled** (unsafe if dirty invariants break) |
| `--opt` | off | Enable ALL of the above at once | **disabled** (all three above disabled) |
| `--bench <SECS>` | — | Run benchmark for N seconds, show score, write report | **working** |

---

## Benchmark System

### What it measures (per frame)

| Metric | Description |
|--------|-------------|
| `frame_us` | Total wall-clock frame time |
| `input_us` | Crossterm event polling |
| `lifecycle_us` | Scene lifecycle / event drain |
| `animator_us` | Stage/step animator tick |
| `hot_reload_us` | Dev hot-reload check |
| `engine_io_us` | Sidecar IPC bridge |
| `behavior_us` | Rhai script + built-in behaviors |
| `audio_us` | Audio system tick |
| `compositor_us` | Layer compositing + sprite rendering |
| `postfx_us` | Post-processing effects |
| `renderer_us` | Diff + flush to terminal |
| `sleep_us` | Frame budget sleep |
| `diff_cells` | Cells changed (back ≠ front) |
| `dirty_cells` | Cells inside dirty region |
| `total_cells` | W × H |
| `write_ops` | Buffer mutation count |

### Report output

- **In-game:** Big-font score + FPS + system breakdown rendered on terminal
- **File:** `reports/benchmark/<YYYYMMDD-HHMMSS>.txt` with full statistics
- **Stats per metric:** avg, min, max, p50, p95, p99
- **Budget chart:** ASCII bar graph showing % of frame time per system
- **Buffer pipeline:** Diff/dirty cell counts and coverage percentages
- **Score formula:** `fps.avg × 10 + (1M / frame.p50) × 5 − frame.p99 / 100`

### Usage

```bash
cargo run -p app -- --bench 5                      # 5s baseline
cargo run -p app -- --opt --bench 5                # 5s with ALL optimizations
cargo run -p app -- --opt-comp --bench 5           # 5s with compositor opts only
```

---

## Bug Fixes (committed `e3fab54`)

| Bug | Root Cause | Fix | File |
|-----|-----------|-----|------|
| "SHELL QUEST I" sprite shows rectangle during transition | `FadeOutEffect` set bg to `TRUE_BLACK` (opaque) instead of transparent | Preserve `cell.bg` during fade; at p≥0.999 clear to `(Reset, Reset)` → blit_from skips | `engine-core/src/effects/builtin/fade.rs` |
| WhiteoutEffect processes empty cells | Transparent cells were being colour-lerped toward white | Skip cells where `symbol==' ' && bg==Reset` | `engine-core/src/effects/builtin/whiteout.rs` |
| NaiveFlusher sends raw Color::Reset | Terminal receives unresolved Reset instead of RGB | Call `resolve_color()` on fg/bg before writing | `engine/src/strategy/flush.rs` |
| Transition flicker on scene leave | `stage_elapsed_ms` not reset in `begin_leave()` | Added `a.stage_elapsed_ms = 0` | `engine/src/systems/scene_lifecycle.rs` |

---

## Optimization Re-enablement Plan

All optimizations were reverted in `f5ae73e` to establish a clean baseline. Re-enablement proceeds one-by-one with benchmark comparison:

| Order | Optimization | Fix Required Before Re-enabling |
|-------|-------------|--------------------------------|
| 1 | `--opt-comp` (DirectLayerCompositor + DirtyRegionPacker) | GIF frame changes don't trigger dirty-rect invalidation |
| 2 | `--opt-present` (HashSkipPresenter) | Skipped frames also skip `fill()` → breaks diff |
| 3 | `--opt-diff` (DirtyRegionDiff) | Misses regions outside dirty bounds during scene transitions |

**Invariant:** `full_redraw_on_scene_change` flag exists in PipelineFlags but is never consumed. Must wire into scene transition path to force full-buffer diff on scene change.

---

## Broader Strategy Opportunities (planned, not yet implemented)

| # | Trait | Subsystem | Implementations |
|---|-------|-----------|-----------------|
| 9 | `SidecarTransport` | engine-io | StdioTransport, TcpTransport, NullTransport |
| 10 | `ModSource` | engine | DirectoryModSource, ZipModSource |
| 11 | `DiagnosticSink` | engine | OverlaySink, FileSink, NullSink |

### Already well-designed (no changes needed)

- **Audio** — `AudioBackend` trait with `NullAudioBackend` + `RodioAudioBackend`
- **Assets** — `SourceLoader` → `SourceAdapter` → `AssetCache` (fs + zip)
- **Effects** — `Effect` trait with 17 built-in implementations + `EffectDispatcher` registry

---

## Git History (optimization-related)

```
dd60a1d feat: comprehensive benchmark mode (--bench <SECS>)
f5ae73e refactor: disable all pipeline optimizations — safe defaults only
12e1b84 feat: SceneCompositor and BehaviorFactory strategy traits
502cb4c refactor: true strategy dispatch — remove boolean oracles, strategies own execution
9c52150 docs: update OPTIMIZATION_PLAN with strategy architecture and --opt-diff
f8246aa feat: beyond-pipeline strategy traits (Phase 3)
8d9c065 feat: strategy pattern scaffolding (Phase 2)
e3fab54 fix: visual glitches in fade-out, whiteout, and NaiveFlusher
b746e65 fix: visual glitches + add --opt-diff flag
92bc873 revert: rollback async render pipeline, keep new splash screen
```
