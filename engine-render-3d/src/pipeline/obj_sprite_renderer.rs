use super::{apply_world_lod_to_source, ObjSpriteSpec, ViewLightingParams};
use crate::prerender::ObjPrerenderedFrames;
use crate::raster::{obj_sprite_dimensions, render_obj_content, try_blit_prerendered};
use crate::scene::{select_lod_level_stable, Renderable3D};
use crate::ObjRenderParams;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::Region;
use engine_core::render_types::ScreenSpaceMetrics;
use engine_core::scene::{CameraSource, HorizontalAlign, VerticalAlign};
use engine_core::scene_runtime_types::{ObjCameraState, SceneCamera3D};

#[derive(Debug, Clone, Copy)]
pub struct SpriteRenderArea {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone)]
pub struct ObjSpriteRenderRuntime<'a> {
    pub sprite_elapsed_ms: u64,
    pub object_offset_x: i32,
    pub object_offset_y: i32,
    pub camera_state: ObjCameraState,
    pub scene_camera_3d: &'a SceneCamera3D,
    pub view_lighting: ViewLightingParams,
    pub asset_root: Option<&'a AssetRoot>,
    pub prerender_frames: Option<&'a ObjPrerenderedFrames>,
}

pub fn render_obj_sprite_to_buffer(
    spec: ObjSpriteSpec<'_>,
    area: SpriteRenderArea,
    runtime: ObjSpriteRenderRuntime<'_>,
    target: &mut Buffer,
) -> Option<Region> {
    let ObjSpriteSpec {
        node,
        id,
        size,
        width,
        height,
        stretch_to_area,
        surface_mode,
        backface_cull,
        clip_y_min,
        clip_y_max,
        rotation_x,
        rotation_y,
        rotation_z,
        rotate_y_deg_per_sec,
        camera_distance,
        camera_source,
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
        atmo_height,
        atmo_density,
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
        ..
    } = spec;

    let Renderable3D::Mesh(mesh_node) = &node.renderable else {
        return None;
    };

    let source = mesh_node.mesh_key.as_str();
    let node_x = node.transform.translation[0].round() as i32;
    let node_y = node.transform.translation[1].round() as i32;
    let node_scale = node.transform.scale[0];
    let node_pitch = node.transform.rotation_deg[0];
    let node_yaw = node.transform.rotation_deg[1];
    let node_roll = node.transform.rotation_deg[2];

    let (sprite_width, sprite_height) = if stretch_to_area {
        (area.width.max(1), area.height.max(1))
    } else if width.is_some() || height.is_some() || size.is_some() {
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
    let effective_source = apply_world_lod_to_source(source, selected_lod);
    let base_x = area.origin_x + resolve_x(node_x, &align_x, area.width, sprite_width);
    let base_y = area.origin_y + resolve_y(node_y, &align_y, area.height, sprite_height);
    let draw_x = base_x.saturating_add(runtime.object_offset_x);
    let draw_y = base_y.saturating_add(runtime.object_offset_y);

    let fg = fg_colour.map(Color::from).unwrap_or(Color::White);
    let bg = bg_colour.map(Color::from).unwrap_or(Color::Reset);
    let draw_glyph = draw_char
        .as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or('#');
    let is_wireframe = surface_mode
        .as_deref()
        .map(|s| s.trim().eq_ignore_ascii_case("wireframe"))
        .unwrap_or(false);

    let elapsed_s = runtime.sprite_elapsed_ms as f32 / 1000.0;
    let live_total_yaw =
        rotation_y.unwrap_or(0.0) + node_yaw + rotate_y_deg_per_sec.unwrap_or(0.0) * elapsed_s;
    let current_pitch = node_pitch;
    let clip_min = clip_y_min.unwrap_or(0.0);
    let clip_max = clip_y_max.unwrap_or(1.0);
    if let (Some(frames), Some(sid)) = (runtime.prerender_frames, id.as_deref()) {
        if try_blit_prerendered(
            Some(frames),
            sid,
            live_total_yaw,
            current_pitch,
            clip_min,
            clip_max,
            draw_x,
            draw_y,
            target,
        ) {
            return Some(visible_region(
                draw_x,
                draw_y,
                sprite_width,
                sprite_height,
                target,
            ));
        }
    }

    let use_scene_camera = camera_source == CameraSource::Scene;
    let scene_camera = runtime.scene_camera_3d;
    render_obj_content(
        effective_source.as_str(),
        Some(sprite_width),
        Some(sprite_height),
        size,
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
            light_point_colour: light_point_colour.map(Color::from),
            light_point_flicker_depth: light_point_flicker_depth.unwrap_or(0.0),
            light_point_flicker_hz: light_point_flicker_hz.unwrap_or(0.0),
            light_point_orbit_hz: light_point_orbit_hz.unwrap_or(0.0),
            light_point_snap_hz: light_point_snap_hz.unwrap_or(0.0),
            light_point_2_x: light_point_2_x.unwrap_or(0.0),
            light_point_2_y: light_point_2_y.unwrap_or(0.0),
            light_point_2_z: light_point_2_z.unwrap_or(0.0),
            light_point_2_intensity: light_point_2_intensity.unwrap_or(0.0),
            light_point_2_colour: light_point_2_colour.map(Color::from),
            light_point_2_flicker_depth: light_point_2_flicker_depth.unwrap_or(0.0),
            light_point_2_flicker_hz: light_point_2_flicker_hz.unwrap_or(0.0),
            light_point_2_orbit_hz: light_point_2_orbit_hz.unwrap_or(0.0),
            light_point_2_snap_hz: light_point_2_snap_hz.unwrap_or(0.0),
            cel_levels: cel_levels.unwrap_or(0),
            shadow_colour: shadow_colour.map(Color::from),
            midtone_colour: midtone_colour.map(Color::from),
            highlight_colour: highlight_colour.map(Color::from),
            tone_mix: tone_mix.unwrap_or(0.0),
            scene_elapsed_ms: runtime.sprite_elapsed_ms,
            camera_pan_x: runtime.camera_state.pan_x,
            camera_pan_y: runtime.camera_state.pan_y,
            camera_look_yaw: runtime.camera_state.look_yaw,
            camera_look_pitch: runtime.camera_state.look_pitch,
            object_translate_x: world_x.unwrap_or(0.0),
            object_translate_y: world_y.unwrap_or(0.0),
            object_translate_z: world_z.unwrap_or(0.0),
            clip_y_min: clip_y_min.unwrap_or(0.0),
            clip_y_max: clip_y_max.unwrap_or(1.0),
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
            ambient_floor: runtime.view_lighting.ambient_floor,
            shadow_contrast: runtime.view_lighting.shadow_contrast,
            exposure: runtime.view_lighting.exposure,
            gamma: runtime.view_lighting.gamma,
            tonemap: runtime.view_lighting.tonemap,
            light_point_falloff: 0.7,
            light_point_2_falloff: 0.7,
            smooth_shading: smooth_shading.unwrap_or(false),
            latitude_bands: latitude_bands.unwrap_or(0),
            latitude_band_depth: latitude_band_depth.unwrap_or(0.0),
            terrain_color: terrain_color.map(|c| {
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
            below_threshold_transparent,
            cloud_alpha_softness: 0.0,
            polar_ice_color: polar_ice_color.map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            polar_ice_start: polar_ice_start.unwrap_or(0.78),
            polar_ice_end: polar_ice_end.unwrap_or(0.92),
            desert_color: desert_color.map(|c| {
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
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([124, 200, 255])),
            atmo_rayleigh_falloff: atmo_rayleigh_falloff.unwrap_or(0.32),
            atmo_haze_amount: atmo_haze_amount.unwrap_or(0.0),
            atmo_haze_color: atmo_haze_color
                .map(|c| {
                    let (r, g, b) = Color::from(c).to_rgb();
                    [r, g, b]
                })
                .or(Some([212, 225, 240])),
            atmo_haze_falloff: atmo_haze_falloff.unwrap_or(0.18),
            atmo_absorption_amount: atmo_absorption_amount.unwrap_or(0.0),
            atmo_absorption_color: atmo_absorption_color.map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            atmo_absorption_height: atmo_absorption_height.unwrap_or(0.55),
            atmo_absorption_width: atmo_absorption_width.unwrap_or(0.18),
            atmo_forward_scatter: atmo_forward_scatter.unwrap_or(0.72),
            atmo_limb_boost: atmo_limb_boost.unwrap_or(1.0),
            atmo_terminator_softness: atmo_terminator_softness.unwrap_or(1.0),
            atmo_night_glow: atmo_night_glow.unwrap_or(0.0)
                * runtime.view_lighting.night_glow_scale,
            atmo_night_glow_color: atmo_night_glow_color.map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            atmo_haze_night_leak: runtime.view_lighting.haze_night_leak,
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
            night_light_color: night_light_color.map(|c| {
                let (r, g, b) = Color::from(c).to_rgb();
                [r, g, b]
            }),
            night_light_threshold: night_light_threshold.unwrap_or(0.82),
            night_light_intensity: night_light_intensity.unwrap_or(0.0),
            heightmap: None,
            heightmap_w: 0,
            heightmap_h: 0,
            heightmap_blend: 0.0,
            depth_sort_faces: false,
        },
        is_wireframe,
        backface_cull.unwrap_or(false),
        draw_glyph,
        fg,
        bg,
        runtime.asset_root,
        draw_x,
        draw_y,
        target,
    );

    Some(visible_region(
        draw_x,
        draw_y,
        sprite_width,
        sprite_height,
        target,
    ))
}

fn visible_region(draw_x: i32, draw_y: i32, width: u16, height: u16, buf: &Buffer) -> Region {
    let x0 = draw_x.max(0);
    let y0 = draw_y.max(0);
    let x1 = (draw_x + width as i32).min(buf.width as i32).max(x0);
    let y1 = (draw_y + height as i32).min(buf.height as i32).max(y0);
    Region {
        x: x0 as u16,
        y: y0 as u16,
        width: (x1 - x0) as u16,
        height: (y1 - y0) as u16,
    }
}

#[inline]
fn resolve_x(offset_x: i32, align_x: &Option<HorizontalAlign>, area_w: u16, sprite_w: u16) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) | None => 0i32,
        Some(HorizontalAlign::Center) => (area_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => area_w.saturating_sub(sprite_w) as i32,
    };
    origin.saturating_add(offset_x)
}

#[inline]
fn resolve_y(offset_y: i32, align_y: &Option<VerticalAlign>, area_h: u16, sprite_h: u16) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) | None => 0i32,
        Some(VerticalAlign::Center) => (area_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => area_h.saturating_sub(sprite_h) as i32,
    };
    origin.saturating_add(offset_y)
}
