# engine-behavior

Behavior runtime, built-in behaviors, and Rhai script execution.

## Purpose

`engine-behavior` owns the per-frame behavior model used by scenes. It defines:

- the `Behavior` trait,
- the `BehaviorContext` shared with each behavior tick,
- the `BehaviorCommand` queue used to report side effects back to the engine,
- built-in behavior implementations,
- Rhai-backed scripted behaviors,
- the built-in behavior factory and metadata lookup.

This crate does not own `World`; it operates on typed context snapshots and
emits commands for higher-level systems to apply.

## Key Types

- `Behavior` — per-tick behavior interface
- `BehaviorContext` — frame-local snapshot of stage, timing, object state, UI state, key state, game state, gameplay world, and collision hits
- `BehaviorCommand` — side-effect envelope such as `SetVisibility`, `SetOffset`, `SetText`, terminal output commands, and script errors
- `RhaiScriptBehavior` — mod or scene-defined scripted behavior
- `SceneAudioBehavior` — built-in scene audio cue emitter
- `BuiltInBehaviorFactory` — authoritative dispatcher for engine-defined behavior names

## Built-in behavior model

Built-in behaviors are constructed from `BehaviorSpec` and updated once per
frame. They do not mutate runtime state directly. Instead they push
`BehaviorCommand` values which the scene runtime or engine systems apply later.

That split is important:

- behavior code stays pure-ish and testable,
- runtime mutation stays centralized,
- script failures can be surfaced cleanly through `ScriptError`.

## Working with this crate

When adding or changing a built-in behavior:

1. add or update the behavior type here,
2. register it in `factory::BuiltInBehaviorFactory`,
3. update metadata exposed through `engine-core` authoring catalogs,
4. update authoring docs if the public script or YAML contract changes,
5. add or update behavior tests.

When changing Rhai scope variables or command shapes, keep the contract aligned
with the runtime behavior system and authored YAML expectations.

Current script-facing API surface includes:

- gameplay world helpers (`world.spawn_object`, `world.entity`, query/count APIs),
- typed gameplay component helpers (`world.set_transform`, `world.set_physics`,
  `world.set_collider_circle`, `world.set_lifetime`, `world.set_visual`),
- per-frame collision reads (`world.collisions()`),
- audio controls (`audio.cue`, `audio.event`, `audio.play_song`, `audio.stop_song`).

## Integration points

- `engine-scene-runtime` owns attached behavior instances and applies emitted commands
- `engine-behavior-registry` supplies mod-defined behavior sources
- `engine-core` provides scene model types, object metadata, and shared runtime snapshot types
