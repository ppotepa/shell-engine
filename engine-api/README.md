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

Scene mutation flow is fully typed. `runtime.scene.objects.find(...).set(...)`,
`scene.object(...).set(...)`, and `scene.mutate(...)` are translated into typed
mutation requests before runtime application. Unsupported object-handle writes
are rejected explicitly: the Rust-side set-path builder returns
`SceneMutationRequestError`, and the Rhai-facing object `set(...)` helpers
return `false` instead of silently dropping the request. Live-handle overlays
are only updated after a valid runtime request is formed and queued, so a
failed `set(...)` does not leave a ghost local mutation behind.

New script examples and docs should lead with `runtime.scene.objects` or the
concise `scene.object(...)` shorthand.

For render3d this now means two typed layers:

- scene-level profile selection/override via `SetProfile` / `SetProfileParam`
- grouped object/domain mutations via additive requests like
  `SetMaterialParams`, `SetAtmosphereParams`, `SetSurfaceParams`,
  `SetGeneratorParams`, `SetBodyParams`, and `SetViewParams`

Object-handle writes into render3d paths normalize into these typed grouped
requests before they leave `engine-api`.

## Main modules

- `runtime` — root runtime handle/service/store registry contracts
- `scene` — scene object access and typed mutation requests
- `commands` — command types and path-to-mutation mapping
- `audio`, `effects`, `collision`, `input` — script-facing domain APIs
- `gameplay` — gameplay context/helpers used by behavior code
- `vehicle` — neutral vehicle-domain facade over active/controlled entity selection
- `rhai` — conversion utilities

## Runtime Handle Model

`engine-api` now owns the typed root runtime contract rather than treating
every script-facing object lookup as a flat helper bag. The model is:

- `runtime.scene` — scene-runtime handles and registries
- `runtime.world` — gameplay-world runtime handle slot
- `runtime.services` — runtime-owned service handle slot
- `runtime.stores` — runtime-owned store/snapshot handle slot

The handle/snapshot split matters:

- live scene handles are discovered through
  `runtime.scene.objects.find(target)` and the stable collection helpers
  `all()`, `by_tag(...)`, and `by_name(...)`; `scene.object(target)` is the
  concise root-scene shorthand; `scene.objects` exposes the same live registry
  surface on the root scene scope; resolved handles support `get(...)` / `set(...)`
- snapshot reads stay on `scene.inspect(target)` and `scene.region(target)`
- live gameplay lookup handles come from `world.objects.find(...)`,
  `world.objects.all()`, `world.objects.by_tag(...)`, and
  `world.objects.by_name(...)`
- full gameplay component/entity access stays on `world.entity(id)`

`engine-api` owns the traits and wrapper types for this root model; adapter
crates decide which handles are currently populated in a given runtime.

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

- `vehicle.button_input()` / `vehicle.button_input_pressed(...)`
- `vehicle.assist_state()` / `vehicle.assist_state_with(...)`
- `vehicle.control_state()` / `vehicle.control_state_with(...)`
- `vehicle.next_ship_profile_id(profile_id)`
- `vehicle.ship_profile_tuning(profile_id)`
- `vehicle.ship_runtime_state_from(#{ ... })`
- `vehicle.ship_runtime_input_from(#{ ... })`
- `vehicle.ship_runtime_step(profile_id, state, input)`
- `vehicle.control_from_intent(intent, assists)`
- `vehicle.packet_envelope_from(#{ ... })`
- `vehicle.packet_telemetry()` / `vehicle.packet_telemetry_from_snapshot(...)`

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

It also now has two distinct gameplay-object seams:

- `world.objects.find(...)` / `all()` / `by_tag(...)` / `by_name(...)` for
  lookup-oriented live object handles resolved by entity id, bound visual
  target, or authored/runtime name
- `world.entity(id)` for the richer per-entity API with transform, physics,
  controller, cooldown, status, and other component helpers

The collection queries return iterable arrays of live handles, so Rhai can use
them directly in loops:

```rhai
for object in world.objects.by_tag("player") {
    let id = object.id();
    let kind = object.kind();
}
```

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

`ScriptSceneApi` also exposes lightweight 2D inspection plus concise
root-scene shorthands around the runtime-root scene handles:

- `runtime.scene.objects.find(target)` — primary live scene-object handle lookup
- `scene.object(target)` — concise live scene-object handle shorthand on the
  root `scene` scope
- `scene.objects.find(target)` / `all()` / `by_tag(...)` / `by_name(...)` —
  same live registry surface when a scene-root shorthand reads better
- `runtime.scene.objects.all()` / `by_tag(...)` / `by_name(...)` — stable
  runtime-root collection helpers for live scene handles
- `scene.inspect(target)` — snapshot map with `id`, `kind`, `state`, `region`,
  `text`, `props`, and `capabilities`
- `scene.region(target)` — runtime layout box map for the resolved object id
- `scene.instantiate(template, target)` / `scene.despawn(target)` — canonical
  scene-level lifecycle helpers

Use `runtime.scene.objects` handles when the script wants pending writes to stay
visible during the same frame; use `scene.object(target)` when a concise
single-target shorthand reads better in Rhai; use `scene.objects.by_tag(...)`
or `scene.objects.by_name(...)` when filtering a live scene registry from the
scene root; use `scene.inspect(...)` / `scene.region(...)` when the script
needs a stable snapshot view; mutate object state through the typed live handle
surface (`obj.text`, `obj.style`, `obj.frame`, `obj.render`) instead of
compatibility root helpers.
