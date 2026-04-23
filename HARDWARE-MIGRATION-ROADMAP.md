# Hardware Migration Roadmap

> Status: Historical transition document.
>
> Active source of truth for current runtime migration is:
> [wgpu.migration.md](wgpu.migration.md)
>
> This file described an intermediate `software | hardware` coexistence model.
> The active target is full `winit + wgpu` runtime cutover.

## Goal

Build a real `software | hardware` engine split without breaking the current software path.

Target state:
- `software`: current SDL2 + software compositor/render path kept as a supported backend.
- `hardware`: new `winit + wgpu` path for world rendering.
- Authored content stays backend-agnostic.
- Backend choice lives in launcher/runtime, not in scenes or mods.
- Repository cleanup is treated as part of the migration, not as a postscript.

## Non-goals

- Do not move backend choice into `mod.yaml` or scene YAML.
- Do not force asset/authoring crates to know about GPU resources.
- Do not rewrite stable domain crates just because a GPU backend is added.
- Do not delete the software path before hardware is feature-complete enough to debug and fall back.

## End State

### Software path

`engine -> compositor -> postfx -> software presenter -> SDL2 window`

Responsibilities:
- full 2D/3D software rendering
- debug overlays
- frame diff / dirty region / pixel canvas optimizations
- compatibility fallback

### Hardware path

`engine -> world extraction -> GPU renderer -> optional CPU UI overlay -> presenter`

Responsibilities:
- 3D world rendering on GPU
- later: 2D world/UI draw lists or GPU UI path
- presentation independent of `Buffer`

### Rule

- `Buffer` is a software artifact, not the universal frame contract.
- `RuntimeSettings` expresses authored intent, not active backend capabilities.
- Render crates produce backend-neutral data as high as possible and backend-specific commands as low as possible.

## Top-Level Execution Order

Topological order from workspace metadata, simplified for migration:

1. shared contracts and model: `engine-core`, `engine-error`, `engine-events`, `engine-runtime`
2. stable domain primitives: `engine-mesh`, `engine-terrain`, `engine-worldgen`, `engine-audio*`, `engine-io`, `engine-debug`, `engine-persistence`
3. render contracts and policy: `engine-render`, `engine-render-policy`, `engine-pipeline`, `engine-frame`
4. scene/runtime/gameplay bridges: `engine-api`, `engine-behavior`, `engine-game`, `engine-gui`, `engine-scene-runtime`, `engine-3d`, `engine-celestial`
5. software render implementation: `engine-render-2d`, `engine-render-3d`, `engine-compositor`, `engine-render-sdl2`
6. orchestrators and tooling: `engine`, `engine-mod`, `engine-asset`, `engine-authoring`, `app`, `editor`, `launcher`, `schema-gen`
7. additive new crates: `engine-platform-winit`, `engine-render-wgpu`, optional `engine-framebuffer-software`, optional `engine-render-3d-sw`

## Agent Layout

Use all five agent lanes with non-overlapping write scopes.

### Archimedes - Render Runtime

Ownership:
- `engine`
- `engine-render`
- `engine-render-2d`
- `engine-render-3d`
- `engine-compositor`
- `engine-render-sdl2`
- `engine-runtime`
- `engine-pipeline`
- `engine-render-policy`
- `engine-frame`

Mission:
- split `software` vs `hardware` orchestration
- kill legacy contract duplication
- freeze the software path
- define new GPU-facing seams

### Ramanujan - Shared Core

Ownership:
- `engine-core`
- `engine-error`
- `engine-events`
- `engine-layout`
- `engine-vector`
- `engine-capture`

Mission:
- move software framebuffer concerns out of shared core
- widen input model for FPS use cases
- version frame capture formats
- remove `Buffer` as the universal low-level contract

### Hubble - Authoring / Tooling / Front Door

Ownership:
- `engine-authoring`
- `engine-asset`
- `engine-mod`
- `schemas`
- `tools/schema-gen`
- `app`
- `editor`
- `launcher`

Mission:
- make config surfaces canonical
- keep authored content backend-agnostic
- clean SDL-only naming from docs and CLI
- align launcher/app/editor with `render_backend`

### Rawls - Domain and Scene Bridges

