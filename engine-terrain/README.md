# engine-terrain

Procedural spherical terrain generation via domain-warped noise.

## Purpose

`engine-terrain` is a Tier-1 crate that owns the full planet generation
pipeline: elevation noise, climate simulation, and biome classification.
It produces a `GeneratedPlanet` from a `PlanetGenParams` seed, which
`engine-compositor` consumes to color and displace mesh vertices.

## Pipeline

```text
PlanetGenParams (seed + 12 knobs)
    │
    ▼
elevation::build()          512×256 lat/lon grid
  two-level domain-warped fBm → organic continent shapes
  ridged noise over land → mountain ranges
  normalise so ocean_fraction of cells are below sea level
    │
    ▼
climate::build()
  latitude-based ITCZ moisture pattern + regional noise
  temperature = latitude + elevation lapse rate
  ice cap strength, rain shadow
    │
    ▼
biome::classify()
  10-type biome grid (Ocean, ShallowOcean, Desert, Grassland,
  Forest, TropicalForest, Tundra, Snow, Mountain, Beach)
    │
    ▼
stats::compute()
  aggregate coverage fractions → PlanetStats
    │
    ▼
GeneratedPlanet { cells, stats, width, height }
```

## Key types

| Type | Module | Description |
|------|--------|-------------|
| `PlanetGenParams` | `params.rs` | Seed + ocean fraction, continent/mountain/climate knobs |
| `WorldGenParams` | `params.rs` | Shape + coloring + subdivisions + `PlanetGenParams` |
| `WorldShape` | `params.rs` | `Flat` / `Sphere` enum |
| `WorldColoring` | `params.rs` | `Altitude` / `Biome` / `None` enum |
| `GeneratedPlanet` | `stats.rs` | Heightmap cells + biome grid + aggregate stats |
| `PlanetStats` | `stats.rs` | Coverage fractions (ocean, forest, desert, snow, mountain, …) |
| `HeightmapCell` | `stats.rs` | Per-cell elevation, moisture, temperature, biome |
| `Biome` | `biome.rs` | 10-type classification enum |

## Modules

| Module | Purpose |
|--------|---------|
| `params.rs` | `PlanetGenParams`, `WorldGenParams`, defaults |
| `elevation.rs` | Domain-warped fBm + ridged mountain noise |
| `climate.rs` | Moisture, temperature, ice caps, rain shadow |
| `biome.rs` | 10-biome classification from elevation/moisture/temperature |
| `coloring.rs` | `biome_color()` → RGB, `altitude_color()` → RGB |
| `noise.rs` | Simplex noise, fBm, ridged fBm primitives |
| `grid.rs` | Lat/lon ↔ 3D sphere coordinate helpers |
| `stats.rs` | Aggregate statistics + output types |
| `lib.rs` | Pipeline entry point `generate()` + global stats cache |

## Integration

`engine-compositor` calls `engine_terrain::generate()` when it encounters a
`world://` URI. The generated planet's cells are mapped onto a
`engine_mesh::cube_sphere(N)` geometry with per-vertex elevation displacement
and per-face biome/altitude coloring.

`engine-behavior` registers a `planet_last_stats()` Rhai function that reads
from the global stats cache, exposing biome coverage to scripts.

## Parameters reference

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `seed` | 0 | 0–9999 | Deterministic random seed |
| `ocean_fraction` | 0.55 | 0.01–0.99 | Target ocean coverage |
| `continent_scale` | 2.5 | 0.5–10 | Landmass size (smaller = larger continents) |
| `continent_warp` | 0.65 | 0–2 | Coastline chaos / organic shapes |
| `continent_octaves` | 5 | 1–8 | Coastline detail level |
| `mountain_scale` | 6.0 | 1–15 | Mountain range spacing |
| `mountain_strength` | 0.45 | 0–1 | Mountain height contribution |
| `mountain_ridge_octaves` | 5 | 1–8 | Ridge jaggedness |
| `moisture_scale` | 3.0 | 0.5–8 | Moisture pattern frequency |
| `ice_cap_strength` | 1.0 | 0–3 | Polar cold zone intensity |
| `lapse_rate` | 0.6 | 0–1.5 | Temperature drop per elevation unit |
| `rain_shadow` | 0.35 | 0–1 | Moisture reduction at altitude |

## Dependency tier

This crate depends only on `serde`. It has **no engine dependencies** and can
be used from tests and tools without pulling in the full pipeline.
