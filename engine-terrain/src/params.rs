//! Generation parameters — YAML-deserializable, seed-based.

use serde::{Deserialize, Serialize};

/// High-level parameters for the noise-based planet generator.
/// These become the canonical "seed" of the planet — same params = same planet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanetGenParams {
    /// Primary random seed.
    pub seed: u64,

    /// Target fraction of ocean surface (0.0–1.0). Drives elevation normalisation.
    #[serde(default = "PlanetGenParams::default_ocean_fraction")]
    pub ocean_fraction: f64,

    /// Continent noise frequency scale. Smaller = larger continents, bigger = archipelagos.
    /// Recommended range 1.5–5.0. Default 2.5.
    #[serde(default = "PlanetGenParams::default_continent_scale")]
    pub continent_scale: f64,

    /// Domain warp strength. 0.0 = smooth continents, 1.5 = very organic/chaotic coastlines.
    #[serde(default = "PlanetGenParams::default_continent_warp")]
    pub continent_warp: f64,

    /// fBm octaves for the continent noise. More octaves = finer coastline detail. Range 3–7.
    #[serde(default = "PlanetGenParams::default_continent_octaves")]
    pub continent_octaves: u8,

    /// Ridged noise frequency for mountain ranges. Higher = narrower mountain chains.
    #[serde(default = "PlanetGenParams::default_mountain_scale")]
    pub mountain_scale: f64,

    /// Mountain elevation contribution over land (0.0–1.0).
    #[serde(default = "PlanetGenParams::default_mountain_strength")]
    pub mountain_strength: f64,

    /// Noise frequency for regional moisture variation.
    #[serde(default = "PlanetGenParams::default_moisture_scale")]
    pub moisture_scale: f64,

    /// Heightmap grid width (longitude cells). Default 512.
    #[serde(default = "PlanetGenParams::default_grid_width")]
    pub grid_width: usize,

    /// Heightmap grid height (latitude cells). Default 256.
    #[serde(default = "PlanetGenParams::default_grid_height")]
    pub grid_height: usize,
}

impl Default for PlanetGenParams {
    fn default() -> Self {
        Self {
            seed: 0,
            ocean_fraction: Self::default_ocean_fraction(),
            continent_scale: Self::default_continent_scale(),
            continent_warp: Self::default_continent_warp(),
            continent_octaves: Self::default_continent_octaves(),
            mountain_scale: Self::default_mountain_scale(),
            mountain_strength: Self::default_mountain_strength(),
            moisture_scale: Self::default_moisture_scale(),
            grid_width: Self::default_grid_width(),
            grid_height: Self::default_grid_height(),
        }
    }
}

impl PlanetGenParams {
    pub fn with_seed(seed: u64) -> Self {
        Self { seed, ..Self::default() }
    }

    fn default_ocean_fraction() -> f64   { 0.55 }
    fn default_continent_scale() -> f64  { 2.5 }
    fn default_continent_warp() -> f64   { 0.65 }
    fn default_continent_octaves() -> u8 { 5 }
    fn default_mountain_scale() -> f64   { 6.0 }
    fn default_mountain_strength() -> f64 { 0.45 }
    fn default_moisture_scale() -> f64   { 3.0 }
    fn default_grid_width() -> usize     { 512 }
    fn default_grid_height() -> usize    { 256 }
}