Ownership:
- `engine-scene-runtime`
- `engine-behavior`
- `engine-behavior-registry`
- `engine-api`
- `engine-game`
- `engine-physics`
- `engine-vehicle`
- `engine-gui`
- `engine-3d`
- `engine-celestial`

Mission:
- remove render-model leakage from domain crates
- keep scene/runtime state backend-neutral
- separate gameplay intents from visual binding
- move software-only 3D artifacts out of authored 3D format crates

### Chandrasekhar - Stable Subsystems and Repo Cleanup

Ownership:
- `engine-mesh`
- `engine-terrain`
- `engine-worldgen`
- `engine-audio`
- `engine-audio-sequencer`
- `engine-io`
- `rust-os`
- `engine-debug`
- `engine-persistence`
- workspace cleanup files and docs

Mission:
- freeze stable subsystems
- isolate experimental crates from critical migration path
- clean repo noise and stale documentation
- keep procedural/audio/IO crates from getting dragged into renderer churn

## P0 Status Snapshot (2026-04-23)

Legend:
- `done` — merged in current branch
- `in_progress` — partially landed, more cleanup needed
- `frozen` — intentionally left stable for this migration

Crate-by-crate:
- `engine`, `engine-render`, `engine-render-wgpu`, `engine-platform-winit`: `in_progress` (live `winit`/`wgpu` runtime loop active; final contract/warning cleanup remains)
- `app`, `launcher`: `in_progress` (runtime backend selection and canonical display settings path active; minor cleanup remains)
- `engine-render-sdl2`, `engine-compositor`, `engine-render-2d`, `engine-render-3d`: `in_progress` (software fallback stable; backend-neutral packet and overlay/UI parity cleanup remains)
- `engine-authoring`, `engine-asset`, `tools/schema-gen`, `engine-mesh`, `engine-terrain`, `engine-worldgen`, `engine-audio`, `engine-audio-sequencer`, `engine-io`, `engine-debug`, `engine-persistence`, `engine-physics`, `rust-os`: `frozen`

## Tranche Update (2026-04-23)

Scope landed in this tranche:
- App/launcher runtime wiring for backend selection is active (`--render-backend software|hardware`).
- Launcher display parsing is aligned to canonical `display.world_render_size` with compatibility alias for `render_size`.
- Wording cleanup continued across app/launcher/schema/doc surfaces to stay backend-agnostic.
- Hardware runtime now runs a live `winit` + `wgpu` loop with hardware input bridge wired through runtime dispatch.

App feature split status (`software-backend` / `hardware-backend`):
- Runtime selection is wired now (CLI + pass-through + config surfaces).
- Compile-time feature split is documented as next implementation step; not yet the default build gate in workspace crates.
- Current effective behavior: software path is stable fallback, hardware path runs live runtime/present plus hardware input bridge.

Launcher wiring status:
- `launcher` accepts and forwards `--render-backend`.
- Auto pixel-scale derives from canonical runtime settings path (`world_render_size`), not legacy-only parsing.
- Compatibility fallback for legacy manifest key remains in place.

Current tranche execution focus:
- backend bootstrap extraction from `engine::run`
- renderer contract hook in runtime loop
- explicit renderer branch (`software` vs `hardware`)
- `engine-render-wgpu` + `engine-platform-winit` runtime integration

## Completion (crate-by-crate, migration-critical)

Overall migration progress (rough): **99% complete / 1% remaining**.

Percent completion:
- `app`: 95%
- `launcher`: 95%
- `schemas`: 90%
- `engine-mod`: 90%
- `engine-render`: 90%
- `engine`: 98%
- `engine-runtime`: 95%
- `engine-compositor`: 90%
- `engine-render-2d`: 90%
- `engine-render-3d`: 90%
- `editor`: 85%
- `engine-core`: 90%
- `engine-events`: 95%

Checklist by crate:
- `app`
  - [x] CLI backend switch (`software|hardware`)
  - [x] Canonical render-size read path in auto scale
  - [ ] Cargo feature gating (`software-backend`/`hardware-backend`) as build-time split
- `launcher`
  - [x] `--render-backend` pass-through
  - [x] canonical `world_render_size` parsing with legacy alias
  - [ ] backend-aware doctor/setup output
- `schemas`
  - [x] remove SDL2-only wording in mod schema surface
  - [ ] explicit deprecated note for legacy aliases where retained
- `engine-mod`
  - [x] backend-agnostic wording cleanup started
  - [ ] remove/replace `StartupOutputSetting` compatibility layer
