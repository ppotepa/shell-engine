use engine_render_3d::raster::{
    blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba, obj_sprite_dimensions,
    render_obj_to_canvas, render_obj_to_rgba_canvas,
};
use engine_celestial::BodyDef;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use engine_render_2d::{resolve_x, resolve_y, RenderArea};
use engine_render_3d::pipeline::{
    render_generated_world_sprite_with, GeneratedWorldRenderCallbacks, GeneratedWorldRenderProfile,
    GeneratedWorldSpriteSpec,
};
use engine_render_3d::scene::Renderable3D;
use std::collections::HashMap;

use super::render::{compute_draw_pos, finalize_sprite, RenderCtx};

const DEFAULT_WORLD_CLOUD_COLOR: &str = "#eaf2f8";

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_generated_world_sprite(
    spec: GeneratedWorldSpriteSpec<'_>,
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
    );

    let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
        obj_sprite_dimensions(width, height, size)
    } else {
        (area.width.max(1), area.height.max(1))
    };
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

fn build_generated_world_profile(
    body: &BodyDef,
    planet: &engine_celestial::PlanetDef,
    surface_scale: f32,
    observer_altitude_km: f32,
) -> GeneratedWorldRenderProfile {
    let sun_dir = [
        planet.sun_dir_x as f32,
        planet.sun_dir_y as f32,
        planet.sun_dir_z as f32,
    ];
    let (cloud_scale, cloud2_scale) = generated_world_cloud_scales(body, surface_scale);
    let atmo_visibility = generated_world_atmosphere_visibility(body, observer_altitude_km);

    GeneratedWorldRenderProfile {
        ambient: planet.ambient as f32,
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

fn body_radius_km(body: &BodyDef) -> Option<f32> {
    body.radius_km.map(|value| value as f32).or_else(|| {
        body.km_per_px
            .map(|km_per_px| (body.radius_px * km_per_px) as f32)
    })
}

fn generated_world_cloud_scales(body: &BodyDef, surface_scale: f32) -> (f32, f32) {
    let Some(radius_km) = body_radius_km(body).filter(|value| *value > f32::EPSILON) else {
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

fn generated_world_atmosphere_visibility(body: &BodyDef, observer_altitude_km: f32) -> f32 {
    let top_km = body
        .atmosphere_top_km
        .map(|value| value as f32)
        .or_else(|| {
            body.atmosphere_top
                .zip(body.km_per_px)
                .map(|(top_px, km_per_px)| (top_px * km_per_px) as f32)
        })
        .unwrap_or(0.0);
    if top_km <= f32::EPSILON {
        return 1.0;
    }
    (1.0 - (observer_altitude_km / (top_km * 8.0)).clamp(0.0, 0.65)).clamp(0.35, 1.0)
}
