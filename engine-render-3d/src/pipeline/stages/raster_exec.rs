use rayon::prelude::*;

use crate::effects::params::{PlanetBiomeParams, PlanetTerrainParams};
use crate::effects::passes::surface::{rasterize_triangle_gouraud, rasterize_triangle_gouraud_rgba};
use crate::pipeline::stages::shade::{FlatFace, GouraudFace};
use crate::raster::rasterize_triangle;

#[derive(Clone, Copy)]
pub(crate) struct GouraudRgbRasterContext {
    pub virtual_w: u16,
    pub virtual_h: u16,
    pub shadow_colour: Option<engine_core::color::Color>,
    pub midtone_colour: Option<engine_core::color::Color>,
    pub highlight_colour: Option<engine_core::color::Color>,
    pub tone_mix: f32,
    pub cel_levels: u8,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub marble_depth: f32,
    pub terrain_relief: f32,
    pub below_threshold_transparent: bool,
    pub biome: Option<PlanetBiomeParams>,
    pub terrain_extra: Option<PlanetTerrainParams>,
}

#[derive(Clone, Copy)]
pub(crate) struct GouraudRgbaRasterContext {
    pub virtual_w: u16,
    pub virtual_h: u16,
    pub cel_levels: u8,
    pub terrain_color: Option<[u8; 3]>,
    pub terrain_threshold: f32,
    pub terrain_noise_scale: f32,
    pub terrain_noise_octaves: u8,
    pub below_threshold_transparent: bool,
    pub cloud_alpha_softness: f32,
    pub biome: Option<PlanetBiomeParams>,
    pub marble_depth: f32,
    pub shadow_colour: Option<engine_core::color::Color>,
    pub midtone_colour: Option<engine_core::color::Color>,
    pub highlight_colour: Option<engine_core::color::Color>,
    pub tone_mix: f32,
    pub latitude_bands: u8,
    pub latitude_band_depth: f32,
}

pub(crate) fn execute_gouraud_rgb_faces(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    faces: &[GouraudFace],
    ctx: GouraudRgbRasterContext,
    clip_min_y: i32,
    clip_max_y: i32,
    row_base: i32,
) {
    for (v0, v1, v2, base_color, s0, s1, s2) in faces {
        rasterize_triangle_gouraud(
            canvas,
            depth,
            ctx.virtual_w,
            ctx.virtual_h,
            *v0,
            *v1,
            *v2,
            *base_color,
            *s0,
            *s1,
            *s2,
            ctx.shadow_colour,
            ctx.midtone_colour,
            ctx.highlight_colour,
            ctx.tone_mix,
            ctx.cel_levels,
            ctx.latitude_bands,
            ctx.latitude_band_depth,
            ctx.terrain_color,
            ctx.terrain_threshold,
            ctx.marble_depth,
            ctx.terrain_relief,
            ctx.below_threshold_transparent,
            ctx.biome,
            ctx.terrain_extra,
            clip_min_y,
            clip_max_y,
            row_base,
        );
    }
}

pub(crate) fn execute_flat_rgb_faces(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    faces: &[FlatFace],
    virtual_w: u16,
    virtual_h: u16,
    clip_min_y: i32,
    clip_max_y: i32,
) {
    for (v0, v1, v2, shaded_color) in faces {
        rasterize_triangle(
            canvas,
            depth,
            virtual_w,
            virtual_h,
            *v0,
            *v1,
            *v2,
            *shaded_color,
            clip_min_y,
            clip_max_y,
        );
    }
}

pub(crate) fn build_parallel_canvas_strips<'a, T>(
    canvas: &'a mut [Option<T>],
    depth: &'a mut [f32],
    row_w: usize,
    virtual_h: u16,
) -> Vec<(i32, &'a mut [Option<T>], &'a mut [f32])> {
    let num_strips = rayon::current_num_threads().max(1);
    let strip_rows = ((virtual_h as usize) + num_strips - 1) / num_strips;
    canvas
        .chunks_mut(strip_rows * row_w)
        .zip(depth.chunks_mut(strip_rows * row_w))
        .enumerate()
        .map(|(i, (cs, ds))| ((i * strip_rows) as i32, cs, ds))
        .collect()
}

pub(crate) fn execute_gouraud_rgb_faces_parallel_strips(
    canvas_strips: &mut [(i32, &mut [Option<[u8; 3]>], &mut [f32])],
    faces: &[GouraudFace],
    row_w: usize,
    clip_min_y: i32,
    clip_max_y: i32,
    ctx: GouraudRgbRasterContext,
) {
    canvas_strips.par_iter_mut().for_each(|(strip_y0, cs, ds)| {
        let strip_y1 = *strip_y0 + (cs.len() / row_w) as i32 - 1;
        let strip_clip_min = (*strip_y0).max(clip_min_y);
        let strip_clip_max = strip_y1.min(clip_max_y);
        if strip_clip_min > strip_clip_max {
            return;
        }
        execute_gouraud_rgb_faces(
            cs,
            ds,
            faces,
            ctx,
            strip_clip_min,
            strip_clip_max,
            *strip_y0,
        );
    });
}

pub(crate) fn execute_gouraud_rgba_faces_parallel_strips(
    canvas_strips: &mut [(i32, &mut [Option<[u8; 4]>], &mut [f32])],
    faces: &[GouraudFace],
    row_w: usize,
    clip_min_y: i32,
    clip_max_y: i32,
    ctx: GouraudRgbaRasterContext,
) {
    canvas_strips.par_iter_mut().for_each(|(strip_y0, cs, ds)| {
        let strip_y1 = *strip_y0 + (cs.len() / row_w) as i32 - 1;
        let strip_clip_min = (*strip_y0).max(clip_min_y);
        let strip_clip_max = strip_y1.min(clip_max_y);
        if strip_clip_min > strip_clip_max {
            return;
        }
        for (v0, v1, v2, base_color, s0, s1, s2) in faces {
            rasterize_triangle_gouraud_rgba(
                cs,
                ds,
                ctx.virtual_w,
                ctx.virtual_h,
                *v0,
                *v1,
                *v2,
                *base_color,
                *s0,
                *s1,
                *s2,
                ctx.cel_levels,
                ctx.terrain_color,
                ctx.terrain_threshold,
                ctx.terrain_noise_scale,
                ctx.terrain_noise_octaves,
                ctx.below_threshold_transparent,
                ctx.cloud_alpha_softness,
                ctx.biome,
                strip_clip_min,
                strip_clip_max,
                *strip_y0,
                ctx.marble_depth,
                ctx.shadow_colour,
                ctx.midtone_colour,
                ctx.highlight_colour,
                ctx.tone_mix,
                ctx.latitude_bands,
                ctx.latitude_band_depth,
            );
        }
    });
}
