use crate::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use crate::ObjRenderParams;

#[inline]
pub(crate) fn build_biome_params(
    params: &ObjRenderParams,
    light_dir_norm: [f32; 3],
    view_dir: [f32; 3],
) -> Option<PlanetBiomeParams> {
    let has_biome = params.polar_ice_color.is_some()
        || params.desert_color.is_some()
        || params.atmo_color.is_some()
        || params.atmo_rayleigh_color.is_some()
        || params.atmo_haze_color.is_some()
        || params.atmo_absorption_color.is_some()
        || params.night_light_color.is_some();
    if !has_biome {
        return None;
    }
    let sun_intensity = (params.light_direction_x * params.light_direction_x
        + params.light_direction_y * params.light_direction_y
        + params.light_direction_z * params.light_direction_z)
        .sqrt()
        .clamp(0.0, 4.0);
    Some(PlanetBiomeParams {
        polar_ice_color: params.polar_ice_color,
        polar_ice_start: params.polar_ice_start,
        polar_ice_end: params.polar_ice_end,
        desert_color: params.desert_color,
        desert_strength: params.desert_strength,
        atmo_color: params.atmo_color,
        atmo_height: params.atmo_height,
        atmo_density: params.atmo_density,
        atmo_strength: params.atmo_strength,
        atmo_rayleigh_amount: params.atmo_rayleigh_amount,
        atmo_rayleigh_color: params.atmo_rayleigh_color,
        atmo_rayleigh_falloff: params.atmo_rayleigh_falloff,
        atmo_haze_amount: params.atmo_haze_amount,
        atmo_haze_color: params.atmo_haze_color,
        atmo_haze_falloff: params.atmo_haze_falloff,
        atmo_absorption_amount: params.atmo_absorption_amount,
        atmo_absorption_color: params.atmo_absorption_color,
        atmo_absorption_height: params.atmo_absorption_height,
        atmo_absorption_width: params.atmo_absorption_width,
        atmo_forward_scatter: params.atmo_forward_scatter,
        atmo_limb_boost: params.atmo_limb_boost,
        atmo_terminator_softness: params.atmo_terminator_softness,
        atmo_night_glow: params.atmo_night_glow,
        atmo_night_glow_color: params.atmo_night_glow_color,
        atmo_haze_night_leak: params.atmo_haze_night_leak,
        atmo_rim_power: params.atmo_rim_power,
        atmo_haze_strength: params.atmo_haze_strength,
        atmo_haze_power: params.atmo_haze_power,
        atmo_veil_strength: params.atmo_veil_strength,
        atmo_veil_power: params.atmo_veil_power,
        night_light_color: params.night_light_color,
        night_light_threshold: params.night_light_threshold,
        night_light_intensity: params.night_light_intensity,
        sun_dir: light_dir_norm,
        sun_intensity,
        view_dir,
        camera_pos: [
            params.camera_world_x,
            params.camera_world_y,
            params.camera_world_z,
        ],
    })
}

#[inline]
pub(crate) fn build_terrain_extra_params(params: &ObjRenderParams) -> Option<PlanetTerrainParams> {
    if params.terrain_color.is_none()
        || (params.normal_perturb_strength <= 0.0
            && params.ocean_specular <= 0.0
            && params.crater_density <= 0.0
            && params.snow_line_altitude <= 0.0
            && params.ocean_noise_scale == 4.0
            && params.ocean_color_rgb.is_none())
    {
        return None;
    }
    Some(PlanetTerrainParams {
        noise_scale: params.terrain_noise_scale,
        normal_perturb: params.normal_perturb_strength,
        ocean_specular: params.ocean_specular,
        crater_density: params.crater_density,
        crater_rim_height: params.crater_rim_height,
        snow_line: params.snow_line_altitude,
        ocean_noise_scale: params.ocean_noise_scale,
        ocean_color_override: params.ocean_color_rgb,
    })
}
