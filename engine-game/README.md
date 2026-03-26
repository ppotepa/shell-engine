# engine-game

Shared game-state and runtime object types.

## Purpose

`engine-game` hosts lightweight stateful types shared across runtime systems:
the persistent `GameState` used by gameplay logic and the `GameObject` model
used by the scene/runtime object graph.

## Key modules

- `game_state` — nested mutable state exposed to gameplay and scripts
- `game_object` — runtime object node model and object-kind discriminants

## Main exports

- `GameState`
- `GameObject`
- `GameObjectKind`

## Working with this crate

- keep state helpers generic and runtime-safe,
- preserve stable path-based access patterns used by scripts and behaviors,
- if object graph behavior changes, coordinate with `engine-scene-runtime`,
- if script-visible state APIs change, verify Rhai integration and authoring docs.
