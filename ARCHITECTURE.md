# Shell Quest Architecture

## 1. Repository Structure

```
shell-quest/
├── app/                       CLI launcher
├── editor/                    TUI authoring tool
├── engine/                    Runtime orchestrator (re-exports all subsystems)
├── engine-core/               Scene model, buffer, effects, metadata
├── engine-celestial/          Bodies, planet presets, regions, systems, sites, routes
├── engine-authoring/          YAML compile/normalize/schema pipeline
├── engine-3d/                 OBJ mesh loading, Scene3D definitions
├── engine-animation/          Stage/step animator
├── engine-audio/              Audio playback (rodio backend)
├── engine-audio-sequencer/    YAML song/SFX runtime + synth note-sheet generation
├── engine-behavior-registry/  Behavior definition registry
├── engine-capture/            Frame capture for regression testing
├── engine-debug/              Debug overlays, log buffer, memory stats
├── engine-error/              Shared error types (EngineError)
├── engine-events/             Input event types
├── engine-frame/              Frame ticket generation tracking
├── engine-game/               Persistent game state (key-value)
├── engine-io/                 Transport-agnostic IPC bridge (sidecar)
├── engine-mod/                Mod manifest loading (dir + zip)
├── engine-pipeline/           Backend-agnostic render pipeline strategies
├── engine-render/             Shared render traits (`RenderBackend`, `OutputBackend`)
├── engine-render-policy/      Render scheduling policies
├── engine-scene-runtime/      Mutable scene instance state + runtime cloning
├── engine-render-sdl2/        SDL2 presenter + input backend
├── engine-runtime/            RuntimeSettings, virtual-size parsing
├── mods/                      Content mods
│   ├── shell-quest/           Main game mod
│   ├── shell-quest-tests/     Automated test mod (no user input)
│   └── playground/            Development playground
├── schemas/                   JSON schemas for YAML validation
└── tools/                     schema-gen, devtool, benchmarks
```

`engine-celestial` is intentionally lightweight: it owns authored celestial
catalog data and loaders so rendering, gameplay, and scene binding can all
depend on one shared domain model without pulling in Rhai or higher-level
runtime systems.

Scenes are loaded as single YAML files (`scenes/*.yml`) or scene packages
(`scenes/<name>/scene.yml` + partials). Asset loading supports unpacked mod
directories and zip-packaged mods.

Scene composition now has an explicit space model:

- scene default: `space: 2d | 3d`
- layer override: `space: inherit | 2d | 3d | screen`

The runtime keeps both a 2D world camera (`camera_x/y`) and a shared scene 3D
camera (`eye`, `look_at`, `up`, `fov`, `near_clip`). `2d` layers consume the
2D camera, `screen` layers stay fixed, and OBJ / Scene3D sprites can opt into
the shared 3D camera with `camera-source: scene`.

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
                         │  engine-render-sdl2    │
                         │  engine-authoring      │
                         └──────────┬───────────┘
                                    │
          ───────────────── Tier 1 ─┼──────────────────────────
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
     engine-animation    engine-render         engine-runtime
     engine-audio        engine-render-policy
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
| 1 | input | `input_us` | Active `InputBackend` polling (SDL2) |
| 2 | lifecycle | `lifecycle_us` | Scene transitions, event drain |
| 3 | animator | `animator_us` | Stage/step advancement via elapsed time |
| 4 | hot_reload | `hot_reload_us` | Dev-mode file change scanning |
| 5 | engine_io | `engine_io_us` | Sidecar IPC bridge (transport-agnostic) |
| 6 | gameplay | `-` | Physics integration, lifetime ticks, component updates |
| 7 | collision | `-` | Collision detection over gameplay entities |
| 8 | gameplay_events (push) | `-` | Publish collision hits to script-visible buffer |
| 9 | behavior | `behavior_us` | Rhai script execution per behavior |
| 10 | visual_binding cleanup | `-` | Despawn scene visuals bound to expired entities before render sync/compositing |
| 11 | particle_physics (collect) | `-` | Write back async worker-particle integration results |
| 12 | particle_ramp | `-` | Apply typed particle colour/radius ramps |
| 13 | audio_sequencer | `audio_us` | Semantic SFX/song sequencing and synth cue scheduling |
| 14 | audio | `audio_us` | Audio backend tick + cue dispatch |
| 15 | visual_sync | `-` | Copy `Transform2D` positions/headings into bound scene runtime targets |
| 16 | compositor | `compositor_us` | Layer blitting + sprite rendering to the render buffer |
| 17 | postfx | `postfx_us` | Post-processing effects (scanline, glitch, etc.) |
| 18 | renderer | `renderer_us` | Double-buffer diff + active output backend present |
| 19 | gameplay_events (clear) | `-` | Clear per-frame collision buffer |
| 20 | sleep | `sleep_us` | Frame budget sleep (target FPS remainder) |

