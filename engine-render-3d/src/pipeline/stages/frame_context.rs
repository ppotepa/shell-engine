use engine_core::color::Color;

use crate::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use crate::effects::passes::planet_params::{build_biome_params, build_terrain_extra_params};
use crate::geom::math::{dot3, normalize3};
use crate::pipeline::stages::raster_exec::{GouraudRgbRasterContext, GouraudRgbaRasterContext};
use crate::pipeline::stages::shade::FlatShadingStageContext;
use crate::ObjRenderParams;

#[derive(Clone, Copy)]
pub(crate) struct FrameLightingContext {
    pub light_dir_norm: [f32; 3],
    pub light_2_dir_norm: [f32; 3],
    pub view_dir: [f32; 3],
    pub half_dir_1: [f32; 3],
    pub half_dir_2: [f32; 3],
}

#[derive(Clone, Copy)]
pub(crate) struct FrameShadingContext {
    pub lighting: FrameLightingContext,
    pub fg_rgb: [u8; 3],
    pub unlit: bool,
    pub ambient: f32,
    pub ambient_floor: f32,
    pub light_2_intensity: f32,
    pub light_point_falloff: f32,
    pub light_point_2_falloff: f32,
    pub cel_levels: u8,
    pub tone_mix: f32,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
    pub shadow_colour: Option<Color>,
    pub midtone_colour: Option<Color>,
    pub highlight_colour: Option<Color>,
    pub light_point_colour: Option<Color>,
    pub light_point_2_colour: Option<Color>,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub terrain_noise_scale: f32,
    pub terrain_noise_octaves: u8,
    pub marble_depth: f32,
    pub terrain_relief: f32,
    pub below_threshold_transparent: bool,
    pub cloud_alpha_softness: f32,
    pub biome: Option<PlanetBiomeParams>,
    pub terrain_extra: Option<PlanetTerrainParams>,
}

impl FrameShadingContext {
    pub fn from_params(params: &ObjRenderParams, fg: Color) -> Self {
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
        let lighting = FrameLightingContext {
            light_dir_norm,
            light_2_dir_norm,
            view_dir,
            half_dir_1: normalize3([
                light_dir_norm[0] + view_dir[0],
                light_dir_norm[1] + view_dir[1],
                light_dir_norm[2] + view_dir[2],
            ]),
            half_dir_2: normalize3([
                light_2_dir_norm[0] + view_dir[0],
                light_2_dir_norm[1] + view_dir[1],
                light_2_dir_norm[2] + view_dir[2],
            ]),
        };
        Self {
            lighting,
            fg_rgb: crate::shading::color_to_rgb(fg),
            unlit: params.unlit,
            ambient: params.ambient,
            ambient_floor: params.ambient_floor,
            light_2_intensity: params.light_2_intensity,
            light_point_falloff: params.light_point_falloff,
            light_point_2_falloff: params.light_point_2_falloff,
            cel_levels: params.cel_levels,
            tone_mix: params.tone_mix,
            latitude_bands: params.latitude_bands,
            latitude_band_depth: params.latitude_band_depth,
            shadow_colour: params.shadow_colour,
            midtone_colour: params.midtone_colour,
            highlight_colour: params.highlight_colour,
            light_point_colour: params.light_point_colour,
            light_point_2_colour: params.light_point_2_colour,
            terrain_color: params.terrain_color,
            terrain_threshold: params.terrain_threshold,
            terrain_noise_scale: params.terrain_noise_scale,
            terrain_noise_octaves: params.terrain_noise_octaves,
            marble_depth: params.marble_depth,
            terrain_relief: params.terrain_relief,
            below_threshold_transparent: params.below_threshold_transparent,
            cloud_alpha_softness: params.cloud_alpha_softness,
            biome: build_biome_params(params, light_dir_norm, view_dir),
            terrain_extra: build_terrain_extra_params(params),
        }
    }

    #[inline]
    pub fn shade_at_vertex(&self, normal: [f32; 3]) -> f32 {
        let ka_lum_ambient = self.ambient.max(self.ambient_floor);
        let light_2_strength = self.light_2_intensity.clamp(0.0, 2.0);
        let lambert_1 = dot3(normal, self.lighting.light_dir_norm).max(0.0);
        let lambert_2 = dot3(normal, self.lighting.light_2_dir_norm).max(0.0) * light_2_strength;
        let lambert = (lambert_1 + lambert_2).clamp(0.0, 1.0);
        (ka_lum_ambient + (1.0 - ka_lum_ambient) * lambert * 0.9).clamp(0.0, 1.0)
    }

    pub fn flat_stage_context(
        &self,
        light_1_pos: [f32; 3],
        light_point_intensity: f32,
        light_2_pos: [f32; 3],
        light_point_2_intensity: f32,
    ) -> FlatShadingStageContext {
        FlatShadingStageContext {
            unlit: self.unlit,
            fg_rgb: self.fg_rgb,
            light_dir_norm: self.lighting.light_dir_norm,
            light_2_dir_norm: self.lighting.light_2_dir_norm,
            half_dir_1: self.lighting.half_dir_1,
            half_dir_2: self.lighting.half_dir_2,
            light_2_intensity: self.light_2_intensity,
            light_1_pos,
            light_point_intensity,
            light_2_pos,
            light_point_2_intensity,
            cel_levels: self.cel_levels,
            tone_mix: self.tone_mix,
            ambient: self.ambient,
            view_dir: self.lighting.view_dir,
            light_point_falloff: self.light_point_falloff,
            light_point_2_falloff: self.light_point_2_falloff,
            shadow_colour: self.shadow_colour,
            midtone_colour: self.midtone_colour,
            highlight_colour: self.highlight_colour,
            light_point_colour: self.light_point_colour,
            light_point_2_colour: self.light_point_2_colour,
        }
    }

    pub fn gouraud_rgb_raster_context(
        &self,
        virtual_w: u16,
        virtual_h: u16,
    ) -> GouraudRgbRasterContext {
        GouraudRgbRasterContext {
            virtual_w,
            virtual_h,
            shadow_colour: self.shadow_colour,
            midtone_colour: self.midtone_colour,
            highlight_colour: self.highlight_colour,
            tone_mix: self.tone_mix,
            cel_levels: self.cel_levels,
            latitude_bands: self.latitude_bands,
            latitude_band_depth: self.latitude_band_depth,
            terrain_color: self.terrain_color,
            terrain_threshold: self.terrain_threshold,
            marble_depth: self.marble_depth,
            terrain_relief: self.terrain_relief,
            below_threshold_transparent: self.below_threshold_transparent,
            biome: self.biome,
            terrain_extra: self.terrain_extra,
        }
    }

    pub fn gouraud_rgba_raster_context(
        &self,
        virtual_w: u16,
        virtual_h: u16,
    ) -> GouraudRgbaRasterContext {
        GouraudRgbaRasterContext {
            virtual_w,
            virtual_h,
            cel_levels: self.cel_levels,
            terrain_color: self.terrain_color,
            terrain_threshold: self.terrain_threshold,
            terrain_noise_scale: self.terrain_noise_scale,
            terrain_noise_octaves: self.terrain_noise_octaves,
            below_threshold_transparent: self.below_threshold_transparent,
            cloud_alpha_softness: self.cloud_alpha_softness,
            biome: self.biome,
            marble_depth: self.marble_depth,
            shadow_colour: self.shadow_colour,
            midtone_colour: self.midtone_colour,
            highlight_colour: self.highlight_colour,
            tone_mix: self.tone_mix,
            latitude_bands: self.latitude_bands,
            latitude_band_depth: self.latitude_band_depth,
        }
    }
}
