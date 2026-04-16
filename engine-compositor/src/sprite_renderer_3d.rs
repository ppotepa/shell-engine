use crate::obj_render::parse_terrain_params_from_uri;
use crate::{
    obj_sprite_dimensions, render_obj_content, try_blit_prerendered, ObjRenderParams,
};
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::scene::{CameraSource, Sprite};
use engine_core::scene_runtime_types::{ObjectRuntimeState, TargetResolver};
use engine_render_3d::pipeline::map_sprite_to_node3d;
use engine_render_3d::scene::Renderable3D;
use engine_render_2d::{resolve_x, resolve_y, RenderArea};
use std::collections::HashMap;

use super::render::{compute_draw_pos, finalize_sprite, RenderCtx};
pub(crate) fn render_obj_sprite(
    sprite: &Sprite,
    area: RenderArea,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    _appear_at: u64,
    sprite_elapsed: u64,
    ctx: &mut RenderCtx<'_>,
) {
    let Some(node) = map_sprite_to_node3d(sprite) else {
        return;
    };
    let Renderable3D::Mesh(mesh_node) = &node.renderable else {
        return;
    };

    let Sprite::Obj {
        id,
        size,
        width,
        height,
        surface_mode,
        backface_cull,
        clip_y_min,
        clip_y_max,
        rotation_x,
        rotation_y,
        rotation_z,
        rotate_y_deg_per_sec,
        camera_distance,
        fov_degrees,
        near_clip,
        light_direction_x,
        light_direction_y,
        light_direction_z,
        light_2_direction_x,
        light_2_direction_y,
        light_2_direction_z,
        light_2_intensity,
        light_point_x,
        light_point_y,
        light_point_z,
        light_point_intensity,
        light_point_colour,
        light_point_flicker_depth,
        light_point_flicker_hz,
        light_point_orbit_hz,
        light_point_snap_hz,
        light_point_2_x,
        light_point_2_y,
        light_point_2_z,
        light_point_2_intensity,
        light_point_2_colour,
        light_point_2_flicker_depth,
        light_point_2_flicker_hz,
        light_point_2_orbit_hz,
        light_point_2_snap_hz,
        cel_levels,
        shadow_colour,
        midtone_colour,
        highlight_colour,
        tone_mix,
        smooth_shading,
        ambient,
        latitude_bands,
        latitude_band_depth,
        terrain_color,
        terrain_threshold,
        terrain_noise_scale,
        terrain_noise_octaves,
        marble_depth,
        below_threshold_transparent,
        polar_ice_color,
        polar_ice_start,
        polar_ice_end,
        desert_color,
        desert_strength,
        atmo_color: _atmo_color,
        atmo_height,
        atmo_density,
        atmo_strength: _atmo_strength,
        atmo_rayleigh_amount,
        atmo_rayleigh_color,
        atmo_rayleigh_falloff,
        atmo_haze_amount,
        atmo_haze_color,
        atmo_haze_falloff,
        atmo_absorption_amount,
        atmo_absorption_color,
        atmo_absorption_height,
        atmo_absorption_width,
        atmo_forward_scatter,
        atmo_limb_boost,
        atmo_terminator_softness,
        atmo_night_glow,
        atmo_night_glow_color,
        atmo_rim_power: _atmo_rim_power,
        atmo_haze_strength: _atmo_haze_strength,
        atmo_haze_power: _atmo_haze_power,
        atmo_veil_strength: _atmo_veil_strength,
        atmo_veil_power: _atmo_veil_power,
        atmo_halo_strength: _atmo_halo_strength,
        atmo_halo_width: _atmo_halo_width,
        atmo_halo_power: _atmo_halo_power,
        night_light_color,
        night_light_threshold,
        night_light_intensity,
        draw_char,
        align_x,
        align_y,
        fg_colour,
        bg_colour,
        world_x,
        world_y,
        world_z,
        cam_world_x,
        cam_world_y,
        cam_world_z,
        view_right_x,
        view_right_y,
        view_right_z,
        view_up_x,
        view_up_y,
        view_up_z,
        view_fwd_x,
        view_fwd_y,
        view_fwd_z,
        camera_source,
        terrain_plane_amplitude,
        terrain_plane_frequency,
        terrain_plane_roughness,
        terrain_plane_octaves,
        terrain_plane_seed_x,
        terrain_plane_seed_z,
        terrain_plane_lacunarity,
        terrain_plane_ridge,
        terrain_plane_plateau,
        terrain_plane_sea_level,
        terrain_plane_scale_x,
        terrain_plane_scale_z,
        world_gen_shape,
        world_gen_base,
        world_gen_coloring,
        world_gen_seed,
        world_gen_ocean_fraction,
        world_gen_continent_scale,
        world_gen_continent_warp,
        world_gen_continent_octaves,
        world_gen_mountain_scale,
        world_gen_mountain_strength,
        world_gen_mountain_ridge_octaves,
        world_gen_moisture_scale,
        world_gen_ice_cap_strength,
        world_gen_lapse_rate,
        world_gen_rain_shadow,
        world_gen_displacement_scale,
        world_gen_subdivisions,
        ..
    } = sprite
    else {
        return;
    };

    let source = mesh_node.source.as_str();
    let node_x = node.transform.translation[0].round() as i32;
    let node_y = node.transform.translation[1].round() as i32;
    let node_scale = node.transform.scale[0];
    let node_pitch = node.transform.rotation_deg[0];
    let node_yaw = node.transform.rotation_deg[1];
    let node_roll = node.transform.rotation_deg[2];

    let effective_source_buf: String;
    let effective_source: &str = if (source.starts_with("terrain-plane://") || source.starts_with("terrain-sphere://") || source.starts_with("earth-sphere://"))
        && (terrain_plane_amplitude.is_some()
            || terrain_plane_frequency.is_some()
            || terrain_plane_roughness.is_some()
            || terrain_plane_octaves.is_some()
            || terrain_plane_seed_x.is_some()
            || terrain_plane_seed_z.is_some()
            || terrain_plane_lacunarity.is_some()
            || terrain_plane_ridge.is_some()
            || terrain_plane_plateau.is_some()
            || terrain_plane_sea_level.is_some()
            || terrain_plane_scale_x.is_some()
            || terrain_plane_scale_z.is_some())
    {
        let scheme = if source.starts_with("terrain-sphere://") {
            "terrain-sphere"
        } else if source.starts_with("earth-sphere://") {
            "earth-sphere"
        } else {
            "terrain-plane"
        };
        let mut params = parse_terrain_params_from_uri(source);
        if let Some(v) = terrain_plane_amplitude  { params.amplitude  = *v; }
        if let Some(v) = terrain_plane_frequency  { params.frequency  = *v; }
        if let Some(v) = terrain_plane_roughness  { params.roughness  = *v; }
        if let Some(v) = terrain_plane_octaves    { params.octaves    = *v; }
        if let Some(v) = terrain_plane_seed_x     { params.seed_x     = *v; }
        if let Some(v) = terrain_plane_seed_z     { params.seed_z     = *v; }
        if let Some(v) = terrain_plane_lacunarity { params.lacunarity = *v; }
        if let Some(v) = terrain_plane_ridge      { params.ridge      = *v; }
        if let Some(v) = terrain_plane_plateau    { params.plateau    = *v; }
        if let Some(v) = terrain_plane_sea_level  { params.sea_level  = *v; }
        if let Some(v) = terrain_plane_scale_x    { params.scale_x    = *v; }
        if let Some(v) = terrain_plane_scale_z    { params.scale_z    = *v; }
        let grid = source
            .splitn(3, "//")
            .nth(1)
            .unwrap_or("32")
            .split('?')
            .next()
            .unwrap_or("32");
        effective_source_buf = format!(
            "{scheme}://{}?amp={}&freq={}&oct={}&rough={}&sx={}&sz={}&lac={}&ridge={}&plat={}&sea={}&scx={}&scz={}",
            grid,
            params.amplitude, params.frequency, params.octaves, params.roughness,
            params.seed_x, params.seed_z, params.lacunarity,
            if params.ridge { 1 } else { 0 },
            params.plateau, params.sea_level, params.scale_x, params.scale_z
        );
        &effective_source_buf
    } else if source.starts_with("world://")
        && (world_gen_seed.is_some()
            || world_gen_ocean_fraction.is_some()
            || world_gen_continent_scale.is_some()
            || world_gen_continent_warp.is_some()
            || world_gen_continent_octaves.is_some()
            || world_gen_mountain_scale.is_some()
            || world_gen_mountain_strength.is_some()
            || world_gen_mountain_ridge_octaves.is_some()
            || world_gen_moisture_scale.is_some()
            || world_gen_ice_cap_strength.is_some()
            || world_gen_lapse_rate.is_some()
            || world_gen_rain_shadow.is_some()
            || world_gen_displacement_scale.is_some()
            || world_gen_subdivisions.is_some()
            || world_gen_shape.is_some()
            || world_gen_base.is_some()
            || world_gen_coloring.is_some())
    {
        let mut p = engine_worldgen::parse_world_params_from_uri(source);
        if let Some(v) = world_gen_shape               { p.shape = engine_worldgen::parse_world_shape(v); }
        if let Some(v) = world_gen_base                { p.base = engine_worldgen::parse_world_base(v); }
        if let Some(v) = world_gen_coloring            { p.coloring = engine_worldgen::parse_world_coloring(v); }
        if let Some(v) = world_gen_subdivisions        { p.subdivisions = *v; }
        if let Some(v) = world_gen_seed                { p.planet.seed = *v; }
        if let Some(v) = world_gen_ocean_fraction      { p.planet.ocean_fraction = *v; }
        if let Some(v) = world_gen_continent_scale     { p.planet.continent_scale = *v; }
        if let Some(v) = world_gen_continent_warp      { p.planet.continent_warp = *v; }
        if let Some(v) = world_gen_continent_octaves   { p.planet.continent_octaves = *v; }
        if let Some(v) = world_gen_mountain_scale      { p.planet.mountain_scale = *v; }
        if let Some(v) = world_gen_mountain_strength   { p.planet.mountain_strength = *v; }
        if let Some(v) = world_gen_mountain_ridge_octaves { p.planet.mountain_ridge_octaves = *v; }
        if let Some(v) = world_gen_moisture_scale      { p.planet.moisture_scale = *v; }
        if let Some(v) = world_gen_ice_cap_strength    { p.planet.ice_cap_strength = *v; }
        if let Some(v) = world_gen_lapse_rate          { p.planet.lapse_rate = *v; }
        if let Some(v) = world_gen_rain_shadow         { p.planet.rain_shadow = *v; }
        if let Some(v) = world_gen_displacement_scale  { p.displacement_scale = *v; }
        effective_source_buf = engine_worldgen::world_uri_from_params(&p);
        &effective_source_buf
    } else {
        source
    };
    let (sprite_width, sprite_height) = if width.is_some() || height.is_some() || size.is_some() {
        obj_sprite_dimensions(*width, *height, *size)
    } else {
        (area.width.max(1), area.height.max(1))
    };
    let base_x = area.origin_x + resolve_x(node_x, align_x, area.width, sprite_width);
    let base_y = area.origin_y + resolve_y(node_y, align_y, area.height, sprite_height);
    let (draw_x, draw_y) = compute_draw_pos(
        base_x,
        base_y,
        sprite.animations(),
        sprite_elapsed,
        object_state,
    );

    let fg = fg_colour.as_ref().map(Color::from).unwrap_or(Color::White);
    let bg = bg_colour.as_ref().map(Color::from).unwrap_or(Color::Reset);
    let draw_glyph = draw_char
        .as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or('#');
    // Avoid allocating a lowercase String by using eq_ignore_ascii_case.
    let is_wireframe = surface_mode
        .as_deref()
        .map(|s| s.trim().eq_ignore_ascii_case("wireframe"))
        .unwrap_or(false);
    let camera_state = id
        .as_deref()
        .and_then(|sid| ctx.obj_camera_states.get(sid))
        .cloned()
        .unwrap_or_default();

    // Prerender fast path: check if this sprite has a cached frame.
    let sprite_id_opt = id.as_deref();
    let elapsed_s = sprite_elapsed as f32 / 1000.0;
    let live_total_yaw =
        rotation_y.unwrap_or(0.0) + node_yaw + rotate_y_deg_per_sec.unwrap_or(0.0) * elapsed_s;
    let current_pitch = node_pitch;
    let clip_min = clip_y_min.unwrap_or(0.0);
    let clip_max = clip_y_max.unwrap_or(1.0);
    if let Some(sid) = sprite_id_opt {
        if try_blit_prerendered(
            sid,
            live_total_yaw,
            current_pitch,
            clip_min,
            clip_max,
            draw_x,
            draw_y,
            ctx.layer_buf,
        ) {
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
            return;
        }
    }

    let use_scene_camera = *camera_source == CameraSource::Scene;
    let scene_camera = ctx.scene_camera_3d;
    render_obj_content(
        effective_source,
        Some(sprite_width),
        Some(sprite_height),
        *size,
        ObjRenderParams {
            scale: node_scale,
            yaw_deg: node_yaw,
            pitch_deg: node_pitch,
            roll_deg: node_roll,
            rotation_x: rotation_x.unwrap_or(0.0),
            rotation_y: rotation_y.unwrap_or(0.0),
            rotation_z: rotation_z.unwrap_or(0.0),
            rotate_y_deg_per_sec: rotate_y_deg_per_sec.unwrap_or(20.0),
            camera_distance: camera_distance.unwrap_or(3.0),
            fov_degrees: fov_degrees.unwrap_or(60.0),
            near_clip: near_clip.unwrap_or(0.001),
            light_direction_x: light_direction_x.unwrap_or(-0.45),
            light_direction_y: light_direction_y.unwrap_or(0.70),
            light_direction_z: light_direction_z.unwrap_or(-0.85),
            light_2_direction_x: light_2_direction_x.unwrap_or(0.0),
            light_2_direction_y: light_2_direction_y.unwrap_or(0.0),
            light_2_direction_z: light_2_direction_z.unwrap_or(-1.0),
            light_2_intensity: light_2_intensity.unwrap_or(0.0),
            light_point_x: light_point_x.unwrap_or(0.0),
            light_point_y: light_point_y.unwrap_or(2.0),
            light_point_z: light_point_z.unwrap_or(0.0),
            light_point_intensity: light_point_intensity.unwrap_or(0.0),
            light_point_colour: light_point_colour.as_ref().map(Color::from),
            light_point_flicker_depth: light_point_flicker_depth.unwrap_or(0.0),
            light_point_flicker_hz: light_point_flicker_hz.unwrap_or(0.0),
            light_point_orbit_hz: light_point_orbit_hz.unwrap_or(0.0),
            light_point_snap_hz: light_point_snap_hz.unwrap_or(0.0),
            light_point_2_x: light_point_2_x.unwrap_or(0.0),
            light_point_2_y: light_point_2_y.unwrap_or(0.0),
            light_point_2_z: light_point_2_z.unwrap_or(0.0),
            light_point_2_intensity: light_point_2_intensity.unwrap_or(0.0),
            light_point_2_colour: light_point_2_colour.as_ref().map(Color::from),
            light_point_2_flicker_depth: light_point_2_flicker_depth.unwrap_or(0.0),
            light_point_2_flicker_hz: light_point_2_flicker_hz.unwrap_or(0.0),
            light_point_2_orbit_hz: light_point_2_orbit_hz.unwrap_or(0.0),
            light_point_2_snap_hz: light_point_2_snap_hz.unwrap_or(0.0),
            cel_levels: cel_levels.unwrap_or(0),
            shadow_colour: shadow_colour.as_ref().map(Color::from),
            midtone_colour: midtone_colour.as_ref().map(Color::from),
            highlight_colour: highlight_colour.as_ref().map(Color::from),
            tone_mix: tone_mix.unwrap_or(0.0),
            scene_elapsed_ms: sprite_elapsed,
            camera_pan_x: camera_state.pan_x,
            camera_pan_y: camera_state.pan_y,
            camera_look_yaw: camera_state.look_yaw,
            camera_look_pitch: camera_state.look_pitch,
            object_translate_x: world_x.unwrap_or(0.0),
            object_translate_y: world_y.unwrap_or(0.0),
            object_translate_z: world_z.unwrap_or(0.0),
            clip_y_min: clip_y_min.unwrap_or(0.0),
            clip_y_max: clip_y_max.unwrap_or(1.0),
            // Cockpit camera override: when cam_world_x/y/z are set, use them; otherwise fall
            // back to the legacy (0, 0, -camera_distance) position.
            camera_world_x: if use_scene_camera {
                scene_camera.eye[0]
            } else {
                cam_world_x.unwrap_or(0.0)
            },
            camera_world_y: if use_scene_camera {
                scene_camera.eye[1]
            } else {
                cam_world_y.unwrap_or(0.0)
            },
            camera_world_z: if use_scene_camera {
                scene_camera.eye[2]
            } else {
                cam_world_z.unwrap_or(-camera_distance.unwrap_or(3.0))
            },
            view_right_x: if use_scene_camera {
                scene_camera.right()[0]
            } else {
                view_right_x.unwrap_or(1.0)
            },
            view_right_y: if use_scene_camera {
                scene_camera.right()[1]
            } else {
                view_right_y.unwrap_or(0.0)
            },
            view_right_z: if use_scene_camera {
                scene_camera.right()[2]
            } else {
                view_right_z.unwrap_or(0.0)
            },
            view_up_x: if use_scene_camera {
                scene_camera.up[0]
            } else {
                view_up_x.unwrap_or(0.0)
            },
            view_up_y: if use_scene_camera {
                scene_camera.up[1]
            } else {
                view_up_y.unwrap_or(1.0)
            },
            view_up_z: if use_scene_camera {
                scene_camera.up[2]
            } else {
                view_up_z.unwrap_or(0.0)
            },
            view_forward_x: if use_scene_camera {
                scene_camera.forward()[0]
            } else {
                view_fwd_x.unwrap_or(0.0)
            },
            view_forward_y: if use_scene_camera {
                scene_camera.forward()[1]
            } else {
                view_fwd_y.unwrap_or(0.0)
            },
            view_forward_z: if use_scene_camera {
                scene_camera.forward()[2]
            } else {
                view_fwd_z.unwrap_or(1.0)
            },
            unlit: false,
            ambient: ambient.unwrap_or(0.15),
            light_point_falloff: 0.7,
            light_point_2_falloff: 0.7,
            smooth_shading: smooth_shading.unwrap_or(false),
            latitude_bands: latitude_bands.unwrap_or(0),
            latitude_band_depth: latitude_band_depth.unwrap_or(0.0),
            terrain_color: terrain_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            terrain_threshold: terrain_threshold.unwrap_or(0.5),
            terrain_noise_scale: terrain_noise_scale.unwrap_or(2.5),
            terrain_noise_octaves: terrain_noise_octaves.unwrap_or(2),
            marble_depth: marble_depth.unwrap_or(0.0),
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
            terrain_displacement: 0.0,
            below_threshold_transparent: *below_threshold_transparent,
            cloud_alpha_softness: 0.0,
            polar_ice_color: polar_ice_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            polar_ice_start: polar_ice_start.unwrap_or(0.78),
            polar_ice_end: polar_ice_end.unwrap_or(0.92),
            desert_color: desert_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            desert_strength: desert_strength.unwrap_or(0.0),
            atmo_color: None,
            atmo_height: atmo_height.unwrap_or(0.12),
            atmo_density: atmo_density.unwrap_or(0.0),
            atmo_strength: 0.0,
            atmo_rayleigh_amount: atmo_rayleigh_amount.unwrap_or(0.0),
            atmo_rayleigh_color: atmo_rayleigh_color
                .as_ref()
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([124, 200, 255])),
            atmo_rayleigh_falloff: atmo_rayleigh_falloff.unwrap_or(0.32),
            atmo_haze_amount: atmo_haze_amount.unwrap_or(0.0),
            atmo_haze_color: atmo_haze_color
                .as_ref()
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([212, 225, 240])),
            atmo_haze_falloff: atmo_haze_falloff.unwrap_or(0.18),
            atmo_absorption_amount: atmo_absorption_amount.unwrap_or(0.0),
            atmo_absorption_color: atmo_absorption_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            atmo_absorption_height: atmo_absorption_height.unwrap_or(0.55),
            atmo_absorption_width: atmo_absorption_width.unwrap_or(0.18),
            atmo_forward_scatter: atmo_forward_scatter.unwrap_or(0.72),
            atmo_limb_boost: atmo_limb_boost.unwrap_or(1.0),
            atmo_terminator_softness: atmo_terminator_softness.unwrap_or(1.0),
            atmo_night_glow: atmo_night_glow.unwrap_or(0.0),
            atmo_night_glow_color: atmo_night_glow_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
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
            night_light_color: night_light_color.as_ref().map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            night_light_threshold: night_light_threshold.unwrap_or(0.82),
            night_light_intensity: night_light_intensity.unwrap_or(0.0),
            heightmap: None,
            heightmap_w: 0,
            heightmap_h: 0,
            heightmap_blend: 0.0,
            // Opaque OBJ/world meshes rely on the depth buffer; transparent layers use dedicated RGBA paths.
            depth_sort_faces: false,
        },
        is_wireframe,
        backface_cull.unwrap_or(false),
        draw_glyph,
        fg,
        bg,
        ctx.asset_root,
        draw_x,
        draw_y,
        ctx.layer_buf,
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

