# Shell Engine Benchmark Guide

Covers performance benchmarking, repeatable benchmark targets, and frame
capture regression testing.

---

## 1. Quick Start

Use an explicit bundled mod and scene path so benchmark results are reproducible.
The default recommended target is the playground 3D scene:

```bash
# Baseline (disable default comp+rowdiff), 10 seconds
cargo run -p app -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --bench 10 --no-opt-comp --no-opt-rowdiff

# With all optimizations
cargo run -p app -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --bench 10 --opt

# Release build example
cargo run -p app --release -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --bench 10 --opt
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

Recommendations: keep default `opt-comp` + `opt-rowdiff`, add `--opt-skip` for
release builds, and use `--opt` when you want every optimization path.

---

## 3. Recommended Benchmark Targets

There is no dedicated in-repo `shell-engine-tests` benchmark mod anymore.
Instead, benchmark against explicit bundled mod/scene pairs.

Recommended default target:

```bash
cargo run -p app -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --bench 10
```

Why this target:

- it exists in the bundled repo,
- it exercises authored scene composition and 3D rendering together,
- it is stable enough for before/after optimization comparisons.

Other useful targets:

- `mods/planet-generator` for generated-world and atmosphere work,
- `mods/terrain-playground` for terrain/worldgen-heavy scenes,
- `mods/gui-playground` for widget/layout-heavy UI work.
- `mods/asteroids` + `/scenes/bench-cloud/scene.yml` for cloud-heavy generated-world stress tests.

The main rule is consistency: keep the mod, start scene, duration, and flag set
fixed between compared runs.

Cloud-heavy reference command:

```bash
cargo run -p app -- --mod asteroids --start-scene /scenes/bench-cloud/scene.yml --bench 10 --opt --skip-splash
```

---

## 4. Benchmark Scenarios

| Scenario | Command | Use Case |
|----------|---------|----------|
| Quick validation | `--bench 5` | Fast iteration on one known scene |
| Full coverage | `--bench 10` | Recommended baseline comparison pass |
| Extended | `--bench 20+` | Statistical stability over longer runs |

### Automated Flag Sweeps

```bash
python3 benchmark.py standard   # 10s, all flag combinations
python3 benchmark.py quick      # 5s, fast check
python3 benchmark.py extended   # 20s, stable metrics
```

Aggregation: `python3 collect-benchmarks.py` produces
`reports/benchmark/results.csv`.

---

## 5. Frame Capture Regression Testing

Binary frame capture for visual regression. This verifies that optimization
changes produce identical rendered output to baseline rendering.

### Workflow

```bash
# 1. Capture baseline
cargo run -p app -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --capture-frames reports/baseline/ --bench 5

# 2. Capture optimized
cargo run -p app -- --mod-source=mods/playground --start-scene=/scenes/3d-scene/scene.yml --opt --capture-frames reports/optimized/ --bench 5

# 3. Compare
FRAME_BASELINE=reports/baseline/ FRAME_OPTIMIZED=reports/optimized/ \
  cargo test -p engine compare_frame_captures -- --ignored --nocapture
```

There is currently no checked-in `capture-frames.sh` /
`capture-frames-tests.sh` wrapper script in the repo. Use the direct commands
above so the benchmark target stays explicit in review history and CI logs.

### Binary Format

Each `.bin` file: `[width:u16 LE][height:u16 LE][cells...]`.
Each cell = 10 bytes:
`[symbol:u32 LE][fg_r:u8][fg_g:u8][fg_b:u8][bg_r:u8][bg_g:u8][bg_b:u8]`.

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
playground-3d-scene              102     62.1    120.3      0.0    131.5      2.1
```

System columns: Compositor (sprite compositing), PostFX (CRT/glow passes),
Renderer (active backend present path; SDL2 diff/present in software mode),
Behavior (Rhai script execution).

### 3D Object Pass Breakdown

When a scene uses 3D sprites, reports also include `3D OBJECT PASSES (us)` with
pass-level timings:

- `3D surface`
- `3D cloud1`
- `3D cloud2`
- `3D halo`
- `3D convert` / `3D comp` / `3D blit`
- `3D tris` / `3D faces` / `3D viewpx` / `3D sprites`

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| No report generated | Check `--mod-source` and `--start-scene`; `reports/benchmark/` is auto-created |
| FPS below 30 | Build with `--release`; compare on the same scene and window setup |
| Frame count mismatch | Ensure both captures use the same mod, scene, and `--bench N` value |
| Color divergence | Run both captures under the same runtime settings; prefer deterministic scenes |
| CSV empty | Verify `ls reports/benchmark/*.txt`; run `python3 collect-benchmarks.py` |

### Software Backend Profiling (SDL2 Implementation)

For software backend stage timings (diff/build, runtime apply, texture upload,
present), enable profiling logs:

```bash
SHELL_ENGINE_SDL_PROFILE=1 cargo run -p app --
```

When run logging is enabled, look for `sdl2.backend` and `sdl2.runtime`
entries in `logs/<date>/run-XXX/run.log` (software backend telemetry, emitted
roughly once per second).
