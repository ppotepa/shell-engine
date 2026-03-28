# Shell Quest Optimization Reference

Pipeline: `simulate -> composite -> postfx -> present -> flush_to_terminal`

Optimizations are either always-on (safe) or configurable via CLI flags.
22 of 24 optimizations complete. 2 deferred.

---

## CLI Flags

| Flag           | Scope       | What it gates                                | Default |
|----------------|-------------|----------------------------------------------|---------|
| `--opt-comp`   | Compositor  | Layer scratch skip, dirty-halfblock narrowing | ON      |
| `--opt-present`| Present     | Hash-based static frame skip                 | OFF     |
| `--opt-diff`   | Buffer diff | DirtyRegionDiff (experimental)               | OFF     |
| `--opt-skip`   | Frame skip  | Unified FrameSkipOracle                      | OFF     |
| `--opt-rowdiff`| Buffer diff | Row-level dirty skip                         | ON      |
| `--opt-async`  | I/O         | Async display sink                           | OFF     |
| `--opt`        | All         | Enables all above                            | OFF     |

---

## Strategy Pattern

Each flag maps to a Strategy trait implementation.
Selected at startup via `PipelineStrategies::from_flags()`.

| Flag               | Strategy Type    | Safe Impl             | Experimental Impl       |
|--------------------|------------------|-----------------------|-------------------------|
| `--opt-diff`       | DiffStrategy     | FullScanDiff          | DirtyRegionDiff         |
| `--opt-comp` (layer)| LayerCompositor | ScratchLayerCompositor| DirectLayerCompositor   |
| `--opt-comp` (pack)| HalfblockPacker  | FullScanPacker        | DirtyRegionPacker       |
| `--opt-present`    | VirtualPresenter | AlwaysPresenter       | HashSkipPresenter       |
| `--opt-skip`       | FrameSkipOracle  | NeverSkipOracle       | UnifiedFrameSkipOracle  |
| `--opt-rowdiff`    | DiffStrategy     | FullScanDiff          | RowSkipDiff             |

---

## Implementation Status

| #  | ID                     | Status             | Notes                              |
|----|------------------------|--------------------|------------------------------------|
| 1  | opt-term-bufwrite      | Always on          | BufWriter 64KB wraps stdout        |
| 2  | opt-term-colorstate    | Always on          | Skip redundant SetColor ANSI       |
| 3  | opt-term-ansibuf       | Always on          | Single write_all per frame         |
| 4  | opt-comp-layerscratch  | Gated --opt-comp   | Direct render when no effects      |
| 5  | opt-comp-halfblock     | Gated --opt-comp   | Pack only dirty-region rows        |
| 6  | opt-comp-effectsref    | Always on          | Raw pointer avoids Vec clone       |
| 7  | opt-postfx-swap        | Always on          | copy_back_from skips front copy    |
| 8  | opt-postfx-passes      | Always on          | All passes use copy_back_from      |
| 9  | opt-img-sheetview      | Always on          | Zero-copy ImageView                |
| 10 | opt-img-quadstack      | Always on          | Stack arrays in quadblock/braille  |
| 11 | opt-sim-objstates      | Always on          | Gen-counter snapshot skip          |
| 12 | opt-sim-rhaiscope      | Always on          | BEHAVIOR_SCOPES rewind             |
| 13 | opt-present-skipstatic | Gated --opt-present| Buffer hash frame skip             |
| 14 | opt-present-fitlut     | Always on          | Precomputed x/y LUT               |
|    | opt-diff               | Gated --opt-diff   | DirtyRegionDiff strategy           |
|    | opt-skip               | Gated --opt-skip   | FrameSkipOracle                    |
|    | opt-rowdiff            | Gated --opt-rowdiff| Row-level dirty skip               |
|    | ANSI reduction         | Always on          | Skip MoveTo, use MoveRight         |
|    | opt-async              | Gated --opt-async  | AsyncDisplaySink                   |
| 15 | opt-comp-skipidle      | Deferred           | Invasive dirty tracking            |
| 16 | opt-postfx-earlyret    | Always on          | Early return when no passes        |
| 17 | opt-comp-regioncache   | Deferred           | Already O(1) HashMap               |
| 18 | opt-buf-cellpack       | Deferred           | Major SoA refactor                 |
| 19 | opt-mem-glowevict      | Already on         | 128-entry GLOW_CACHE               |
| 20 | opt-comp-borrowstr     | Deferred           | Invasive lifetime propagation      |

---

## Summary Stats

- Always on: 10 optimizations (safe, no flag needed)
- Gated behind flags: 7 optimizations
- Already in codebase: 3
- Deferred: 4

---

## Key Invariants

- `fill()` marks entire buffer dirty -- never reset dirty after fill (causes ghosting).
- PostFX must preserve combined dirty region across all passes.
- FrameSkipOracle prevents animation flickering with content hash.

---

## Running with Optimizations

```bash
# All optimizations
cargo run -p app -- --opt

# Disable defaults for A/B comparison
cargo run -p app -- --no-opt-comp --no-opt-rowdiff

# Benchmark with optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt
```
