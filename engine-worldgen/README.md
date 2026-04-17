# engine-worldgen

World URI parsing, mesh build keys, and procedural generated-world meshes.

## Purpose

`engine-worldgen` owns the bridge between `engine-terrain` and `engine-mesh`.
It provides:

- `world://` URI parsing into `WorldGenParams`
- canonical URI serialization and normalized mesh build keys
- base-sphere selection (`cube`, `uv`, `tetra`, `octa`, `icosa`)
- per-vertex elevation displacement
- per-face biome/altitude coloring
- `GeneratedWorldMesh` output consumed by `engine-render-3d`

This keeps generated-world mesh construction outside compositor frame assembly.

## Key functions

| Function | Description |
|----------|-------------|
| `parse_world_params_from_uri(uri)` | Parse `world://N?...` into `WorldGenParams` |
| `world_uri_from_params(p)` | Canonical URI string for cache keys and runtime updates |
| `world_mesh_build_key_from_uri(uri)` | Stable normalized build key for generated meshes |
| `prepare_world_gen_from_uri(uri)` | Metadata-only prep (`params` + normalized build key) |
| `build_world_mesh(p)` | Generate `GeneratedWorldMesh` from params |

## URI format

```text
world://SUBDIVISIONS?shape=sphere&base=cube&coloring=biome&seed=42&ocean=0.55&...
```

All query parameters are optional. Missing ones fall back to
`WorldGenParams::default()`.

## Dependencies

- `engine-terrain` — climate, elevation, biomes
- `engine-mesh` — base geometry primitives
