# Changelog

All notable changes to Shell Quest will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Prologue architecture** (March 2026)
  - `Difficulty` enum: 5 levels (MouseEnjoyer → Su) with `MachineSpec` hardware config
  - `MachineSpec.FromDifficulty()` single source of truth for CPU/RAM/NIC/disk per difficulty
  - Engine passes difficulty from `GameState` to sidecar via `IoRequest::Hello`

- **Shell commands** (March 2026)
  - `cd` — change directory with relative/absolute/~ path support
  - `pwd` — print working directory (shows /home/linus equivalent)
  - `cp` — copy files via `IMutableFileSystem`, checks disk space from MachineSpec
  - `ftp` — enter FTP client mode, switches `SessionMode` to FtpSession
  - `CommandContext.ResolvePath()` — cwd-aware path resolution for all commands

- **FTP client session** (March 2026)
  - Full FTP simulation: open, binary, ascii, put, ls, cd, pwd, status, bye
  - Default mode is ASCII (historically accurate — corrupts binary archives)
  - Simulated DNS table, anonymous login, transfer delays from NIC speed
  - Core prologue puzzle: player must discover `binary` command

- **Mutable filesystem** (March 2026)
  - `IMutableFileSystem` interface: TryCopy, TryWrite, TryMkdir
  - `ZipVirtualFileSystem` implements mutable ops as in-memory overlay
  - `SeedEpochFiles()` populates linux-0.01/ working tree at boot

- **Quest state tracking** (March 2026)
  - `QuestState` in Models.cs: FtpTransferMode, UploadAttempted, BackupMade, UploadSuccess
  - `AppHost` routes input between Shell and FtpSession modes
  - Boot sequence hardware lines driven by MachineSpec

- **Timeline validation system** (March 2026)
  - Compile-time sprite timeline validation in `engine-authoring`
  - Warns when sprite `appear_at_ms >= scene_duration` (will never be visible)
  - Warns when sprite `disappear_at_ms <= appear_at_ms` (always hidden)
  - Debug mode only validation with stderr warnings
  - Added `Stage::duration_ms()` and `Scene::on_enter_duration_ms()` helpers
  - Test coverage: 3 regression tests for validation diagnostics
  
- **Snap teleport lighting for OBJ sprites** (March 2026)
  - `light-point-snap-hz` and `light-point-2-snap-hz` fields
  - Instant pseudo-random position jumps (no smooth interpolation)
  - Deterministic hash-based angle generation with different seeds per light
  - Priority: snap > orbit > static lighting modes
  - Applied to difficulty menu 3D portraits for dramatic neon effect

- **Documentation expansion** (March 2026)
  - New `timeline-architecture.md` — complete timeline system documentation
  - New `obj-lighting.md` — 3D lighting features (directional, point, orbit, snap, cel)
  - Updated `scene-centric-authoring.md` with timeline section (9.1)
  - Updated `README.md` with full documentation index

### Fixed
- **Boot sequence choreography** (March 2026)
  - Fixed VGA/BIOS sprite "leak" into login scene (was authoring error, not engine bug)
  - Removed problematic terminal-monitor object from `05-intro-cpu-on` scene
  - Logo shortened to 2s (was 8.5s)
  - BIOS compressed to 6s with white CPU text for visibility
  - Boot GIF timing restored to match actual duration (10530ms)
  - Realistic boot delays in C# sidecar (longer I/O, shorter init)
  
- **Scene transition cleanup verified** (March 2026)
  - Confirmed `world.clear_scoped()` properly cleans up SceneRuntime
  - No sprite state leak between scenes (architecture working as designed)

### Changed
- **Timeline architecture clarification** (March 2026)
  - Documented that sprite timing is **absolute** (relative to scene start, not layer)
  - Documented that layers have no `appear_at_ms`/`disappear_at_ms` fields
  - Documented that layer visibility does NOT cascade to children timeline
  - Scene duration for timing is `on_enter` stage only

## Architecture Decisions

### Timeline System (March 2026)

**Decision**: Keep sprite timing absolute (scene-relative) rather than layer-relative.

**Rationale**:
- Current architecture: layers have static `visible: bool`, sprites have `appear_at_ms`/`disappear_at_ms`
- Adding layer timeline would require significant engine redesign
- Relative timing modes increase complexity and debugging difficulty
- Compile-time validation catches authoring errors effectively
- Runtime `layer.visible` control via Rhai provides dynamic visibility

**Alternatives considered**:
1. Add `appear_at_ms`/`disappear_at_ms` to Layer struct
2. Support relative sprite timing to layer start
3. Hierarchical visibility cascade from layer to children
4. Timeline compiler with automatic clamping

**Future improvements** (not prioritized):
- These would require changing authoring model
- Need careful design to avoid confusion
- Current validation approach is sufficient

### Snap vs Orbit Lighting (March 2026)

**Decision**: Prioritize snap teleport over smooth orbit when both are specified.

**Rationale**:
- Snap has zero interpolation overhead (deterministic)
- More dramatic visual impact for static portraits
- Smooth orbit better for animated scenes
- Priority order allows easy override: `snap-hz > orbit-hz > static`

**Use cases**:
- Snap: difficulty menu, character select, dramatic reveals
- Orbit: cutscenes, interactive 3D exploration
- Static: performance-critical scenes, many simultaneous OBJ sprites

## Migration Notes

### Timeline Validation (March 2026)

If you see warnings like:
```
⚠️  Scene 'X': sprite #2 in layer 'Y' has appear_at_ms=8200 
    but on_enter ends at 6000ms (sprite will never be visible)
```

**Fix**: Adjust sprite `appear_at_ms` to be within scene `on_enter` duration:
```yaml
# Before (broken)
stages:
  on_enter:
    steps:
      - duration: 6000
layers:
  - sprites:
      - appear_at_ms: 8200  # ❌ Beyond scene duration

# After (fixed)
stages:
  on_enter:
    steps:
      - duration: 10000  # Extend scene OR...
layers:
  - sprites:
      - appear_at_ms: 5500  # ...reduce sprite timing
```

**Note**: Validation only runs in debug builds (`cargo build`), not release builds.

### Snap Lighting Migration (March 2026)

To convert from orbit to snap:

```yaml
# Before (smooth orbit)
sprites:
  - type: obj
    light-point-orbit-hz: 0.1

# After (instant snap)
sprites:
  - type: obj
    light-point-snap-hz: 0.2  # ~5s intervals
```

Both fields can coexist (snap takes priority), allowing gradual migration.

## Testing

All changes include test coverage:
- **206 engine tests** pass
- **73 engine-authoring tests** pass (includes 3 new timeline validation tests)
- **79 engine-core tests** pass

Run tests:
```bash
cargo test -p engine
cargo test -p engine-authoring
cargo test -p engine-core
```

## See Also

- **Timeline**: `timeline-architecture.md`
- **Lighting**: `obj-lighting.md`
- **Authoring**: `scene-centric-authoring.md`
- **Tooling**: `AGENTS.md`
