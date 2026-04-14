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
  sidecar IO snapshot, raw key metadata, and mouse position (`mouse_x: f32`,
  `mouse_y: f32` in output-space pixels — exposed to Rhai as `gui.mouse_x/y`).
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
  `cfg` into the prefab's `ArcadeController` config.
- Ship thrust smoke emitters are keyed by `ship_id` plus optional `thrust_ms`;
  the built-in emitter computes spawn position and velocity from the ship.

## World generation module (`scripting/world.rs`)

Registers `planet_last_stats()` — a Rhai function returning biome coverage
from the most recently generated `world://` mesh. Returns a map with keys:
`ocean`, `shallow`, `desert`, `grassland`, `forest`, `cold`, `mountain`
(all `f64` fractions 0.0–1.0). Depends on `engine-terrain`.
