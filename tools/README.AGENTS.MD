# Development Tools — Benchmarking, Capture, Schema

## Benchmark Runners

### benchmark.py (Automated multi-scenario runner)

```bash
python3 benchmark.py
```

Runs multiple benchmarks automatically:
- Scenario 1: Baseline (no flags)
- Scenario 2: All optimizations (`--opt`)
- Scenario 3: Individual flags (`--opt-comp`, `--opt-skip`, etc.)

Output:
- Console display: FPS, timing breakdown
- File: `reports/benchmark/YYYYMMDD-HHMMSS.txt`
- Line-by-line: all metrics

### benchmark.sh (Shell batch runner)

```bash
./benchmark.sh <MOD_SOURCE> <DURATION> <FLAGS>
```

Example:
```bash
./benchmark.sh mods/shell-quest-tests 10 "--opt-skip --opt-rowdiff"
```

Runs single benchmark with custom flags. Simpler than Python version.

### collect-benchmarks.py (CSV aggregator)

```bash
python3 collect-benchmarks.py reports/benchmark/ reports/benchmark/results.csv
```

Parses all .txt reports in directory:
1. Extract metrics (fps, frame_us, compositor_us, etc.)
2. Parse SCENE BREAKDOWN section
3. Create dynamic columns: `scene_00_frames`, `scene_00_fps`, `scene_00_comp`, `scene_00_pfx`, `scene_00_rend`
4. Output as CSV (Excel-compatible)

Result: Single spreadsheet with all scenario comparisons and per-scene breakdown.

## Frame Capture Tools

### capture-frames.sh (Baseline capture)

```bash
./capture-frames.sh <MOD_SOURCE> <OUTPUT_DIR> <DURATION>
```

Example:
```bash
./capture-frames.sh mods/shell-quest-tests reports/baseline 10
```

Captures binary frame data:
- One file per frame: `frame_000000.bin`, `frame_000001.bin`, ...
- Format: raw cell bytes (char + color codes)
- No compression

### capture-frames-tests.sh (Compare baseline vs optimized)

```bash
./capture-frames-tests.sh <BASELINE_DIR> <OPTIMIZED_DIR> <DURATION>
```

Example:
```bash
./capture-frames-tests.sh reports/baseline reports/optimized 10
```

Two-phase capture:
1. Capture baseline with safe defaults
2. Capture optimized with all flags
3. Compare frame-by-frame
4. Report divergences (pixel count, affected regions)

Useful for regression detection.

## Schema Tools

### refresh-schemas.sh (Regenerate YAML schemas)

```bash
./refresh-schemas.sh
```

Equivalent to:
```bash
cargo run -p schema-gen -- --all-mods
```

Regenerates all YAML schemas in `schemas/`:
- `scene.yml.schema.json`
- `mod.yaml.schema.json`
- `behavior.yml.schema.json`
- `effect.yml.schema.json`

Use after any model changes.

### schema-gen (Drift checking)

```bash
cargo run -p schema-gen -- --all-mods --check
```

Compares generated schemas against existing ones:
- Returns exit code 0 if identical
- Returns exit code 1 if diverged
- Useful for CI validation

## Refactoring Tools

### rename_case.py (Snake case normalization)

```bash
python3 tools/rename_case.py --manifest <MANIFEST> --scan <PATH> [--dry-run]
```

Manifest-driven renaming:
```yaml
renames:
  - old: camelCase_var
    new: camel_case_var
  - old: old_function_name
    new: new_function_name
```

Modes:
- `--dry-run`: Show changes without applying
- Default: Apply changes to all files in `<PATH>`

Useful for maintaining consistent naming conventions.

## Test & Build Commands

### Core Tests

```bash
# Engine tests (233 tests)
cargo test -p engine

# Engine-authoring tests (73 tests)
cargo test -p engine-authoring

# Engine-core tests (79 tests)
cargo test -p engine-core

# All tests
cargo test
```

### Build

```bash
# Debug build (slower, full symbols)
cargo build

# Release build (optimized)
cargo build --release

# Specific binary
cargo build -p app
cargo build -p editor
```

### Running

```bash
# Play game (built-in mod)
cargo run -p app

# Play with test mod
cargo run -p app -- --mod-source=mods/shell-quest-tests

# Play with custom mod
cargo run -p app -- --mod-source=/path/to/mod

# Play with benchmarking
cargo run -p app -- --bench 10

# Play with frame capture
cargo run -p app -- --capture-frames /tmp/frames --bench 10

# Play with all optimizations
cargo run -p app -- --opt

# Editor
cargo run -p editor

# Editor with custom mod
cargo run -p editor -- --mod-source=mods/shell-quest
```

## Automation Workflow

### Full benchmark suite

```bash
# 1. Run benchmarks (writes to reports/benchmark/YYYYMMDD-*.txt)
python3 benchmark.py

# 2. Aggregate results into CSV
python3 collect-benchmarks.py reports/benchmark/ reports/benchmark/results.csv

# 3. Open CSV in Excel/LibreOffice for analysis
# Check which flags provide best fps, lowest frame latency, etc.
```

### Regression testing

```bash
# 1. Capture baseline
./capture-frames-tests.sh reports/baseline reports/optimized 10

# 2. Review divergences
# Check for visual artifacts, glitches, old frames

# 3. If clean, commit optimization
git commit -m "optimization: enable --opt-flag for 10% speedup"
```

### Schema validation

```bash
# 1. Make model changes (add field to scene.yml schema)
# 2. Regenerate schemas
./refresh-schemas.sh

# 3. Check for drift (in CI)
cargo run -p schema-gen -- --all-mods --check
```

## Key Environment Variables

- `SHELL_QUEST_MOD_SOURCE` — Override mod directory (CLI `--mod-source` takes precedence)
- `FRAME_BASELINE`, `FRAME_OPTIMIZED` — For frame capture comparison tests
