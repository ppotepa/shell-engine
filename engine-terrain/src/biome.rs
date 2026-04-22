//! Biome classification.

use serde::{Deserialize, Serialize};

/// Coarse biome classification for each grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Biome {
    Ocean = 0,
    ShallowWater = 1,
    Beach = 2,
    Desert = 3,
    Grassland = 4,
    Forest = 5,
    Tundra = 6,
    Snow = 7,
    Mountain = 8,
    Volcanic = 9,
}

impl Default for Biome {
    fn default() -> Self {
        Biome::Ocean
    }
}

/// Classify every cell into a `Biome` based on elevation, moisture, temperature.
pub fn classify(
    elevation: &[f32],
    moisture: &[f32],
    temperature: &[f32],
    has_ocean: bool,
    width: usize,
    height: usize,
) -> Vec<Biome> {
    let total = width * height;
    let mut biomes = vec![Biome::Ocean; total];
    for i in 0..total {
        biomes[i] = classify_cell(elevation[i], moisture[i], temperature[i], has_ocean);
    }
    biomes
}

fn classify_cell(elev: f32, moist: f32, temp: f32, has_ocean: bool) -> Biome {
    if has_ocean {
        if elev < 0.45 {
            return Biome::Ocean;
        }
        if elev < 0.50 {
            return Biome::ShallowWater;
        }
        if elev < 0.52 {
            return Biome::Beach;
        }
    }

    // High elevation → mountain or snow
    if elev > 0.80 {
        return if temp < 0.3 {
            Biome::Snow
        } else {
            Biome::Mountain
        };
    }
    if elev > 0.70 && temp < 0.2 {
        return Biome::Snow;
    }

    // Cold zones
    if temp < 0.25 {
        return Biome::Tundra;
    }

    // Dry zones
    if moist < 0.25 {
        return Biome::Desert;
    }

    // Temperate
    if moist > 0.6 {
        return Biome::Forest;
    }

    Biome::Grassland
}

#[cfg(test)]
mod tests {
    use super::{classify, Biome};

    #[test]
    fn classify_without_ocean_avoids_water_biomes() {
        let elevation = vec![0.10, 0.47, 0.51, 0.85];
        let moisture = vec![0.80, 0.50, 0.40, 0.20];
        let temperature = vec![0.70, 0.60, 0.60, 0.10];

        let biomes = classify(&elevation, &moisture, &temperature, false, 2, 2);

        assert!(!biomes
            .iter()
            .any(|biome| { matches!(biome, Biome::Ocean | Biome::ShallowWater | Biome::Beach) }));
    }
}
