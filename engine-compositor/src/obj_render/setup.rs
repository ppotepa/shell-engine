use super::ObjRenderParams;
use crate::obj_render_helpers::{normalize3, PlanetBiomeParams, PlanetTerrainParams};

#[inline]
pub(super) fn normalized_light_and_view_dirs(
    params: &ObjRenderParams,
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let light_dir_norm = normalize3([
        params.light_direction_x,
        params.light_direction_y,
        params.light_direction_z,
    ]);
    let light_2_dir_norm = normalize3([
        params.light_2_direction_x,
        params.light_2_direction_y,
        params.light_2_direction_z,
    ]);
    let view_dir = normalize3([
        -params.view_forward_x,
        -params.view_forward_y,
        -params.view_forward_z,
    ]);
    (light_dir_norm, light_2_dir_norm, view_dir)
}

#[inline]
pub(super) fn build_biome_params(
    params: &ObjRenderParams,
    light_dir_norm: [f32; 3],
    view_dir: [f32; 3],
) -> Option<PlanetBiomeParams> {
    let has_biome = params.polar_ice_color.is_some()
        || params.desert_color.is_some()
        || params.atmo_color.is_some()
        || params.night_light_color.is_some();
    if !has_biome {
        return None;
    }
    Some(PlanetBiomeParams {
        polar_ice_color: params.polar_ice_color,
        polar_ice_start: params.polar_ice_start,
        polar_ice_end: params.polar_ice_end,
        desert_color: params.desert_color,
        desert_strength: params.desert_strength,
        atmo_color: params.atmo_color,
        atmo_strength: params.atmo_strength,
        atmo_rim_power: params.atmo_rim_power,
        atmo_haze_strength: params.atmo_haze_strength,
        atmo_haze_power: params.atmo_haze_power,
        atmo_veil_strength: params.atmo_veil_strength,
        atmo_veil_power: params.atmo_veil_power,
        night_light_color: params.night_light_color,
        night_light_threshold: params.night_light_threshold,
        night_light_intensity: params.night_light_intensity,
        sun_dir: light_dir_norm,
        view_dir,
        camera_pos: [
            params.camera_world_x,
            params.camera_world_y,
            params.camera_world_z,
        ],
    })
}

#[inline]
pub(super) fn build_terrain_extra_params(params: &ObjRenderParams) -> Option<PlanetTerrainParams> {
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
