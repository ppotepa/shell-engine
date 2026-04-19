use engine_core::color::Color;
use engine_core::render_types::{Light3D, LodHint, Transform3D};
use engine_core::scene::TonemapOperator;

use crate::pipeline::generated_world_renderer::GeneratedWorldRenderProfile;
use crate::scene::materials::MaterialInstance;
use crate::ObjRenderParams;

#[derive(Debug, Clone, Default)]
pub struct FrameGeometry3D {
    pub source: Option<String>,
    pub mesh_key_debug: Option<String>,
    pub transform: Transform3D,
    pub lod_hint: Option<LodHint>,
    pub visible: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FrameSurfaceProfile {
    pub material: Option<MaterialInstance>,
    pub unlit: bool,
    pub smooth_shading: bool,
    pub ambient: f32,
    pub ambient_floor: f32,
    pub cel_levels: u8,
    pub shadow_colour: Option<Color>,
    pub midtone_colour: Option<Color>,
    pub highlight_colour: Option<Color>,
    pub tone_mix: f32,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub terrain_noise_scale: f32,
    pub terrain_noise_octaves: u8,
    pub marble_depth: f32,
    pub terrain_relief: f32,
    pub terrain_displacement: f32,
    pub polar_ice_color: Option<[u8; 3]>,
    pub polar_ice_start: f32,
    pub polar_ice_end: f32,
    pub desert_color: Option<[u8; 3]>,
    pub desert_strength: f32,
    pub noise_seed: f32,
    pub warp_strength: f32,
    pub warp_octaves: u8,
    pub noise_lacunarity: f32,
    pub noise_persistence: f32,
    pub normal_perturb_strength: f32,
    pub ocean_specular: f32,
    pub ocean_noise_scale: f32,
    pub ocean_color_rgb: Option<[u8; 3]>,
    pub crater_density: f32,
    pub crater_rim_height: f32,
    pub snow_line_altitude: f32,
    pub below_threshold_transparent: bool,
    pub cloud_alpha_softness: f32,
    pub night_light_color: Option<[u8; 3]>,
    pub night_light_threshold: f32,
    pub night_light_intensity: f32,
    pub heightmap: Option<std::sync::Arc<Vec<f32>>>,
    pub heightmap_w: u32,
    pub heightmap_h: u32,
    pub heightmap_blend: f32,
}

#[derive(Debug, Clone, Default)]
pub struct FrameAtmosphereProfile {
    pub color: Option<[u8; 3]>,
    pub height: f32,
    pub density: f32,
    pub strength: f32,
    pub rayleigh_amount: f32,
    pub rayleigh_color: Option<[u8; 3]>,
    pub rayleigh_falloff: f32,
    pub haze_amount: f32,
    pub haze_color: Option<[u8; 3]>,
    pub haze_falloff: f32,
    pub absorption_amount: f32,
    pub absorption_color: Option<[u8; 3]>,
    pub absorption_height: f32,
    pub absorption_width: f32,
    pub forward_scatter: f32,
    pub limb_boost: f32,
    pub terminator_softness: f32,
    pub night_glow: f32,
    pub night_glow_color: Option<[u8; 3]>,
    pub haze_night_leak: f32,
    pub rim_power: f32,
    pub haze_strength: f32,
    pub haze_power: f32,
    pub veil_strength: f32,
    pub veil_power: f32,
    pub halo_strength: f32,
    pub halo_width: f32,
    pub halo_power: f32,
    pub visibility: f32,
}

#[derive(Debug, Clone)]
pub struct FrameLightingProfile {
    pub direction_lights: Vec<Light3D>,
    pub point_lights: Vec<FramePointLightProfile>,
    pub shadow_contrast: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub tonemap: TonemapOperator,
    pub depth_sort_faces: bool,
}

impl Default for FrameLightingProfile {
    fn default() -> Self {
        Self {
            direction_lights: Vec::new(),
            point_lights: Vec::new(),
            shadow_contrast: 1.0,
            exposure: 1.0,
            gamma: 2.2,
            tonemap: TonemapOperator::Linear,
            depth_sort_faces: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FramePointLightProfile {
    pub position: [f32; 3],
    pub intensity: f32,
    pub colour: Option<Color>,
    pub falloff: f32,
    pub flicker_depth: f32,
    pub flicker_hz: f32,
    pub orbit_hz: f32,
    pub snap_hz: f32,
}

#[derive(Debug, Clone, Default)]
pub struct FrameEnvironmentProfile {
    pub background_color: Option<Color>,
    pub background_floor: Option<f32>,
    pub sun_direction: Option<[f32; 3]>,
    pub cloud_color: Option<Color>,
    pub cloud_threshold: Option<f32>,
    pub cloud_ambient: Option<f32>,
    pub cloud_noise_scale: Option<f32>,
    pub cloud_noise_octaves: Option<u8>,
    pub cloud_scale: Option<f32>,
    pub cloud2_scale: Option<f32>,
    pub cloud_render_scale_1: Option<f32>,
    pub cloud_render_scale_2: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct FramePostProcessProfile {
    pub exposure: f32,
    pub gamma: f32,
    pub tonemap: TonemapOperator,
    pub shadow_contrast: f32,
}

impl Default for FramePostProcessProfile {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            gamma: 2.2,
            tonemap: TonemapOperator::Linear,
            shadow_contrast: 1.0,
        }
    }
}

impl FrameSurfaceProfile {
    pub fn from_obj_params(params: &ObjRenderParams) -> Self {
        Self {
            material: None,
            unlit: params.unlit,
            smooth_shading: params.smooth_shading,
            ambient: params.ambient,
            ambient_floor: params.ambient_floor,
            cel_levels: params.cel_levels,
            shadow_colour: params.shadow_colour,
            midtone_colour: params.midtone_colour,
            highlight_colour: params.highlight_colour,
            tone_mix: params.tone_mix,
            latitude_bands: params.latitude_bands,
            latitude_band_depth: params.latitude_band_depth,
            terrain_color: params.terrain_color,
            terrain_threshold: params.terrain_threshold,
            terrain_noise_scale: params.terrain_noise_scale,
            terrain_noise_octaves: params.terrain_noise_octaves,
            marble_depth: params.marble_depth,
            terrain_relief: params.terrain_relief,
            terrain_displacement: params.terrain_displacement,
            polar_ice_color: params.polar_ice_color,
            polar_ice_start: params.polar_ice_start,
            polar_ice_end: params.polar_ice_end,
            desert_color: params.desert_color,
            desert_strength: params.desert_strength,
            noise_seed: params.noise_seed,
            warp_strength: params.warp_strength,
            warp_octaves: params.warp_octaves,
            noise_lacunarity: params.noise_lacunarity,
            noise_persistence: params.noise_persistence,
            normal_perturb_strength: params.normal_perturb_strength,
            ocean_specular: params.ocean_specular,
            ocean_noise_scale: params.ocean_noise_scale,
            ocean_color_rgb: params.ocean_color_rgb,
            crater_density: params.crater_density,
            crater_rim_height: params.crater_rim_height,
            snow_line_altitude: params.snow_line_altitude,
            below_threshold_transparent: params.below_threshold_transparent,
            cloud_alpha_softness: params.cloud_alpha_softness,
            night_light_color: params.night_light_color,
            night_light_threshold: params.night_light_threshold,
            night_light_intensity: params.night_light_intensity,
            heightmap: params.heightmap.clone(),
            heightmap_w: params.heightmap_w,
            heightmap_h: params.heightmap_h,
            heightmap_blend: params.heightmap_blend,
        }
    }

    pub fn from_generated_world_profile(profile: &GeneratedWorldRenderProfile) -> Self {
        Self {
            material: None,
            unlit: false,
            smooth_shading: true,
            ambient: profile.ambient,
            ambient_floor: profile.ambient_floor,
            cel_levels: profile.cel_levels,
            shadow_colour: profile.shadow_color,
            midtone_colour: profile.midtone_color,
            highlight_colour: profile.highlight_color,
            tone_mix: profile.tone_mix,
            latitude_bands: profile.latitude_bands,
            latitude_band_depth: profile.latitude_band_depth,
            terrain_color: profile.terrain_color,
            terrain_threshold: profile.terrain_threshold,
            terrain_noise_scale: profile.terrain_noise_scale,
            terrain_noise_octaves: profile.terrain_noise_octaves,
            marble_depth: profile.marble_depth,
            terrain_relief: profile.terrain_relief,
            terrain_displacement: profile.terrain_displacement,
            polar_ice_color: profile.polar_ice_color,
            polar_ice_start: profile.polar_ice_start,
            polar_ice_end: profile.polar_ice_end,
            desert_color: profile.desert_color,
            desert_strength: profile.desert_strength,
            noise_seed: profile.noise_seed,
            warp_strength: profile.warp_strength,
            warp_octaves: profile.warp_octaves,
            noise_lacunarity: profile.noise_lacunarity,
            noise_persistence: profile.noise_persistence,
            normal_perturb_strength: profile.normal_perturb_strength,
            ocean_specular: profile.ocean_specular,
            ocean_noise_scale: profile.ocean_noise_scale,
            ocean_color_rgb: Some(color_to_rgb(profile.ocean_color)),
            crater_density: profile.crater_density,
            crater_rim_height: profile.crater_rim_height,
            snow_line_altitude: profile.snow_line_altitude,
            below_threshold_transparent: false,
            cloud_alpha_softness: 0.0,
            night_light_color: profile.night_light_color,
            night_light_threshold: profile.night_light_threshold,
            night_light_intensity: profile.night_light_intensity,
            heightmap: profile.generated_heightmap.clone(),
            heightmap_w: profile.generated_heightmap_w,
            heightmap_h: profile.generated_heightmap_h,
            heightmap_blend: profile.heightmap_blend,
        }
    }
}

impl FrameAtmosphereProfile {
    pub fn from_obj_params(params: &ObjRenderParams) -> Self {
        Self {
            color: params.atmo_color,
            height: params.atmo_height,
            density: params.atmo_density,
            strength: params.atmo_strength,
            rayleigh_amount: params.atmo_rayleigh_amount,
            rayleigh_color: params.atmo_rayleigh_color,
            rayleigh_falloff: params.atmo_rayleigh_falloff,
            haze_amount: params.atmo_haze_amount,
            haze_color: params.atmo_haze_color,
            haze_falloff: params.atmo_haze_falloff,
            absorption_amount: params.atmo_absorption_amount,
            absorption_color: params.atmo_absorption_color,
            absorption_height: params.atmo_absorption_height,
            absorption_width: params.atmo_absorption_width,
            forward_scatter: params.atmo_forward_scatter,
            limb_boost: params.atmo_limb_boost,
            terminator_softness: params.atmo_terminator_softness,
            night_glow: params.atmo_night_glow,
            night_glow_color: params.atmo_night_glow_color,
            haze_night_leak: params.atmo_haze_night_leak,
            rim_power: params.atmo_rim_power,
            haze_strength: params.atmo_haze_strength,
            haze_power: params.atmo_haze_power,
            veil_strength: params.atmo_veil_strength,
            veil_power: params.atmo_veil_power,
            halo_strength: params.atmo_halo_strength,
            halo_width: params.atmo_halo_width,
            halo_power: params.atmo_halo_power,
            visibility: 1.0,
        }
    }

    pub fn from_generated_world_profile(profile: &GeneratedWorldRenderProfile) -> Self {
        Self {
            color: profile.atmo_color,
            height: 0.0,
            density: 0.0,
            strength: profile.atmo_strength,
            rayleigh_amount: 0.0,
            rayleigh_color: None,
            rayleigh_falloff: 0.0,
            haze_amount: 0.0,
            haze_color: None,
            haze_falloff: 0.0,
            absorption_amount: 0.0,
            absorption_color: None,
            absorption_height: 0.0,
            absorption_width: 0.0,
            forward_scatter: 0.0,
            limb_boost: 0.0,
            terminator_softness: 0.0,
            night_glow: 0.0,
            night_glow_color: None,
            haze_night_leak: profile.haze_night_leak,
            rim_power: 0.0,
            haze_strength: 0.0,
            haze_power: 0.0,
            veil_strength: 0.0,
            veil_power: 0.0,
            halo_strength: 0.0,
            halo_width: 0.0,
            halo_power: 0.0,
            visibility: profile.atmo_visibility,
        }
    }
}

impl FrameLightingProfile {
    pub fn from_obj_params(params: &ObjRenderParams) -> Self {
        Self {
            direction_lights: Vec::new(),
            point_lights: vec![
                FramePointLightProfile {
                    position: [params.light_point_x, params.light_point_y, params.light_point_z],
                    intensity: params.light_point_intensity,
                    colour: params.light_point_colour,
                    falloff: params.light_point_falloff,
                    flicker_depth: params.light_point_flicker_depth,
                    flicker_hz: params.light_point_flicker_hz,
                    orbit_hz: params.light_point_orbit_hz,
                    snap_hz: params.light_point_snap_hz,
                },
                FramePointLightProfile {
                    position: [
                        params.light_point_2_x,
                        params.light_point_2_y,
                        params.light_point_2_z,
                    ],
                    intensity: params.light_point_2_intensity,
                    colour: params.light_point_2_colour,
                    falloff: params.light_point_2_falloff,
                    flicker_depth: params.light_point_2_flicker_depth,
                    flicker_hz: params.light_point_2_flicker_hz,
                    orbit_hz: params.light_point_2_orbit_hz,
                    snap_hz: params.light_point_2_snap_hz,
                },
            ],
            shadow_contrast: params.shadow_contrast,
            exposure: params.exposure,
            gamma: params.gamma,
            tonemap: params.tonemap,
            depth_sort_faces: params.depth_sort_faces,
        }
    }

    pub fn from_generated_world_profile(profile: &GeneratedWorldRenderProfile) -> Self {
        Self {
            direction_lights: Vec::new(),
            point_lights: Vec::new(),
            shadow_contrast: profile.shadow_contrast,
            exposure: profile.exposure,
            gamma: profile.gamma,
            tonemap: profile.tonemap,
            depth_sort_faces: false,
        }
    }
}

impl FrameEnvironmentProfile {
    pub fn from_generated_world_profile(profile: &GeneratedWorldRenderProfile) -> Self {
        Self {
            background_color: None,
            background_floor: None,
            sun_direction: Some(profile.sun_dir),
            cloud_color: Some(profile.cloud_color),
            cloud_threshold: Some(profile.cloud_threshold),
            cloud_ambient: Some(profile.cloud_ambient),
            cloud_noise_scale: Some(profile.cloud_noise_scale),
            cloud_noise_octaves: Some(profile.cloud_noise_octaves),
            cloud_scale: Some(profile.cloud_scale),
            cloud2_scale: Some(profile.cloud2_scale),
            cloud_render_scale_1: Some(profile.cloud_render_scale_1),
            cloud_render_scale_2: Some(profile.cloud_render_scale_2),
        }
    }
}

impl FramePostProcessProfile {
    pub fn from_obj_params(params: &ObjRenderParams) -> Self {
        Self {
            exposure: params.exposure,
            gamma: params.gamma,
            tonemap: params.tonemap,
            shadow_contrast: params.shadow_contrast,
        }
    }

    pub fn from_generated_world_profile(profile: &GeneratedWorldRenderProfile) -> Self {
        Self {
            exposure: profile.exposure,
            gamma: profile.gamma,
            tonemap: profile.tonemap,
            shadow_contrast: profile.shadow_contrast,
        }
    }
}

fn color_to_rgb(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb { r, g, b } => [r, g, b],
        _ => color.to_rgb().into(),
    }
}
