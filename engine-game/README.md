# engine-game

Shared game-state, runtime object types, and component-backed gameplay world data.

## Purpose

`engine-game` hosts gameplay-facing stateful types shared across runtime systems:
- persistent `GameState` used by behaviors,
- `GameObject` model used by the scene/runtime object graph,
- `GameplayWorld` entity store plus typed components (`Transform2D`,
  `PhysicsBody2D`, `Collider2D`, `Lifetime`, `VisualBinding`),
- `GameplayWorld` controlled-entity slot for active gameplay ownership
  independent of specific controller implementations,
- lightweight vehicle-domain surfaces (`VehicleProfile`, `VehicleTelemetry`)
  derived from generic motion components for runtime seams,
- pluggable gameplay strategy traits for physics integration and collision.

## Key modules

- `game_state` — nested mutable state exposed to gameplay and scripts
- `game_object` — runtime object node model and object-kind discriminants
- `gameplay` — dynamic gameplay entity store
- `components` — typed gameplay component structs
- `strategy` — strategy traits and default implementations
- `collision` — broadphase/narrowphase + wrap-aware collision utilities

## Main exports

- `GameState`
- `GameObject`
- `GameObjectKind`
- `GameplayWorld`
- `Transform2D`, `PhysicsBody2D`, `Collider2D`, `Lifetime`, `VisualBinding`
- `VehicleProfile`, `VehicleFacing`, `MotionFrame`, `VehicleTelemetry`
- `GameplayStrategies`, `CollisionStrategies`
- `DespawnVisual` — controls visual cleanup on entity despawn

## Working with this crate

- keep state helpers generic and runtime-safe,
- preserve stable path-based access patterns used by scripts and behaviors,
- if object graph behavior changes, coordinate with `engine-scene-runtime`,
- keep component APIs stable when exposed to Rhai (`world.set_*` / `world.*`),
- keep strategy contracts swappable (physics/collision implementations),
- `VisualBinding` supports primary + additional visuals (`additional_visuals: Vec<String>`) — use `all_visual_ids()` for cleanup,
- `GameplayWorld::add_visual(id, visual_id)` registers additional visual bindings,
- `GameplayWorld::ids_with_visual_binding()` queries entities for visual sync system,
- `GameplayWorld::set_controlled_entity(...)`, `controlled_entity()`, and
  `clear_controlled_entity()` provide a generic active-entity contract for
  gameplay scenes without baking mod-specific `player` semantics into the engine,
- `GameplayWorld::snapshot_vehicle_profile(...)` /
  `snapshot_vehicle_telemetry(...)` build neutral vehicle snapshots from
  `ArcadeController`, `AngularBody`, `LinearBrake`, `ThrusterRamp`,
- `VehicleRuntimePrimitives` is the lower-level projection seam from generic
  gameplay components into `engine-vehicle` profile/telemetry inputs,
- `VehicleStateCache` keeps controlled-entity selection plus optional cached
  vehicle profile/telemetry snapshots without making `engine-game` the owner of
  the vehicle domain vocabulary,
- ship-friendly control semantics, profile helpers, and handoff rules still
  live in `engine-vehicle` / `engine-api` rather than in gameplay storage,
- `GameplayWorld::sync_vehicle_profile(...)`,
  `sync_vehicle_telemetry(...)`, and `sync_vehicle_runtime_state(...)`
  optionally cache those snapshots for downstream runtime systems,
- if script-visible state APIs change, verify Rhai integration and authoring docs.
