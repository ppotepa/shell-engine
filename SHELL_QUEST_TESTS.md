# Test Mod for Automated Benchmarking & Frame Capture

## Overview

`shell-quest-tests` is a lightweight variant of the main shell-quest mod designed for automated benchmarking and frame capture regression testing. It eliminates all user input requirements by replacing them with automatic timeouts, compresses scene timings for efficient benchmarking, and loops continuously for any-duration benchmarks.

## Key Features

- **No user input required** — all `any-key` triggers replaced with timeouts
- **Compressed timings** — ~9.4 seconds per full loop (vs ~37s original)
- **Continuous looping** — scene 04 loops back to scene 00
- **Splash auto-skip** — `--bench` automatically skips splash screen

## Using the Test Mod

### Benchmarking (primary use case)

```bash
# 10-second benchmark covering all 5 scenes (recommended)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10

# With all optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt

# Extended 20-second run (2 full loops)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 20
```

### Frame Capture

```bash
# Baseline capture
cargo run -p app -- --mod-source=mods/shell-quest-tests --capture-frames reports/baseline --bench 10

# Optimized capture
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt --capture-frames reports/optimized --bench 10
```

### Environment Variable

```bash
export SHELL_QUEST_MOD_SOURCE=mods/shell-quest-tests
cargo run -p app -- --bench 10
```

## Test Scenes

### Scene Timeline (per loop)

| Scene | ID | Duration | Trigger | Effects |
|-------|----|----------|---------|---------|
| 00 Intro Logo | `00.intro.logo` | ~1680ms | timeout 600ms | CRT-on, shine, flash/whiteout/fade |
| 01 Intro Date | `01.intro.date` | ~1900ms | timeout 400ms | Scanlines 1.4s |
| 02 Intro Boot | `02.intro.boot` | ~2180ms | timeout 200ms | Fade-in, scanlines 1.6s, fade-out |
| 03 Lab Enter | `03.intro.lab-enter` | ~1120ms | timeout 200ms | Fade-in, pause, fade-out |
| 04 Difficulty | `04.intro.difficulty-select` | ~2550ms | timeout 2000ms | 4× PostFX (heaviest scene) |
| **Total loop** | | **~9430ms** | | **Loops back to scene 00** |

### Benchmark Coverage by Duration

| Duration | Coverage |
|----------|---------|
| `--bench 5` | Scenes 00-02 (partial loop) |
| `--bench 10` | All 5 scenes + start of 2nd loop |
| `--bench 20` | ~2 full loops |
| `--bench 30` | ~3 full loops |

### Original vs Compressed Timings

| Scene | Original | Compressed | Ratio |
|-------|----------|-----------|-------|
| 00 | 7000ms | 1680ms | 4.2× |
| 01 | 9780ms | 1900ms | 5.1× |
| 02 | 12290ms | 2180ms | 5.6× |
| 03 | 3160ms | 1120ms | 2.8× |
| 04 | 5000ms | 2550ms | 2.0× |
| **Total** | **37230ms** | **9430ms** | **3.9×** |

Scene 04 has the least compression because its PostFX passes need enough time to stress-test the pipeline.

## Files Structure

```
mods/shell-quest-tests/
├── mod.yaml                          # Test mod manifest
└── scenes/
    ├── 00-intro-logo/
    │   ├── scene.yml                 # Compressed: 1680ms, CRT-on + shine + flash
    │   └── layers/main.yml
    ├── 01-intro-date/
    │   ├── scene.yml                 # Compressed: 1900ms, scanlines
    │   └── layers/main.yml
    ├── 02-intro-boot/
    │   ├── scene.yml                 # Compressed: 2180ms, fade + scanlines
    │   └── layers/main.yml
    ├── 03-intro-lab-enter/
    │   ├── scene.yml                 # Compressed: 1120ms, fade in/out
    │   └── layers/main.yml
    └── 04-difficulty-select/
        ├── scene.yml                 # Compressed: 2550ms, 4× PostFX, loops to 00
        └── layers/main.yml
```

## Per-Scene Benchmark Metrics

Benchmark reports include per-scene breakdown showing:
- Frame count per scene
- Average FPS per scene
- Compositor, PostFX, renderer, and behavior time per scene

This helps identify which scenes benefit most from each optimization flag. Scene 04 (difficulty-select) is intentionally the heaviest due to 4 PostFX passes (crt-underlay, crt-distort, crt-scan-glitch, crt-ruby).

## Regression Workflow

```bash
# 1. Capture baseline
cargo run -p app -- --mod-source=mods/shell-quest-tests --capture-frames reports/baseline --bench 10

# 2. Capture optimized
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt --capture-frames reports/optimized --bench 10

# 3. Compare
export FRAME_BASELINE=reports/baseline FRAME_OPTIMIZED=reports/optimized
cargo test -p engine compare_frame_captures -- --ignored --nocapture
```

## Notes

- Test mod inherits all effects, PostFX, and visual styling from parent mod
- Only scene lifecycle triggers and timings are modified
- Visual content is identical to main mod (same layers, sprites, effects)
- Scene 04 `next: 00.intro.logo` enables continuous looping (original has `next: null`)
- Timing compression preserves effect types but shortens durations proportionally
