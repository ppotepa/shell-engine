# engine-render-3d

Shared 3D rendering domain crate.

## Purpose

`engine-render-3d` owns the reusable 3D domain that should not live in
compositor assembly:

- neutral frame-level 3D render inputs,
- sprite/domain-to-frame producer seams,
- generated-world rendering,
- Scene3D prerender/runtime-store logic,
- software raster helpers,
- shading, atmosphere, terrain, and biome effect kernels.
- consumption of resolved scene-level lighting/environment profiles without
  mod-specific knowledge of planets or particular scenes.

The compositor should delegate to this crate instead of reimplementing 3D
internals.

## Main modules

- `api` / `frame_input` / `frame_profiles` — neutral 3D frame contract and
  profile types (`Render3dFrameInput`, `Frame*Profile`)
- `scene` — typed 3D scene graph/runtime data
  - includes LOD policy seam (`scene::lod::{select_lod_level, select_lod_level_stable}`) and `Node3DInstance::lod_hint`
- `pipeline` — prepared-item producers, sprite specs, and render execution helpers
  - generated-world path includes cloud cadence/reuse hooks for CPU raster cost control
- `effects::passes` — reusable effect/pass seams (`surface`, `halo`,
  `postprocess`, `RenderPassContext`)
- `prerender` — Scene3D atlas/runtime-store/work-item orchestration
- `raster` — low-level software raster helpers shared by 3D paths
- `effects` — atmosphere, biome, terrain, and related effect kernels
- `geom` / `shading` — math, clip, rendering types, and color/shading utilities

## Ownership split

- `engine-render-2d` owns 2D draw logic
- `engine-render-3d` owns 3D draw logic
- `engine-compositor` assembles final frames using both

## Integration

- `engine-compositor` should consume `PreparedRender3dItem` /
  `render_prepared_render3d_item_to_buffer(...)` instead of rebuilding 3D
  dispatch logic locally
- `engine-worldgen` supplies generated mesh/build-key inputs
  - including optional LOD-tagged mesh build-key domains for future adaptive LOD rollout
- `engine-scene-runtime` and `engine-api` feed typed-first runtime mutation data

## Public seams

The current neutralization work centers around these public seams:

- `Render3dFrameInput` and `Frame*Profile` for renderer-facing frame data
- `PreparedRender3dItem` / `PreparedRender3dSource` for producer output
- `prepare_render3d_item(...)` for sprite/spec extraction into prepared items
- `render_prepared_render3d_item_to_buffer(...)` for common prepared-item dispatch
- `RenderPassContext` for pass-level effect execution
