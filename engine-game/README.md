# engine-game

Shared game-state, runtime object types, and component-backed gameplay world data.

## Purpose

`engine-game` hosts gameplay-facing stateful types shared across runtime systems:
- persistent `GameState` used by behaviors,
- `GameObject` model used by the scene/runtime object graph,
- `GameplayWorld` entity store plus typed components (`Transform2D`,
  `PhysicsBody2D`, `Collider2D`, `Lifetime`, `VisualBinding`),
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
- `GameplayStrategies`, `CollisionStrategies`

## Working with this crate

- keep state helpers generic and runtime-safe,
- preserve stable path-based access patterns used by scripts and behaviors,
- if object graph behavior changes, coordinate with `engine-scene-runtime`,
- keep component APIs stable when exposed to Rhai (`world.set_*` / `world.*`),
- keep strategy contracts swappable (physics/collision implementations),
- if script-visible state APIs change, verify Rhai integration and authoring docs.
