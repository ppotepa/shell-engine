# Test Mod for Benchmarking — Structure & Usage

## Purpose

`shell-quest-tests` is a lightweight variant of the main mod for:
- Automated benchmarking (no user interaction needed)
- Frame capture regression testing
- Consistent per-scene performance measurement

## Scene Timeline

All scenes have **timeout triggers** (no user input required):

| Scene | ID | Original | Compressed | Trigger | Notes |
|-------|----|---------|-----------|---------|----|
| 00 Logo | `00.intro.logo` | 7000ms | 1680ms | timeout 600ms | CRT-on, shine, flash |
| 01 Date | `01.intro.date` | 9780ms | 1900ms | timeout 400ms | Scanlines |
| 02 Boot | `02.intro.boot` | 12290ms | 2180ms | timeout 200ms | Fade, scanlines |
| 03 Lab Enter | `03.intro.lab-enter` | 3160ms | 1120ms | timeout 200ms | Fade in/out |
| 04 Difficulty | `04.intro.difficulty-select` | 5000ms | 2550ms | timeout 2000ms | 4× PostFX |
| **Total loop** | | **37230ms** | **9430ms** | | Loops back to 00 |

## Compression Ratios

Compression preserves effect types but shortens durations:

| Scene | Ratio | Reason |
|-------|-------|--------|
| 00 | 4.2× | Minimal effect, can compress heavily |
| 01 | 5.1× | Text fade, can compress |
| 02 | 5.6× | Animation, can compress |
| 03 | 2.8× | Mixed effects, moderate compression |
| 04 | 2.0× | 4 PostFX passes need time to stress-test |

**One full loop**: ~9.4 seconds → `--bench 10` covers all 5 scenes + starts 2nd loop

## Benchmark Coverage

| Duration | Coverage | Scenes |
|----------|----------|--------|
| `--bench 5` | Partial | 00-02 |
| `--bench 10` | Full + start of 2nd loop | 00-04 + 00 (partial) |
| `--bench 20` | ~2 full loops | 00-04 ×2 |
| `--bench 30` | ~3 full loops | 00-04 ×3 |

## Per-Scene Metrics

Benchmark reports include breakdown:

```
SCENE BREAKDOWN
───────────────────────────────────
Scene ID            Frames  FPS  Comp(%)  Postfx(%) Rend(%)
00.intro.logo        100   60.0    8.2       12.1     15.3
01.intro.date         114   60.0    5.1       18.2     14.0
02.intro.boot         131   60.0    6.7       22.5     16.1
03.intro.lab-enter     67   60.0    4.2        8.0     12.5
04.difficulty-select  153   60.0   12.3       45.2     18.0
───────────────────────────────────
Total                565   60.0    7.3       21.1     15.1
```

Allows identification of which scenes benefit from which optimizations.

## Looping Behavior

Scene 04 has `next: 00.intro.logo` (main mod has `next: null`).

This enables continuous looping for any-duration benchmarks:
- Completion of scene 04 triggers transition to scene 00
- Infinite loop until `--bench <SECS>` timeout
- Smooth transitions between scenes

## Frame Capture Integration

Test mod works seamlessly with frame capture:

```bash
# Capture baseline (safe defaults)
cargo run -p app -- --mod-source=mods/shell-quest-tests \
  --capture-frames reports/baseline --bench 10

# Capture optimized (all flags)
cargo run -p app -- --mod-source=mods/shell-quest-tests \
  --opt --capture-frames reports/optimized --bench 10

# Compare
export FRAME_BASELINE=reports/baseline FRAME_OPTIMIZED=reports/optimized
cargo test -p engine compare_frame_captures -- --ignored --nocapture
```

## Running Benchmarks

### Baseline (no optimizations)
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### Specific optimization flag
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt-skip --bench 10
```

### All optimizations
```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --opt --bench 10
```

### CSV aggregation
```bash
python3 collect-benchmarks.py reports/benchmark/ reports/benchmark/results.csv
```

Output CSV includes per-scene columns: `scene_00_frames`, `scene_00_fps`, `scene_00_comp`, etc.

## File Structure

```
mods/shell-quest-tests/
├── mod.yaml                      Mod manifest (inherits from parent)
└── scenes/
    ├── 00-intro-logo/
    │   ├── scene.yml             (Modified: timeout instead of any-key)
    │   └── layers/main.yml       (Inherited reference)
    ├── 01-intro-date/
    │   ├── scene.yml
    │   └── layers/main.yml
    ├── 02-intro-boot/
    │   ├── scene.yml
    │   └── layers/main.yml
    ├── 03-intro-lab-enter/
    │   ├── scene.yml
    │   └── layers/main.yml
    └── 04-difficulty-select/
        ├── scene.yml             (Modified: next: 00.intro.logo)
        └── layers/main.yml
```

## Modifications from Main Mod

**Changes to scene.yml files:**
1. All `on_idle.trigger: any-key` → `on_idle.trigger: timeout`
2. Set `on_idle.timeout: <DURATION>ms` (auto-advance)
3. Scene 04: `next: null` → `next: 00.intro.logo` (enable looping)

**Preserved:**
- All effects, PostFX, visual styling
- Layer structure, sprites, objects
- Custom behaviors & effects
- Asset references

## Notes

- Test mod inherits all visual styling from parent mod
- No user input required (fully automated)
- Ideal for CI/CD regression testing
- Per-scene metrics enable targeted optimization analysis
- Continuous looping supports any-duration benchmarks
