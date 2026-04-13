//! Derives `PlanetDef` visual parameters from a `GeneratedPlanet`.
//!
//! Mapping strategy:
//! - Noise params (scale, warp, seed) are copied directly from `PlanetGenParams`
//!   so the renderer generates the same continent shapes as the generator.
//! - Visual params (colors, atmosphere) come from biome archetype lookup tables.
//! All values can be overridden by authored YAML fields on top.

use engine_terrain::{BiomeArchetype, GeneratedPlanet};
use crate::PlanetDef;

/// Derive a `PlanetDef` from generator output.
/// Authored YAML overrides are applied separately by the catalog loader.
pub fn planet_def_from_generated(planet: &GeneratedPlanet) -> PlanetDef {
    let s = &planet.stats;
    let p = &planet.params;
    let mut def = PlanetDef::default_generated();

    // ── Terrain structure ──────────────────────────────────────────────────────
    def.terrain_threshold = 0.5; // normalise_to_ocean_fraction sets sea level at 0.5

    // Copy noise params directly → renderer generates matching continent shapes
    def.noise_seed          = (p.seed % 100_000) as f64;
    def.terrain_noise_scale = p.continent_scale;
    def.terrain_noise_octaves = p.continent_octaves;
    def.warp_strength       = p.continent_warp * 1.8; // renderer warp scale is slightly different
    def.warp_octaves        = 2;
    def.noise_lacunarity    = 2.0;
    def.noise_persistence   = 0.5;

    // Elevation std → terrain roughness
    let roughness = s.elevation_std as f64;
    def.terrain_relief          = (roughness * 1.2).clamp(0.15, 0.55);
    def.normal_perturb_strength = (roughness * 0.8).clamp(0.10, 0.50);

    // Mountain fraction → slight snow line adjustment
    def.snow_line_altitude = (0.80 - s.mountain_fraction as f64 * 0.4).clamp(0.40, 0.88);

    // ── Climate → ice caps ────────────────────────────────────────────────────
    let cold = s.cold_fraction as f64;
    let polar_t = s.polar_temperature as f64;
    def.polar_ice_start = (1.0 - polar_t * 0.5 - cold * 0.3).clamp(0.30, 0.95);
    def.polar_ice_end   = (def.polar_ice_start + 0.12).min(1.0);

    // ── Moisture → desert & clouds ────────────────────────────────────────────
    let moist = s.land_moisture as f64;
    def.desert_strength     = ((1.0 - moist) * 0.9).clamp(0.0, 0.95);
    def.cloud_threshold     = (0.85 - moist * 0.35).clamp(0.50, 0.95);
    def.cloud_noise_scale   = 3.0 + moist * 1.5;
    def.cloud_noise_octaves = if moist > 0.6 { 4 } else { 3 };

    // ── Ocean specular ─────────────────────────────────────────────────────────
    def.ocean_specular = if s.ocean_fraction > 0.5 { 0.55 } else { 0.25 };

    // ── Latitude bands ────────────────────────────────────────────────────────
    if roughness < 0.08 {
        def.latitude_bands      = 14;
        def.latitude_band_depth = 0.30;
    } else {
        def.latitude_bands      = if s.mean_temperature > 0.6 { 5 } else { 3 };
        def.latitude_band_depth = 0.07;
    }

    // ── Archetype-based visual presets ─────────────────────────────────────────
    apply_archetype_visuals(&mut def, s.archetype, s.ocean_fraction, moist, cold);

    // ── Store heightmap for renderer ───────────────────────────────────────────
    // heightmap_blend = 0: renderer noise drives both displacement AND coloring, keeping them in sync.
    // The generated heightmap (engine-terrain) is still computed for stats/archetype but not blended in.
    def.generated_heightmap   = None;
    def.generated_heightmap_w = 0;
    def.generated_heightmap_h = 0;
    def.heightmap_blend       = 0.0;

    // ── Vertex displacement ───────────────────────────────────────────────────
    // Controlled by YAML terrain_displacement override; set a sensible archetype default here.
    def.terrain_displacement = match s.archetype {
        BiomeArchetype::Oceanic     => 0.06,
        BiomeArchetype::Terrestrial => 0.14,
        BiomeArchetype::Arid        => 0.20,
        BiomeArchetype::Volcanic    => 0.24,
        BiomeArchetype::Frozen      => 0.10,
    };

    def
}

