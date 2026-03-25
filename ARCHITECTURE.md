# Shell Quest Architecture

## 1. Repository Structure

```
shell-quest/
├── app/                       CLI launcher
├── editor/                    TUI authoring tool
├── engine/                    Runtime orchestrator (re-exports all subsystems)
├── engine-core/               Scene model, buffer, effects, metadata
├── engine-authoring/          YAML compile/normalize/schema pipeline
├── engine-3d/                 OBJ mesh loading, Scene3D definitions
├── engine-animation/          Stage/step animator
├── engine-audio/              Audio playback (rodio backend)
├── engine-behavior-registry/  Behavior definition registry
├── engine-capture/            Frame capture for regression testing
├── engine-debug/              Debug overlays, log buffer, memory stats
├── engine-error/              Shared error types (EngineError)
├── engine-events/             Input event types
├── engine-frame/              Frame ticket generation tracking
├── engine-game/               Persistent game state (key-value)
├── engine-io/                 Transport-agnostic IPC bridge (sidecar)
├── engine-mod/                Mod manifest loading (dir + zip)
├── engine-pipeline/           PipelineFlags (pure data)
├── engine-render/             RenderBackend trait
├── engine-render-policy/      Render scheduling policies
├── engine-render-terminal/    Crossterm terminal presenter
├── engine-runtime/            RuntimeSettings, virtual-size parsing
├── engine-terminal/           Terminal detection and configuration
├── mods/                      Content mods
│   ├── shell-quest/           Main game mod
│   ├── shell-quest-tests/     Automated test mod (no user input)
│   └── playground/            Development playground
├── schemas/                   JSON schemas for YAML validation
└── tools/                     schema-gen, devtool, benchmarks
```

Scenes are loaded as single YAML files (`scenes/*.yml`) or scene packages
(`scenes/<name>/scene.yml` + partials). Asset loading supports unpacked mod
directories and zip-packaged mods.

## 2. Crate Dependency Graph

```
                         ┌─────────┐  ┌────────┐
                         │   app   │  │ editor │
                         └────┬────┘  └───┬────┘
                              │            │
                         ┌────▼────────────▼────┐
          Tier Top       │       engine         │  (re-exports everything)
                         └──────────┬───────────┘
                                    │
          ───────────────── Tier 3 ─┼──────────────────────────
                         ┌──────────┴───────────┐
                         │  engine-mod           │
                         │  engine-io            │
                         └──────────┬───────────┘
                                    │
          ───────────────── Tier 2 ─┼──────────────────────────
                         ┌──────────┴───────────┐
                         │  engine-render-terminal│
                         │  engine-authoring      │
                         └──────────┬───────────┘
                                    │
          ───────────────── Tier 1 ─┼──────────────────────────
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
     engine-animation    engine-render         engine-runtime
     engine-audio        engine-render-policy  engine-terminal
     engine-3d           engine-behavior-registry
     engine-capture      (all depend on engine-core)
              │                     │                     │
          ───────────────── Tier 0 ─┼──────────────────────────
              │                     │                     │
     engine-pipeline     engine-frame          engine-debug
     engine-error        engine-events         engine-game
              (no engine dependencies)
```

## 3. Per-Frame Systems

Executed in this fixed order every frame inside `game_loop.rs`:

| # | System | Timing Field | Purpose |
|---|--------|-------------|---------|
| 1 | input | `input_us` | Crossterm event polling (keyboard, mouse, resize) |
| 2 | lifecycle | `lifecycle_us` | Scene transitions, event drain |
| 3 | animator | `animator_us` | Stage/step advancement via elapsed time |
| 4 | hot_reload | `hot_reload_us` | Dev-mode file change scanning |
| 5 | engine_io | `engine_io_us` | Sidecar IPC bridge (transport-agnostic) |
| 6 | behavior | `behavior_us` | Rhai script execution per behavior |
| 7 | audio | `audio_us` | Audio playback tick |
| 8 | compositor | `compositor_us` | Layer blitting + sprite rendering to virtual buffer |
| 9 | postfx | `postfx_us` | Post-processing effects (scanline, glitch, etc.) |
| 10 | renderer | `renderer_us` | Double-buffer diff + terminal flush pipeline |
| 11 | sleep | `sleep_us` | Frame budget sleep (target FPS remainder) |

After step 10, the frame-skip oracle is notified via `oracle.frame_advanced()`.

## 4. Strategy Pattern Architecture

All rendering strategies live in `PipelineStrategies`, a `World` resource using
trait-based dispatch. Strategies are selected from CLI flags at startup.

```rust
pub struct PipelineStrategies {
    pub diff:      Box<dyn DiffStrategy>,
    pub layer:     Box<dyn LayerCompositor>,
    pub halfblock: Box<dyn HalfblockPacker>,
    pub present:   Box<dyn VirtualPresenter>,
    pub flush:     Box<dyn TerminalFlusher>,
}
```

