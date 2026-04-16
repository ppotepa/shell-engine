# App Layer — CLI, Startup, Configuration

## Build Script

`app/build.rs` watches `.git/HEAD` and `.git/index` so that any commit or
staged file change triggers a rebuild. It embeds the git hash as
`BUILD_GIT_HASH`, printed in the startup banner:
`ShellEngine [a1b2c3d] initialized with mod source: …`

## CLI Flags

All flags passed via command-line or environment variables:

| Flag | Purpose | Type |
|------|---------|------|
| `--mod-source <PATH>` | Load mod from directory instead of built-in | String |
| `--sdl-window-ratio <RATIO>` | SDL startup ratio (`16:9`, `4:3`, `free`) | String |
| `--sdl-pixel-scale <N>` | SDL startup logical pixel scale | Number |
| `--no-sdl-vsync` | Disable SDL VSync | Boolean |
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
  render_size: 320x180
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

## Engine Integration Points

- **PipelineStrategies**: Created via `PipelineFlags::from_cli()` → strategies chosen per flag
- **BenchmarkState**: Registered once, populated per frame
- **World setup**: Subsystems added via `app.register_system()`
- **Event loop**: Routed through `InputBackend` (`Sdl2InputBackend`), frame pacing via sleep