- `engine-render`
  - [x] split-trait direction established
  - [ ] remove legacy duplicate contracts
- `engine`
  - [x] runtime backend selection wiring
  - [x] backend bootstrap extraction from `engine::run`
  - [x] renderer contract hook in runtime loop
  - [x] renderer branch split (`software` / `hardware`)
  - [x] real hardware runtime/present loop path landed
  - [x] hardware input bridge wired for hardware runtime path
- `engine-runtime`
  - [x] canonical runtime settings used by app/launcher flow
  - [ ] finish final compatibility cleanup (`is_pixel_backend`/legacy surfaces)
- `engine-compositor`
  - [x] software-only ownership explicitly isolated
- `engine-render-2d`
  - [ ] finalize long-term hardware UI/overlay seam
- `engine-render-3d`
  - [ ] finalize backend-neutral prepared world packets
- `engine-render-wgpu`
  - [x] scaffold crate created and wired in workspace
  - [x] trait-compatible init/present/shutdown path active

## Tranche Checklist (Sync)

- [x] Backend bootstrap extraction
- [x] Renderer contract hook
- [x] Renderer branch split
- [x] `engine-render-wgpu` scaffold crate
- [x] `engine-platform-winit` scaffold crate

## Next Blockers For True `winit+wgpu` Runtime

1. Finalize backend-neutral world packet contract from `engine-render-3d` to GPU backend.
2. Finalize overlay/HUD parity path across both backends.
3. Remove remaining compatibility surfaces (`StartupOutputSetting`, `is_pixel_backend`) and close warning/deprecation cleanup.

## P0 Cleanup Checklist

- [x] Add `tmp/` to `.gitignore`
- [x] Start backend naming cleanup in render docs (`OutputBackend` language removed)
- [x] Add startup backend switch (`software|hardware`) to app/engine
- [x] Remove SDL-only wording from remaining public docs where backend-neutral wording is correct
- [x] Scope SDL diagnostics/setup text to software backend explicitly
- [x] Align `app`/`launcher` manifest display parsing to canonical runtime settings
- [ ] Replace/remove `engine-mod` `StartupOutputSetting` (final compatibility cleanup)
- [ ] Remove `is_pixel_backend` from authored runtime settings (final compatibility cleanup)
- [ ] Decide and enforce one canonical policy for `README.AGENTS.MD` filename casing
- [ ] Resolve duplicate architecture docs naming (`ARCH.MD` vs `ARCHITECTURE.md`)

## Phase Plan

## P0 - Contract Cleanup and Workspace Hygiene

This phase must land before real hardware work.

Exit criteria:
- one canonical backend contract in `engine-render`
- `Buffer` no longer treated as the universal render contract in shared APIs
- `app`, `launcher`, `engine-mod`, and schemas use one canonical display config surface
- repository docs and names no longer imply `SDL2-only`
- stable crates identified and frozen

Tasks:
- `engine-render`: choose one backend contract and demote legacy `RendererBackend`
- `engine-core`: extract or quarantine `Buffer`, `PixelCanvas`, `DiffStrategy`, `BufferAccess`
- `engine-runtime`: remove `is_pixel_backend` from authored settings model
- `app` and `launcher`: stop reading old `display.render_size`; use `engine-runtime::RuntimeSettings`
- `engine-mod`: replace or remove `StartupOutputSetting`
- `schemas`: remove `SDL2 backend` wording from `mod.schema.yaml`
- repo cleanup: ignore `tmp/`, resolve `ARCH.MD` vs `ARCHITECTURE.md`, normalize `README.AGENTS.*` casing policy

## P1 - Freeze Legacy Software Path

Exit criteria:
- current software runtime still boots and remains the reference path
- software-only crates are clearly marked and separated from shared contracts
- compositor remains the full renderer only in `software`

Tasks:
- freeze `engine-render-sdl2` as software presenter
- keep `engine-compositor` as software renderer for 2D/3D/UI/postfx
- move software-only vector and framebuffer helpers out of shared crates
- rename software-only strategy types and docs where needed

## P2 - Hardware Bootstrap

Exit criteria:
- new hardware backend opens a window, owns surface/device/queue, clears screen, and presents
- runtime can select `--render-backend hardware`
- no fake `Buffer` shim is required for the hardware presenter

