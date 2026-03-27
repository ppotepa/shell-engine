# App Layer — CLI, Startup, Configuration

## CLI Flags

All flags passed via command-line or environment variables:

| Flag | Purpose | Type |
|------|---------|------|
| `--mod-source <PATH>` | Load mod from directory instead of built-in | String |
| `--bench <SECS>` | Benchmark mode, run for N seconds | Number |
| `--capture-frames <DIR>` | Capture binary frames to directory | String |
| `--output <terminal\|sdl2\|prompt>` | Override startup backend selection | String |
| `--audio` | Enable audio output (default: off) | Boolean |
| `--opt-comp` | Compositor optimizations | Boolean |
| `--opt-diff` | Dirty-region diff scan | Boolean |
| `--opt-present` | Hash-based frame skip | Boolean |
| `--opt-skip` | FrameSkipOracle (frame dedup) | Boolean |
| `--opt-rowdiff` | Row-level dirty skip | Boolean |
| `--opt` | Enable ALL optimizations | Boolean |
| `--debug-feature` | Enable debug overlays (F1, ~, F3/F4) | Boolean |

## Configuration

**EngineConfig** (set from CLI flags):
```
pub struct EngineConfig {
    pub output_backend: BackendKind,       // resolved from --output or mod.yaml
    pub renderer_mode: Option<String>,     // --renderer-mode <MODE>
    pub bench_secs: Option<f32>,           // --bench <SECS>
    pub capture_frames_dir: Option<PathBuf>, // --capture-frames <DIR>
    pub audio: bool,                       // --audio
    pub debug_feature: bool,               // --dev / --debug-feature
}
```

**Mod manifest startup output**:
```yaml
output_backend: prompt   # terminal | sdl2 | prompt
```

Precedence:
1. CLI `--output` when provided
2. mod.yaml `output_backend`
3. fallback `terminal`

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
   - Extract `--mod-source`, `--output`, `--bench`, `--capture-frames`, opt flags
   - Set `EngineConfig` and `PipelineFlags`

2. **Initialize audio** (if `--audio` enabled)
   - Select audio backend (RodioAudioBackend or NullAudioBackend)
   - Scan mod assets for WAV/MP3/OGG files

3. **Load mod** (if `--mod-source` provided)
   - Otherwise use built-in shell-quest mod
   - Load scenes, assets, schemas

4. **Create World** (engine registry)
   - Register all systems
   - Register `PipelineStrategies` (optimization strategy implementations)
   - Register `BenchmarkState` if `--bench` active

5. **Run main loop** (game_loop.rs)
   - Per-frame: input → lifecycle → animate → compose → postfx → render
   - Optional: capture frames (if `--capture-frames`)
   - Optional: benchmark sampling (if `--bench`)

## Environment Variables

- `SHELL_QUEST_MOD_SOURCE` — Same as `--mod-source` (CLI takes precedence)

## Engine Integration Points

- **PipelineStrategies**: Created via `PipelineFlags::from_cli()` → strategies chosen per flag
- **BenchmarkState**: Registered once, populated per frame
- **World setup**: Subsystems added via `app.register_system()`
- **Event loop**: Routed through `InputBackend` (`TerminalInputBackend` or `Sdl2InputBackend`), frame pacing via sleep