After renderer (step 18), the frame-skip oracle is notified via
`oracle.frame_advanced()`.

## 4. Strategy Pattern Architecture

All rendering strategies live in `PipelineStrategies`, a `World` resource using
trait-based dispatch. Strategies are selected from CLI flags at startup.

```rust
pub struct PipelineStrategies {
    pub diff:    Box<dyn DiffStrategy>,
    pub layer:   Box<dyn LayerCompositor>,
    pub present: Box<dyn VirtualPresenter>,
}
```

| Flag | Strategy Trait | Safe (default) | Optimized |
|------|---------------|----------------|-----------|
| `--opt-diff` | `DiffStrategy` | `FullScanDiff` | `DirtyRegionDiff` |
| `--opt-rowdiff` | `DiffStrategy` | `FullScanDiff` | `RowSkipDiff` |
| `--opt-comp` (layer) | `LayerCompositor` | `ScratchLayerCompositor` | `DirectLayerCompositor` |
| `--opt-present` | `VirtualPresenter` | `AlwaysPresenter` | `HashSkipPresenter` |
| `--opt-skip` | `FrameSkipOracle` | `AlwaysRender` | `CoordinatedSkip` |

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

### Scene3D Render Path

Scene3D assets now have two complementary runtime surfaces:

- **`Scene3DAtlas`**: pre-baked buffers for static named frames.
- **`Scene3DRuntimeStore`**: parsed Scene3D definitions kept live for clip frames.

During compositing, static Scene3D frame ids blit directly from the atlas. Bare
clip ids such as `solar-orbit` are instead evaluated against the current
`elapsed_ms` and rendered on demand from the runtime store, which keeps clip
tweens, orbit motion, and vertical reveal masks (`clip_y_min/max`) live without
forcing authors to reference generated `clip-0..N` frame names.

## 6. Buffer Architecture

The rendering pipeline uses a double-buffer with dirty tracking:

```
 Back Buffer ──(compositor writes)──► Front Buffer
      │                                    │
      └──── diff scan (strategy) ──────────┘
                     │
              dirty cells list
                     │
              SDL2 present (GlyphPatch upload)
```

- **Double buffer**: back (current frame) vs front (previous frame).
- **Dirty tracking**: bounding-box region + per-row `BitSet`.
- **Diff strategies**: `FullScanDiff` scans every cell; `DirtyRegionDiff`
  restricts to tracked bounding box; `RowSkipDiff` skips clean rows entirely.
- **SDL2 output**: dirty cells are uploaded as `GlyphPatch` records to the SDL2
  worker thread, which renders glyphs to a pixel canvas and presents the texture.

## 7. SDL2 Pixel Rendering

The SDL2 backend renders the virtual cell buffer to a pixel canvas:

```
Virtual Buffer  W x H     (cell-based in-memory canvas)
    │
    ▼
Diff scan       (DiffStrategy)
    │
    ▼
GlyphPatch list (dirty cells only)
    │
    ▼
SDL2 pixel canvas  (glyph rasterization per patch)
    │
    ▼
SDL2 texture present
```

