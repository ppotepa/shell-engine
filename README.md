# Shell Engine

Shell Engine is a scene engine in active runtime migration from legacy SDL2
paths toward a `winit + wgpu` runtime, built around YAML authoring, Rhai
runtime scripting, and mod-local content packages.

Current migration checkpoint: hardware-first defaults are active in `app` and
`engine`, backend-neutral `FrameSubmission` is wired in the engine render
system, and hardware submit currently keeps a compatibility fallback to
software presentation on failure.

The repo is currently used as a reusable engine playground. The bundled mods
focus on renderer development, UI experiments, and procedural world/planet
generation rather than one single shipped game campaign.

## What It Is

- YAML-authored scenes, layers, sprites, effects, and GUI widgets
- Rhai-driven runtime behaviors and scene mutation APIs
- split rendering architecture:
  - `engine-render-2d` owns 2D sprite/layout rendering
  - `engine-render-3d` owns 3D scene/raster/prerender logic
  - `engine-compositor` assembles frame output and PostFX
- procedural world generation through `engine-terrain`, `engine-mesh`, and
  `engine-worldgen`
- mod loading from directories or zip archives

Optional sidecar and IPC crates still exist in the workspace, but they are not
the defining center of the current engine architecture.

## Getting Started

1. Install Rust.
2. Clone the repo:

```bash
git clone https://github.com/ppotepa/shell-engine.git
cd shell-engine
```

3. Start the interactive launcher:

```bash
cargo run -p app
```

4. Or launch a specific mod directly:

```bash
cargo run -p app -- --mod playground
cargo run -p app -- --mod-source=mods/planet-generator
```

5. Run the editor:

```bash
cargo run -p editor
```

## Included Mods

- `mods/playground` — general engine sandbox
- `mods/planet-generator` — procedural world/planet tuning mod
- `mods/gui-playground` — GUI/widget behavior playground
- `mods/terrain-playground` — terrain and worldgen experiments
- `mods/asteroids` — gameplay-heavy orbital combat prototype

## Repo Map

- `app/` — CLI entrypoint
- `launcher/` — interactive launcher UI
- `editor/` — authoring/editor tooling
- `engine/` — top-level runtime orchestration
- `engine-core/` — shared scene/runtime/model types
- `engine-authoring/` — YAML compile/normalize/validate pipeline
- `engine-api/` — script-facing engine API surface
- `engine-render-2d/` — 2D sprite/layout rendering
- `engine-render-3d/` — 3D scene, raster, prerender, generated-world rendering
- `engine-compositor/` — frame assembly and PostFX
- `engine-worldgen/` — `world://` parsing, mesh build keys, generated meshes
- `engine-terrain/` — climate/biome/elevation generation
- `engine-mesh/` — procedural geometry primitives
- `mods/` — bundled content mods
- `schemas/` — shared and generated YAML schemas
- `tools/` — schema-gen and support tools

## Documentation

### Root docs

- [README.md](README.md) — overview
- [wgpu.migration.md](wgpu.migration.md) — active runtime migration source of truth (`winit + wgpu`)
- [ARCHITECTURE.md](ARCHITECTURE.md) — crate boundaries, system order, rendering flow
- [AUTHORING.md](AUTHORING.md) — authored scene contract, assets, sprites, Rhai
- [MODS.md](MODS.md) — bundled mods and mod structure
- [UNITS.md](UNITS.md) — unified unit model (`screen_px`, `virtual_px`, `wu`, `m/km`)
- [BENCHMARKING.md](BENCHMARKING.md) — benchmark workflow and capture
- [OPTIMIZATIONS.md](OPTIMIZATIONS.md) — optimization flags and invariants
- [CHANGELOG.md](CHANGELOG.md) — development log

### Subsystem docs

- `app/README.md` and `app/README.AGENTS.MD`
- `engine/README.md` and `engine/README.AGENTS.MD`
- `engine-core/README.md` and `engine-core/README.AGENTS.MD`
- `engine-compositor/README.md` and `engine-compositor/README.AGENTS.md`
- `engine-scene-runtime/README.md` and `engine-scene-runtime/README.AGENTS.md`
- `engine-render/README.md`
- `engine-render-2d/README.md`
- `engine-render-3d/README.md`
- `engine-worldgen/README.md`
- `engine-mesh/README.md`
- `engine-terrain/README.md`
- `engine-behavior/README.md` and `engine-behavior/README.AGENTS.md`
- `mods/README.md`

## Status

The large 2D/3D ownership split is now in place in code:

- 2D and 3D rendering live in separate crates,
- compositor no longer owns the 3D raster domain,
- runtime mutation flow is typed-first,
- remaining raw string-path mutation support is intentionally narrow and
  documented.

Latest highlights (April 2026):

- unified spatial/unit baseline documented in [UNITS.md](UNITS.md),
- generated-world performance pass landed for cloud-heavy planet rendering,
- benchmark workflow now includes a dedicated cloud stress scene:
  `mods/asteroids/scenes/bench-cloud/scene.yml`.

Runtime migration snapshot (2026-04-23):
- runtime defaults: `app` + `engine` now default to `hardware-backend`
- `FrameSubmission` seam is live in runtime path
- SDL2 runtime bootstrap is detached from active/default runtime path
- app CLI is compat-only for legacy software knobs (`--compat-window-ratio`, `--compat-pixel-scale`, `--no-compat-vsync`), and `--render-backend software` is rejected
- no active `--sdl-*` CLI aliases remain in `app`/`launcher` sources (`rg -n "sdl-window-ratio|sdl-pixel-scale|no-sdl-vsync" app/src launcher/src -g "*.rs"` returns no matches)
- no active `is_pixel_backend` usage remains in runtime/policy hot path (`rg -n "is_pixel_backend" engine-runtime/src engine-render-policy/src -g "*.rs"` returns no matches)
- active migration blockers are:
  - migration closure gates outside renderer bridge cleanup (verification artifacts and PERF/DOD completion)
- canonical migration status and acceptance gates: [wgpu.migration.md](wgpu.migration.md) and [STATUS.md](STATUS.md)
