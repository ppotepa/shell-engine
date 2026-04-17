# engine-render-3d

Shared 3D rendering domain crate.

## Purpose

`engine-render-3d` owns the reusable 3D domain that should not live in
compositor assembly:

- typed 3D scene/render inputs,
- sprite-to-3D spec extraction,
- generated-world rendering,
- Scene3D prerender/runtime-store logic,
- software raster helpers,
- shading, atmosphere, terrain, and biome effect kernels.

The compositor should delegate to this crate instead of reimplementing 3D
internals.

## Main modules

- `api` — concrete 3D pipeline input/output types and seams
- `scene` — typed 3D scene graph/runtime data
- `pipeline` — prepared sprite specs and render execution helpers
- `prerender` — Scene3D atlas/runtime-store/work-item orchestration
- `raster` — low-level software raster helpers shared by 3D paths
- `effects` — atmosphere, biome, terrain, and related effect kernels
- `geom` / `shading` — math, clip, rendering types, and color/shading utilities

## Ownership split

- `engine-render-2d` owns 2D draw logic
- `engine-render-3d` owns 3D draw logic
- `engine-compositor` assembles final frames using both

## Integration

- `engine-compositor` uses prepared 3D sprite specs and 3D callbacks from here
- `engine-worldgen` supplies generated mesh/build-key inputs
- `engine-scene-runtime` and `engine-api` feed typed-first runtime mutation data