| Flag | Strategy Trait | Safe (default) | Optimized |
|------|---------------|----------------|-----------|
| `--opt-diff` | `DiffStrategy` | `FullScanDiff` | `DirtyRegionDiff` |
| `--opt-rowdiff` | `DiffStrategy` | `FullScanDiff` | `RowSkipDiff` |
| `--opt-comp` (layer) | `LayerCompositor` | `ScratchLayerCompositor` | `DirectLayerCompositor` |
| `--opt-comp` (pack) | `HalfblockPacker` | `FullScanPacker` | `DirtyRegionPacker` |
| `--opt-present` | `VirtualPresenter` | `AlwaysPresenter` | `HashSkipPresenter` |
| `--opt-skip` | `FrameSkipOracle` | `AlwaysRender` | `CoordinatedSkip` |

Additional non-flagged strategies:

| Trait | Implementations | Selection |
|-------|----------------|-----------|
| `TerminalFlusher` | `NaiveFlusher`, `AnsiBatchFlusher` | `AnsiBatchFlusher` is default |
| `SceneCompositor` | `CellSceneCompositor`, `HalfblockSceneCompositor` | Auto from renderer mode |

The umbrella flag `--opt` enables all optimizations at once.

## 5. Scene Model

```
Scene
├── stages[]
│   └── steps[]
│       └── effects[]         (per-step visual effects)
├── layers[]
│   └── sprites[]             (positioned images with timing)
├── objects[]                  (named data objects)
├── behaviors[]                (Rhai scripts)
├── postfx[]                   (post-processing chain)
└── next                       (scene transition target)
```

Scenes are authored in YAML. The `engine-authoring` crate compiles raw YAML into
the normalized `Scene` model consumed by the runtime. Scene packages split layers,
sprites, and effects into partial YAML files merged at load time.

## 6. Buffer Architecture

The rendering pipeline uses a double-buffer with dirty tracking:

```
 Back Buffer ──(compositor writes)──► Front Buffer
      │                                    │
      └──── diff scan (strategy) ──────────┘
                     │
              dirty cells list
                     │
              terminal flush (AnsiBatchFlusher)
```

- **Double buffer**: back (current frame) vs front (previous frame).
- **Dirty tracking**: bounding-box region + per-row `BitSet`.
- **Diff strategies**: `FullScanDiff` scans every cell; `DirtyRegionDiff`
  restricts to tracked bounding box; `RowSkipDiff` skips clean rows entirely.

## 7. Halfblock Rendering

Halfblock mode doubles vertical resolution by encoding two pixel rows per
terminal cell using Unicode half-block characters.

```
Terminal  W x H     (physical terminal size)
    │
    ▼
Virtual   W x (H*2) (double-height pixel buffer)
    │
    ▼
Output    W x H     (packed halfblock cells)
```

Recommended virtual-size tiers:

| Tier | Resolution | Notes |
|------|-----------|-------|
| 1 (safe) | 120 x 60 | Works on most terminals |
| 2 (recommended) | 160 x 80 | Good balance of detail and performance |
| 3 (high-res) | 200+ x 100+ | Requires `--unconstrained` or large terminal |

## 8. Timeline System

Sprite timing uses **absolute** offsets relative to scene start (not relative to
layer or stage start). Key rules:

- `visible-from` / `visible-to` on sprites are absolute milliseconds from scene
  entry.
- Layer visibility is a **static boolean** (`visible: true/false`), not animated.
- In debug mode, validation warns about impossible timings (e.g., `visible-from`
  after scene duration).

## 9. Input System

**Input profiles** configure which key bindings are active:

| Profile | Use Case |
|---------|----------|
| `obj-viewer` | 3D object viewer controls |
| `terminal-size-tester` | Terminal capability probing |
| `terminal-shell` | Full shell interaction mode |

**Rhai key bridge** — variables available in behavior scripts:

| Variable | Type | Description |
|----------|------|-------------|
| `key.pressed` | `bool` | Whether a key was pressed this frame |
| `key.code` | `String` | Key name (e.g., `"a"`, `"Enter"`) |
| `key.ctrl` | `bool` | Ctrl modifier held |
| `key.alt` | `bool` | Alt modifier held |
| `key.shift` | `bool` | Shift modifier held |

**Debug keys** (when `--dev` is active):