pub(crate) fn render_scene_clip_sprite(
    sprite: &Sprite,
    area: RenderArea,
    object_id: Option<&str>,
    object_state: &ObjectRuntimeState,
    object_regions: &mut HashMap<String, Region>,
    ctx: &mut RenderCtx<'_>,
) {
    let Some(node) = map_sprite_to_node3d(sprite) else {
        return;
    };
    let Renderable3D::SceneClip(scene_clip) = node.renderable else {
        return;
    };
    use crate::scene3d_atlas::Scene3DAtlas;
    use crate::scene3d_runtime_store::Scene3DRuntimeStore;
    use engine_render::rasterizer::blit;
    let draw_x = area
        .origin_x
        .saturating_add(node.transform.translation[0].round() as i32)
        .saturating_add(object_state.offset_x)
        .max(0) as u16;
    let draw_y = area
        .origin_y
        .saturating_add(node.transform.translation[1].round() as i32)
        .saturating_add(object_state.offset_y)
        .max(0) as u16;

    // Real-time path: if the frame string names a clip (no "-N" suffix with a numeric keyframe
    // index), look up the parsed scene definition and render the current animation frame live.
    // This gives true 60fps 3D animation without startup prerender cost for clip frames.
    let rendered_realtime = if let (Some(entry), Some(asset_root)) =
        (Scene3DRuntimeStore::current_get(&scene_clip.source), ctx.asset_root)
    {
        if entry.def.frames.contains_key(scene_clip.frame.as_str()) {
            let buf = crate::scene3d_prerender::render_scene3d_frame_at(
                entry,
                &scene_clip.frame,
                ctx.scene_elapsed_ms,
                asset_root,
                scene_clip.use_scene_camera.then_some(ctx.scene_camera_3d),
            );
            if let Some(buf) = buf {
                blit(&buf, ctx.layer_buf, draw_x, draw_y);
                if let Some(id) = object_id {
                    object_regions.insert(
                        id.to_string(),
                        engine_core::effects::Region {
                            x: draw_x,
                            y: draw_y,
                            width: buf.width,
                            height: buf.height,
                        },
                    );
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    // Fallback: look up prerendered buffer from world-scoped atlas via thread-local pointer.
    // Used for static frames and when no runtime store is available.
    if !rendered_realtime && !scene_clip.use_scene_camera {
        if let Some(buf) = Scene3DAtlas::current_get(&scene_clip.source, &scene_clip.frame) {
            blit(&buf, ctx.layer_buf, draw_x, draw_y);
            if let Some(id) = object_id {
                object_regions.insert(
                    id.to_string(),
                    engine_core::effects::Region {
                        x: draw_x,
                        y: draw_y,
                        width: buf.width,
                        height: buf.height,
                    },
                );
            }
        }
    }
}



