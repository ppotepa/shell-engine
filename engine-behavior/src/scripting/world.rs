//! Rhai API for world/planet generation readback.

use rhai::{Engine as RhaiEngine, Map as RhaiMap};

/// Register world/planet scripting functions.
pub fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_fn("planet_last_stats", planet_last_stats);
}

/// Returns the last generated planet's biome coverage as a Rhai map.
///
/// Keys: ocean, shallow, beach, desert, grassland, forest, cold, mountain
/// Values: f64 fractions (0.0–1.0).
/// Returns an empty map if no planet has been generated yet.
fn planet_last_stats() -> RhaiMap {
    let mut map = RhaiMap::new();
    if let Some(stats) = engine_terrain::last_planet_stats() {
        map.insert(
            "ocean".into(),
            rhai::Dynamic::from_float(stats.ocean_fraction as f64),
        );
        map.insert(
            "shallow".into(),
            rhai::Dynamic::from_float(stats.shallow_fraction as f64),
        );
        map.insert(
            "desert".into(),
            rhai::Dynamic::from_float(stats.desert_fraction as f64),
        );
        map.insert(
            "grassland".into(),
            rhai::Dynamic::from_float(stats.grassland_fraction as f64),
        );
        map.insert(
            "forest".into(),
            rhai::Dynamic::from_float(stats.forest_fraction as f64),
        );
        map.insert(
            "cold".into(),
            rhai::Dynamic::from_float(stats.cold_fraction as f64),
        );
        map.insert(
            "mountain".into(),
            rhai::Dynamic::from_float(stats.mountain_fraction as f64),
        );
    }
    map
}
