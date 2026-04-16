use crate::pipeline::GeneratedWorldSpriteSpec;
use crate::scene::Renderable3D;
use crate::ObjRenderParams;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::{CameraSource, SpriteSizePreset};
use engine_core::scene_runtime_types::SceneCamera3D;

const DEFAULT_WORLD_CLOUD_2_COLOR: Color = Color::Rgb {
    r: 0xd7,
    g: 0xe2,
    b: 0xec,
};

pub struct GeneratedWorldRenderProfile {
    pub ambient: f32,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
    pub terrain_displacement: f32,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub terrain_noise_scale: f32,
    pub terrain_noise_octaves: u8,
    pub marble_depth: f32,
    pub terrain_relief: f32,
    pub polar_ice_color: Option<[u8; 3]>,
    pub polar_ice_start: f32,
    pub polar_ice_end: f32,
    pub desert_color: Option<[u8; 3]>,
    pub desert_strength: f32,
    pub atmo_strength: f32,
    pub atmo_color: Option<[u8; 3]>,
    pub night_light_color: Option<[u8; 3]>,
    pub night_light_threshold: f32,
    pub night_light_intensity: f32,
    pub shadow_color: Option<Color>,
    pub midtone_color: Option<Color>,
    pub highlight_color: Option<Color>,
    pub tone_mix: f32,
    pub cel_levels: u8,
    pub noise_seed: f32,
    pub generated_heightmap: Option<std::sync::Arc<Vec<f32>>>,
    pub generated_heightmap_w: u32,
    pub generated_heightmap_h: u32,
    pub heightmap_blend: f32,
    pub warp_strength: f32,
    pub warp_octaves: u8,
    pub noise_lacunarity: f32,
    pub noise_persistence: f32,
    pub normal_perturb_strength: f32,
    pub ocean_specular: f32,
    pub ocean_noise_scale: f32,
    pub crater_density: f32,
    pub crater_rim_height: f32,
    pub snow_line_altitude: f32,
    pub ocean_color: Color,
    pub cloud_color: Color,
    pub cloud_threshold: f32,
    pub cloud_ambient: f32,
    pub cloud_noise_scale: f32,
    pub cloud_noise_octaves: u8,
    pub cloud_scale: f32,
    pub cloud2_scale: f32,
    pub atmo_visibility: f32,
    pub sun_dir: [f32; 3],
}

pub type RenderObjToCanvasFn = fn(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    backface_cull: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 3]>>, u16, u16)>;

pub type RenderObjToRgbaCanvasFn = fn(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    size: Option<SpriteSizePreset>,
    params: ObjRenderParams,
    wireframe: bool,
    fg: Color,
    asset_root: Option<&AssetRoot>,
) -> Option<(Vec<Option<[u8; 4]>>, u16, u16)>;

pub type ConvertCanvasToRgbaFn = fn(Vec<Option<[u8; 3]>>) -> Vec<Option<[u8; 4]>>;
pub type CompositeRgbaOverFn = fn(&mut [Option<[u8; 4]>], &[Option<[u8; 4]>]);
pub type BlitRgbaCanvasFn = fn(&mut Buffer, &[Option<[u8; 4]>], u16, u16, u16, u16, u16, u16);

pub struct GeneratedWorldRenderCallbacks {
    pub render_obj_to_canvas: RenderObjToCanvasFn,
    pub render_obj_to_rgba_canvas: RenderObjToRgbaCanvasFn,
    pub convert_canvas_to_rgba: ConvertCanvasToRgbaFn,
    pub composite_rgba_over: CompositeRgbaOverFn,
    pub blit_rgba_canvas: BlitRgbaCanvasFn,
}

