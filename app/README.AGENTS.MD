# App Layer — CLI, Startup, Configuration

## Build Script

`app/build.rs` watches `.git/HEAD` and `.git/index` so that any commit or
staged file change triggers a rebuild. It embeds the git hash as
`BUILD_GIT_HASH`, printed in the startup banner:
`ShellEngine [a1b2c3d] initialized with mod source: …`

## Cargo Features (Hardware-Only)

- Default build (`cargo run -p app`) enables `hardware-backend`.
- `hardware-backend` forwards to `engine/hardware-backend`.
- No software backend Cargo feature is available.

## CLI Flags

All flags passed via command-line or environment variables:

| Flag | Purpose | Type |
|------|---------|------|
| `--mod-source <PATH>` | Load mod from directory instead of built-in | String |
| `--render-backend <software|hardware>` | Select render backend family (`software` currently rejected; `hardware` is default) | String |
| `--compat-window-ratio <RATIO>` | Legacy compatibility arg; ignored on hardware (`16:9`, `4:3`, `free`) | String |
| `--compat-pixel-scale <N>` | Legacy compatibility arg; ignored on hardware | Number |
| `--no-compat-vsync` | Legacy compatibility arg; ignored on hardware | Boolean |
| `--bench <SECS>` | Benchmark mode, run for N seconds | Number |
| `--capture-frames <DIR>` | Capture binary frames to directory | String |
| `--audio` | Enable audio output (default: off) | Boolean |
| `--start-scene <SCENE>` | Jump directly to a scene path | String |
| `--skip-splash` | Skip splash screen (auto-on for `--bench`) | Boolean |
| `--target-fps <FPS>` | Override target FPS | Number |
| `--check-scenes` | Run startup validation and exit | Boolean |
| `--opt-comp` | Compositor optimizations | Boolean |
| `--no-opt-comp` | Disable default compositor optimizations | Boolean |
| `--opt-diff` | Dirty-region diff scan | Boolean |
| `--opt-present` | Hash-based frame skip | Boolean |
| `--opt-skip` | FrameSkipOracle (frame dedup) | Boolean |
| `--opt-rowdiff` | Row-level dirty skip | Boolean |
| `--no-opt-rowdiff` | Disable default row diff optimization | Boolean |
| `--opt-async` | Async display sink | Boolean |
| `--opt` | Enable ALL optimizations | Boolean |
| `--debug-feature` | Enable debug overlays (F1, ~, F3/F4) — alias for `--dev` | Boolean |
| `--dev` | Enable dev helpers (F1 overlay, scene nav). On by default in debug builds | Boolean |
| `--no-dev` | Disable dev helpers even in debug builds | Boolean |

### Debug overlay (F1, Tab, ~)

Visible when `--debug-feature` is on. The Stats panel shows scene id, stage, render sizing, per-system timings, and the live object count; recent script errors are listed when present. Tab switches Stats/Logs, `~` toggles visibility.

## Configuration

**EngineConfig** (set from CLI flags):
```
pub struct EngineConfig {
    pub bench_secs: Option<f32>,           // --bench <SECS>
    pub capture_frames_dir: Option<PathBuf>, // --capture-frames <DIR>
    pub audio: bool,                       // --audio
    pub debug_feature: bool,               // --dev / --debug-feature
}
```

**Mod manifest display settings**:
```yaml
display:
  world_render_size: 320x180
```

**PipelineFlags** (optimization toggles):
```
pub struct PipelineFlags {
    pub opt_comp: bool,      // --opt-comp
    pub opt_diff: bool,      // --opt-diff
    pub opt_present: bool,   // --opt-present
    pub opt_skip: bool,      // --opt-skip
    pub opt_rowdiff: bool,   // --opt-rowdiff
}
```

## App Startup Flow

1. **Parse CLI args** (`cli.rs`)
   - Extract `--mod-source`, `--bench`, `--capture-frames`, opt flags
   - Set `EngineConfig` and `PipelineFlags`

2. **Optional startup validation** (`--check-scenes`)
   - Run scene graph, level config, Rhai script, effect/font/image, and audio sequencer checks
   - Current compatibility behavior: scene checks emit `StartupOutputSetting::Compatibility` via `StartupOutputSetting::compatibility_default()`
   - Print report and exit before engine boot

3. **Initialize audio**
   - Select audio backend (RodioAudioBackend or NullAudioBackend)
   - Synthesize mod `audio/synth/*.yml` note sheets into in-memory cue buffers
   - Load optional `audio/sfx.yaml` and `audio/songs/**/*.yml`

4. **Load mod** (if `--mod-source` provided)
   - Otherwise use built-in shell-engine mod
   - Load scenes, assets, schemas

5. **Create World** (engine registry)
   - Register all systems
   - Register `PipelineStrategies` (optimization strategy implementations)
   - Register `BenchmarkState` if `--bench` active

6. **Run main loop** (`game_loop.rs`)
   - Per-frame: input → lifecycle → animate → hot_reload → engine_io → behavior → audio_sequencer → audio → compose → postfx → render
   - Optional: capture frames (if `--capture-frames`)
   - Optional: benchmark sampling (if `--bench`)

## Environment Variables

- `SHELL_ENGINE_MOD_SOURCE` — Same as `--mod-source` (CLI takes precedence)

## Validation Notes (2026-04-23)

- `app/src/main.rs` validates that `--render-backend software` is rejected (`validate_render_backend_selection`, tests `software_backend_is_rejected_by_backend_validation` and `software_backend_compat_options_return_unavailable_error`).
- `app/src/main.rs` exposes compat-only legacy knobs (`--compat-window-ratio`, `--compat-pixel-scale`, `--no-compat-vsync`) and does not expose `--sdl-*` aliases.
- `app/src/main.rs` uses `StartupOutputSetting::compatibility_default()` for scene checks (`output_for_scene_checks`, test `scene_checks_use_compat_output_setting`).
- `rg -n -F "StartupOutputSetting::Sdl2" app/src engine/src launcher/src engine-mod/src` returns no matches.
- `rg -n "sdl-window-ratio|sdl-pixel-scale|no-sdl-vsync" app/src launcher/src -g "*.rs"` returns no matches.
- `rg -n "is_pixel_backend" engine-runtime/src engine-render-policy/src -g "*.rs"` returns no matches.

## Engine Integration Points

- **PipelineStrategies**: Created via `PipelineFlags::from_cli()` → strategies chosen per flag
- **BenchmarkState**: Registered once, populated per frame
- **World setup**: Subsystems added via `app.register_system()`
- **Event loop**: Routed through backend-specific `InputBackend` implementation, frame pacing via sleep
