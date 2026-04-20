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
- `vehicle` — neutral vehicle-domain facade over active/controlled entity selection
- `rhai` — conversion utilities

## Vehicle helper surface

`vehicle` is a thin domain adapter over the runtime controlled-entity slot.
Today it exposes a minimal neutral contract:

- `vehicle.set_active(id)`
- `vehicle.active()`
- `vehicle.clear_active()`

Alongside that runtime-selection facade, `engine-api` now re-exports the
neutral vehicle crate types used by higher layers:

- assembly plans and config DTOs (`VehicleAssembly`, `ArcadeConfig`,
  `AngularBodyConfig`, `LinearBrakeConfig`, `ThrusterRampConfig`)
- descriptor/capability types (`VehicleDescriptor`, `VehicleModel`,
  `VehicleCapabilities`, `VehicleKind`, `ShipModel`)
- input/assist state (`VehicleInputIntent`, `VehicleControlState`,
  `VehicleAssistState`, `VehicleMotionIntent`,
  `VehicleTranslationIntent`, `VehicleRotationIntent`)
- frame/telemetry seams (`VehicleReferenceFrame`,
  `VehicleEnvironmentBinding`, `VehicleTelemetrySnapshot`)
- handoff packet DTOs (`VehicleLaunchPacket`, `VehicleReturnPacket`, ...)
- telemetry/profile types (`VehicleProfile`, `VehicleTelemetry`, ...)

The Rhai-facing `vehicle` scope also provides thin typed wrappers around the
neutral DTOs from `engine-vehicle`, for example:

- `vehicle.button_input_from(#{ ... })`
- `vehicle.assist_state_from(#{ ... })`
- `vehicle.next_ship_profile_id(profile_id)`
- `vehicle.ship_profile_tuning(profile_id)`
- `vehicle.ship_runtime_state_from(#{ ... })`
- `vehicle.ship_runtime_input_from(#{ ... })`
- `vehicle.ship_runtime_step(profile_id, state, input)`
- `vehicle.control_from_intent(intent, assists)`
- `vehicle.packet_envelope_from(#{ ... })`
- `vehicle.packet_telemetry_from(#{ ... })`

Those helpers intentionally stay in the "thin facade" category: they normalize
or forward typed values into `engine-vehicle`; even the required
`ship_runtime_*` passthrough used by current mods delegates straight to
`engine-vehicle::ShipModel` instead of re-owning runtime semantics in
`engine-api`.

It intentionally does not mirror the generic `world.controlled_entity()` names.
Those stay on the gameplay API; the vehicle module is a domain-scoped
facade that runtimes can inject as a dedicated `vehicle` scope.

## Gameplay helper surface

The gameplay API remains entity- and scene-agnostic. Recent additions include a
generic controlled-entity contract:

- `world.set_controlled_entity(id)`
- `world.controlled_entity()`
- `world.clear_controlled_entity()`

This is the preferred engine-level seam for "currently piloted / possessed /
actively driven" gameplay objects instead of relying on mod-specific tags or
roles as runtime authority.

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

## Scene text/layout helper surface

`ScriptSceneApi` also exposes lightweight 2D inspection and text-update helpers
above raw `scene.set(...)` string paths:

- `inspect(target)` — snapshot map with `id`, `kind`, `state`, `region`, `text`,
  `props`, and `capabilities`
- `region(target)` — runtime layout box map for the resolved object id
- `set_text(target, text)` — typed text-body mutation
- `set_text_style(target, #{ fg, bg, font })` — ergonomic style wrapper over the
  supported text/style paths

These helpers stay on the same typed mutation boundary as `scene.set(...)`;
they do not add a separate string-path execution branch in the runtime.