Virtual render sizes are authored in `mod.yaml` under the `display:` block and
define the in-memory canvas dimensions independent of the window size.

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
| `free-look-camera` | Scene-level camera fly controls |

**Rhai key bridge** — variables available in behavior scripts:

| Variable | Type | Description |
|----------|------|-------------|
| `key.pressed` | `bool` | Whether a key was pressed this frame |
| `key.code` | `String` | Key name (e.g., `"a"`, `"Enter"`) |
| `key.ctrl` | `bool` | Ctrl modifier held |
| `key.alt` | `bool` | Alt modifier held |
| `key.shift` | `bool` | Shift modifier held |

**Rhai behavior state model**

- `local[]` is scoped to a single behavior instance, not the whole scene.
- Use `game.set/get` for cross-script state handoff such as entity ids needed by
  both gameplay and render-sync behaviors.
- Use `local[]` only for script-private frame-to-frame state.
- Scene runtime object state is immediate-mode. `reset_frame_state()` clears
  transient visibility/offset state before behaviors run, so scripts that drive
  parallax, camera-relative layers, or other runtime-only presentation state
  must re-emit those values every frame.

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

## 12. Editor Architecture

The editor is a YAML-first authoring tool built on `engine-core` and
`engine-authoring`. The terminal TUI has been replaced with an SDL2-backed stub;
full editor UI is not yet implemented.

**Current modules** (`editor/src/`):

| Module | Purpose |
|--------|---------|
| `app.rs` | Editor lifecycle, main editor loop |
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
| Render/compositor | Verify compositor + renderer + backend presentation interactions |
| Transitions/lifecycle | Verify scoped reset behavior, scene loader reference resolution |
| Rhai script API | `BehaviorContext`, scope push block in `RhaiScriptBehavior::update`, `AUTHORING.md`, regression tests in `engine-behavior` |
| Debug/diagnostics | Push to `DebugLogBuffer` via `BehaviorCommand::ScriptError` or direct `world.get_mut` |

When changing gameplay wrapping or bounds behavior, also verify the Rhai-facing
`world.set_world_bounds(min_x, min_y, max_x, max_y)` contract stays aligned with
the underlying runtime order.

## 14. CLI Quick Reference

### App (`cargo run -p app`)

| Flag | Description |
|------|-------------|
| `--mod <NAME>` | Mod to load by name (default: `shell-quest`) |
| `--mod-source <PATH>` | Full mod source path (dir or .zip), overrides `--mod` |
| `--dev` | Enable dev helpers (overlays, scene nav). Auto in debug builds |
| `--no-dev` | Disable dev helpers even in debug builds |
| `--sdl-window-ratio <RATIO>` | SDL startup window ratio (`16:9`, `4:3`, `free`) |
| `--sdl-pixel-scale <N>` | SDL startup pixel scale multiplier |
| `--no-sdl-vsync` | Disable SDL VSync |
| `--audio` | Enable audio playback |
| `--logs` | Force-enable run logging |
| `--no-logs` | Force-disable run logging |
| `--log-root <DIR>` | Override log root directory (default: `./logs`) |
| `--start-scene <SCENE>` | Jump to a specific scene |
| `--skip-splash` | Skip engine splash screen |
| `--check-scenes` | Run startup validation for the selected mod and exit |
| `--target-fps <FPS>` | Override target FPS (default: from mod manifest, 60) |
| `--opt` | Enable ALL optimizations |
| `--opt-comp` | Compositor optimizations (layer scratch skip) |
| `--opt-diff` | Dirty-region diff scan |
| `--opt-present` | Hash-based static frame skip |
| `--opt-skip` | Unified frame-skip coordination |
| `--opt-rowdiff` | Row-level dirty skip in diff scan |
| `--bench [SECS]` | Benchmark mode (default 5s), saves report |
| `--capture-frames <DIR>` | Capture frames for visual regression testing |

**Environment variables**: `SHELL_QUEST_DEV`, `SHELL_QUEST_DEBUG_FEATURE`, `SHELL_QUEST_MOD_SOURCE`

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
