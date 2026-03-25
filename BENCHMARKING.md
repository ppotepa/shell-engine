# Shell Quest Benchmark Guide

## Quick Start

The Shell Quest engine has built-in benchmark mode for profiling all optimization flags. 

### Running Individual Benchmarks

Run any benchmark with the `--bench <SECONDS>` flag:

```bash
# Baseline (no optimizations), 5 seconds
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5

# With specific optimization
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --opt-rowdiff

# With all optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --opt
```

### Benchmark Output

The app will:
1. Run the benchmark for the specified duration
2. Display results in-game on a full-screen benchmark screen
3. Save a detailed text report to `reports/benchmark/<TIMESTAMP>.txt`

### Available Optimization Flags

| Flag | Description | Impact |
|------|-------------|--------|
| `--opt-comp` | Compositor optimizations | Layer scratch skip, dirty-region narrowing |
| `--opt-diff` | Dirty-region diff scan | Experimental, may cause artifacts |
| `--opt-skip` | Unified frame-skip oracle | Prevents animation flickering |
| `--opt-rowdiff` | Row-level dirty skip | Skips unchanged rows in diff scan |
| `--opt` | All optimizations | Combines all flags |

---

## Benchmark Scenarios

### Scenario 1: Quick Test (2 seconds)
Fast validation, useful during development:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 2
```

### Scenario 2: Standard Benchmark (5 seconds)
Default benchmark duration, balanced accuracy:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5
```

### Scenario 3: Extended Benchmark (10 seconds)
More stable metrics, recommended for comparative analysis:
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

---

## Automated Benchmarking

### Python Script (Recommended)

Run all flag combinations automatically:

```bash
# Standard 5-second benchmarks
python3 benchmark.py standard

# Quick 2-second test of all flags
python3 benchmark.py quick

# Extended 10-second benchmarks
python3 benchmark.py extended
```

The script will:
- Build the engine in release mode
- Test all 11 flag combinations sequentially
- Parse reports automatically
- Generate `reports/benchmark/results.csv` with full metrics
- Display performance comparison table

### Bash Script (Alternative)

```bash
./benchmark.sh standard
```

---

## CSV Report Format

The benchmark generates `reports/benchmark/results.csv` with these columns:

| Column | Description |
|--------|-------------|
| `scenario` | Test duration (quick/standard/extended) |
| `name` | Flag combination name |
| `flags` | Actual CLI flags used |
| `score` | Performance score (higher is better) |
| `frames` | Total frames rendered |
| `fps_avg` | Average FPS |
| `frame_time` | Average frame time (microseconds) |
| `comp_time` | Compositor time (microseconds) |
| `rend_time` | Renderer time (microseconds) |
| `diff_cells` | Diff cells per frame (average) |
| `dirty_cells` | Dirty cells per frame (average) |
| `dirty_pct` | Dirty coverage % |
| `diff_pct` | Diff coverage % |

---

## Interpreting Results

### FPS vs Frame Time

- **FPS**: Frames per second (higher = better)
- **Frame Time**: Microseconds per frame (lower = better)
- Formula: `FPS = 1,000,000 / Frame Time`

### System Breakdown

Each benchmark shows time spent in each system:
- **Compositor**: Object/sprite rendering
- **Renderer**: Terminal diff/flush
- **Behavior**: Rhai script execution
- **PostFX**: Post-processing effects

### Buffer Pipeline

- **Diff cells**: Cells that changed vs previous frame
- **Dirty cells**: Cells marked dirty by compositor
- **Dirty coverage %**: Fraction of screen marked dirty
- **Diff coverage %**: Fraction of screen that actually changed

**Goal**: Lower diff/dirty percentages = faster rendering.

---

## Example Workflow

### 1. Establish Baseline

```bash
# Run baseline (no flags)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5

# Note the FPS and frame times from the in-game display
# Report is auto-saved to reports/benchmark/TIMESTAMP.txt
```

### 2. Test Individual Optimizations

```bash
# Test opt-rowdiff alone
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 5 --opt-rowdiff

# Compare FPS to baseline
```

### 3. Run Full Comparison

```bash
# Automated test of all combinations
python3 benchmark.py standard

# Results in reports/benchmark/results.csv
```

### 4. Analyze Results

Open `results.csv` in your spreadsheet application:
- Compare FPS across all flag combinations
- Look for regressions (lower FPS than baseline)
- Identify best-performing combination

---

## Benchmark Metrics Explained

### Score

Calculated as:
```
score = (avg_fps * 10) + (median_fps * 5) - (p99_frame_time / 100)
```

- Dominated by average FPS
- Penalized by high frame-time variance
- Higher is better

### FPS Statistics

- **AVG**: Average FPS across all frames
- **P50** (median): 50th percentile frame time
- **P95**: 95th percentile (worst 5%)
- **P99**: 99th percentile (worst 1%)

**Goal**: Maximize average FPS and minimize variance.

### Example Report

```
FPS           avg=    62.2  min=    62.1  max=    62.5  p50=    62.3  p95=    62.3  p99=    62.3
Frame         avg= 16069.1us  min= 16011.0us  max= 16114.0us  p50= 16060.0us  p95= 16107.0us  p99= 16112.0us
```

- Consistent 62.2 FPS
- Tight variance (62.1-62.5)
- Frame time ~16ms (16,069 microseconds)

---

## Troubleshooting

### "No benchmark report generated"

- Check that `reports/benchmark/` directory exists
- Verify the app ran for the full duration
- Check that `--mod-source=mods/shell-quest-tests` path is correct

### CSV is empty or incomplete

- Ensure the Python script is executable: `chmod +x benchmark.py`
- Check that benchmark reports exist in `reports/benchmark/`
- Run manually and verify reports are created

### FPS is very low (< 30)

- Running in debug mode? Use `--release` flag
- Terminal resolution too high? Reduce with `--renderer-mode`
- Try with `--skip-splash` to skip intro animation

---

## Next Steps

After benchmarking:

1. **Commit results**: Save CSV to version control
2. **Document regressions**: Flag any unexpected FPS drops
3. **Profile hotspots**: Use `--debug-feature` to see system breakdown
4. **Iterate**: Test new optimizations and compare

---

## Flag Recommendation

Based on prior testing:

- **Baseline**: Use for reference only
- **--opt-skip**: Prevents flickering, recommended always
- **--opt-rowdiff**: ~10-20% gain on static scenes, combine with --opt-comp
- **--opt**: All optimizations together (recommended for release)

