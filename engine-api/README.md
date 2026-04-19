# engine-api

Script-facing engine API facade.

## Purpose

`engine-api` is the public-facing surface used by Rhai scripts and other
engine-side consumers. It groups engine capabilities by domain instead of
exposing runtime internals directly.

## What it owns

- `BehaviorCommand`
- typed scene mutation request types
- scene/audio/effects/input/gameplay API registration helpers
- Rhai conversion helpers shared by script-facing modules
- typed-first render3d mutation requests for scene-level profiles and grouped
  node/domain params

## Important note

Scene mutation flow is fully typed. `scene.mutate(...)` and supported
`scene.set(...)` paths are translated into typed mutation requests before
runtime application. Unsupported `scene.set(...)` paths do not enqueue runtime
commands.

For render3d this now means two typed layers:

- scene-level profile selection/override via `SetProfile` / `SetProfileParam`
- grouped object/domain mutations via additive requests like
  `SetMaterialParams`, `SetAtmosphereParams`, `SetSurfaceParams`,
  `SetGeneratorParams`, `SetBodyParams`, and `SetViewParams`

Legacy `scene.set(target, "...", value)` render paths are compatibility shims
that normalize into these typed requests before they leave `engine-api`.

## Main modules

- `scene` — scene object access and typed mutation requests
- `commands` — command types and path-to-mutation mapping
- `audio`, `effects`, `collision`, `input` — script-facing domain APIs
- `gameplay` — gameplay context/helpers used by behavior code
- `rhai` — conversion utilities

## Render3D helper surface

`ScriptSceneApi` exposes grouped helpers for high-level render3d control:

- `set_render3d_profile(...)`
- `set_render3d_profile_param(...)`
- `set_material_params(...)`
- `set_atmosphere_params(...)`
- `set_surface_params(...)`
- `set_generator_params(...)`
- `set_body_params(...)`
- `set_view_params(...)`