| Key | Action |
|-----|--------|
| F1 | Toggle stats overlay (scene id, virtual size, errors) |
| ~ / ` | Toggle logs overlay (recent runtime log entries) |
| F3 | Previous scene |
| F4 | Next scene |

## 10. Logging System

**Log path**: `logs/<dd-mm-yy>/run-XXX/run.log`

**Activation**:

| Condition | Logging |
|-----------|---------|
| `--logs` | Force on |
| `--no-logs` | Force off |
| Debug build | On by default |
| Release build | Off unless `--logs` or env var |

**In-memory buffers**:

| Buffer | Capacity | Purpose |
|--------|----------|---------|
| Ring buffer | 500 entries | Debug overlay (~ key) |
| `DebugLogBuffer` | 64 entries | Rhai script errors (F1 overlay) |

`DebugLogBuffer` is fed by `BehaviorCommand::ScriptError` emitted on Rhai
compile/runtime failures.

## 11. Schema System

JSON schemas are generated from Rust types and written per-mod for YAML
validation in editors.

| Command | Purpose |
|---------|---------|
| `cargo run -p schema-gen -- --all-mods` | Regenerate all schemas |
| `cargo run -p schema-gen -- --all-mods --check` | Drift check (CI) |
| `./refresh-schemas.sh` | Helper script |

## 12. Editor Architecture

The editor is a YAML-first TUI authoring tool built on `engine-core` and
`engine-authoring`.

**Current modules** (`editor/src/`):

| Module | Purpose |
|--------|---------|
| `app.rs` | Terminal lifecycle, main editor loop |
| `cli.rs` | CLI options (`--mod-source`) |
| `domain/` | Scene/effect/asset indexes, diagnostics |
| `io/` | File scanning, YAML I/O |
| `input/` | Key mapping, commands |
| `state/` | Application state |
| `ui/` | Draw, layout, focus, filter, theme |

**Current features**: project explorer, scene preview, effects browser, file
editor, start screen, live refresh (~1.2s).

**Target architecture**: modular window system with `Window` trait,
`LayoutManager`, and `WindowRegistry`.

## 13. Change Playbook

| Change Type | Files to Update |
|------------|----------------|
| Scene model/fields | `engine-core` model, `engine-authoring` compile/normalize, schema surfaces, runtime consumption |
| Effect params | Effect metadata, schema generation, editor consumption |
| Render/compositor | Verify compositor + renderer + virtual buffer interactions |
| Transitions/lifecycle | Verify scoped reset behavior, scene loader reference resolution |
| Rhai script API | `BehaviorContext`, scope push block in `RhaiScriptBehavior::update`, `scene-centric-authoring.md` sec 13, regression test in `behavior::tests` |
| Debug/diagnostics | Push to `DebugLogBuffer` via `BehaviorCommand::ScriptError` or direct `world.get_mut` |

## 14. CLI Quick Reference

### App (`cargo run -p app`)

| Flag | Description |
|------|-------------|
| `--mod <NAME>` | Mod to load by name (default: `shell-quest`) |
| `--mod-source <PATH>` | Full mod source path (dir or .zip), overrides `--mod` |
| `--renderer-mode <MODE>` | Force renderer: `cell`, `halfblock`, `quadblock`, `braille` |
| `--dev` | Enable dev helpers (overlays, scene nav). Auto in debug builds |
| `--no-dev` | Disable dev helpers even in debug builds |
| `--audio` | Enable audio playback |
| `--logs` | Force-enable run logging |
| `--no-logs` | Force-disable run logging |
| `--log-root <DIR>` | Override log root directory (default: `./logs`) |
| `--start-scene <SCENE>` | Jump to a specific scene |
| `--skip-splash` | Skip engine splash screen |
| `--target-fps <FPS>` | Override target FPS (default: from mod manifest, 60) |
| `--opt` | Enable ALL optimizations |
| `--opt-comp` | Compositor optimizations (layer skip, dirty halfblock) |
| `--opt-diff` | Dirty-region diff scan |
| `--opt-present` | Hash-based static frame skip |
| `--opt-skip` | Unified frame-skip coordination |
| `--opt-rowdiff` | Row-level dirty skip in diff scan |
| `--opt-async` | Async display sink (background terminal I/O thread) |
| `--bench [SECS]` | Benchmark mode (default 5s), saves report |
| `--capture-frames <DIR>` | Capture frames for visual regression testing |

**Environment variables**: `SHELL_QUEST_DEV`, `SHELL_QUEST_MOD_SOURCE`

### Editor (`cargo run -p editor`)

| Flag | Description |
|------|-------------|
| `--mod-source <PATH>` | Path to mod root (default: `mods/shell-quest`) |
| `--logs` | Force-enable run logging |
| `--no-logs` | Force-disable run logging |
| `--log-root <DIR>` | Override log root directory |

### Devtool (`cargo run -p devtool`)

| Subcommand | Description |
|------------|-------------|
| `create mod <name>` | Scaffold a new mod |
| `create scene` | Create a new scene package |
| `create layer` | Create a new layer partial |
| `create sprite <src>` | Copy asset and add sprite entry |
| `create effect` | Create effects partial |
| `edit sprite` | Edit sprite properties |
| `schema refresh` | Regenerate per-mod local schemas |
| `schema check` | Check schema drift |
