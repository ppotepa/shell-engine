use super::{GeneratedWorldRenderProfile, ViewLightingParams};
use engine_celestial::{BodyDef, PlanetDef};
use engine_core::color::Color;
use engine_core::spatial::SpatialContext;

const DEFAULT_WORLD_CLOUD_COLOR: &str = "#eaf2f8";

pub fn build_generated_world_render_profile(
    body: &BodyDef,
    planet: &PlanetDef,
    surface_scale: f32,
    observer_altitude_km: f32,
    spatial_context: SpatialContext,
    view_lighting: ViewLightingParams,
) -> GeneratedWorldRenderProfile {
    let sun_dir = [
        planet.sun_dir_x as f32,
        planet.sun_dir_y as f32,
        planet.sun_dir_z as f32,
    ];
    let (cloud_scale, cloud2_scale) =
        generated_world_cloud_scales(body, surface_scale, spatial_context);
    let atmo_visibility =
        generated_world_atmosphere_visibility(body, observer_altitude_km, spatial_context);

    GeneratedWorldRenderProfile {
        ambient: planet.ambient as f32,
        ambient_floor: view_lighting.ambient_floor,
        shadow_contrast: view_lighting.shadow_contrast,
        exposure: view_lighting.exposure,
        gamma: view_lighting.gamma,
        tonemap: view_lighting.tonemap,
        night_glow_scale: view_lighting.night_glow_scale,
        haze_night_leak: view_lighting.haze_night_leak,
        latitude_bands: planet.latitude_bands,
        latitude_band_depth: planet.latitude_band_depth as f32,
        terrain_displacement: planet.terrain_displacement as f32,
        terrain_color: colour_rgb(Some(planet.land_color.as_str())),
        terrain_threshold: planet.terrain_threshold as f32,
        terrain_noise_scale: planet.terrain_noise_scale as f32,
        terrain_noise_octaves: planet.terrain_noise_octaves,
        marble_depth: planet.marble_depth as f32,
        terrain_relief: planet.terrain_relief as f32,
        polar_ice_color: planet
            .polar_ice_color
            .as_deref()
            .and_then(|value| colour_rgb(Some(value))),
        polar_ice_start: planet.polar_ice_start as f32,
        polar_ice_end: planet.polar_ice_end as f32,
        desert_color: planet
            .desert_color
            .as_deref()
            .and_then(|value| colour_rgb(Some(value))),
        desert_strength: planet.desert_strength as f32,
        atmo_strength: planet.atmo_strength as f32,
        atmo_color: planet
            .atmo_color
            .as_deref()
            .and_then(|value| colour_rgb(Some(value))),
        night_light_color: planet
            .night_light_color
            .as_deref()
            .and_then(|value| colour_rgb(Some(value))),
        night_light_threshold: planet.night_light_threshold as f32,
        night_light_intensity: planet.night_light_intensity as f32,
        shadow_color: planet
            .shadow_color
            .as_deref()
            .map(|s| colour_value(Some(s), Color::Black)),
        midtone_color: planet
            .midtone_color
            .as_deref()
            .map(|s| colour_value(Some(s), Color::White)),
        highlight_color: planet
            .highlight_color
            .as_deref()
            .map(|s| colour_value(Some(s), Color::White)),
        tone_mix: planet.tone_mix as f32,
        cel_levels: planet.cel_levels,
        noise_seed: planet.noise_seed as f32,
        generated_heightmap: planet.generated_heightmap.clone(),
        generated_heightmap_w: planet.generated_heightmap_w,
        generated_heightmap_h: planet.generated_heightmap_h,
        heightmap_blend: planet.heightmap_blend as f32,
        warp_strength: planet.warp_strength as f32,
        warp_octaves: planet.warp_octaves,
        noise_lacunarity: planet.noise_lacunarity as f32,
        noise_persistence: planet.noise_persistence as f32,
        normal_perturb_strength: planet.normal_perturb_strength as f32,
        ocean_specular: planet.ocean_specular as f32,
        ocean_noise_scale: planet.ocean_noise_scale as f32,
        crater_density: planet.crater_density as f32,
        crater_rim_height: planet.crater_rim_height as f32,
        snow_line_altitude: planet.snow_line_altitude as f32,
        ocean_color: colour_value(Some(planet.ocean_color.as_str()), Color::White),
        cloud_color: colour_value(
            Some(
                planet
                    .cloud_color
                    .as_deref()
                    .unwrap_or(DEFAULT_WORLD_CLOUD_COLOR),
            ),
            Color::White,
        ),
        cloud_threshold: planet.cloud_threshold as f32,
        cloud_ambient: planet.cloud_ambient as f32,
        cloud_noise_scale: planet.cloud_noise_scale as f32,
        cloud_noise_octaves: planet.cloud_noise_octaves,
        cloud_scale,
        cloud2_scale,
        cloud_render_scale_1: 0.58,
        cloud_render_scale_2: 0.42,
        atmo_visibility,
        sun_dir,
    }
}

