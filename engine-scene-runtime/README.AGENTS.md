# engine-scene-runtime — mutable scene instance

## Purpose

`engine-scene-runtime` owns the mutable runtime state of a single active scene.

It is responsible for:

- building the runtime object graph,
- maintaining stable object aliases and lookup,
- caching object and effective state snapshots,
- attaching and updating behaviors,
- applying behavior commands,
- UI focus state,
- terminal shell state,
- OBJ viewer camera state,
- runtime-owned lifecycle control helpers.

## Internal module split

- `construction` — build `SceneRuntime` from a compiled `Scene`
- `object_graph` — resolver, object lookup, effective state snapshots
- `materialization` — property snapshots and sprite mutation helpers
- `behavior_runner` — update behaviors and apply commands
- `ui_focus` — focus order, theme state, text layout
- `terminal_shell` — shell transcript, editing, key routing
- `camera_3d` — OBJ viewer camera/orbit helpers
- `lifecycle_controls` — terminal shell, object viewer, and terminal-size control routing used by engine lifecycle orchestration

## Working with this crate

- keep authored-scene schema changes out of this crate,
- keep reusable behavior logic in `engine-behavior`,
- add scene-instance mutation and control-state logic here,
- preserve resolver stability and cache invalidation rules,
- prefer narrow helper methods that engine orchestration can call.

## Important boundary

Engine lifecycle orchestration still lives in `engine`, but runtime-owned control
logic now belongs here. If a feature is specific to mutable scene-instance input
state, start here before adding more logic to `engine/src/systems/scene_lifecycle.rs`.