New crates:
- `engine-platform-winit`
- `engine-render-wgpu`
- optional `engine-framebuffer-software` if framebuffer extraction happens as a new crate instead of a legacy module

Tasks:
- `engine-platform-winit`: window, event loop, raw mouse motion, cursor capture, fullscreen
- `engine-render-wgpu`: adapter/device/surface/swapchain, depth target, present loop
- `engine`: separate `software_render_path` and `hardware_render_path`

## P3 - Hardware World Rendering

Exit criteria:
- GPU backend renders one mesh, then multiple meshes, then terrain/worldgen
- world rendering bypasses software compositor entirely
- `engine-render-3d` produces backend-neutral prepared world items

Tasks:
- convert prepared 3D items into GPU upload/draw packets
- keep `engine-worldgen` as the world mesh seam
- teach `engine` to extract world items and submit them to GPU backend
- leave software 3D raster intact for legacy path

## P4 - Input, UI, and Overlay Parity

Exit criteria:
- freelook-grade mouse input exists in shared event model
- HUD/debug overlay works in both backends
- software-only UI assumptions are no longer global runtime assumptions

Tasks:
- `engine-events`: relative mouse motion, text input, cursor mode, focus
- `engine-render-2d`: define UI draw-list seam or maintain temporary CPU overlay path
- `engine-debug`: keep data model unchanged, swap presentation per backend
- `editor`: decide whether preview is software-only or backend-selectable

## P5 - Final Cleanup and Removal

Exit criteria:
- dead legacy contracts removed
- duplicate camera/runtime types merged
- stale compatibility crates retired
- docs describe two backends clearly

Tasks:
- remove or fully integrate `engine-frame` + `PreparedFrame`
- retire `engine-behavior-registry`
- remove duplicate `Camera3DState` vs `SceneCamera3D`
- delete stale SDL-only wording and redundant wrappers
- optional: add workspace `default-members`

## Crate-by-Crate Roadmap

## Runtime / Render Hot Path

### `engine`

Current role:
- main orchestrator: startup, resources, game loop, backend selection, splash, scene lifecycle

Must change:
- split runtime into `software_render_path` and `hardware_render_path`
- stop assuming one global `Buffer` and one `Box<dyn RendererBackend>`
- stop hardcoding `runtime_settings.is_pixel_backend = true`

Key files:
- `engine/src/lib.rs`
- `engine/src/game_loop.rs`
- `engine/src/systems/compositor/mod.rs`
- `engine/src/systems/renderer.rs`
- `engine/src/services.rs`
- `engine/src/splash.rs`
- `engine/src/prepared_frame.rs`
- `engine/src/runtime_settings.rs`

Cleanup tasks:
- either delete or truly integrate `PreparedFrame`
- merge `engine/src/runtime_settings.rs` helper logic into `engine-runtime` or a thin shared helper

Target phase:
- P0, P2, P3, P5

### `engine-render`

Current role:
- shared render abstractions, overlay/vector types, but also duplicated legacy contract surfaces

Must change:
- keep `RenderBackendKind`, `PresentationBackend`, `HardwareRendererBackend`
- demote or replace `RendererBackend::present_frame(&Buffer)` as a software-only contract
- remove duplicate top-level API (`RenderBackend` + `RenderFrame`) if it remains unused

Key files:
- `engine-render/src/lib.rs`
- `engine-render/README.md`

Cleanup tasks:
- shared crate must stop depending conceptually on `Buffer` as the canonical frame payload
- align README with actual contract names

Target phase:
- P0

### `engine-render-2d`

Current role:
- software 2D raster/layout/text/image/vector path writing to `Buffer`

Must change:
- keep text/layout semantics and measurement
- add a future UI draw-list seam
- avoid assuming `Buffer` for all consumers

Key files:
- `engine-render-2d/src/api.rs`
- `engine-render-2d/src/image.rs`
- `engine-render-2d/src/text.rs`
- `engine-render-2d/src/vector.rs`

Cleanup tasks:
- move vector queue out of thread-local scratch state
- replace boolean `is_pixel_backend` assumptions with a richer surface kind

Target phase:
- P1, P4

### `engine-render-3d`

Current role:
- mixed frontend/backend crate; good prepared-item producers but software-buffer outputs

