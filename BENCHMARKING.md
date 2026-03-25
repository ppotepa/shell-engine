# Shell Quest Benchmark Guide

## Quick Start

The Shell Quest engine has built-in benchmark mode for profiling all optimization flags. Splash is automatically skipped during benchmarks.

### Running Individual Benchmarks

Run any benchmark with the `--bench <SECONDS>` flag:

```bash
# Baseline (no optimizations), 10 seconds — recommended
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10

# With specific optimization
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-rowdiff

# With all optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt
```

> **Note:** `--skip-splash` is not needed — `--bench` automatically skips the splash screen.

### Benchmark Output

The app will:
1. Run the benchmark for the specified duration
2. Display results in-game on a full-screen benchmark screen
3. Save a detailed text report to `reports/benchmark/<TIMESTAMP>.txt`
4. Report includes per-scene breakdown showing FPS/timing for each scene individually

### Available Optimization Flags

| Flag | Description | Impact |
|------|-------------|--------|
| `--opt-comp` | Compositor optimizations | Layer scratch skip, dirty-region narrowing |
| `--opt-diff` | Dirty-region diff scan | Experimental, may cause artifacts |
| `--opt-present` | Hash-based static frame skip | Skip unchanged frames |
| `--opt-skip` | Unified frame-skip oracle | Prevents animation flickering |
| `--opt-rowdiff` | Row-level dirty skip | Skips unchanged rows in diff scan |
| `--opt` | All optimizations | Combines all flags |

---

## Test Mod: shell-quest-tests

The benchmark uses `mods/shell-quest-tests`, a lightweight variant of the main mod with:

- **Timeout triggers** instead of user input (fully automated)
- **Compressed timings** (~9.4 seconds per full loop vs ~37s original)
- **Scene looping** — scene 04 transitions back to scene 00 for continuous benchmarking

### Scene Timeline (per loop)

| Scene | Duration | Effects | Notes |
|-------|----------|---------|-------|
| 00 Intro Logo | ~1680ms | CRT-on, shine, flash/whiteout | Transitions |
| 01 Intro Date | ~1900ms | Scanlines | Static content |
| 02 Intro Boot | ~2180ms | Fade-in, scanlines, fade-out | Mixed |
| 03 Lab Enter | ~1120ms | Fade-in, pause, fade-out | Light |
| 04 Difficulty | ~2550ms | 4× PostFX (CRT underlay/distort/glitch/ruby) | **Heaviest scene** |
| **Total loop** | **~9430ms** | | Loops back to scene 00 |

### Benchmark Coverage

| Duration | Scenes Covered |
|----------|---------------|
| `--bench 5` | Scenes 00-02 (partial) |
| `--bench 10` | All 5 scenes + starts 2nd loop |
| `--bench 20` | ~2 full loops |
| `--bench 30` | ~3 full loops |

**Recommended:** `--bench 10` covers all scenes in one full pass.

---

## Benchmark Scenarios

### Scenario 1: Quick Validation (5 seconds)
Fast validation, useful during development:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5
```

### Scenario 2: Standard Benchmark (10 seconds) — **RECOMMENDED**
Covers all 5 scenes in one full loop:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### Scenario 3: Extended Benchmark (20 seconds)
Two full loops for more stable metrics:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 20
```

---

## Automated Benchmarking

### Python Script (Recommended)

Run all flag combinations automatically:

```bash
# Standard 10-second benchmarks
python3 benchmark.py standard

# Quick 5-second test of all flags
python3 benchmark.py quick

# Extended 20-second benchmarks
python3 benchmark.py extended
```

The script will:
- Build the engine in release mode
- Test all flag combinations sequentially
- Parse reports automatically
- Generate `reports/benchmark/results.csv` with full metrics
- Display performance comparison table

### Bash Script (Alternative)

```bash
./benchmark.sh standard
```

---

## CSV Report Format

Run `python3 collect-benchmarks.py` to aggregate all reports into `reports/benchmark/results.csv`.

### Core Columns

| Column | Description |
|--------|-------------|
| `flag_name` | Flag combination (e.g., "comp+skip+rowdiff") |
| `opt_comp`, `opt_diff`, `opt_skip`, `opt_rowdiff` | Individual flag status |
| `score` | Performance score (higher is better) |
| `total_frames` | Total frames rendered |
| `fps_avg`, `fps_min`, `fps_max`, `fps_p99` | Frame rate statistics |
| `frame_time_avg`, `frame_time_p50`, `frame_time_p99` | Frame timing (microseconds) |
| `comp_time_avg`, `rend_time_avg`, `behavior_time_avg` | System breakdown |
| `diff_cells_avg`, `dirty_cells_avg` | Buffer pipeline |
| `dirty_coverage_pct`, `diff_coverage_pct` | Coverage percentages |

### Per-Scene Columns (dynamic)

| Column Pattern | Description |
|--------|-------------|
| `scene_count` | Number of unique scenes in benchmark |
| `scene_ids` | Comma-separated list of scene IDs |
| `scene_<ID>_frames` | Frame count for specific scene |
| `scene_<ID>_fps` | Average FPS for specific scene |
| `scene_<ID>_comp` | Compositor time for specific scene |
| `scene_<ID>_pfx` | PostFX time for specific scene |
| `scene_<ID>_rend` | Renderer time for specific scene |

---

## Interpreting Results

### Per-Scene Breakdown

The report includes a scene breakdown table:
```
── SCENE BREAKDOWN ───────────────────────────────────────────
  SCENE                          FRAMES  FPS avg  COMP us   PFX us  REND us   BHV us
  ──────────────────────────────────────────────────────────────────────────────
  00.intro.logo                     102     62.1    120.3      0.0    131.5      2.1
  04.intro.difficulty-select        155     61.9    164.2    892.4    149.8      1.8
```

This helps identify which scenes are heaviest and where optimization effort should focus.

### Score Formula

```
score = (avg_fps × 10) + (1M / median_frame_time × 5) − (p99_frame_time / 100)
```

- Dominated by average FPS
- Penalized by high frame-time variance
- Higher is better

### System Breakdown

- **Compositor**: Object/sprite rendering
- **PostFX**: Post-processing effects (CRT, glow, etc.)
- **Renderer**: Terminal diff/flush
- **Behavior**: Rhai script execution

---

## Example Workflow

### 1. Establish Baseline

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### 2. Test Individual Optimizations

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-rowdiff
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt-skip
```

### 3. Aggregate and Compare

```bash
python3 collect-benchmarks.py
# Opens reports/benchmark/results.csv
```

### 4. Analyze Per-Scene

Check which scenes benefit most from each optimization flag by comparing per-scene columns in the CSV.

---

## Troubleshooting

### "No benchmark report generated"

- Verify the app ran for the full duration
- Check that `--mod-source=mods/shell-quest-tests` path is correct
- `reports/benchmark/` directory is auto-created

### CSV is empty or incomplete

- Check that benchmark reports exist: `ls reports/benchmark/*.txt`
- Run aggregator: `python3 collect-benchmarks.py`

### FPS is very low (< 30)

- Running in debug mode? Use `--release` flag
- Terminal resolution too high? Check RESOLUTIONS.md
- Too many PostFX? Scene 04 is heaviest due to 4 CRT passes

---

## Flag Recommendation

Based on testing:

- **Baseline**: Use for reference only
- **--opt-skip**: Prevents flickering, recommended always
- **--opt-rowdiff**: ~10-20% gain on static scenes
- **--opt-comp --opt-skip --opt-rowdiff**: Balanced, low risk
- **--opt**: All optimizations together (recommended for release)

