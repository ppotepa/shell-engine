//! Aggregate statistics and output types.

use crate::{Biome, PlanetGenParams};
use serde::{Deserialize, Serialize};

/// Single heightmap cell — the elementary output unit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HeightmapCell {
    /// Normalised elevation 0.0–1.0. 0.5 = sea level.
    pub elevation: f32,
    /// Normalised moisture 0.0–1.0.
    pub moisture: f32,
    /// Normalised temperature 0.0–1.0 (1.0 = hottest).
    pub temperature: f32,
    /// Biome classification.
    pub biome: Biome,
}

/// High-level planet archetype derived from dominant biomes.
/// Used to look up default visual parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeArchetype {
    /// Mostly water, temperate continents.
    Oceanic,
    /// Mostly arid / desert.
    Arid,
    /// Mostly cold / polar.
    Frozen,
    /// Mostly lava / volcanic.
    Volcanic,
    /// Mix of land and ocean — generic terrestrial.
    Terrestrial,
}

/// Aggregate statistics derived from the heightmap.
/// These drive the `PlanetDef` parameter derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetStats {
    /// Fraction of cells with elevation < 0.5 (ocean).
    pub ocean_fraction: f32,
    /// Mean moisture across all land cells.
    pub land_moisture: f32,
    /// Mean temperature across all cells.
    pub mean_temperature: f32,
    /// Temperature at polar cells (y near 0 or height-1).
    pub polar_temperature: f32,
    /// Standard deviation of elevation — proxy for terrain roughness.
    pub elevation_std: f32,
    /// Mean elevation of mountain cells (elev > 0.7).
    pub mountain_fraction: f32,
    /// Dominant biome archetype.
    pub archetype: BiomeArchetype,
    /// Fraction of desert cells among land cells.
    pub desert_fraction: f32,
    /// Fraction of snow/tundra cells among land cells.
    pub cold_fraction: f32,
    /// Fraction of ocean cells that are shallow water.
    pub shallow_fraction: f32,
    /// Fraction of forest cells among land cells.
    pub forest_fraction: f32,
    /// Fraction of grassland cells among land cells.
    pub grassland_fraction: f32,
}

/// The full output of the tectonic generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPlanet {
    /// Input parameters (retained for display and reproducibility).
    pub params: PlanetGenParams,
    /// Flat heightmap cells, row-major (y * width + x).
    pub cells: Vec<HeightmapCell>,
    pub width: usize,
    pub height: usize,
    /// Aggregate statistics.
    pub stats: PlanetStats,
}

impl GeneratedPlanet {
    /// Access a cell at grid coordinates.
    #[inline]
    pub fn cell(&self, x: usize, y: usize) -> &HeightmapCell {
        &self.cells[y * self.width + x]
    }
}

/// Compute aggregate statistics from the heightmap fields.
pub fn compute(
    elevation: &[f32],
    moisture: &[f32],
    temperature: &[f32],
    biomes: &[Biome],
    width: usize,
    height: usize,
) -> PlanetStats {
    let total = elevation.len() as f32;

    let ocean_count = biomes
        .iter()
        .filter(|&&biome| matches!(biome, Biome::Ocean | Biome::ShallowWater))
        .count();
    let ocean_fraction = ocean_count as f32 / total;

    // Land cell stats
    let land: Vec<usize> = (0..elevation.len())
        .filter(|&i| !matches!(biomes[i], Biome::Ocean | Biome::ShallowWater))
        .collect();
    let land_moisture = if land.is_empty() {
        0.5
    } else {
        land.iter().map(|&i| moisture[i]).sum::<f32>() / land.len() as f32
    };

    let mean_temperature = temperature.iter().sum::<f32>() / total;

    // Polar cells: top and bottom 10% of rows
    let polar_rows = (height / 10).max(1);
    let polar_cells: Vec<f32> = (0..width * polar_rows)
        .chain((height - polar_rows) * width..width * height)
        .map(|i| temperature[i])
        .collect();
    let polar_temperature = if polar_cells.is_empty() {
        0.1
    } else {
        polar_cells.iter().sum::<f32>() / polar_cells.len() as f32
    };

    // Elevation std
    let mean_elev = elevation.iter().sum::<f32>() / total;
    let var = elevation
        .iter()
        .map(|&e| (e - mean_elev).powi(2))
        .sum::<f32>()
        / total;
    let elevation_std = var.sqrt();

    let mountain_count = elevation.iter().filter(|&&e| e > 0.7).count();
    let mountain_fraction = mountain_count as f32 / total;

    let desert_count = biomes.iter().filter(|&&b| b == Biome::Desert).count();
    let desert_fraction = if land.is_empty() {
        0.0
    } else {
        desert_count as f32 / land.len() as f32
    };

    let cold_count = biomes
        .iter()
        .filter(|&&b| matches!(b, Biome::Snow | Biome::Tundra))
        .count();
    let cold_fraction = if land.is_empty() {
        0.0
    } else {
        cold_count as f32 / land.len() as f32
    };

    let shallow_count = biomes.iter().filter(|&&b| b == Biome::ShallowWater).count();
    let shallow_fraction = if ocean_count == 0 {
        0.0
    } else {
        shallow_count as f32 / ocean_count as f32
    };

    let forest_count = biomes.iter().filter(|&&b| b == Biome::Forest).count();
    let forest_fraction = if land.is_empty() {
        0.0
    } else {
        forest_count as f32 / land.len() as f32
    };

    let grassland_count = biomes.iter().filter(|&&b| b == Biome::Grassland).count();
    let grassland_fraction = if land.is_empty() {
        0.0
    } else {
        grassland_count as f32 / land.len() as f32
    };

    let archetype = derive_archetype(
        ocean_fraction,
        desert_fraction,
        cold_fraction,
        mountain_fraction,
    );

    PlanetStats {
        ocean_fraction,
        land_moisture,
        mean_temperature,
        polar_temperature,
        elevation_std,
        mountain_fraction,
        archetype,
        desert_fraction,
        cold_fraction,
        shallow_fraction,
        forest_fraction,
        grassland_fraction,
    }
}

fn derive_archetype(ocean: f32, desert: f32, cold: f32, mountain: f32) -> BiomeArchetype {
    if ocean > 0.80 {
        return BiomeArchetype::Oceanic;
    }
    if desert > 0.55 {
        return BiomeArchetype::Arid;
    }
    if cold > 0.55 {
        return BiomeArchetype::Frozen;
    }
    if mountain > 0.35 {
        return BiomeArchetype::Volcanic;
    }
    BiomeArchetype::Terrestrial
}

#[cfg(test)]
mod tests {
    use super::compute;
    use crate::Biome;

    #[test]
    fn ocean_fraction_counts_water_biomes_not_raw_elevation() {
        let elevation = vec![0.10, 0.20, 0.80, 0.90];
        let moisture = vec![0.50; 4];
        let temperature = vec![0.50; 4];
        let biomes = vec![
            Biome::Desert,
            Biome::Grassland,
            Biome::Mountain,
            Biome::Snow,
        ];

        let stats = compute(&elevation, &moisture, &temperature, &biomes, 2, 2);

        assert_eq!(stats.ocean_fraction, 0.0);
    }
}
