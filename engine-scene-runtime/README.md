# engine-scene-runtime

Materialized scene runtime, object graph, UI state, and runtime-side control handling.

## Purpose

`engine-scene-runtime` turns a compiled `Scene` into mutable runtime state that
other systems can consume frame by frame. It owns:

- stable runtime object IDs and alias resolution,
- object runtime state snapshots,
- behavior attachment and behavior command application,
- UI focus and theme state,
- OBJ viewer camera state,
- runtime-side lifecycle helpers for object viewer and UI focus controls.

This crate is the mutable scene instance, not the scene compiler and not the
global world container.

## Internal split

The crate is intentionally split by responsibility:

- `construction` — build the runtime object graph from the authored scene
- `object_graph` — object lookup, resolver, and effective-state snapshots
- `materialization` — text/object property snapshots and sprite mutation helpers
- `behavior_runner` — behavior updates and command application
- `ui_focus` — focus order, theme state, and text layout helpers
- `camera_3d` — OBJ viewer camera and orbit helpers
- `lifecycle_controls` — runtime-owned control routing consumed by engine lifecycle orchestration

## Key types

- `SceneRuntime` — main mutable runtime container
- `TargetResolver` — stable lookup for scene/layer/sprite/object aliases
- `ObjectRuntimeState` — visibility and offset state per object
- `ObjCameraState` — free-camera state for OBJ viewer scenes
- `RawKeyEvent` / `SidecarIoFrameState` — per-frame input and sidecar snapshots

## Runtime Contracts That Matter

- Runtime object state is immediate-mode. `reset_frame_state()` clears transient
  offsets, visibility, and related derived state before behaviors run each
  frame, so render-driving scripts must re-assert those values every tick.
- `TargetResolver` treats explicit aliases as the authoritative targeting
  surface. Runtime clone target names are reserved for the cloned layer/object;
  child sprite display names are only fallback lookup keys.
- Runtime-cloned layers may contain child sprites with the same authored name as
  the clone target. Resolver stability and child-alias cleanup must preserve the
  parent layer as the target for gameplay visual sync and soft-despawn paths.

## Working with this crate

When changing runtime scene behavior:

- keep authored model changes in `engine-core` and `engine-authoring`,
- keep `SceneRuntime` focused on per-scene mutable state,
- attach behaviors here, but keep behavior implementations in `engine-behavior`,
- prefer adding runtime-local control logic here rather than back in `engine`,
- preserve resolver stability because behaviors, compositor targeting, and UI focus all depend on it,
- preserve alias precedence: explicit aliases first, generated object names only
  as fallback lookup keys.

If you add a new runtime-owned control surface, model it here and let the engine
call a narrow helper instead of duplicating scene-specific logic.

## Integration points

- `engine` registers `SceneRuntime` as a scoped resource on scene activation
- `engine-behavior` provides behavior implementations and command types
- `engine-compositor` consumes snapshots, object regions, and camera state
- `engine-animation` provides stage information used while updating behaviors