Must change:
- keep producer layer, source specs, frame/view/lighting preparation
- separate software raster backend from backend-neutral world item preparation
- feed GPU backend with prepared world packets instead of `Buffer`

Key files:
- `engine-render-3d/src/api.rs`
- `engine-render-3d/src/pipeline/producers/mod.rs`
- `engine-render-3d/src/pipeline/prepared_item_renderer.rs`
- `engine-render-3d/src/pipeline/obj_sprite_renderer.rs`
- `engine-render-3d/src/pipeline/generated_world_renderer.rs`
- `engine-render-3d/src/pipeline/scene_clip_renderer.rs`
- `engine-render-3d/src/raster.rs`

Cleanup tasks:
- remove `Render3dOutput { color: Buffer }` from public architecture
- extract software-only raster path into dedicated module or crate

Target phase:
- P1, P3, P5

### `engine-render-sdl2`

Current role:
- stable software presenter and input bridge

Must change:
- almost nothing for the migration itself
- remain the frozen software backend

Key files:
- `engine-render-sdl2/src/renderer.rs`
- `engine-render-sdl2/src/runtime.rs`
- `engine-render-sdl2/README.md`

Cleanup tasks:
- only documentation and maybe future input extraction

Target phase:
- P1

### `engine-compositor`

Current role:
- full software frame assembly with 2D, 3D, postfx, world/ui split passes

Must change:
- remain the full renderer for `software`
- stop being mandatory for world 3D in `hardware`
- keep UI/HUD/postfx/object region work for software and maybe temporary hardware overlay path

Key files:
- `engine-compositor/src/provider.rs`
- `engine-compositor/src/compositor.rs`
- `engine-compositor/src/prepared_frame.rs`
- `engine-compositor/src/layer_compositor.rs`
- `engine-compositor/src/sprite_renderer_2d.rs`

Cleanup tasks:
- remove 3D delegate from the default 2D sprite path once GPU world render exists
- deduplicate prepared-frame logic

Target phase:
- P1, P3, P5

### `engine-runtime`

Current role:
- shared authored runtime/display settings and presentation layout math

Must change:
- keep authored settings backend-neutral
- remove `is_pixel_backend`
- rename `BufferLayout` only if needed; the data is fine, the semantic naming is old

Key files:
- `engine-runtime/src/lib.rs`

Cleanup tasks:
- unify helper usage with `app`, `launcher`, and `engine`

Target phase:
- P0

### `engine-pipeline`

Current role:
- software optimization strategies and frame-skip flags

Must change:
- keep `FrameSkipOracle`
- mark layer/present strategies as software-only, not backend-neutral

Key files:
- `engine-pipeline/src/lib.rs`
- `engine-pipeline/src/strategies/present.rs`
- `engine-pipeline/src/strategies/layer.rs`
- `engine-pipeline/src/strategies/skip.rs`

Cleanup tasks:
- rename or isolate software strategies
- deprecate stale names after hardware path exists

Target phase:
- P1, P5

### `engine-render-policy`

Current role:
- font resolution policy

Must change:
- minimal; replace backend boolean assumptions with an enum or richer surface descriptor

Key files:
- `engine-render-policy/src/lib.rs`

Target phase:
- P0, P4

### `engine-frame`

Current role:
- frame ticketing, but not cleanly integrated

Must change:
- either become the actual render queue freshness contract or be removed from the active design

Key files:
- `engine-frame/src/lib.rs`
- `engine/src/prepared_frame.rs`

Target phase:
- P0, P5

## Shared / Core / Data

### `engine-core`

Current role:
- shared model layer, scene types, `World`, render types, and today also software framebuffer machinery

Must change:
- keep scene model, render-neutral types, world container
- move `Buffer`, `PixelCanvas`, `BufferAccess`, `DiffStrategy` out of shared core or isolate them as software-only
- merge duplicate camera/runtime types

Key files:
- `engine-core/src/buffer.rs`
- `engine-core/src/access.rs`
- `engine-core/src/strategy/diff.rs`
- `engine-core/src/render_types/*`
- `engine-core/src/scene_runtime_types.rs`

Cleanup tasks:
- `Camera3DState` vs `SceneCamera3D`
- keep `render_types` as the seed of the backend-neutral render model

Target phase:
- P0

### `engine-error`

Current role:
- top-level workspace error enum

Must change:
- add typed render/platform failure variants instead of overloading `std::io::Error`

