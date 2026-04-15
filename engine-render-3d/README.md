# engine-render-3d

Shared 3D rendering domain crate.

## Purpose

`engine-render-3d` centralizes reusable 3D render-domain logic that should not
live in compositor orchestration:

- geometry helpers (`geom::math`, `geom::clip`, `geom::types`),
- procedural effect kernels (`effects::*`),
- shading and color-space transforms (`shading`),
- pipeline seam contracts (`api::Render3dPipeline`).

This keeps `engine-compositor` focused on layer/sprite orchestration and final
buffer composition.

## Modules

- `api` — generic render pipeline seam trait.
- `geom` — vector math, line clipping, shared rendering types.
- `effects` — atmosphere, biome, terrain, and noise helpers.
- `shading` — tone quantization, shading/tint composition, color-space utilities.

## Integration

`engine-compositor` imports this crate for planet/OBJ render-domain internals.
Higher-level engine crates should continue calling compositor entry points.
