# engine-compositor

Frame assembly, sprite dispatch orchestration, and PostFX.

## Purpose

`engine-compositor` sits between runtime scene state and the final back buffer.
It no longer owns the 3D raster/render domain. Instead it assembles the frame
by combining:

- 2D rendering from `engine-render-2d`,
- 3D rendering and prerender outputs from `engine-render-3d`,
- prepared layer/frame inputs,
- PostFX and dirty-region propagation.

This crate should stay focused on composition and delegation.

## What it owns

- composition entry points (`dispatch_composite*`)
- layer/frame preparation helpers
- prepared 2D/3D sprite dispatch wiring
- PostFX execution
- engine-facing provider/access seams
- prerender orchestration helpers that register or consume data owned elsewhere

## What it does not own

- 2D sprite raster internals
- 3D rasterization or shading
- generated-world mesh building
- Scene3D work-item rendering internals

Those belong to `engine-render-2d`, `engine-render-3d`, and `engine-worldgen`.

## Key modules

- `compositor` — composition entry points
- `scene_compositor` — frame input structs and prepared layer helpers
- `prepared_frame` — preclassified 2D/3D sprite inputs
- `provider` / `access` — engine integration seams
- `prerender` — compositor-side preparation orchestration
- `effect_applicator` / `systems::postfx` — PostFX execution

## Working with this crate

- keep world/resource extraction in `engine`,
- keep sprite raster logic in render crates,
- preserve object-region reporting for targeted effects and behavior consumers,
- preserve dirty-region correctness through PostFX and final assembly,
- keep prerender helpers data-oriented instead of mutating `World` directly.

## Invariants

- PostFX must preserve or widen the accumulated dirty region.
- Prepared-frame paths and direct sprite dispatch paths must remain behaviorally equivalent.
- Optional accelerators such as OBJ prerender and Scene3D atlas prerender must not become the only render path.

## Integration points

- `engine` calls compositor entry points from its compositor system
- `engine-scene-runtime` supplies object states, target resolver, and camera state
- `engine-render-2d` provides 2D rendering primitives
- `engine-render-3d` provides 3D sprite specs, raster helpers, and prerender seams