fn apply_archetype_visuals(def: &mut PlanetDef, arch: BiomeArchetype, ocean_frac: f32, moist: f64, cold: f64) {
    match arch {
        BiomeArchetype::Oceanic => {
            def.ocean_color        = "#091c3a".to_string();
            def.land_color         = "#3a6028".to_string();
            def.atmo_color         = Some("#5aaae0".to_string());
            def.atmo_strength      = 0.40;
            def.polar_ice_color    = Some("#e8f4ff".to_string());
            def.shadow_color       = Some("#020810".to_string());
            def.midtone_color      = Some("#0d3060".to_string());
            def.highlight_color    = Some("#5ab8ee".to_string());
            def.tone_mix           = 0.80;
            def.cloud_color        = Some("#d4ecff".to_string());
            def.night_light_intensity = 0.0;
        }
        BiomeArchetype::Arid => {
            def.ocean_color        = "#1a0a04".to_string();
            def.land_color         = "#8b3a10".to_string();
            def.desert_color       = Some("#c87840".to_string());
            def.atmo_color         = Some("#c87038".to_string());
            def.atmo_strength      = 0.18;
            def.shadow_color       = Some("#0a0200".to_string());
            def.midtone_color      = Some("#601808".to_string());
            def.highlight_color    = Some("#d08840".to_string());
            def.tone_mix           = 0.85;
            def.cloud_threshold    = 0.95;
            def.night_light_intensity = 0.0;
        }
        BiomeArchetype::Frozen => {
            def.ocean_color        = "#0a1830".to_string();
            def.land_color         = "#8ca8c0".to_string();
            def.polar_ice_color    = Some("#f0f4ff".to_string());
            def.polar_ice_start    = 0.15;
            def.polar_ice_end      = 0.98;
            def.atmo_color         = Some("#8abce0".to_string());
            def.atmo_strength      = 0.28;
            def.shadow_color       = Some("#040c1a".to_string());
            def.midtone_color      = Some("#1a3c60".to_string());
            def.highlight_color    = Some("#a8ccee".to_string());
            def.tone_mix           = 0.82;
            def.cloud_color        = Some("#d8ecff".to_string());
            def.night_light_intensity = 0.0;
        }
        BiomeArchetype::Volcanic => {
            def.ocean_color        = "#080200".to_string();
            def.land_color         = "#180800".to_string();
            def.desert_color       = Some("#4a1808".to_string());
            def.atmo_color         = Some("#e06018".to_string());
            def.atmo_strength      = 0.38;
            def.atmo_rim_power     = 3.2;
            def.shadow_color       = Some("#030100".to_string());
            def.midtone_color      = Some("#280800".to_string());
            def.highlight_color    = Some("#6a1800".to_string());
            def.tone_mix           = 0.90;
            def.night_light_color  = Some("#ff5500".to_string());
            def.night_light_threshold = 0.55;
            def.night_light_intensity = 1.2;
            def.cloud_color        = Some("#604030".to_string());
            def.cloud_threshold    = 0.92;
        }
        BiomeArchetype::Terrestrial => {
            let blue = ocean_frac.clamp(0.3, 0.8) as f64;
            def.ocean_color        = if blue > 0.55 { "#0b2748".to_string() } else { "#0f3020".to_string() };
            def.land_color         = if moist > 0.5 { "#3a6b28".to_string() } else { "#6b5028".to_string() };
            def.atmo_color         = Some("#4a90d9".to_string());
            def.atmo_strength      = 0.30 + moist * 0.10;
            def.polar_ice_color    = Some("#ddeeff".to_string());
            def.shadow_color       = Some("#020810".to_string());
            def.midtone_color      = Some("#0d3464".to_string());
            def.highlight_color    = Some("#5abde8".to_string());
            def.tone_mix           = 0.78;
            def.cloud_color        = Some("#d0e8ff".to_string());
            def.night_light_color  = Some("#e8c46a".to_string());
            def.night_light_threshold = 0.80;
            def.night_light_intensity = if cold > 0.4 { 0.0 } else { 0.5 };
        }
    }

    if def.atmo_strength > 0.0 {
        def.atmo_rim_power = 3.8;
    }
}


