use engine_asset::MeshBuildKey;
use engine_celestial::BodyDef;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::render_types::ScreenSpaceMetrics;
use engine_core::scene::TonemapOperator;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use engine_core::spatial::SpatialContext;
use engine_render_2d::{resolve_x, resolve_y, RenderArea};
use engine_render_3d::pipeline::{
    render_generated_world_sprite_with, GeneratedWorldRenderCallbacks, GeneratedWorldRenderProfile,
    GeneratedWorldSpriteSpec,
};
use engine_render_3d::raster::{
    blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba, obj_sprite_dimensions,
    render_obj_to_canvas, render_obj_to_rgba_canvas,
};
use engine_render_3d::scene::{select_lod_level_stable, Renderable3D};
use std::collections::HashMap;

use super::render::{compute_draw_pos, finalize_sprite, RenderCtx};

const DEFAULT_WORLD_CLOUD_COLOR: &str = "#eaf2f8";

fn resolved_ambient_floor(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .black_level
        .unwrap_or(0.06)
}

fn resolved_exposure(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .exposure
        .unwrap_or(1.0)
        .max(0.0)
}

fn resolved_gamma(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .gamma
        .unwrap_or(2.2)
        .clamp(0.1, 4.0)
}

fn resolved_tonemap(ctx: &RenderCtx<'_>) -> TonemapOperator {
    ctx.resolved_view_profile
        .lighting
        .tonemap
        .unwrap_or(TonemapOperator::Linear)
}

fn resolved_shadow_contrast(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .shadow_contrast
        .unwrap_or(1.0)
        .clamp(0.25, 4.0)
}

fn resolved_night_glow_scale(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .night_glow_scale
        .unwrap_or(1.0)
        .clamp(0.0, 2.0)
}

fn resolved_haze_night_leak(ctx: &RenderCtx<'_>) -> f32 {
    ctx.resolved_view_profile
        .lighting
        .haze_night_leak
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_generated_world_sprite(
    mut spec: GeneratedWorldSpriteSpec<'_>,
    area: RenderArea,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let GeneratedWorldSpriteSpec {
        sprite,
        node,
        size,
        width,
        height,
        observer_altitude_km,
        align_x,
        align_y,
        ..
    } = spec.clone();
    let Renderable3D::GeneratedWorld(generated_world) = node.renderable else {
        return;
    };

    let Some(catalogs) = ctx.celestial_catalogs else {
        return;
    };
    let Some(body) = catalogs.bodies.get(generated_world.body_id.as_str()) else {
        return;
    };
    let preset_id = generated_world
        .preset_id
        .as_deref()
        .or(body.planet_type.as_deref());
    let Some(planet) = preset_id.and_then(|id| catalogs.planet_types.get(id)) else {
        return;
    };

    let profile = build_generated_world_profile(
        body,
        planet,
        node.transform.scale[0],
        observer_altitude_km.unwrap_or(0.0),
        ctx.spatial_context,
        resolved_ambient_floor(ctx),
        resolved_shadow_contrast(ctx),
        resolved_exposure(ctx),
        resolved_gamma(ctx),
        resolved_tonemap(ctx),
        resolved_night_glow_scale(ctx),
        resolved_haze_night_leak(ctx),
    );

    let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
        obj_sprite_dimensions(width, height, size)
    } else {
        (area.width.max(1), area.height.max(1))
    };
    let selected_lod = select_lod_level_stable(
        node.id.as_str(),
        node.lod_hint.as_ref(),
        ScreenSpaceMetrics {
            projected_radius_px: (sprite_width.min(sprite_height) as f32) * 0.5,
            viewport_area_px: sprite_width as u32 * sprite_height as u32,
        },
    );
    if let Renderable3D::GeneratedWorld(world) = &mut spec.node.renderable {
        let effective_source = apply_world_lod_to_source(world.mesh_key.as_str(), selected_lod);
        world.mesh_key = MeshBuildKey::from_source(effective_source);
    }
    let base_x = area.origin_x
        + resolve_x(
            node.transform.translation[0].round() as i32,
            &align_x,
            area.width,
            sprite_width,
        );
    let base_y = area.origin_y
        + resolve_y(
            node.transform.translation[1].round() as i32,
            &align_y,
            area.height,
            sprite_height,
        );
    let (draw_x, draw_y) = compute_draw_pos(
        base_x,
        base_y,
        sprite.animations(),
        sprite_elapsed,
        object_state,
    );

    let rendered = render_generated_world_sprite_with(
        spec,
        &profile,
        sprite_width,
        sprite_height,
        draw_x,
        draw_y,
        sprite_elapsed,
        ctx.scene_camera_3d,
        ctx.asset_root,
        ctx.layer_buf,
        GeneratedWorldRenderCallbacks {
            render_obj_to_canvas,
            render_obj_to_rgba_canvas,
            convert_canvas_to_rgba,
            composite_rgba_over,
            blit_rgba_canvas,
        },
    );
    if !rendered {
        return;
    }

    let sprite_region = Region {
        x: draw_x,
        y: draw_y,
        width: sprite_width,
        height: sprite_height,
    };
    finalize_sprite(
        object_id,
        sprite_region,
        sprite_elapsed,
        sprite.stages(),
        ctx,
        target_resolver,
        object_regions,
    );
}

fn apply_world_lod_to_source(
    source: &str,
    lod_level: engine_core::render_types::LodLevel,
) -> String {
    if !source.starts_with("world://") {
        return source.to_string();
    }
    engine_worldgen::apply_world_lod_to_uri(source, lod_level.0)
}

fn build_generated_world_profile(
    body: &BodyDef,
    planet: &engine_celestial::PlanetDef,
    surface_scale: f32,
    observer_altitude_km: f32,
    spatial_context: SpatialContext,
    ambient_floor: f32,
    shadow_contrast: f32,
    exposure: f32,
    gamma: f32,
    tonemap: TonemapOperator,
    night_glow_scale: f32,
    haze_night_leak: f32,
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
        ambient_floor,
        shadow_contrast,
        exposure,
        gamma,
        tonemap,
        night_glow_scale,
        haze_night_leak,
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

fn generated_world_atmosphere_visibility(
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
