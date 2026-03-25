# Optimization, Strategy & Benchmark — Progress Report

> Updated as of `optimizations` branch, March 2026.

---

## File Tree

```
shell-quest/
├── app/
│   └── src/
│       └── main.rs                          # CLI flags: --opt, --opt-comp/present/diff/skip/rowdiff, --bench <SECS>
│
├── engine-core/
│   └── src/
│       ├── buffer.rs                        # Double-buffer: dirty tracking, dirty_rows, diff, write_count, last_diff_count
│       ├── effects/
│       │   └── builtin/
│       │       ├── fade.rs                  # FadeOut glitch fix (transparent at p≥0.999)
│       │       └── whiteout.rs              # Whiteout glitch fix (skip transparent cells)
│       └── strategy/
│           ├── mod.rs                       # re-exports: DiffStrategy, ModEffectFactory
│           ├── diff.rs                      # trait DiffStrategy → FullScanDiff, DirtyRegionDiff, RowSkipDiff
│           └── effect_factory.rs            # trait ModEffectFactory (mod-defined effects hook)
│
├── engine/
│   └── src/
│       ├── bench.rs                         # Benchmark: FrameSample (scene_id), BenchmarkState, BenchResults, per-scene breakdown
│       ├── game_loop.rs                     # Per-frame timing for 11 systems + buffer metrics + scene_id → BenchmarkState
│       ├── lib.rs                           # EngineConfig (bench_secs, opt_*), BenchmarkState registration
│       ├── pipeline_flags.rs                # PipelineFlags struct (opt_comp/present/diff/skip/rowdiff)
│       ├── systems/
│       │   ├── renderer.rs                  # Strategy dispatch (diff, flush, present), ANSI payload optimisation
│       │   └── scene_lifecycle.rs           # begin_leave fix (stage_elapsed_ms = 0)
│       └── strategy/
│           ├── mod.rs                       # PipelineStrategies container, from_flags(), default_safe()
│           ├── display.rs                   # trait DisplaySink → SyncDisplaySink, AsyncDisplaySink (not yet wired)
│           ├── layer.rs                     # trait LayerCompositor → ScratchLayerCompositor, DirectLayerCompositor
│           ├── halfblock.rs                 # trait HalfblockPacker → FullScanPacker, DirtyRegionPacker
│           ├── present.rs                   # trait VirtualPresenter → AlwaysPresenter, HashSkipPresenter
│           ├── flush.rs                     # trait TerminalFlusher → AnsiBatchFlusher, NaiveFlusher
│           ├── scene_compositor.rs          # trait SceneCompositor → CellSceneCompositor, HalfblockSceneCompositor
│           └── behavior_factory.rs          # trait BehaviorFactory → BuiltInBehaviorFactory
│
├── collect-benchmarks.py                    # CSV aggregator: parses report .txt files, per-scene columns
├── benchmark.py                             # Automated multi-scenario runner
├── benchmark.sh                             # Shell batch benchmark runner
│
├── reports/
│   └── benchmark/                           # (gitignored) timestamped .txt benchmark reports
│
├── mods/
│   └── shell-quest-tests/                   # Benchmark test mod: compressed scenes, looping
│       ├── mod.yaml
│       └── scenes/                          # 5 scenes, ~9.4s per loop, loops continuously
│
└── .gitignore                               # includes reports/benchmark/
```

---

## Strategy Pattern — Traits & Implementations

| # | Trait | Crate | File | Implementations | CLI Gate |
|---|-------|-------|------|-----------------|----------|
| 1 | `DiffStrategy` | engine-core | `strategy/diff.rs` | **FullScanDiff** (safe), **DirtyRegionDiff**, **RowSkipDiff** | `--opt-diff` / `--opt-rowdiff` |
| 2 | `LayerCompositor` | engine | `strategy/layer.rs` | **ScratchLayerCompositor** (safe), **DirectLayerCompositor** | `--opt-comp` |
| 3 | `HalfblockPacker` | engine | `strategy/halfblock.rs` | **FullScanPacker** (safe), **DirtyRegionPacker** | `--opt-comp` |
| 4 | `VirtualPresenter` | engine | `strategy/present.rs` | **AlwaysPresenter** (safe), **HashSkipPresenter** | `--opt-present` |
| 5 | `TerminalFlusher` | engine | `strategy/flush.rs` | **AnsiBatchFlusher** (default), **NaiveFlusher** (debug) | — |
| 6 | `SceneCompositor` | engine | `strategy/scene_compositor.rs` | **CellSceneCompositor**, **HalfblockSceneCompositor** | auto per scene |
| 7 | `BehaviorFactory` | engine | `strategy/behavior_factory.rs` | **BuiltInBehaviorFactory** (10 built-in behaviors) | — |
| 8 | `ModEffectFactory` | engine-core | `strategy/effect_factory.rs` | *(mod-defined, future)* | — |
| 9 | `DisplaySink` | engine | `strategy/display.rs` | **SyncDisplaySink** (safe), **AsyncDisplaySink** (not wired) | — |

