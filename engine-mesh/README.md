# engine-mesh

Procedural 3D mesh generation for Shell Engine.

## Purpose

`engine-mesh` is a low-dependency geometry crate that owns reusable runtime
mesh generation. It provides a `Mesh` type and primitive generators without
depending on compositor orchestration.

## Key types

- `Mesh` — triangle mesh (`vertices`, `normals`, `faces`)
- `primitives::cube_sphere(N)` — uniform cube-sphere
- `primitives::uv_sphere(lat, lon)` — classic lat/lon sphere
- `primitives::tetra_sphere(levels)` — subdivided tetrahedron sphere
- `primitives::octa_sphere(levels)` — subdivided octahedron sphere
- `primitives::icosa_sphere(levels)` — subdivided icosphere

## Integration

`engine-worldgen` and `engine-render-3d` consume these primitives and build
keys, while compositor-side adapters only see the prepared render inputs.

Scene YAML authors reference generated sphere meshes via:

```yaml
mesh-source: cube-sphere://64
```

The generated mesh is resolved through the shared asset/worldgen path and
cached under its normalized build key.

## `world://`

The `world://N` URI extends generated sphere meshes with procedural terrain.
`engine-worldgen` owns the translation from `WorldGenParams` to
`GeneratedWorldMesh`, including elevation displacement and biome/altitude
coloring.
