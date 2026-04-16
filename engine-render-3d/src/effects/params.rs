/// Optional planet biome and atmosphere parameters for Gouraud rasterization.
#[derive(Clone, Copy)]
pub struct PlanetBiomeParams {
    pub polar_ice_color: Option<[u8; 3]>,
    pub polar_ice_start: f32,
    pub polar_ice_end: f32,
    pub desert_color: Option<[u8; 3]>,
    pub desert_strength: f32,
    pub atmo_color: Option<[u8; 3]>,
    pub atmo_strength: f32,
    pub atmo_rim_power: f32,
    pub atmo_haze_strength: f32,
    pub atmo_haze_power: f32,
    pub atmo_veil_strength: f32,
    pub atmo_veil_power: f32,
    pub night_light_color: Option<[u8; 3]>,
    pub night_light_threshold: f32,
    pub night_light_intensity: f32,
    pub sun_dir: [f32; 3],
    pub view_dir: [f32; 3],
    pub camera_pos: [f32; 3],
}

/// Extra per-pixel terrain rendering parameters for Gouraud rasterization.
#[derive(Clone, Copy)]
pub struct PlanetTerrainParams {
    pub noise_scale: f32,
    pub normal_perturb: f32,
    pub ocean_specular: f32,
    pub crater_density: f32,
    pub crater_rim_height: f32,
    pub snow_line: f32,
    pub ocean_noise_scale: f32,
    pub ocean_color_override: Option<[u8; 3]>,
}
