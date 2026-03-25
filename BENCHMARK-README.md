# Shell Quest Engine Benchmarking System

Comprehensive performance benchmarking for all optimization flags.

## Quick Start

### Run a Single Benchmark

```bash
# Baseline (no flags), 5 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash

# With opt-rowdiff, 5 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash --opt-rowdiff

# All optimizations, 5 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash --opt
```

### Aggregate Existing Reports to CSV

```bash
python3 collect-benchmarks.py
```

This creates `reports/benchmark/results.csv` with all existing benchmark data.

---

## How It Works

1. **Run benchmarks** with `--bench <SECS>` flag
2. **App displays results** in-game on a full-screen display
3. **Report is saved** to `reports/benchmark/TIMESTAMP.txt`
4. **Run aggregator** to convert all reports to CSV
5. **Open CSV** in your spreadsheet for analysis

---

## Optimization Flags

### Available

| Flag | Description | Est. Impact |
|------|-------------|------------|
| `--opt-comp` | Layer scratch skip, dirty-region narrowing | ~5% |
| `--opt-diff` | Dirty-region diff scan (experimental) | ~20% |
| `--opt-skip` | Unified frame-skip oracle (recommended) | ~0-5% |
| `--opt-rowdiff` | Row-level dirty skip | ~10-20% |
| `--opt` | All optimizations | ~30-50% |

### Recommended Combinations

1. **Baseline**: No flags
2. **Safe**: `--opt-skip` (prevents flickering)
3. **Balanced**: `--opt-comp --opt-skip`
4. **Aggressive**: `--opt` (all flags)

---

## Benchmark Scenarios

### Quick Test (2 seconds)
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 2 --skip-splash
```
- Fast iteration during development
- Less stable metrics

### Standard Benchmark (5 seconds) — **RECOMMENDED**
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash
```
- Balanced accuracy
- Good for comparative analysis

### Extended Benchmark (10 seconds)
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --skip-splash
```
- Most stable metrics
- Slower iteration

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

---

## Workflow: Comparing Flags

### 1. Establish Baseline

```bash
# Run baseline
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash

# Note the FPS from in-game display
# (also saved to reports/benchmark/TIMESTAMP.txt)
```

### 2. Test Individual Flags

```bash
# Test opt-rowdiff
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash --opt-rowdiff

# Test opt-skip
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash --opt-skip

# Test combined
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash --opt-comp --opt-rowdiff
```

### 3. Aggregate Results

```bash
python3 collect-benchmarks.py
# Creates results.csv with all benchmarks
```

### 4. Analyze in Spreadsheet

```bash
# Open in LibreOffice Calc, Excel, or similar
open reports/benchmark/results.csv
```

Then:
- Sort by `fps_avg` to find fastest combination
- Check `frame_time_p99` for jitter/variance
- Compare system breakdown to find bottlenecks

---

## Example Results

From recent testing (mods/shell-quest-tests, 5-second benchmarks):

| Flag | FPS | Frame Time (us) | Compositor (us) | Renderer (us) | Dirty % |
|------|-----|-----------------|-----------------|---------------|---------|
| baseline | 62.2 | 16,066 | 120 | 132 | 6.7% |
| opt-comp | 62.2 | 16,072 | 164 | 149 | 6.7% |
| opt-diff | 62.2 | 16,068 | 129 | 145 | 6.7% |
| **opt-comp+diff** | **62.2** | **16,069** | **64** | **67** | **7.3%** |

**Observations:**
- All combinations achieve ~62 FPS (60 Hz target)
- Combined `--opt-comp --opt-diff` has lowest system times
- Diff cells actually higher (more precision in tracking)

---

## Tips & Tricks

### Building for Speed

Use release mode (faster iteration):
```bash
cargo run --release -p app -- --mod-source=mods/shell-quest-tests --bench 5 --skip-splash
```

### Filtering CSV Results

In spreadsheet software:
1. Apply filter to `flag_name`
2. Sort by `fps_avg` descending
3. Compare variations of interest

### Detecting Regressions

Compare P99 frame times:
- If `frame_time_p99` is much higher than `frame_time_avg`, there's jitter
- Good flags: P99 < 1.1 × AVG
- Bad flags: P99 > 1.5 × AVG

### Low FPS Debugging

Check in this order:
1. **Compositor bottleneck** if `comp_time_avg` > `rend_time_avg`
2. **Renderer bottleneck** if `rend_time_avg` is high
3. **Behavior bottleneck** if `behavior_time_avg` is high
4. **Output bottleneck** if `diff_cells_avg` is very high

---

## Troubleshooting

### No reports generated

- Check `reports/benchmark/` exists
- Verify app ran for full duration (wait for in-game results screen)
- Try running manually first: `cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 2`

### CSV is empty

- Ensure reports exist: `ls reports/benchmark/*.txt`
- Run aggregator: `python3 collect-benchmarks.py`
- Check for parse errors in Python output

### FPS is too low (< 50)

- Running in debug mode? Add `--release` flag
- Resolution too high? Try `--renderer-mode Cell`
- Many optimizations active? Try `--opt-skip` alone first

---

## Next Steps

After benchmarking and analysis:

1. **Document results** — Save CSV and notes to repo
2. **Identify best combo** — Usually `--opt` or `--opt-comp --opt-rowdiff`
3. **Make default** — Update game launch to use recommended flags
4. **Iterate** — Re-benchmark after engine changes