fn colour_value(raw: Option<&str>, fallback: Color) -> Color {
    raw.and_then(engine_core::scene::color::parse_colour_str)
        .map(|value| Color::from(&value))
        .unwrap_or(fallback)
}

fn colour_rgb(raw: Option<&str>) -> Option<[u8; 3]> {
    let colour = raw.and_then(engine_core::scene::color::parse_colour_str)?;
    let (r, g, b) = Color::from(&colour).to_rgb();
    Some([r, g, b])
}

fn body_radius_km(body: &BodyDef, spatial_context: SpatialContext) -> Option<f32> {
    body.resolved_radius_km(Some(spatial_context.scale.meters_per_world_unit))
        .map(|value| value as f32)
}

fn generated_world_cloud_scales(
    body: &BodyDef,
    surface_scale: f32,
    spatial_context: SpatialContext,
) -> (f32, f32) {
    let Some(radius_km) =
        body_radius_km(body, spatial_context).filter(|value| *value > f32::EPSILON)
    else {
        return (surface_scale, surface_scale);
    };
    let cloud_bottom = body.cloud_bottom_km.unwrap_or(0.0) as f32;
    let cloud_top = body.cloud_top_km.unwrap_or(cloud_bottom as f64) as f32;
    let cloud_mid = ((cloud_bottom + cloud_top) * 0.5).max(0.0);
    let cloud_high = (cloud_top + (cloud_top - cloud_bottom).max(6.0) * 1.5).max(cloud_mid);
    (
        surface_scale * (1.0 + cloud_mid / radius_km),
        surface_scale * (1.0 + cloud_high / radius_km),
    )
}

pub fn generated_world_atmosphere_visibility(
    body: &BodyDef,
    observer_altitude_km: f32,
    spatial_context: SpatialContext,
) -> f32 {
    let top_km = body
        .resolved_atmosphere_top_km(Some(spatial_context.scale.meters_per_world_unit))
        .unwrap_or(0.0) as f32;
    if top_km <= f32::EPSILON {
        return 1.0;
    }
    (1.0 - (observer_altitude_km / (top_km * 8.0)).clamp(0.0, 0.65)).clamp(0.35, 1.0)
}

#[cfg(test)]
mod tests {
    use super::generated_world_atmosphere_visibility;
    use engine_celestial::BodyDef;
    use engine_core::spatial::SpatialContext;

    #[test]
    fn km_per_world_unit_prefers_body_mapping_over_scene_spatial_scale() {
        let body = BodyDef {
            km_per_px: Some(42.0),
            ..BodyDef::default()
        };
        let mut spatial = SpatialContext::default();
        spatial.scale.meters_per_world_unit = 2000.0;
        let resolved = body
            .km_per_world_unit(Some(spatial.scale.meters_per_world_unit))
            .expect("km mapping");
        assert!((resolved - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn atmosphere_visibility_uses_scene_spatial_scale_without_body_km_fields() {
        let body = BodyDef {
            atmosphere_top: Some(10.0),
            ..BodyDef::default()
        };
        let default_visibility =
            generated_world_atmosphere_visibility(&body, 20.0, SpatialContext::default());
        assert!(
            (default_visibility - 1.0).abs() < f32::EPSILON,
            "without body km mapping or authored spatial scale, visibility should remain neutral"
        );

        let mut authored_spatial = SpatialContext::default();
        authored_spatial.scale.meters_per_world_unit = 2000.0;
        let scaled_visibility =
            generated_world_atmosphere_visibility(&body, 20.0, authored_spatial);
        assert!(
            scaled_visibility < 1.0,
            "authored scene spatial scale should produce finite atmosphere attenuation"
        );
    }
}
