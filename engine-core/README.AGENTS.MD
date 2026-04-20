# Engine Core — Shared Model Layer

## Purpose

`engine-core` is the lowest-level shared crate in the workspace. It contains the
types that need to be reused across authoring, runtime, compositor, behavior,
editor, and tooling code without pulling in engine orchestration.

## What lives here

- scene model types (`Scene`, `Layer`, `Sprite`, stage/step data, UI config)
- buffer and cell primitives
- effect types and authoring metadata
- asset-source abstractions
- game object and scene-runtime snapshot data
- `World` and core world-access helpers
- shared logging and utility modules that must not depend on higher crates

## Scene model notes

- scenes are normalized runtime data, not raw authored YAML
- scene timing is stage-driven (`on_enter`, `on_idle`, `on_leave`)
- sprite timing fields are absolute scene-relative values where applicable
- scene packages and authored partials are assembled before values reach this crate

If the authored contract changes, update `engine-authoring` and schemas in lockstep.

## Buffer model

- double-buffered cell storage supports back/front comparison
- dirty tracking is part of the buffer contract
- compositor, PostFX, and renderer code rely on dirty-region correctness

Do not treat dirty tracking as an optional optimization detail; it is part of
render correctness.

## Runtime snapshot types

`scene_runtime_types.rs` contains pure data shared across crates, including:

- `TargetResolver`
- `ObjectRuntimeState`
- `ObjCameraState`
- `RawKeyEvent`
- `SidecarIoFrameState`

These types exist so crates like `engine-behavior`, `engine-scene-runtime`, and
`engine-compositor` can share state shapes without depending on each other’s
full implementations.

## World ownership

`World` now lives here as the shared type-indexed resource container. Higher
crates provide domain-specific access traits or provider impls on top of it.

Keep `World` generic and low-level. Engine-specific lifecycle or orchestration
logic should not move into this crate.

## World generation fields on `Sprite::Obj`

`Sprite::Obj` carries 17 `world_gen_*` fields (seed, has_ocean, ocean_fraction,
continent_scale/warp/octaves, mountain_scale/strength/ridge_octaves,
moisture_scale, ice_cap_strength, lapse_rate, rain_shadow,
displacement_scale, subdivisions, coloring, shape). These are set at
compile time from YAML and at runtime via `world.*` Rhai property paths
in `engine-scene-runtime`. The compositor builds a `world://` URI from
these fields; each unique URI triggers a fresh mesh generation via
`engine-terrain`.

## Scene GUI widget definitions

`SceneGuiWidgetDef` in `scene/model.rs` is the YAML serde enum for widget
authoring. Variants: `Slider` (with `handle`, `hit-padding`, `min`, `max`,
`default`), `Button`, `Toggle`, `Panel`. At construction time,
`engine-scene-runtime` converts each variant into a `Box<dyn GuiControl>` from
`engine-gui`. The enum stays in `engine-core` as the stable YAML contract.

## Change playbook

When changing scene model or shared runtime types:

1. update the types here,
2. update `engine-authoring` compile/normalize code,
3. update schema generation,
4. update runtime consumers (`engine-scene-runtime`, `engine-compositor`, `engine`, editor),
5. run focused tests in the affected crates.
