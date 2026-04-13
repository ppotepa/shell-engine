# engine-mesh

Procedural 3D mesh generation for Shell Quest.

## Purpose

`engine-mesh` is a zero-dependency, Tier-1 crate that owns all runtime
geometry generation. It provides a [`Mesh`] type and primitive generators
that the compositor (and future tooling) consume to build renderably meshes
without reading `.obj` files from disk.

## Key types

- `Mesh` — triangle mesh: `vertices`, `normals`, `faces` (index triples)
- `primitives::cube_sphere(N)` — uniform cube-sphere, N subdivisions per edge
- `primitives::uv_sphere(lat, lon)` — classic lat/lon sphere (matches original `sphere.obj` topology)

## Primitive comparison

| Generator | N / bands | Vertices | Triangles | Notes |
|-----------|-----------|----------|-----------|-------|
| `cube_sphere` | 32  |  6 534  |  12 288 | Fast, coarse |
| `cube_sphere` | 64  | 25 350  |  49 152 | Good quality, used by default |
| `cube_sphere` | 128 | 99 846  | 196 608 | High-resolution |
| `uv_sphere`   | 40×80 | 3 362 |   6 240 | Matches legacy `sphere.obj` |

`cube_sphere` produces a nearly uniform triangle distribution and has no
pole singularity, making it ideal for procedural planet rendering.

## Integration with `engine-compositor`

`engine-compositor` converts `Mesh` → `ObjMesh` and injects it into the
global mesh cache via the `cube-sphere://N` URI scheme. Scene YAML authors
reference it via:

```yaml
mesh-source: cube-sphere://64
```

The compositor intercepts this URI in `get_or_load_obj_mesh`, calls
`engine_mesh::primitives::cube_sphere(64)`, converts with
`obj_loader::mesh_to_obj_mesh`, and caches the result under that key.

No file I/O occurs — generation happens once per subdivision level at
first use and is shared across all sprites via `Arc<ObjMesh>`.

## Adding new primitives

1. Add a module under `src/primitives/`.
2. Re-export from `primitives/mod.rs`.
3. Add a URI scheme handler in `engine-compositor/src/obj_render.rs`
   inside `get_or_load_obj_mesh`.

## Dependency tier

This crate has **no engine dependencies**. It can be used from tests,
tools, and future renderers without pulling in the full pipeline.
