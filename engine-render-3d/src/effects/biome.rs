use super::noise::value_noise_3d;

/// Smoothstep cubic interpolation: maps [edge0, edge1] -> [0,1].
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Desert latitude weighting: peaks in subtropics, fades toward equator/poles.
#[inline]
pub fn desert_equatorial_weight(lat_abs: f32) -> f32 {
    (1.0 - ((lat_abs - 0.28) * 3.5).abs()).clamp(0.0, 1.0)
}

/// Convert noise + latitude weight into desert blend mask.
#[inline]
pub fn desert_noise_mask(noise: f32, lat_abs: f32, strength: f32) -> f32 {
    smoothstep(0.62, 0.82, noise) * desert_equatorial_weight(lat_abs) * strength.max(0.0)
}

/// Polar ice blend mask.
#[inline]
pub fn polar_ice_mask(lat_abs: f32, start: f32, end: f32) -> f32 {
    smoothstep(start, end, lat_abs)
}

/// Night-side weighting from sun-facing dot product (1.0 on dark side).
#[inline]
pub fn night_side_factor(sun_dot: f32) -> f32 {
    smoothstep(0.10, -0.08, sun_dot)
}

/// City light blend mask.
#[inline]
pub fn city_light_mask(noise: f32, threshold: f32, night: f32, intensity: f32) -> f32 {
    smoothstep(threshold, 1.0, noise) * night * intensity.max(0.0)
}

#[derive(Debug, Clone, Copy)]
pub struct LandBiomeSignals {
    pub normal: [f32; 3],
    pub lat_abs: f32,
    pub desert_mask: f32,
    pub ice_mask: f32,
    pub city_mask: f32,
}

/// Normalized surface normal from world/view-space position.
#[inline]
pub fn surface_normal(view_pos: [f32; 3]) -> [f32; 3] {
    let len = (view_pos[0] * view_pos[0] + view_pos[1] * view_pos[1] + view_pos[2] * view_pos[2])
        .sqrt()
        .max(1e-6);
    [view_pos[0] / len, view_pos[1] / len, view_pos[2] / len]
}

/// Absolute latitude from a normalized surface normal.
#[inline]
pub fn lat_abs_from_normal(normal: [f32; 3]) -> f32 {
    normal[1].abs()
}

/// Desert blend mask computed from local-space noise and latitude.
#[inline]
pub fn desert_mask_from_local(local_pos: [f32; 3], lat_abs: f32, strength: f32) -> f32 {
    let noise = value_noise_3d(local_pos[0] * 7.0, local_pos[1] * 7.0, local_pos[2] * 7.0);
    desert_noise_mask(noise, lat_abs, strength)
}

/// Polar ice mask for land: latitude with elevation bias.
#[inline]
pub fn polar_ice_mask_land(
    lat_abs: f32,
    noise: f32,
    terrain_threshold: f32,
    start: f32,
    end: f32,
) -> f32 {
    let elev_boost = (noise - terrain_threshold) * 0.15;
    polar_ice_mask(lat_abs + elev_boost, start, end)
}

/// Polar ice mask for ocean (slightly tighter threshold than land).
#[inline]
pub fn polar_ice_mask_ocean(lat_abs: f32, start: f32, end: f32) -> f32 {
    polar_ice_mask(lat_abs, start + 0.05, end)
}

/// Polar ice mask for ocean from interpolated sphere/view position.
#[inline]
pub fn polar_ice_mask_ocean_from_view(view_pos: [f32; 3], start: f32, end: f32) -> f32 {
    let lat_abs = lat_abs_from_normal(surface_normal(view_pos));
    polar_ice_mask_ocean(lat_abs, start, end)
}

/// Night-side city light mask using local-space detail noise.
#[inline]
pub fn city_light_mask_from_local(
    local_pos: [f32; 3],
    normal: [f32; 3],
    sun_dir: [f32; 3],
    threshold: f32,
    intensity: f32,
) -> f32 {
    let sun_dot = normal[0] * sun_dir[0] + normal[1] * sun_dir[1] + normal[2] * sun_dir[2];
    let night = night_side_factor(sun_dot);
    if night <= 0.01 || intensity <= 0.0 {
        return 0.0;
    }
    let noise = value_noise_3d(
        local_pos[0] * 18.0,
        local_pos[1] * 18.0,
        local_pos[2] * 18.0,
    );
    city_light_mask(noise, threshold, night, intensity)
}

/// Combined land-biome signals for a single surface sample.
pub fn land_biome_signals(
    local_pos: [f32; 3],
    view_pos: [f32; 3],
    noise: f32,
    terrain_threshold: f32,
    desert_strength: f32,
    ice_start: f32,
    ice_end: f32,
    city_threshold: f32,
    city_intensity: f32,
    sun_dir: [f32; 3],
) -> LandBiomeSignals {
    let normal = surface_normal(view_pos);
    let lat_abs = lat_abs_from_normal(normal);
    let desert_mask = if desert_strength > 0.0 {
        desert_mask_from_local(local_pos, lat_abs, desert_strength)
    } else {
        0.0
    };
    let ice_mask = polar_ice_mask_land(lat_abs, noise, terrain_threshold, ice_start, ice_end);
    let city_mask =
        city_light_mask_from_local(local_pos, normal, sun_dir, city_threshold, city_intensity);
    LandBiomeSignals {
        normal,
        lat_abs,
        desert_mask,
        ice_mask,
        city_mask,
    }
}
