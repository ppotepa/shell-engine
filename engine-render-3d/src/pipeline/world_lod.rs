pub fn apply_world_lod_to_source(
    source: &str,
    lod_level: engine_core::render_types::LodLevel,
) -> String {
    if !source.starts_with("world://") {
        return source.to_string();
    }
    engine_worldgen::apply_world_lod_to_uri(source, lod_level.0)
}
