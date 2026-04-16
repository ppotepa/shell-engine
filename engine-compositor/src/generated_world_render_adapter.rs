use crate::{
    blit_rgba_canvas, composite_rgba_over, convert_canvas_to_rgba, obj_sprite_dimensions,
    render_obj_to_canvas, render_obj_to_rgba_canvas, ObjRenderParams,
};
use engine_celestial::BodyDef;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene::CameraSource;
use engine_core::scene_runtime_types::{ObjectRuntimeState, SceneCamera3D, TargetResolver};
use engine_render_2d::{resolve_x, resolve_y, RenderArea};
use engine_render_3d::pipeline::GeneratedWorldSpriteSpec;
use engine_render_3d::scene::Renderable3D;
use std::collections::HashMap;

use super::render::{compute_draw_pos, finalize_sprite, RenderCtx};
const DEFAULT_WORLD_CLOUD_COLOR: &str = "#eaf2f8";
const DEFAULT_WORLD_CLOUD_2_COLOR: &str = "#d7e2ec";

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
        spin_deg,
        cloud_spin_deg,
        cloud2_spin_deg,
        observer_altitude_km,
        camera_distance,
        camera_source,
        fov_degrees,
        near_clip,
        sun_dir_x,
        sun_dir_y,
        sun_dir_z,
        align_x,
        align_y,
    } = spec;
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

    let use_scene_camera = camera_source == CameraSource::Scene;
    let scene_camera = ctx.scene_camera_3d;
    let sun_dir = [
        sun_dir_x.unwrap_or(planet.sun_dir_x as f32),
        sun_dir_y.unwrap_or(planet.sun_dir_y as f32),
        sun_dir_z.unwrap_or(planet.sun_dir_z as f32),
    ];
    let surface_scale = node.transform.scale[0];
    let (cloud_scale, cloud2_scale) = generated_world_cloud_scales(body, surface_scale);
    let mesh_path = generated_world.mesh_key.as_str();
    let base_yaw = node.transform.rotation_deg[1];
    let pitch = node.transform.rotation_deg[0];
    let roll = node.transform.rotation_deg[2];
    let camera_distance = camera_distance.unwrap_or(3.0);
    let fov_degrees = fov_degrees.unwrap_or(60.0);
    let near_clip = near_clip.unwrap_or(0.001);
    let atmo_visibility =
        generated_world_atmosphere_visibility(body, observer_altitude_km.unwrap_or(0.0));

    let mut surface_params = build_generated_world_base_params(
        surface_scale,
        base_yaw + spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    surface_params.ambient = planet.ambient as f32;
    surface_params.smooth_shading = true;
    surface_params.latitude_bands = planet.latitude_bands;
    surface_params.latitude_band_depth = planet.latitude_band_depth as f32;
    surface_params.terrain_displacement = planet.terrain_displacement as f32;
    surface_params.terrain_color = colour_rgb(Some(planet.land_color.as_str()));
    surface_params.terrain_threshold = planet.terrain_threshold as f32;
    surface_params.terrain_noise_scale = planet.terrain_noise_scale as f32;
    surface_params.terrain_noise_octaves = planet.terrain_noise_octaves;
    surface_params.marble_depth = planet.marble_depth as f32;
    surface_params.terrain_relief = planet.terrain_relief as f32;
    surface_params.polar_ice_color = planet
        .polar_ice_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.polar_ice_start = planet.polar_ice_start as f32;
    surface_params.polar_ice_end = planet.polar_ice_end as f32;
    surface_params.desert_color = planet
        .desert_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.desert_strength = planet.desert_strength as f32;
    surface_params.atmo_color = None;
    surface_params.atmo_height = 0.12;
    surface_params.atmo_density = (planet.atmo_strength as f32 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_strength = 0.0;
    surface_params.atmo_rayleigh_amount =
        (planet.atmo_strength as f32 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_rayleigh_color = planet
        .atmo_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.atmo_rayleigh_falloff = 0.32;
    surface_params.atmo_haze_amount =
        (planet.atmo_strength as f32 * 0.45 * atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_haze_color = surface_params.atmo_rayleigh_color;
    surface_params.atmo_haze_falloff = 0.18;
    surface_params.atmo_absorption_amount = 0.0;
    surface_params.atmo_absorption_color = None;
    surface_params.atmo_absorption_height = 0.55;
    surface_params.atmo_absorption_width = 0.18;
    surface_params.atmo_forward_scatter = 0.72;
    surface_params.atmo_limb_boost = 1.35;
    surface_params.atmo_terminator_softness = 1.05;
    surface_params.atmo_night_glow = 0.0;
    surface_params.atmo_night_glow_color = None;
    surface_params.atmo_rim_power = 4.5;
    surface_params.atmo_haze_strength = 0.0;
    surface_params.atmo_haze_power = 1.8;
    surface_params.atmo_veil_strength = 0.0;
    surface_params.atmo_veil_power = 1.6;
    surface_params.atmo_halo_strength = 0.0;
    surface_params.atmo_halo_width = 0.12;
    surface_params.atmo_halo_power = 2.2;
    surface_params.night_light_color = planet
        .night_light_color
        .as_deref()
        .and_then(|value| colour_rgb(Some(value)));
    surface_params.night_light_threshold = planet.night_light_threshold as f32;
    surface_params.night_light_intensity = planet.night_light_intensity as f32;
    // Wire tone palette — this is what gives the ocean its colour (shadow→midtone→highlight
    // maps the Lambertian shade range onto authored ocean colours). Without tone_mix > 0 the
    // ocean renders as shaded white/grey from the sphere mesh face colour.
    surface_params.shadow_colour = planet
        .shadow_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::Black));
    surface_params.midtone_colour = planet
        .midtone_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::White));
    surface_params.highlight_colour = planet
        .highlight_color
        .as_deref()
        .map(|s| colour_value(Some(s), Color::White));
    surface_params.tone_mix = planet.tone_mix as f32;
    surface_params.cel_levels = planet.cel_levels;
    surface_params.noise_seed = planet.noise_seed as f32;
    surface_params.heightmap = planet.generated_heightmap.clone();
    surface_params.heightmap_w = planet.generated_heightmap_w;
    surface_params.heightmap_h = planet.generated_heightmap_h;
    surface_params.heightmap_blend = planet.heightmap_blend as f32;
    surface_params.warp_strength = planet.warp_strength as f32;
    surface_params.warp_octaves = planet.warp_octaves;
    surface_params.noise_lacunarity = planet.noise_lacunarity as f32;
    surface_params.noise_persistence = planet.noise_persistence as f32;
    surface_params.normal_perturb_strength = planet.normal_perturb_strength as f32;
    surface_params.ocean_specular = planet.ocean_specular as f32;
    surface_params.ocean_noise_scale = planet.ocean_noise_scale as f32;
    surface_params.crater_density = planet.crater_density as f32;
    surface_params.crater_rim_height = planet.crater_rim_height as f32;
    surface_params.snow_line_altitude = planet.snow_line_altitude as f32;

    // ── RGBA compositing pipeline: surface → cloud1 → cloud2 → blit ──────────
    let ocean_fg = colour_value(Some(planet.ocean_color.as_str()), Color::White);
    let (ocean_r, ocean_g, ocean_b) = ocean_fg.to_rgb();
    surface_params.ocean_color_rgb = Some([ocean_r, ocean_g, ocean_b]);

    // 1. Render surface (opaque) to RGB canvas, then convert to RGBA.
    let Some((surface_rgb, virtual_w, virtual_h)) = render_obj_to_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        surface_params,
        false,
        false,
        ocean_fg,
        ctx.asset_root,
    ) else {
        return;
    };
    let mut composited = convert_canvas_to_rgba(surface_rgb);

    // 2. Cloud layer 1 — soft alpha edges, per-pixel noise.
    let cloud_colour = planet
        .cloud_color
        .as_deref()
        .unwrap_or(DEFAULT_WORLD_CLOUD_COLOR);
    let cloud_rgb = colour_rgb(Some(cloud_colour));
    let cloud_threshold = (planet.cloud_threshold as f32).clamp(0.0, 0.999);
    let mut cloud_params = build_generated_world_base_params(
        cloud_scale,
        base_yaw + cloud_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    cloud_params.ambient = planet.cloud_ambient as f32;
    cloud_params.smooth_shading = true;
    cloud_params.terrain_color = cloud_rgb;
    cloud_params.terrain_threshold = cloud_threshold;
    cloud_params.terrain_noise_scale = planet.cloud_noise_scale as f32;
    cloud_params.terrain_noise_octaves = planet.cloud_noise_octaves.max(1);
    cloud_params.marble_depth = (planet.marble_depth as f32 * 0.5).max(0.003);
    cloud_params.below_threshold_transparent = true;
    cloud_params.cloud_alpha_softness = 0.12;

    if let Some((cloud1_rgba, _, _)) = render_obj_to_rgba_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        cloud_params,
        false,
        colour_value(Some(cloud_colour), Color::White),
        ctx.asset_root,
    ) {
        composite_rgba_over(&mut composited, &cloud1_rgba);
    }

    // 3. Cloud layer 2 — sparse high-altitude breakup.
    let cloud2_colour = DEFAULT_WORLD_CLOUD_2_COLOR;
    let mut cloud2_params = build_generated_world_base_params(
        cloud2_scale,
        base_yaw + 180.0 + cloud2_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera,
        sun_dir,
    );
    cloud2_params.ambient = 0.004;
    cloud2_params.smooth_shading = true;
    cloud2_params.terrain_color = colour_rgb(Some(cloud2_colour));
    cloud2_params.terrain_threshold = (cloud_threshold + 0.12).min(0.992);
    cloud2_params.terrain_noise_scale = (planet.cloud_noise_scale as f32 * 0.35).max(1.1);
    cloud2_params.terrain_noise_octaves = planet.cloud_noise_octaves.clamp(1, 2);
    cloud2_params.marble_depth = (planet.marble_depth as f32 * 0.2).max(0.002);
    cloud2_params.below_threshold_transparent = true;
    cloud2_params.cloud_alpha_softness = 0.08;

    if let Some((cloud2_rgba, _, _)) = render_obj_to_rgba_canvas(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        cloud2_params,
        false,
        colour_value(Some(cloud2_colour), Color::White),
        ctx.asset_root,
    ) {
        composite_rgba_over(&mut composited, &cloud2_rgba);
    }

    // 4. Blit composited RGBA canvas to buffer.
    let (target_w, _target_h) =
        obj_sprite_dimensions(Some(sprite_width), Some(sprite_height), size);
    blit_rgba_canvas(
        ctx.layer_buf,
        &composited,
        virtual_w,
        virtual_h,
        target_w,
        sprite_height,
        draw_x,
        draw_y,
    );

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

