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
- resolved scene-level view/lighting/environment state and runtime-only
  lighting/environment overrides,
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
- `render3d_state` — resolved scene-view profile storage and runtime overlay state
- `dirty_tracking` — narrow dirty-mask mapping for typed 3D mutations
- `lifecycle_controls` — runtime-owned control routing consumed by engine lifecycle orchestration; includes `update_gui` (input → `GuiSystem`) and `sync_widget_visuals` (trait-based `visual_sync()` → sprite offsets)

## Key types

- `SceneRuntime` — main mutable runtime container
- `TargetResolver` — stable lookup for scene/layer/sprite/object aliases
- `ObjectRuntimeState` — visibility and offset state per object
- `ObjCameraState` — free-camera state for OBJ viewer scenes
- `ResolvedViewProfile` — effective scene-wide lighting/environment contract
- `RawKeyEvent` / `SidecarIoFrameState` — per-frame input and sidecar snapshots
- `gui_widgets: Vec<Box<dyn GuiControl>>` — trait-object widget collection, built from `SceneGuiWidgetDef` at construction

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

## GUI widget integration

Scenes with a `gui:` block have their widget definitions converted to trait
objects (`Box<dyn GuiControl>`) during construction (`scene_gui_widget_to_control`).
Each frame:

1. `update_gui()` feeds `InputEvent`s into `GuiSystem::update` (trait dispatch).
2. After behaviors run, `sync_widget_visuals()` iterates all widgets, calls
   `visual_sync()`, resolves sprite aliases via `TargetResolver`, and applies
   `offset_x` to `ObjectRuntimeState`.

This keeps slider handle positioning (and future widget visual sync) at the
engine level — Rhai scripts only need to read values, not manually position
handle sprites.

## Scene view profiles

Scene runtime now materializes one effective scene-wide look contract from:

- authored `scene.view`,
- optional asset-backed `view-profile`,
- optional asset-backed `lighting-profile`,
- optional asset-backed `space-environment-profile`,
- optional typed runtime overrides.

Preferred mutations in this area are typed:

- `SetProfile { slot, profile }`
- `SetProfileParam { slot, name/value }`

This should stay the preferred path. Do not add new render-facing string-path
semantics when typed scene mutations can express the same intent.

## Render3D grouped runtime mutations

`materialization.rs` handles typed `Render3DMutation` variants for authored 3D
sprites, but runtime mutation semantics are now grouped by render-domain
responsibility instead of growing new string-path branches.

Canonical grouped mutations:

| Group | Runtime enum backing | Typical responsibility |
|-------|----------------------|------------------------|
| `material` | `Render3DGroupedParam::Material(ObjMaterialParam)` | motion, material, light vectors, object placement |
| `atmosphere` | `Render3DGroupedParam::Atmosphere(AtmosphereParam)` | atmosphere shell / halo / haze controls |
| `surface` | `Render3DGroupedParam::Surface(TerrainParam)` | terrain/displacement-facing surface params |
| `generator` | `Render3DGroupedParam::Generator(WorldgenParam)` | world generation inputs and rebuild-affecting params |
| `body` | `Render3DGroupedParam::Body(PlanetParam)` | planet/body animation and observer inputs |
| `view` | `Render3DGroupedParam::View(ViewParam)` | view-local runtime controls such as distance/yaw/pitch/roll |

Compatibility `scene.set(id, "obj.*", value)` / `scene.set(id, "planet.*",
value)` / `scene.set(id, "terrain.*", value)` calls are still supported, but
they are normalized into grouped typed mutations before runtime application.
The runtime should converge on grouped/profile mutations, not add more
render-facing string-path branches.

## Integration points

- `engine` registers `SceneRuntime` as a scoped resource on scene activation
- `engine-behavior` provides behavior implementations and command types
- `engine-compositor` consumes snapshots, object regions, and camera state
- `engine-animation` provides stage information used while updating behaviors
