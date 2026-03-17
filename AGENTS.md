# AGENTS.md

Local machine guide for AI agents working in this repository.
This file is intentionally local/ignored (`**/*.md` ignored; only root `README.md` is tracked).

## 1) Project identity

`shell-quest` is a Rust workspace with three connected products:
- terminal game runtime (`app` + `engine`)
- shared runtime/authoring contracts (`engine-core`, `engine-authoring`)
- terminal editor for content workflows (`editor`)

Content is YAML-first (`mods/*`). Runtime is fixed-step and system-based.

## 2) Workspace map (what each crate owns)

- `app/`
  - CLI entrypoint for running the game.
  - Picks mod source from `--mod-source` or `SHELL_QUEST_MOD_SOURCE`.
  - Creates and runs `engine::ShellEngine`.

- `engine/`
  - Runtime orchestration (boot, event loop, systems, scene transitions, terminal output).
  - Owns runtime-only concerns: `World`, lifecycle manager, renderer integration, repositories.
  - System chain: `animator -> behavior -> audio -> compositor -> renderer`.

- `engine-core/`
  - Pure/shared model + primitives.
  - Scene model (`scene`), effects runtime primitives (`effects`), terminal buffers (`buffer`), animations, markup.
  - Used by runtime, editor, and authoring tooling.

- `engine-authoring/`
  - Authoring pipeline boundary.
  - Compile/validate/schema/package/repository/document logic for YAML content.
  - Runtime scene compile in `engine` delegates to this crate.

- `editor/`
  - TUI editor/browser (`ratatui`, `crossterm`).
  - Modules: `domain`, `io`, `input`, `state`, `ui`.
  - Reads mod content and shared metadata; does not execute game loop.

- `mods/shell-quest/`
  - Main gameplay mod (entrypoint, scenes, assets, schemas).

- `mods/playground/`
  - Sandbox mod for feature demos and experiments (including 3D scenes).

- `tools/schema-gen/`
  - Generates per-mod schema fragments from authoring metadata/contracts.
  - Supports `--all-mods` and `--check`.

- `tools/ttf-rasterizer/`
  - Font tooling support.

## 3) Runtime architecture from YAML to terminal

### 3.1 Authoring input layer

Primary input files:
- `mod.yaml`
  - identity/version/entrypoint
  - terminal constraints + runtime settings (`use_virtual_buffer`, `virtual_size`, `virtual_policy`, renderer override)
- scene YAML package(s) under `scenes/`
  - scene metadata, stages/steps/effects, layers/sprites, behaviors, menu options, audio cues
- object/asset YAML references

Authoring principle:
- authored YAML is the editable source
- runtime gets compiled/normalized typed model

### 3.2 Loading and compilation

`engine/src/repositories.rs`:
- creates repository by source type (filesystem mod dir or zip mod archive)
- discovers scenes
- loads/assembles scene packages (`scene.yml` + partials) via `engine-authoring::package`
- compiles scene YAML into typed runtime `Scene` via `scene_compiler`

`engine/src/scene_loader.rs`:
- builds scene id index
- resolves transition refs:
  - `/scenes/x.yml` => path
  - `scene-id` => id lookup

Startup checks run before entering game loop (`engine/src/pipelines/startup/*`).

### 3.3 Boot and world construction

`ShellEngine::run` (`engine/src/lib.rs`):
1. Load manifest + entrypoint.
2. Run startup pipeline checks.
3. Load entry scene.
4. Read terminal/runtime settings.
5. Build `World` resources:
   - `EventQueue`
   - output `Buffer`
   - optional `VirtualBuffer`
   - `AudioRuntime`
   - `RuntimeSettings`
   - `AssetRoot`
   - `TerminalRenderer`
   - `SceneLoader`
   - scoped `SceneRuntime`
   - scoped `Animator`
6. Enter fixed-step loop.

### 3.4 World/resource model

`engine/src/world.rs`:
- type-erased container split into:
  - singletons (persistent across scene transitions)
  - scoped (cleared on transition)

`engine/src/services.rs`:
- `EngineWorldAccess` provides typed accessors used by systems.

Design intent:
- cheap resource sharing without full ECS overhead
- explicit scoped reset semantics for per-scene state

### 3.5 Per-frame pipeline

`engine/src/game_loop.rs`:
1. Poll input events (key/mouse/resize), push to `EventQueue`.
2. Drain queue.
3. Lifecycle handling (`SceneLifecycleManager`).
4. Run systems in fixed order:
   - `animator_system`
   - `behavior_system`
   - `audio_system`
   - `compositor_system`
   - `renderer_system`
5. Sleep to target FPS budget.

Why fixed order:
- deterministic timing/state transitions
- predictable behavior->render data flow

### 3.6 Animator (time and scene stage control)

`engine/src/systems/animator.rs`:
- tracks `SceneStage` (`OnEnter`, `OnIdle`, `OnLeave`, `Done`)
- tracks `step_idx`, `elapsed_ms`, stage and scene elapsed time
- emits scene transition event when finished and next scene exists
- handles empty and zero-duration steps safely (no freeze)
- loops stage when configured