fn build_generated_world_base_params(
    scale: f32,
    yaw_deg: f32,
    pitch_deg: f32,
    roll_deg: f32,
    camera_distance: f32,
    fov_degrees: f32,
    near_clip: f32,
    scene_elapsed_ms: u64,
    use_scene_camera: bool,
    scene_camera: &SceneCamera3D,
    sun_dir: [f32; 3],
) -> ObjRenderParams {
    ObjRenderParams {
        scale,
        yaw_deg,
        pitch_deg,
        roll_deg,
        rotation_x: 0.0,
        rotation_y: 0.0,
        rotation_z: 0.0,
        rotate_y_deg_per_sec: 0.0,
        camera_distance,
        fov_degrees,
        near_clip,
        light_direction_x: sun_dir[0],
        light_direction_y: sun_dir[1],
        light_direction_z: sun_dir[2],
        light_2_direction_x: 0.0,
        light_2_direction_y: 0.0,
        light_2_direction_z: -1.0,
        light_2_intensity: 0.0,
        light_point_x: 0.0,
        light_point_y: 2.0,
        light_point_z: 0.0,
        light_point_intensity: 0.0,
        light_point_colour: None,
        light_point_flicker_depth: 0.0,
        light_point_flicker_hz: 0.0,
        light_point_orbit_hz: 0.0,
        light_point_snap_hz: 0.0,
        light_point_2_x: 0.0,
        light_point_2_y: 0.0,
        light_point_2_z: 0.0,
        light_point_2_intensity: 0.0,
        light_point_2_colour: None,
        light_point_2_flicker_depth: 0.0,
        light_point_2_flicker_hz: 0.0,
        light_point_2_orbit_hz: 0.0,
        light_point_2_snap_hz: 0.0,
        cel_levels: 0,
        shadow_colour: None,
        midtone_colour: None,
        highlight_colour: None,
        tone_mix: 0.0,
        scene_elapsed_ms,
        camera_pan_x: 0.0,
        camera_pan_y: 0.0,
        camera_look_yaw: 0.0,
        camera_look_pitch: 0.0,
        object_translate_x: 0.0,
        object_translate_y: 0.0,
        object_translate_z: 0.0,
        clip_y_min: 0.0,
        clip_y_max: 1.0,
        camera_world_x: if use_scene_camera {
            scene_camera.eye[0]
        } else {
            0.0
        },
        camera_world_y: if use_scene_camera {
            scene_camera.eye[1]
        } else {
            0.0
        },
        camera_world_z: if use_scene_camera {
            scene_camera.eye[2]
        } else {
            -camera_distance
        },
        view_right_x: if use_scene_camera {
            scene_camera.right()[0]
        } else {
            1.0
        },
        view_right_y: if use_scene_camera {
            scene_camera.right()[1]
        } else {
            0.0
        },
        view_right_z: if use_scene_camera {
            scene_camera.right()[2]
        } else {
            0.0
        },
        view_up_x: if use_scene_camera {
            scene_camera.up[0]
        } else {
            0.0
        },
        view_up_y: if use_scene_camera {
            scene_camera.up[1]
        } else {
            1.0
        },
        view_up_z: if use_scene_camera {
            scene_camera.up[2]
        } else {
            0.0
        },
        view_forward_x: if use_scene_camera {
            scene_camera.forward()[0]
        } else {
            0.0
        },
        view_forward_y: if use_scene_camera {
            scene_camera.forward()[1]
        } else {
            0.0
        },
        view_forward_z: if use_scene_camera {
            scene_camera.forward()[2]
        } else {
            1.0
        },
        unlit: false,
        ambient: 0.05,
        light_point_falloff: 0.7,
        light_point_2_falloff: 0.7,
        smooth_shading: true,
        latitude_bands: 0,
        latitude_band_depth: 0.0,
        terrain_displacement: 0.0,
        terrain_color: None,
        terrain_threshold: 0.5,
        terrain_noise_scale: 2.5,
        terrain_noise_octaves: 2,
        marble_depth: 0.0,
        terrain_relief: 0.0,
        noise_seed: 0.0,
        warp_strength: 0.0,
        warp_octaves: 2,
        noise_lacunarity: 2.0,
        noise_persistence: 0.5,
        normal_perturb_strength: 0.0,
        ocean_specular: 0.0,
        crater_density: 0.0,
        crater_rim_height: 0.35,
        snow_line_altitude: 0.0,
        below_threshold_transparent: false,
        cloud_alpha_softness: 0.0,
        polar_ice_color: None,
        polar_ice_start: 0.78,
        polar_ice_end: 0.92,
        desert_color: None,
        desert_strength: 0.0,
        atmo_color: None,
        atmo_height: 0.12,
        atmo_density: 0.0,
        atmo_strength: 0.0,
        atmo_rayleigh_amount: 0.0,
        atmo_rayleigh_color: None,
        atmo_rayleigh_falloff: 0.32,
        atmo_haze_amount: 0.0,
        atmo_haze_color: None,
        atmo_haze_falloff: 0.18,
        atmo_absorption_amount: 0.0,
        atmo_absorption_color: None,
        atmo_absorption_height: 0.55,
        atmo_absorption_width: 0.18,
        atmo_forward_scatter: 0.72,
        atmo_limb_boost: 1.0,
        atmo_terminator_softness: 1.0,
        atmo_night_glow: 0.0,
        atmo_night_glow_color: None,
        atmo_rim_power: 4.5,
        atmo_haze_strength: 0.0,
        atmo_haze_power: 1.8,
        atmo_veil_strength: 0.0,
        atmo_veil_power: 1.6,
        atmo_halo_strength: 0.0,
        atmo_halo_width: 0.12,
        atmo_halo_power: 2.2,
        ocean_noise_scale: 4.0,
        ocean_color_rgb: None,
        night_light_color: None,
        night_light_threshold: 0.82,
        night_light_intensity: 0.0,
        heightmap: None,
        heightmap_w: 0,
        heightmap_h: 0,
        heightmap_blend: 0.0,
        depth_sort_faces: false,
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
