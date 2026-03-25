# Shell Quest Engine Benchmarking System

Comprehensive performance benchmarking for all optimization flags with per-scene breakdown.

## Quick Start

### Run a Single Benchmark

```bash
# Baseline (no flags), 10 seconds (recommended)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10

# With opt-rowdiff, 10 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-rowdiff

# All optimizations, 10 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt
```

> **Note:** Splash is automatically skipped when `--bench` is active. No need for `--skip-splash`.

### Aggregate Existing Reports to CSV

```bash
python3 collect-benchmarks.py
```

This creates `reports/benchmark/results.csv` with all existing benchmark data including per-scene breakdown.

---

## How It Works

1. **Run benchmarks** with `--bench <SECS>` flag (splash auto-skipped)
2. **App displays results** in-game on a full-screen display
3. **Report is saved** to `reports/benchmark/TIMESTAMP.txt` (includes per-scene breakdown)
4. **Run aggregator** to convert all reports to CSV
5. **Open CSV** in your spreadsheet for analysis

### Test Mod Looping

The test mod (`mods/shell-quest-tests`) loops continuously — scene 04 transitions back to scene 00. One full loop takes ~9.4 seconds, so `--bench 10` covers all 5 scenes.

---

## Optimization Flags

### Available

| Flag | Description | Est. Impact |
|------|-------------|------------|
| `--opt-comp` | Layer scratch skip, dirty-region halfblock packing | ~5% |
| `--opt-diff` | Dirty-region diff scan (experimental) | ~20% |
| `--opt-present` | Hash-based static frame skip | ~0-5% |
| `--opt-skip` | Unified frame-skip oracle (prevents flickering) | ~0-5% |
| `--opt-rowdiff` | Row-level dirty skip in diff scan | ~10-20% |
| `--opt` | All optimizations | ~30-50% |

### Recommended Combinations

1. **Baseline**: No flags
2. **Safe**: `--opt-skip` (prevents flickering)
3. **Balanced**: `--opt-comp --opt-skip --opt-rowdiff`
4. **Aggressive**: `--opt` (all flags)

---

## Benchmark Scenarios

### Quick Validation (5 seconds)
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5
```
- Covers scenes 00-02 partially
- Fast iteration during development

### Standard Benchmark (10 seconds) — **RECOMMENDED**
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```
- Covers all 5 scenes in one full loop
- Includes the PostFX-heavy scene 04
- Best balance of accuracy and speed

### Extended Benchmark (20 seconds)
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 20
```
- Two full loops for more stable metrics
- Better for comparative analysis

---

## CSV Columns Explained

### Configuration
- `flag_name`: Human-readable flag combination (e.g., "comp+skip+rowdiff")
- `opt_comp`, `opt_diff`, `opt_skip`, `opt_rowdiff`: Individual flag status (ON/off)

### Performance Score
- `score`: Overall performance metric (higher = better)
- `total_frames`: Frames rendered during benchmark
- `fps_avg`, `fps_min`, `fps_max`, `fps_p99`: Frame rate statistics

### Frame Timing (microseconds)
- `frame_time_avg`: Average frame duration
- `frame_time_p50`: Median frame time (50th percentile)
- `frame_time_p99`: Worst 1% of frames (99th percentile)

### System Breakdown (microseconds)
- `comp_time_avg`: Compositor time
- `rend_time_avg`: Renderer time
- `behavior_time_avg`: Behavior/script execution time

### Buffer Pipeline
- `diff_cells_avg`: Cells that changed per frame (average)
- `dirty_cells_avg`: Cells marked dirty per frame (average)
- `dirty_coverage_pct`: Fraction of screen marked dirty
- `diff_coverage_pct`: Fraction of screen that actually changed

### Per-Scene Columns (dynamic)
- `scene_count`: Number of unique scenes captured
- `scene_ids`: Comma-separated scene ID list
- `scene_<ID>_frames`: Frame count per scene
- `scene_<ID>_fps`: Average FPS per scene
- `scene_<ID>_comp`: Compositor time per scene
- `scene_<ID>_pfx`: PostFX time per scene
- `scene_<ID>_rend`: Renderer time per scene

---

## Workflow: Comparing Flags

### 1. Establish Baseline

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### 2. Test Individual Flags

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-rowdiff
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-skip
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-comp --opt-rowdiff
```

### 3. Aggregate Results

```bash
python3 collect-benchmarks.py
```

### 4. Analyze in Spreadsheet

```bash
open reports/benchmark/results.csv
```

Then:
- Sort by `fps_avg` to find fastest combination
- Check `frame_time_p99` for jitter/variance
- Compare per-scene columns to find bottlenecks (e.g., scene 04 PostFX)

---

## Tips & Tricks

### Building for Speed

Use release mode for accurate profiling:
```bash
cargo run --release -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### Identifying Scene Bottlenecks

Check the per-scene breakdown in reports to find which scenes are heaviest:
- **Scene 04** (difficulty-select) has 4 PostFX passes — expect highest PFX times
- **Scenes 00-03** have no PostFX — compare compositor/renderer only

### Detecting Regressions

Compare P99 frame times:
- Good flags: P99 < 1.1 × AVG
- Bad flags: P99 > 1.5 × AVG

### Low FPS Debugging

Check in this order:
1. **PostFX bottleneck** if scene 04 FPS much lower than others
2. **Compositor bottleneck** if `comp_time_avg` > `rend_time_avg`
3. **Renderer bottleneck** if `rend_time_avg` is high
4. **Behavior bottleneck** if `behavior_time_avg` is high

---

## Troubleshooting

### No reports generated

- Verify app ran for full duration (wait for in-game results screen)
- Try: `cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5`

### CSV is empty

- Ensure reports exist: `ls reports/benchmark/*.txt`
- Run aggregator: `python3 collect-benchmarks.py`

### FPS is too low (< 50)

- Running in debug mode? Add `--release` flag
- Resolution too high? See RESOLUTIONS.md
- Scene 04 heavy? This is expected (4 PostFX passes)

