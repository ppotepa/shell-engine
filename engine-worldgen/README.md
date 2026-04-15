# engine-worldgen

World URI parsing, base-sphere selection, and procedural world mesh building.

## Purpose

`engine-worldgen` owns the bridge between `engine-terrain` (planet heightmap
generation) and `engine-mesh` (geometry primitives). It provides:

- `world://` URI parsing → `WorldGenParams`
- Canonical URI serialization for cache keys
- Base-sphere selection (`cube`, `uv`, `tetra`, `octa`, `icosa`)
- Per-vertex elevation displacement
- Per-face biome/altitude coloring
- `GeneratedWorldMesh` output consumed by `engine-compositor`

This keeps `engine-compositor` focused on orchestration and buffer
composition — the world mesh pipeline is self-contained here.

## Key functions

| Function | Description |
|----------|-------------|
| `parse_world_params_from_uri(uri)` | Parse `world://N?...` into `WorldGenParams` |
| `world_uri_from_params(p)` | Canonical URI string for cache key / runtime updates |
| `build_world_mesh(p)` | Generate `GeneratedWorldMesh` (mesh + face colors) from params |

## URI format

```
world://SUBDIVISIONS?shape=sphere&base=cube&coloring=biome&seed=42&ocean=0.55&...
```

All query parameters are optional — missing ones fall back to `WorldGenParams::default()`.

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `shape` | `sphere` / `flat` | `sphere` | Overall shape |
| `base` | `cube` / `uv` / `tetra` / `octa` / `icosa` | `cube` | Sphere topology |
| `coloring` | `biome` / `altitude` / `none` | `biome` | Face coloring strategy |
| `seed` | 0–9999 | 0 | Planet random seed |
| `ocean` | 0.01–0.99 | 0.55 | Ocean fraction |
| `cscale` | 0.5–10 | 2.5 | Continent scale |
| `cwarp` | 0–2 | 0.65 | Coastline chaos |
| `coct` | 2–8 | 5 | Continent noise octaves |
| `mscale` | 1–20 | 6.0 | Mountain spacing |
| `mstr` | 0–1 | 0.45 | Mountain strength |
| `mroct` | 2–8 | 5 | Ridge jaggedness |
| `moistscale` | 0.5–10 | 3.0 | Moisture scale |
| `ice` | 0–3 | 1.0 | Polar ice intensity |
| `lapse` | 0–1 | 0.6 | Altitude cooling |
| `rainshadow` | 0–1 | 0.35 | Rain shadow |
| `disp` | 0–1 | 0.22 | Vertex displacement scale |

## Dependencies

- `engine-terrain` — heightmap generation and biome pipeline
- `engine-mesh` — geometry primitives (cube/uv/poly spheres)