### 3.7 Behavior system (runtime state mutation)

`engine/src/systems/behavior.rs` + `engine/src/behavior.rs`:
- resets transient runtime state each frame
- evaluates behavior specs attached to scene/layer/sprite
- updates visibility/offset/effect-targeted runtime state
- emits commands (including audio cue commands)

Behavior result drives compositor decisions.

### 3.8 Scene runtime materialization and resolver

`engine/src/scene_runtime.rs`:
- converts typed `Scene` into runtime object graph
- builds stable object IDs for scene/layer/sprite hierarchy
- stores mutable runtime object state
- stores target resolver maps (id/name/alias/path lookup)
- caches object regions for effect targeting
- tracks camera/object-viewer runtime state for interactive OBJ scenes

Critical mapping rule:
- resolver index maps must stay aligned with z-sorted runtime scene structure.

### 3.9 Lifecycle and transitions

`engine/src/systems/scene_lifecycle.rs`:
- classifies quit/key/mouse/transition/resize events
- handles any-key and menu-driven transition behavior
- applies scene transitions:
  - load by ref
  - apply scene virtual size override if configured
  - clear scoped resources
  - register fresh `SceneRuntime` + `Animator`
- handles terminal resize and virtual buffer resize in max-available mode

### 3.10 Composition and rendering

`engine/src/systems/compositor/*`:
- reads authored scene + runtime state + stage/effect context
- renders scene into selected target buffer
- supports render modes from `SceneRenderedMode`:
  - `cell`
  - `halfblock`
  - `quadblock`
  - `braille`
- applies effects with target resolution (scene/layer/sprite scope)

`engine/src/systems/renderer.rs`:
- optional present from virtual buffer to output buffer (fit/strict policy)
- computes buffer diff
- flushes only changed terminal cells
- swaps front/back buffers

Performance pattern:
- minimize terminal writes (diff flush)
- keep virtual/output responsibilities separate

### 3.11 Audio

`engine/src/systems/audio.rs`:
- flushes queued audio commands each frame
- behavior system is primary command producer

## 4) Scene/effect model details relevant for agents

`engine-core/src/scene/model.rs`:
- Scene has lifecycle stages with sequential steps and optional looping.
- `Step::duration_ms()` uses max(explicit duration, max effect duration).
- `StageTrigger`: `AnyKey`, `Timeout`, `None`.
- Scene supports:
  - `rendered-mode`
  - `virtual-size-override`
  - menu options and next-scene references
  - scene/layer/sprite behaviors
  - stage-based audio cues

`engine-core/src/effects/metadata.rs`:
- builtin effect metadata is intended as source of truth for editor/docs/schema tooling.

## 5) Mod and asset structure contract

Each mod root should contain:
- `mod.yaml`
- `scenes/`
- `assets/`
- `schemas/` (generated fragments)

Scenes can be:
- single YAML files
- packaged scene dirs (`scene.yml` + partials)

Asset loading supports both unpacked directories and zip-packaged mods.

## 6) Editor architecture (short map)

`editor/src`:
- `app.rs`: terminal lifecycle and main editor loop
- `cli.rs`: CLI options (`--mod-source`)
- `domain/`: scene/effect/asset indexes, diagnostics
- `io/`: file scanning, yaml, recents
- `input/`: command + key mapping
- `state/`: app state
- `ui/`: draw/layout/focus/filters/theme

Editor shares model/metadata from `engine-core` and `engine-authoring`.

## 7) Tooling and schema workflow

Schema generation:
- `cargo run -p schema-gen -- --all-mods`

Schema verification in CI/local checks:
- `cargo run -p schema-gen -- --all-mods --check`

Helper script:
- `./refresh-schemas.sh` (single run or loop mode)

## 8) Operational commands

- Run game:
  - `cargo run -p app`
- Run editor:
  - `cargo run -p editor`
- Run with playground mod:
  - `SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app`
- Core runtime tests:
  - `cargo test -p engine`
  - `cargo test -p engine-core`

## 9) Critical invariants (must preserve)

- Keep system order stable unless explicit architecture change is requested.
- Preserve resolver correctness against sorted layer/sprite runtime order.
- Apply scene `virtual-size-override` on transitions.
- Keep virtual buffer synced with terminal resize in max-available mode.
- Do not reintroduce animator freeze for empty/0ms stages.
- Reset per-frame behavior runtime state before behavior application.
- Maintain compatibility with existing YAML mod structure.

## 10) Change playbook for AI agents

When changing:
- scene model/fields:
  - update `engine-core` model + runtime consumption + schema/authoring surfaces
- effect params:
  - update effect metadata + schema generation path + editor consumption
- render pipeline:
  - verify compositor + renderer + virtual buffer interactions
- transitions/lifecycle:
  - verify scoped reset behavior and scene loader ref resolution

Bias:
- prefer minimal, local, type-safe changes
- avoid hidden fallback behavior
- test changed surfaces with existing crate tests