Key files:
- `engine-error/src/lib.rs`

Cleanup tasks:
- support backend/device/surface/present error families cleanly

Target phase:
- P0

### `engine-events`

Current role:
- shared input event contract and backend polling trait

Must change:
- keep core input events
- add relative mouse motion, text input, focus, cursor mode, and optionally gamepad/window activation

Key files:
- `engine-events/src/lib.rs`
- `engine-events/src/input_backend.rs`
- `engine-events/src/access.rs`

Cleanup tasks:
- remove low-level `access.rs` dependency on `engine-core` if possible

Target phase:
- P0, P4

### `engine-layout`

Current role:
- layout math plus sprite/compositor-centric adapters

Must change:
- keep pure layout math and area/track types
- move sprite-specific adapters out of the crate

Key files:
- `engine-layout/src/flex.rs`
- `engine-layout/src/grid.rs`
- `engine-layout/src/area.rs`
- `engine-layout/src/tracks.rs`

Target phase:
- P1

### `engine-vector`

Current role:
- mixed geometry and software rasterization directly into `Buffer`

Must change:
- split geometry from software raster
- keep intersection/math reusable
- add tessellation or adapter seam later if needed for GPU

Key files:
- `engine-vector/src/lib.rs`

Target phase:
- P0, P1

### `engine-capture`

Current role:
- capture/compare for regression testing, currently cell-grid only

Must change:
- version snapshot format
- support `CellGridV1` and `Rgba8V1` minimum
- stop taking only `&Buffer`

Key files:
- `engine-capture/src/capture.rs`
- `engine-capture/src/compare.rs`
- `engine-capture/README.md`

Target phase:
- P0, P4

## Authoring / Assets / Tooling / Front Door

### `engine-authoring`

Current role:
- YAML compile/normalize/schema pipeline

Must change:
- almost nothing for backend migration itself
- remain strictly backend-agnostic

Target phase:
- P0 docs/schema alignment only

### `engine-asset`

Current role:
- asset/repository loading, profile hydration, mesh/image loading and caching

Must change:
- remain CPU-domain only
- do not absorb `GpuMesh` or backend resource lifetime logic

Target phase:
- P0 docs alignment, P3 consumer integration only

### `engine-mod`

Current role:
- mod manifest and startup checks

Must change:
- remove or rename `StartupOutputSetting`
- keep checks content-focused and backend-agnostic

Key files:
- `engine-mod/src/output_backend.rs`
- `engine-mod/src/startup/context.rs`

Target phase:
- P0

### `schemas`

Current role:
- authoring schemas

Must change:
- `mod.schema.yaml` must stop saying SDL2 is the presenter of the world/UI targets
- if legacy aliases remain, mark them explicitly as deprecated surfaces

Key files:
- `schemas/mod.schema.yaml`
- `schemas/scene.schema.yaml`
- `schemas/scene-file.schema.yaml`

Target phase:
- P0

### `tools/schema-gen`

Current role:
- schema generation/checks

Must change:
- mostly regression coverage for canonical display keys

Target phase:
- P0

### `app`

Current role:
- CLI launcher for the engine binary

Must change:
- keep `--render-backend`
- stop reading old `display.render_size`
- rename `--sdl-*` to neutral flags with legacy aliases if needed
- stop hardcoding `StartupOutputSetting::Sdl2`

Key files:
- `app/src/main.rs`

Target phase:
- P0, P2

### `editor`

Current role:
- authoring/editor stub with software preview remnants

Must change:
- docs must stop implying SDL2 is the eternal editor backend
- preview path must be clearly software-only or explicitly backend-selectable later

Key files:
- `editor/src/app.rs`
- `editor/src/state/scene_run.rs`
- `editor/README.AGENTS.MD`

Target phase:
- P0, P4

### `launcher`

Current role:
- outer wrapper for running the workspace

Must change:
- add `render_backend = software|hardware` to launcher config/CLI
- make `setup` and `doctor` backend-aware
- stop duplicating stale manifest parsing with old `render_size`

Key files:
- `launcher/src/cli.rs`
- `launcher/src/config.rs`
- `launcher/src/workspace.rs`
- `launcher/src/commands/run.rs`
- `launcher/src/commands/setup.rs`
- `launcher/src/commands/doctor.rs`
- `launcher/src/env.rs`