#[allow(clippy::too_many_arguments)]
pub fn render_generated_world_sprite_with(
    spec: GeneratedWorldSpriteSpec<'_>,
    profile: &GeneratedWorldRenderProfile,
    sprite_width: u16,
    sprite_height: u16,
    draw_x: u16,
    draw_y: u16,
    sprite_elapsed: u64,
    scene_camera_3d: &SceneCamera3D,
    asset_root: Option<&AssetRoot>,
    target: &mut Buffer,
    callbacks: GeneratedWorldRenderCallbacks,
) -> bool {
    let GeneratedWorldSpriteSpec {
        node,
        size,
        spin_deg,
        cloud_spin_deg,
        cloud2_spin_deg,
        camera_distance,
        camera_source,
        fov_degrees,
        near_clip,
        sun_dir_x,
        sun_dir_y,
        sun_dir_z,
        ..
    } = spec;
    let Renderable3D::GeneratedWorld(generated_world) = node.renderable else {
        return false;
    };

    let use_scene_camera = camera_source == CameraSource::Scene;
    let sun_dir = [
        sun_dir_x.unwrap_or(profile.sun_dir[0]),
        sun_dir_y.unwrap_or(profile.sun_dir[1]),
        sun_dir_z.unwrap_or(profile.sun_dir[2]),
    ];
    let surface_scale = node.transform.scale[0];
    let mesh_path = generated_world.mesh_key.as_str();
    let base_yaw = node.transform.rotation_deg[1];
    let pitch = node.transform.rotation_deg[0];
    let roll = node.transform.rotation_deg[2];
    let camera_distance = camera_distance.unwrap_or(3.0);
    let fov_degrees = fov_degrees.unwrap_or(60.0);
    let near_clip = near_clip.unwrap_or(0.001);

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
        scene_camera_3d,
        sun_dir,
    );
    surface_params.ambient = profile.ambient;
    surface_params.smooth_shading = true;
    surface_params.latitude_bands = profile.latitude_bands;
    surface_params.latitude_band_depth = profile.latitude_band_depth;
    surface_params.terrain_displacement = profile.terrain_displacement;
    surface_params.terrain_color = profile.terrain_color;
    surface_params.terrain_threshold = profile.terrain_threshold;
    surface_params.terrain_noise_scale = profile.terrain_noise_scale;
    surface_params.terrain_noise_octaves = profile.terrain_noise_octaves;
    surface_params.marble_depth = profile.marble_depth;
    surface_params.terrain_relief = profile.terrain_relief;
    surface_params.polar_ice_color = profile.polar_ice_color;
    surface_params.polar_ice_start = profile.polar_ice_start;
    surface_params.polar_ice_end = profile.polar_ice_end;
    surface_params.desert_color = profile.desert_color;
    surface_params.desert_strength = profile.desert_strength;
    surface_params.atmo_color = None;
    surface_params.atmo_height = 0.12;
    surface_params.atmo_density = (profile.atmo_strength * profile.atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_strength = 0.0;
    surface_params.atmo_rayleigh_amount =
        (profile.atmo_strength * profile.atmo_visibility).clamp(0.0, 1.0);
    surface_params.atmo_rayleigh_color = profile.atmo_color;
    surface_params.atmo_rayleigh_falloff = 0.32;
    surface_params.atmo_haze_amount =
        (profile.atmo_strength * 0.45 * profile.atmo_visibility).clamp(0.0, 1.0);
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
    surface_params.night_light_color = profile.night_light_color;
    surface_params.night_light_threshold = profile.night_light_threshold;
    surface_params.night_light_intensity = profile.night_light_intensity;
    surface_params.shadow_colour = profile.shadow_color;
    surface_params.midtone_colour = profile.midtone_color;
    surface_params.highlight_colour = profile.highlight_color;
    surface_params.tone_mix = profile.tone_mix;
    surface_params.cel_levels = profile.cel_levels;
    surface_params.noise_seed = profile.noise_seed;
    surface_params.heightmap = profile.generated_heightmap.clone();
    surface_params.heightmap_w = profile.generated_heightmap_w;
    surface_params.heightmap_h = profile.generated_heightmap_h;
    surface_params.heightmap_blend = profile.heightmap_blend;
    surface_params.warp_strength = profile.warp_strength;
    surface_params.warp_octaves = profile.warp_octaves;
    surface_params.noise_lacunarity = profile.noise_lacunarity;
    surface_params.noise_persistence = profile.noise_persistence;
    surface_params.normal_perturb_strength = profile.normal_perturb_strength;
    surface_params.ocean_specular = profile.ocean_specular;
    surface_params.ocean_noise_scale = profile.ocean_noise_scale;
    surface_params.crater_density = profile.crater_density;
    surface_params.crater_rim_height = profile.crater_rim_height;
    surface_params.snow_line_altitude = profile.snow_line_altitude;

    let (ocean_r, ocean_g, ocean_b) = profile.ocean_color.to_rgb();
    surface_params.ocean_color_rgb = Some([ocean_r, ocean_g, ocean_b]);

    let Some((surface_rgb, virtual_w, virtual_h)) = (callbacks.render_obj_to_canvas)(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        surface_params,
        false,
        false,
        profile.ocean_color,
        asset_root,
    ) else {
        return false;
    };
    let mut composited = (callbacks.convert_canvas_to_rgba)(surface_rgb);

    let mut cloud_params = build_generated_world_base_params(
        profile.cloud_scale,
        base_yaw + cloud_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera_3d,
        sun_dir,
    );
    cloud_params.ambient = profile.cloud_ambient;
    cloud_params.smooth_shading = true;
    cloud_params.terrain_color = Some(color_to_rgb(profile.cloud_color));
    cloud_params.terrain_threshold = profile.cloud_threshold.clamp(0.0, 0.999);
    cloud_params.terrain_noise_scale = profile.cloud_noise_scale;
    cloud_params.terrain_noise_octaves = profile.cloud_noise_octaves.max(1);
    cloud_params.marble_depth = (profile.marble_depth * 0.5).max(0.003);
    cloud_params.below_threshold_transparent = true;
    cloud_params.cloud_alpha_softness = 0.12;

    if let Some((cloud1_rgba, _, _)) = (callbacks.render_obj_to_rgba_canvas)(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        cloud_params,
        false,
        profile.cloud_color,
        asset_root,
    ) {
        (callbacks.composite_rgba_over)(&mut composited, &cloud1_rgba);
    }

    let mut cloud2_params = build_generated_world_base_params(
        profile.cloud2_scale,
        base_yaw + 180.0 + cloud2_spin_deg.unwrap_or(0.0),
        pitch,
        roll,
        camera_distance,
        fov_degrees,
        near_clip,
        sprite_elapsed,
        use_scene_camera,
        scene_camera_3d,
        sun_dir,
    );
    cloud2_params.ambient = 0.004;
    cloud2_params.smooth_shading = true;
    cloud2_params.terrain_color = Some(color_to_rgb(DEFAULT_WORLD_CLOUD_2_COLOR));
    cloud2_params.terrain_threshold = (profile.cloud_threshold + 0.12).min(0.992);
    cloud2_params.terrain_noise_scale = (profile.cloud_noise_scale * 0.35).max(1.1);
    cloud2_params.terrain_noise_octaves = profile.cloud_noise_octaves.clamp(1, 2);
    cloud2_params.marble_depth = (profile.marble_depth * 0.2).max(0.002);
    cloud2_params.below_threshold_transparent = true;
    cloud2_params.cloud_alpha_softness = 0.08;

    if let Some((cloud2_rgba, _, _)) = (callbacks.render_obj_to_rgba_canvas)(
        mesh_path,
        Some(sprite_width),
        Some(sprite_height),
        size,
        cloud2_params,
        false,
        DEFAULT_WORLD_CLOUD_2_COLOR,
        asset_root,
    ) {
        (callbacks.composite_rgba_over)(&mut composited, &cloud2_rgba);
    }

    (callbacks.blit_rgba_canvas)(
        target,
        &composited,
        virtual_w,
        virtual_h,
        sprite_width,
        sprite_height,
        draw_x,
        draw_y,
    );

    true
}

fn color_to_rgb(color: Color) -> [u8; 3] {
    let (r, g, b) = color.to_rgb();
    [r, g, b]
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
