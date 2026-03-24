# Test Mod for Automated Frame Capture

## Overview

`shell-quest-tests` is a lightweight variant of the main shell-quest mod designed specifically for automated frame capture regression testing. It eliminates all user input requirements (`any-key` triggers) by replacing them with automatic timeouts.

## Problem

The main shell-quest mod includes interactive elements:
- `00.intro.logo`: "Press any key" to continue
- `04.difficulty-select`: Menu selection (requires key press)

These block automated testing. The test mod removes these blockers.

## Solution

**Modified scenes use timeout triggers instead of any-key:**

```yaml
# Original (blocks on user input)
on_idle:
  trigger: any-key
  looping: true
  steps: [ ... ]

# Test variant (auto-continues)
on_idle:
  trigger: timeout
  timeout: 3500ms
  steps: [ ... ]
```

## Using the Test Mod

### Option 1: Run with --mod-source (if supported)
```bash
cargo run -p app -- --mod-source mods/shell-quest-tests --capture-frames /tmp/capture --bench 2
```

### Option 2: Set environment variable
```bash
export SHELL_QUEST_MOD_SOURCE=mods/shell-quest-tests
cargo run -p app -- --capture-frames /tmp/capture --bench 2
```

### Option 3: Use automated script
```bash
./capture-frames-tests.sh reports/baseline reports/optimized 2
```

## Test Scenes Included

| Scene | ID | Trigger | Timeout | Notes |
|-------|----|---------|---------| ----|
| 00 Intro Logo | `00.intro.logo` | timeout | 3500ms | CRT-on + title shine |
| 01 Intro Date | `01.intro.date` | timeout | 1000ms | Date card appears |
| 02 Intro Boot | `02.intro.boot` | timeout | 1000ms | Linus animation |
| 03 Lab Enter | `03.intro.lab-enter` | timeout | 1000ms | Lab fade in/out |
| 04 Difficulty | `04.intro.difficulty-select` | timeout | 4000ms | Auto-advances menu |

All other scenes are inherited from parent mod (via layer references).

## Frame Count Expectations

With `--bench N`:
- `00.intro.logo`: ~30 frames (CRT-on + shine effect, timeout)
- `01.intro.date`: ~8-10 frames (date fade in/out timing)
- `02.intro.boot`: ~10-12 frames (boot animation)
- Total per run: ~60-70 frames (depends on scene timing)

## Regression Workflow

```bash
# 1. Capture baseline (safe defaults)
./capture-frames-tests.sh reports/baseline reports/optimized 2

# This automatically:
#   - Captures with safe defaults
#   - Captures with all optimizations
#   - Compares and reports
```

## Scene Modifications

### 00-intro-logo (timeouts instead of any-key)

**Original:**
```yaml
on_idle:
  trigger: any-key
  looping: true
  steps:
    - pause: 1800ms
      effects: [ ... ]
```

**Test:**
```yaml
on_idle:
  trigger: timeout
  timeout: 3500ms  # Auto-advance after shine effect
  steps:
    - pause: 1800ms
      effects: [ ... ]
```

### 04-difficulty-select (timeouts instead of menu selection)

**Original:**
```yaml
on_idle:
  trigger: any-key
  looping: true
  steps:
    - pause: 1800ms
      effects: [ ... ]
```

**Test:**
```yaml
on_idle:
  trigger: timeout
  timeout: 4000ms  # Auto-advance after effects
  steps:
    - pause: 1800ms
      effects: [ ... ]
```

Menu options remain defined but unused (auto-timeout selects first one).

## Files Structure

```
mods/shell-quest-tests/
├── mod.yaml                          # Test mod manifest
└── scenes/
    ├── 00-intro-logo/
    │   ├── scene.yml                 # Modified: any-key → timeout
    │   └── layers/main.yml           # Copied from parent
    ├── 01-intro-date/
    │   ├── scene.yml                 # Modified: ensure timeout
    │   └── layers/main.yml           # Copied from parent
    ├── 02-intro-boot/
    │   ├── scene.yml                 # Modified: ensure timeout
    │   └── layers/main.yml           # Copied from parent
    ├── 03-intro-lab-enter/
    │   ├── scene.yml                 # Modified: ensure timeout
    │   └── layers/main.yml           # Copied from parent
    └── 04-difficulty-select/
        ├── scene.yml                 # Modified: any-key → timeout
        └── layers/main.yml           # Copied from parent
```

## Integration with Frame Capture

The test mod works seamlessly with the frame capture system:

```bash
# Baseline (safe defaults, test mod with timeouts)
cargo run -p app -- --capture-frames reports/baseline --bench 2

# Optimized (all flags, test mod with timeouts)
cargo run -p app -- --opt-comp --opt-present --opt-diff --capture-frames reports/optimized --bench 2

# Compare
export FRAME_BASELINE=reports/baseline FRAME_OPTIMIZED=reports/optimized
cargo test -p engine compare_frame_captures -- --ignored --nocapture
```

Scenes flow automatically without requiring user interaction.

## Future Use

- **CI/CD pipelines** — run automated regression tests on every commit
- **Optimization validation** — capture before/after optimization changes
- **Performance profiling** — consistent benchmark runs without interaction
- **Screenshot generation** — generate scene previews at specific frames

## Notes

- Test mod inherits all effects, postfx, and visual styling from parent mod
- Only scene lifecycle triggers are modified (any-key → timeout)
- Menu options remain defined but are never reached (auto-timeout)
- Visual content is identical to main mod
- Timing tweaks (timeout values) can be adjusted for different capture needs

## Troubleshooting

### Scenes transition too quickly
Increase timeout values in scene definitions (e.g., 3500ms → 5000ms)

### Scenes don't transition
Check that `trigger: timeout` and `timeout: XXXX ms` are set correctly

### Frames not captured
- Verify directory exists and is writable
- Check disk space
- Review logs for errors

### Frame divergences
May indicate visual glitch in optimization — debug as usual with `FRAME_BASELINE` and `FRAME_OPTIMIZED` env vars.
