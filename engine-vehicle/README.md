# engine-vehicle

Owns the neutral vehicle domain used across the engine.

## What it owns

- hierarchy / taxonomy:
  - `VehicleKind`
  - `VehicleCapabilities`
  - `VehicleDescriptor`
  - `VehicleModel` / `VehicleModelRef`
  - `ShipModel`
- runtime-neutral state:
  - `VehicleProfile`
  - `VehicleFacing`
  - `MotionFrame`
  - `VehicleTelemetry`
  - `BrakePhase`
- assembly:
  - `VehicleAssembly`
  - `VehicleAssemblyPlan`
  - `VehicleAssemblyDescriptor`
  - `VehicleAssemblySink`
- input / frame / telemetry seams:
  - `VehicleButtonInput`
  - `VehicleInputIntent`
  - `VehicleMotionIntent`
  - `VehicleControlState`
  - `VehicleReferenceFrame`
  - `VehicleEnvironmentBinding`
  - `VehicleTelemetrySnapshot`
- ship-local runtime model helpers:
  - `ShipRuntimeState`
  - `ShipRuntimeInput`
  - `ShipRuntimeOutput`
- handoff:
  - `VehiclePacketEnvelope`
  - `VehicleLaunchPacket`
  - `VehicleReturnPacket`

## What it does not own

- low-level gameplay primitives like `Transform2D`, `PhysicsBody2D`,
  `ArcadeController`, `AngularBody`, `LinearBrake`, `ThrusterRamp`
  stay in `engine-game`
- Rhai registration stays in `engine-api` / `engine-behavior`
- mod-specific camera / HUD / scene logic stays in mods

## Design direction

The crate is intentionally typed-dispatch-first. It does not start with one
large `dyn Vehicle` object tree. Instead it exposes a small hierarchy of thin
model seams:

- `VehicleAssemblyModel`
- `VehicleControllerModel`
- `VehicleReferenceFrameModel`
- `VehicleTelemetryModel`

`ShipModel` is the first concrete implementation. Future vehicles should be
added through `VehicleKind + VehicleDescriptor + model` instead of copying
behavior glue across mods.

## Module map

- `assembly`: runtime-neutral stack descriptors, Rhai map parsing, and sink
  application order for attaching vehicle gameplay primitives.
- `input`: normalized control state, reference-frame helpers, environment
  bindings, and telemetry snapshots shared by UI/runtime handoff flows.
- `handoff`: canonical launch/return packet DTOs, with legacy
  `vehicle_handoff` launch packets still accepted on read and normalized to
  explicit `vehicle_launch` / `vehicle_return` kinds.
- `models` + `runtime`: typed dispatch seams, concrete vehicle models such as
  `ShipModel`, and pure Rust ship-runtime helpers for adapters that want a
  typed state step without putting that logic into gameplay storage or script
  registration layers.

## Ship Tuning And Runtime DTOs

- built-in ship tuning is centralized in `input` through
  `VehicleShipProfile`, `BUILTIN_SHIP_VEHICLE_PROFILE_IDS`,
  `next_builtin_ship_profile_id`, and `VehicleControlState` helpers like
  `cycle_ship_profile`, `toggle_altitude_hold`, and `toggle_heading_hold`
- button-to-control normalization also stays in `engine-vehicle` through
  `VehicleButtonInput`, so gameplay/script layers can consume one shared
  control vocabulary instead of re-deriving ship axes locally
- Rust-side ship runtime transitions stay typed in `runtime` through
  `ShipReferenceFrameState`, `ShipRuntimeState`, `ShipRuntimeInput`, and
  `ShipRuntimeOutput`; packet/telemetry handoff remains bridged through
  `VehicleTelemetrySnapshot` and `VehiclePacketTelemetry`
- profile-specific ship-runtime ownership now also includes grounded vs
  surface-locked vs detached transitions, local-horizon anchor retention, and
  co-rotation/runtime-output flag semantics inside `ShipModel` rather than in
  gameplay or scripting adapters
- packet/telemetry roundtrips are expected to preserve whether a snapshot is
  grounded or detached while carrying any authored local-horizon anchor data
  (`spawn_angle_deg`, `radius_wu`, `altitude_km`) across launch/return packets

## Boundary Notes

- `engine-game` owns primitive runtime components and only caches/projects
  neutral vehicle DTOs; it does not own vehicle-domain control semantics.
- `engine-api` should stay a thin `vehicle.*` facade over selection plus typed
  value / handoff helpers; it is not the owner of concrete ship-runtime logic.
- `engine-behavior` adapts `VehicleAssemblyPlan` and typed DTO helpers onto
  gameplay primitives; it should not redefine vehicle-domain parsing or
  semantics locally.

## Validation

```bash
cargo test -p engine-vehicle --lib
```
