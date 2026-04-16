# Shell Engine Benchmark Guide

Covers performance benchmarking, the test mod, and frame capture regression testing.

---

## 1. Quick Start

```bash
# Baseline (disable default comp+rowdiff), 10 seconds
cargo run -p app -- --mod-source=mods/shell-engine-tests --bench 10 --no-opt-comp --no-opt-rowdiff

# With all optimizations
cargo run -p app -- --mod-source=mods/shell-engine-tests --bench 10 --opt

# Release build example
cargo run -p app --release -- --mod-source=mods/shell-engine-tests --bench 10 --opt
```

`--bench` automatically skips the splash screen. Reports are saved to
`reports/benchmark/YYYYMMDD-HHMMSS.txt` with per-scene breakdown.

---

## 2. Optimization Flags

| Flag | Description |
|------|-------------|
| `--opt-comp` | Compositor: layer scratch skip (enabled by default; use `--no-opt-comp` to disable) |
| `--opt-diff` | Dirty-region diff (experimental) |
| `--opt-present` | Hash-based static frame skip |
| `--opt-skip` | Unified frame-skip oracle |
| `--opt-rowdiff` | Row-level dirty skip (enabled by default; use `--no-opt-rowdiff` to disable) |
| `--opt` | All optimizations combined |

Recommendations: keep default `opt-comp` + `opt-rowdiff`, add `--opt-skip` for release builds, and use `--opt` when you want every optimization path.

---

## 3. Test Mod (shell-engine-tests)

Lightweight variant of the main mod: timeout triggers instead of user input,
compressed timings (~9.4s per loop), continuous looping (scene 04 wraps to 00).

### Scene Timeline

| Scene | Duration | Trigger | Effects |
|-------|----------|---------|---------|
| 00 Intro Logo | ~1680ms | timeout 600ms | CRT-on, shine, flash |
| 01 Intro Date | ~1900ms | timeout 400ms | Scanlines |
| 02 Intro Boot | ~2180ms | timeout 200ms | Fade-in, scanlines |
| 03 Lab Enter | ~1120ms | timeout 200ms | Fade-in/out |
| 04 Difficulty | ~2550ms | timeout 2000ms | 4x PostFX (heaviest) |
| **Total loop** | **~9430ms** | | |

### Coverage by Duration

| Duration | Scenes Covered |
|----------|----------------|
| `--bench 5` | 00-02 (partial loop) |
| `--bench 10` | All 5 scenes + 2nd loop start |
| `--bench 20` | ~2 full loops |
| `--bench 30` | ~3 full loops |

---

## 4. Benchmark Scenarios

| Scenario | Command | Use Case |
|----------|---------|----------|
| Quick validation | `--bench 5` | Partial coverage, fast iteration |
| Full coverage | `--bench 10` | Recommended -- all scenes in one pass |
| Extended | `--bench 20+` | Statistical stability, multiple loops |

### Automated Flag Sweeps

```bash
python3 benchmark.py standard   # 10s, all flag combinations
python3 benchmark.py quick      # 5s, fast check
python3 benchmark.py extended   # 20s, stable metrics
```

Aggregation: `python3 collect-benchmarks.py` produces `reports/benchmark/results.csv`.

---

## 5. Frame Capture Regression Testing

Binary frame capture for visual regression -- verifies that optimizations produce
identical output to baseline rendering.

### Workflow

```bash
# 1. Capture baseline
cargo run -p app -- --capture-frames reports/baseline/ --bench 5

# 2. Capture optimized
cargo run -p app -- --opt --capture-frames reports/optimized/ --bench 5

# 3. Compare
FRAME_BASELINE=reports/baseline/ FRAME_OPTIMIZED=reports/optimized/ \
  cargo test -p engine compare_frame_captures -- --ignored --nocapture
```

### Quick Script

```bash
./capture-frames.sh reports/baseline reports/optimized 5
```

### Binary Format

Each `.bin` file: `[width:u16 LE][height:u16 LE][cells...]`.
Each cell = 10 bytes: `[symbol:u32 LE][fg_r:u8][fg_g:u8][fg_b:u8][bg_r:u8][bg_g:u8][bg_b:u8]`.

Colors are serialized as 8-bit RGB triples. `Color::Reset` maps to `(0,0,0)`;
ANSI palette colors are expanded to standard RGB.

---

## 6. Benchmark Reports

Reports are saved to `reports/benchmark/YYYYMMDD-HHMMSS.txt`.

### Score Formula

```
score = (fps.avg * 10) + (1_000_000 / frame.p50 * 5) - (frame.p99 / 100)
```

Higher is better. Dominated by average FPS, penalized by frame-time variance.

### Per-Scene Breakdown

```
SCENE                          FRAMES  FPS avg  COMP us   PFX us  REND us   BHV us
00.intro.logo                     102     62.1    120.3      0.0    131.5      2.1
04.intro.difficulty-select        155     61.9    164.2    892.4    149.8      1.8
```

System columns: Compositor (sprite compositing), PostFX (CRT/glow passes),
Renderer (SDL2 diff/present), Behavior (Rhai script execution).

---

## 7. Automated Testing Script

The `capture-frames-tests.sh` script captures baseline and optimized frames using
the `shell-engine-tests` mod automatically:

```bash
./capture-frames-tests.sh baseline optimized 2
```

This runs both captures with `--mod-source=mods/shell-engine-tests`, then invokes the
comparison test. Useful for CI and pre-merge visual regression checks.

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| No report generated | Check `--mod-source` path; `reports/benchmark/` is auto-created |
| FPS below 30 | Build with `--release`; check terminal resolution (see RESOLUTIONS.md) |
| Frame count mismatch | Ensure both captures use the same `--bench N` value |
| Color divergence | Run both captures in the same terminal; prefer `Color::Rgb` in test scenes |
| CSV empty | Verify `ls reports/benchmark/*.txt`; run `python3 collect-benchmarks.py` |

### SDL2 Pipeline Profiling

For SDL2 backend stage timings (diff/build, runtime apply, texture upload, present),
enable profiling logs:

```bash
SHELL_ENGINE_SDL_PROFILE=1 cargo run -p app --
```

When run logging is enabled, look for `sdl2.backend` and `sdl2.runtime`
entries in `logs/<date>/run-XXX/run.log` (emitted roughly once per second).