### PipelineStrategies Container

```
struct PipelineStrategies {
    diff:      Box<dyn DiffStrategy>,       // FullScanDiff | DirtyRegionDiff | RowSkipDiff
    layer:     Box<dyn LayerCompositor>,     // ScratchLayerCompositor | DirectLayerCompositor
    halfblock: Box<dyn HalfblockPacker>,     // FullScanPacker | DirtyRegionPacker
    present:   Box<dyn VirtualPresenter>,    // AlwaysPresenter | HashSkipPresenter
    flush:     Box<dyn TerminalFlusher>,     // AnsiBatchFlusher | NaiveFlusher
    display:   Box<dyn DisplaySink>,         // SyncDisplaySink | AsyncDisplaySink
}
```

Registered once in `World` at startup. Systems call trait methods — zero boolean branching in hot loops.

**Current state:** `from_flags()` selects implementations based on `--opt-*` CLI flags. `default_safe()` returns all-safe defaults.

---

## CLI Flags

| Flag | Default | Purpose | Status |
|------|---------|---------|--------|
| `--opt-comp` | off | Skip scratch buffer for effectless layers + dirty-region halfblock packing | **working** (fixes applied) |
| `--opt-present` | off | Hash-based frame skip when virtual buffer unchanged | **working** (disables postfx cache for correctness) |
| `--opt-diff` | off | DirtyRegionDiff instead of FullScanDiff (~90% diff savings) | **experimental** (dirty invariant edge cases) |
| `--opt-skip` | off | FrameSkipOracle — skip redundant frames by content hash | **working** |
| `--opt-rowdiff` | off | RowSkipDiff — row-level dirty tracking, skip unchanged rows | **working** |
| `--opt` | off | Enable ALL of the above at once | **working** |
| `--bench <SECS>` | — | Run benchmark for N seconds, per-scene breakdown, write report | **working** |

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
- **Per-scene breakdown:** Frame count, FPS, compositor/postfx/renderer/behavior per scene
- **Score formula:** `fps.avg × 10 + (1M / frame.p50) × 5 − frame.p99 / 100`

### Usage

```bash
# Recommended: 10-second benchmark with test mod (covers all scenes)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10

# With optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt --bench 10

# Specific flags
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt-comp --opt-skip --bench 10
```

### CSV Aggregation

```bash
# Parse all .txt reports into a single CSV
python collect-benchmarks.py reports/benchmark/ reports/benchmark/results.csv
```

Output includes per-scene columns: `scene_<ID>_frames`, `scene_<ID>_fps`, `scene_<ID>_comp`, `scene_<ID>_pfx`, `scene_<ID>_rend`.

---

## Bug Fixes (committed `e3fab54`)

| Bug | Root Cause | Fix | File |
|-----|-----------|-----|------|
| "SHELL QUEST I" sprite shows rectangle during transition | `FadeOutEffect` set bg to `TRUE_BLACK` (opaque) instead of transparent | Preserve `cell.bg` during fade; at p≥0.999 clear to `(Reset, Reset)` → blit_from skips | `engine-core/src/effects/builtin/fade.rs` |
| WhiteoutEffect processes empty cells | Transparent cells were being colour-lerped toward white | Skip cells where `symbol==' ' && bg==Reset` | `engine-core/src/effects/builtin/whiteout.rs` |
| NaiveFlusher sends raw Color::Reset | Terminal receives unresolved Reset instead of RGB | Call `resolve_color()` on fg/bg before writing | `engine/src/strategy/flush.rs` |
| Transition flicker on scene leave | `stage_elapsed_ms` not reset in `begin_leave()` | Added `a.stage_elapsed_ms = 0` | `engine/src/systems/scene_lifecycle.rs` |

---

## Optimization Re-enablement Status

All optimizations reverted in `f5ae73e`, then re-enabled one-by-one with bug fixes:

| Optimization | Status | Fix(es) Applied |
|-------------|--------|-----------------|
| `--opt-comp` | ✅ working | Force scratch for timed sprites (f330c3b), image dirty tracking, fill dirty preservation (d8aeb2c) |
| `--opt-present` | ✅ working | Disable postfx cache when present active (4b6f06c) |
| `--opt-diff` | ⚠️ experimental | PostFX dirty region preservation (877ac10); still fragile with edge cases |
| `--opt-skip` | ✅ working | FrameSkipOracle prevents redundant frame processing |
| `--opt-rowdiff` | ✅ working | Row-level dirty tracking in Buffer, skip unchanged rows in diff |

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
