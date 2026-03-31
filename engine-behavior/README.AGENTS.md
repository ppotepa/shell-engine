# engine-behavior — behavior execution contract

## Purpose

`engine-behavior` defines the behavior runtime contract used by the rest of the
engine:

- `Behavior`,
- `BehaviorContext`,
- `BehaviorCommand`,
- built-in behaviors,
- Rhai-script behaviors,
- built-in behavior factory dispatch.

It does not own the scene runtime container. It consumes snapshots and emits
commands.

## How the contract works

1. `engine-scene-runtime` builds a frame-local `BehaviorContext`.
2. Each attached behavior receives the current object, scene, and context.
3. Behaviors emit `BehaviorCommand` values instead of mutating runtime state directly.
4. `engine-scene-runtime` applies those commands after behavior updates finish.

This split keeps behavior code testable and keeps runtime mutation centralized.

## Must-remember surfaces

- `BehaviorContext` carries stage timing, object snapshots, UI state, game state,
  sidecar IO snapshot, and raw key metadata.
- `BehaviorCommand::ScriptError` is not ignored; higher-level systems surface it
  into debug logging.
- Built-in behavior dispatch lives in `factory::BuiltInBehaviorFactory`.

## When changing behavior APIs

- update the behavior type or Rhai binding here,
- update factory registration,
- update authoring metadata in `engine-core`,
- update runtime integration if command shapes changed,
- add or update tests.

## Script-facing contract reminders

- `local[]` is behavior-local, not scene-global. If two Rhai files need to
  share state, use `game.set/get`.
- `world.set_world_bounds` is authored as
  `world.set_world_bounds(min_x, min_y, max_x, max_y)`.
- `spawn_prefab("ship", #{ cfg: ... })` merges runtime controller overrides from
  `cfg` into the prefab's `TopDownShipController` config.
- Ship thrust smoke emitters are keyed by `ship_id` plus optional `thrust_ms`;
  the built-in emitter computes spawn position and velocity from the ship.
