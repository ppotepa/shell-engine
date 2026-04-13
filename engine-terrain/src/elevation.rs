//! Elevation field via domain-warped noise — continent shapes + mountain ridges.
//!
//! Algorithm:
//! 1. For each lat/lon cell, convert to 3D sphere coordinates.
//! 2. Apply two-level domain warp to fBm noise → organic continent mask.
//! 3. Blend ridged noise over land to add mountain ranges.
//! 4. Normalise so that `ocean_fraction` of cells are below 0.5.

use crate::grid::cell_to_xyz;
use crate::noise::{continent_noise, ridged_fbm};
use crate::params::PlanetGenParams;

/// Build a normalised elevation grid (values 0.0–1.0).
/// 0.5 = sea level. Exactly `params.ocean_fraction` of cells will be below 0.5.
pub fn build(params: &PlanetGenParams) -> Vec<f32> {
    let w = params.grid_width;
    let h = params.grid_height;
    let seed = params.seed;
    let s = params.continent_scale;
    let warp = params.continent_warp;
    let oct = params.continent_octaves;
    let ms = params.mountain_scale;
    let mstr = params.mountain_strength;

    let mut elev = vec![0.0f32; w * h];

    for y in 0..h {
        for x in 0..w {
            let (cx, cy, cz) = cell_to_xyz(x, y, w, h);

            // Domain-warped continent base noise → [0.0, 1.0)
            let continent = continent_noise(cx, cy, cz, s, warp, oct, seed);

            // Ridged noise for mountain ranges — prominent only over land
            let ridge = ridged_fbm(cx * ms, cy * ms, cz * ms, 5, seed + 700);
            // Smooth land factor: 0 in ocean, 1 on land (peaks above threshold)
            let land_t = ((continent - 0.5) * 5.0).clamp(0.0, 1.0);
            let combined = continent + ridge * mstr * land_t * 0.35;

            elev[y * w + x] = combined as f32;
        }
    }

    // Normalise so ocean_fraction cells are below 0.5
    normalise_to_ocean_fraction(&mut elev, params.ocean_fraction as f32);
    for v in elev.iter_mut() {
        *v = v.clamp(0.0, 1.0);
    }
    elev
}

/// Adjust global elevation so that exactly `ocean_fraction` of cells have elevation < 0.5.
pub(crate) fn normalise_to_ocean_fraction(elev: &mut [f32], ocean_fraction: f32) {
    let mut sorted: Vec<f32> = elev.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let target_idx = ((ocean_fraction * sorted.len() as f32) as usize).min(sorted.len() - 1);
    let sea_level = sorted[target_idx];
    let shift = 0.5 - sea_level;
    for v in elev.iter_mut() {
        *v += shift;
    }
}