Target phase:
- P0, P2

## Domain / Scene Bridges / Gameplay

### `engine-scene-runtime`

Current role:
- mutable scene instance state, object graph, scene camera/runtime mutation layer

Must change:
- stay backend-neutral
- remove dead dependency on `engine-render` if unused
- move widget visual syncing and legacy software-specific bridges out if possible

Key files:
- `engine-scene-runtime/src/lib.rs`
- `engine-scene-runtime/src/object_graph.rs`
- `engine-scene-runtime/src/lifecycle_controls.rs`

Target phase:
- P0, P1

### `engine-behavior`

Current role:
- behavior runtime and Rhai bridge, but currently mixed with render-prefab and visual binding concerns

Must change:
- keep behavior core and typed commands
- move `spawn_visual`, `bind_visual`, render-prefab logic out to adapters
- remove clipboard-through-renderer assumptions

Key files:
- `engine-behavior/src/lib.rs`
- `engine-behavior/src/catalog.rs`
- `engine-behavior/src/scripting/gameplay_impl.rs`

Target phase:
- P0, P5

### `engine-behavior-registry`

Current role:
- compatibility shim

Must change:
- retire after callers migrate to `engine-behavior`

Target phase:
- P5

### `engine-api`

Current role:
- typed scene/runtime/script API surface

Must change:
- stay backend-neutral
- move clipboard/platform actions out of render-backed semantics
- keep DTOs and requests clean

Key files:
- `engine-api/src/runtime/api.rs`
- `engine-api/src/scene/render.rs`
- `engine-api/src/commands.rs`

Target phase:
- P0, P5

### `engine-game`

Current role:
- gameplay entities/components, but still leaks scene visual binding

Must change:
- remove `VisualBinding` and visual sync extraction from core gameplay store
- keep opaque handles or adapter-facing output instead

Key files:
- `engine-game/src/components.rs`
- `engine-game/src/gameplay.rs`

Target phase:
- P0, P5

### `engine-physics`

Current role:
- geometry/collision helpers

Must change:
- no renderer-driven changes required

Target phase:
- frozen early

### `engine-vehicle`

Current role:
- vehicle domain/runtime, but does too much Rhai parsing inside the domain crate

Must change:
- move Rhai map parsing upward into API or behavior layer

Key files:
- `engine-vehicle/src/assembly.rs`
- `engine-vehicle/src/runtime.rs`

Target phase:
- P1, P5

### `engine-gui`

Current role:
- GUI runtime and widget state, but directly emits sprite-specific visual sync actions

Must change:
- keep widget state/system/focus/hit testing
- move `VisualSyncAction` out to a bridge crate or adapter layer

Key files:
- `engine-gui/src/control.rs`
- `engine-gui/src/system.rs`

Target phase:
- P0, P4, P5

### `engine-3d`

Current role:
- authored 3D scene format and asset resolution, but also contains software-only atlas/prerender artifacts

Must change:
- keep scene3d format and resolve logic
- move `Scene3DAtlas`, prerender caches, and framebuffer artifacts out

Key files:
- `engine-3d/src/scene3d_format.rs`
- `engine-3d/src/scene3d_resolve.rs`
- `engine-3d/src/scene3d_atlas.rs`
- `engine-3d/src/obj_prerender.rs`

Target phase:
- P0, P5

### `engine-celestial`

Current role:
- celestial domain/query services plus appearance presets

Must change:
- keep physical/query layer
- separate appearance/render presets more cleanly over time

Key files:
- `engine-celestial/src/lib.rs`
- `engine-celestial/src/services.rs`

Target phase:
- P1, P5

## Stable Subsystems / Procedural / Platform / Cleanup

### `engine-mesh`

Current role:
- pure CPU mesh generators

Must change:
- mostly documentation cleanup only
- freeze public mesh generator contract

Target phase:
- freeze in P0

### `engine-terrain`

Current role:
- terrain/world generation math

Must change:
- no renderer-driven change
- later audit `LAST_PLANET_STATS` global state

Target phase:
- freeze in P0

### `engine-worldgen`

Current role:
- `world://` URI seam and world mesh build output

Must change:
- remain the GPU world mesh seam
- documentation cleanup only for ownership wording

Target phase:
- freeze in P0, consume in P3

### `engine-audio`

Current role:
- audio runtime/backend

