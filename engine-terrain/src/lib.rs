//! Procedural spherical terrain generation via domain-warped noise.
//!
//! Produces a `GeneratedPlanet` from a `PlanetGenParams` seed + high-level params.
//! Output feeds into `engine-celestial`'s `PlanetDef` derivation layer.
//!
//! ## World generation pipeline
//!
//! ```text
//! WorldGenParams
//!   ├── planet: PlanetGenParams  → engine_terrain::generate() → GeneratedPlanet
//!   ├── shape: flat | sphere     → engine-mesh selects mesh generator
//!   ├── coloring: altitude|biome → engine_terrain::coloring::* maps cells → [u8;3]
//!   └── displacement_scale       → radial vertex offset on sphere
//! ```
//!
//! `engine-compositor` owns the bridge between this crate and `engine-mesh`.
//!
//! Algorithm overview:
//! 1. Convert each cell of a 512×256 lat/lon grid to a 3D unit sphere point.
//! 2. Apply two-level domain-warped fBm to generate organic continent shapes.
//! 3. Blend ridged noise over land cells to add mountain ranges.
//! 4. Normalise so `ocean_fraction` of cells are below 0.5 (sea level).
//! 5. Derive moisture (latitude ITCZ pattern + regional noise) and temperature
//!    (latitude + elevation lapse rate).
//! 6. Classify every cell into a `Biome`.
//! 7. Compute aggregate statistics used by the `PlanetDef` mapping layer.

pub mod biome;
pub mod climate;
pub mod coloring;
pub mod elevation;
pub mod grid;
pub mod noise;
pub mod params;
pub mod stats;

pub use biome::Biome;
pub use coloring::{altitude_color, biome_color};
pub use params::{PlanetGenParams, WorldColoring, WorldGenParams, WorldShape};
pub use stats::{BiomeArchetype, GeneratedPlanet, HeightmapCell, PlanetStats};

/// Run the full noise-based pipeline and return a `GeneratedPlanet`.
///
/// Deterministic: identical `params` always produce identical output.
pub fn generate(params: &PlanetGenParams) -> GeneratedPlanet {
    // 1. Elevation via domain-warped noise
    let elevation = elevation::build(params);

    // 2. Climate fields
    let (moisture, temperature) = climate::build(
        &elevation,
        params.grid_width,
        params.grid_height,
        params.seed,
        params.moisture_scale,
    );

    // 3. Biome grid
    let biomes = biome::classify(&elevation, &moisture, &temperature, params.grid_width, params.grid_height);

    // 4. Statistics
    let planet_stats = stats::compute(&elevation, &moisture, &temperature, &biomes, params.grid_width, params.grid_height);

    // 5. Pack cells
    let cells: Vec<HeightmapCell> = (0..params.grid_width * params.grid_height)
        .map(|i| HeightmapCell {
            elevation: elevation[i],
            moisture: moisture[i],
            temperature: temperature[i],
            biome: biomes[i],
        })
        .collect();

    GeneratedPlanet {
        params: params.clone(),
        cells,
        width: params.grid_width,
        height: params.grid_height,
        stats: planet_stats,
    }
}
