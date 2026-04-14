//! Biome and altitude colour palettes for world mesh face colouring.
//!
//! These functions are the single source of truth for planet surface colours.
//! Both `WorldColoring::Biome` and `WorldColoring::Altitude` use functions here
//! so the palette is consistent regardless of mesh shape or URI scheme.

use crate::Biome;

/// Map a [`Biome`] to an sRGB face colour.
///
/// Colours are tuned for a terrestrial Earth-like palette with clear biome
/// boundaries visible at cube-sphere subdivision levels 24–48.
pub fn biome_color(biome: Biome) -> [u8; 3] {
    match biome {
        Biome::Ocean       => [13,  43,  82],   // #0d2b52 deep ocean
        Biome::ShallowWater => [26,  95, 160],  // #1a5fa0 shallow water
        Biome::Beach       => [194, 165,  96],  // #c2a560 sand
        Biome::Desert      => [212, 168,  85],  // #d4a855 hot desert
        Biome::Grassland   => [58,  140,  58],  // #3a8c3a temperate grass
        Biome::Forest      => [30,  107,  30],  // #1e6b1e temperate forest
        Biome::Tundra      => [122, 140, 106],  // #7a8c6a cold scrubland
        Biome::Snow        => [232, 238, 245],  // #e8eef5 snow / ice cap
        Biome::Mountain    => [154, 138, 122],  // #9a8a7a exposed rock
        Biome::Volcanic    => [90,  42,  26],   // #5a2a1a volcanic rock
    }
}

/// Map a normalised elevation (0.0–1.0, sea level = 0.5) to an sRGB face colour.
///
/// Used by `WorldColoring::Altitude` — a simpler gradient that doesn't require
/// running the full climate pipeline.
pub fn altitude_color(elevation: f32) -> [u8; 3] {
    if elevation < 0.45 {
        // Deep ocean
        let t = (elevation / 0.45).clamp(0.0, 1.0);
        lerp_rgb([8, 20, 55], [18, 80, 140], t)
    } else if elevation < 0.50 {
        // Shallow water
        let t = ((elevation - 0.45) / 0.05).clamp(0.0, 1.0);
        lerp_rgb([18, 80, 140], [30, 110, 180], t)
    } else if elevation < 0.53 {
        // Beach / sand
        [194, 165, 96]
    } else if elevation < 0.68 {
        // Lowland / grassland
        let t = ((elevation - 0.53) / 0.15).clamp(0.0, 1.0);
        lerp_rgb([58, 150, 50], [40, 110, 35], t)
    } else if elevation < 0.80 {
        // Highland / shrubland
        let t = ((elevation - 0.68) / 0.12).clamp(0.0, 1.0);
        lerp_rgb([100, 85, 55], [130, 105, 70], t)
    } else if elevation < 0.90 {
        // Mountain rock
        let t = ((elevation - 0.80) / 0.10).clamp(0.0, 1.0);
        lerp_rgb([140, 128, 115], [168, 158, 148], t)
    } else {
        // Snow cap
        let t = ((elevation - 0.90) / 0.10).clamp(0.0, 1.0);
        lerp_rgb([200, 205, 215], [235, 240, 248], t)
    }
}

fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_colors_all_variants() {
        let biomes = [
            Biome::Ocean, Biome::ShallowWater, Biome::Beach, Biome::Desert,
            Biome::Grassland, Biome::Forest, Biome::Tundra, Biome::Snow,
            Biome::Mountain, Biome::Volcanic,
        ];
        for b in biomes {
            let c = biome_color(b);
            assert!(c.iter().all(|&v| v <= 255));
        }
    }

    #[test]
    fn altitude_colors_sample_points() {
        // Deep ocean should be bluish
        let deep = altitude_color(0.2);
        assert!(deep[2] > deep[0], "deep ocean should be blue-dominant");
        // Snow cap should be bright
        let snow = altitude_color(0.95);
        assert!(snow[0] > 180 && snow[1] > 180, "snow should be bright");
    }
}