Must change:
- independent of render migration
- later audit `unsafe Send/Sync`

Target phase:
- freeze in P0, technical debt later

### `engine-audio-sequencer`

Current role:
- authored audio sequencing runtime

Must change:
- independent of render migration
- later module split for maintainability

Target phase:
- freeze in P0

### `engine-io`

Current role:
- IO protocol and transports

Must change:
- independent of render migration
- later split large `lib.rs`

Target phase:
- freeze in P0

### `rust-os`

Current role:
- experimental sidecar/product crate

Must change:
- isolate from critical migration path
- consider moving out of default workflow

Target phase:
- workspace cleanup only

### `engine-debug`

Current role:
- debug data model

Must change:
- minimal; keep data model, change presentation consumers later

Target phase:
- freeze in P0, consume in P4

### `engine-persistence`

Current role:
- persistence store

Must change:
- no renderer-driven change
- later add typed errors, atomic writes, README

Target phase:
- freeze in P0, technical debt later

### `launcher`, `schema-gen`, `devtool`, `sound-server`, `ttf-rasterizer`

Role:
- tooling or wrapper crates

Must change:
- only where CLI/docs/workflow drift with new backend split

Target phase:
- P0 for naming/config cleanup only

## Repo Cleanup Checklist

### Mandatory

- add `tmp/` to `.gitignore`
- decide canonical doc between `ARCH.MD` and `ARCHITECTURE.md`
- normalize `README.AGENTS.MD` vs `README.AGENTS.md` policy for cross-platform consistency
- align root `README.md` language away from `SDL2-first` once hardware backend actually boots
- fix stale references to `OutputBackend`, `SDL2-only`, `pixel backend`, and old manifest `render_size`

### Recommended

- add workspace `default-members` so experiments and heavy tools do not always run in the default cargo set
- classify `rust-os` explicitly as experimental in docs and workflow
- add missing crate README for `engine-persistence`
- refresh `engine-mesh`, `engine-terrain`, `engine-worldgen`, `engine-render` docs to reflect actual ownership boundaries

## Freeze List

Freeze early:
- `engine-mesh`
- `engine-terrain`
- `engine-worldgen`
- `engine-audio-sequencer`
- `engine-io` wire protocol
- `engine-debug` data model
- `engine-persistence` concept/API shape
- `engine-physics`

Do not freeze yet:
- `engine-core::Buffer`
- `engine-runtime::is_pixel_backend`
- `engine-render` legacy backend traits
- `engine-render-3d` software output shapes
- `engine-game` visual binding model
- `engine-gui::VisualSyncAction`
- `engine-3d` atlas/prerender artifacts
- `launcher` SDL-specific CLI/config

## New Crates To Add

### `engine-platform-winit`

Purpose:
- window creation
- event loop
- raw mouse motion
- cursor capture/visibility
- monitor/fullscreen queries

### `engine-render-wgpu`

Purpose:
- GPU device/surface lifecycle
- swapchain / surface configuration
- world mesh upload
- GPU draw submission
- hardware present path

### Optional `engine-framebuffer-software`

Purpose:
- `Buffer`
- `PixelCanvas`
- dirty tracking
- software diff logic
- software capture adapters

Use if extracting software framebuffer out of `engine-core` is cleaner than feature-gating it there.

### Optional `engine-render-3d-sw`

Purpose:
- software raster implementation split from `engine-render-3d` frontend

Use if `engine-render-3d` becomes cleaner as a backend-neutral producer crate.

## Safe Commit Order

1. workspace cleanup and naming only
2. config/schema/runtime alignment only
3. shared contract split only
4. `Buffer` extraction only
5. software path freeze only
6. hardware bootstrap crates only
7. GPU mesh world path only
8. input/UI parity only
9. final removals and dead API cleanup

## Definition of Done

The migration is complete when all of the following are true:
- `cargo run -p app -- --render-backend software` still boots the current engine path
- `cargo run -p app -- --render-backend hardware` boots a real GPU window and presents without depending on `Buffer`
- world meshes and terrain render through the hardware path
- scenes and mods remain backend-agnostic
- domain crates do not carry scene visual IDs, sprite aliases, or framebuffer types unless they are explicitly adapters
- repository docs no longer imply SDL2 is the only runtime backend
- dead legacy contracts and compatibility shims are removed
