use engine_asset::ObjMesh;
use rayon::prelude::*;

use crate::geom::types::ProjectedVertex;
use crate::shading::{
    apply_point_light_tint, apply_shading, apply_tone_palette, face_shading_with_specular,
};

pub(crate) type GouraudFace = (
    ProjectedVertex,
    ProjectedVertex,
    ProjectedVertex,
    [u8; 3],
    f32,
    f32,
    f32,
);

pub(crate) type FlatFace = (ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3]);

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlatShadingStageContext {
    pub unlit: bool,
    pub fg_rgb: [u8; 3],
    pub light_dir_norm: [f32; 3],
    pub light_2_dir_norm: [f32; 3],
    pub half_dir_1: [f32; 3],
    pub half_dir_2: [f32; 3],
    pub light_2_intensity: f32,
    pub light_1_pos: [f32; 3],
    pub light_point_intensity: f32,
    pub light_2_pos: [f32; 3],
    pub light_point_2_intensity: f32,
    pub cel_levels: u8,
    pub tone_mix: f32,
    pub ambient: f32,
    pub view_dir: [f32; 3],
    pub light_point_falloff: f32,
    pub light_point_2_falloff: f32,
    pub shadow_colour: Option<engine_core::color::Color>,
    pub midtone_colour: Option<engine_core::color::Color>,
    pub highlight_colour: Option<engine_core::color::Color>,
    pub light_point_colour: Option<engine_core::color::Color>,
    pub light_point_2_colour: Option<engine_core::color::Color>,
}

pub(crate) fn prepare_gouraud_faces_into<F>(
    mesh: &ObjMesh,
    sorted_faces: &[(f32, usize)],
    face_limit: usize,
    projected: &[Option<ProjectedVertex>],
    unlit: bool,
    fg_rgb: [u8; 3],
    shade_at_vertex: F,
    shaded_gouraud: &mut Vec<GouraudFace>,
) where
    F: Fn([f32; 3]) -> f32 + Sync,
{
    shaded_gouraud.clear();
    shaded_gouraud.par_extend(
        sorted_faces[..face_limit]
            .par_iter()
            .filter_map(|(_, face_idx)| {
                let face = &mesh.faces[*face_idx];
                let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                let (s0, s1, s2) = if unlit {
                    (1.0, 1.0, 1.0)
                } else {
                    (
                        shade_at_vertex(v0.normal),
                        shade_at_vertex(v1.normal),
                        shade_at_vertex(v2.normal),
                    )
                };
                let base_color = if unlit { fg_rgb } else { face.color };
                Some((v0, v1, v2, base_color, s0, s1, s2))
            }),
    );
}

pub(crate) fn prepare_flat_faces_into(
    mesh: &ObjMesh,
    sorted_faces: &[(f32, usize)],
    face_limit: usize,
    projected: &[Option<ProjectedVertex>],
    ctx: FlatShadingStageContext,
    shaded_faces: &mut Vec<FlatFace>,
) {
    shaded_faces.clear();
    shaded_faces.par_extend(
        sorted_faces[..face_limit]
            .par_iter()
            .filter_map(|(_, face_idx)| {
                let face = &mesh.faces[*face_idx];
                let v0 = projected.get(face.indices[0]).and_then(|p| *p)?;
                let v1 = projected.get(face.indices[1]).and_then(|p| *p)?;
                let v2 = projected.get(face.indices[2]).and_then(|p| *p)?;
                if ctx.unlit {
                    return Some((v0, v1, v2, ctx.fg_rgb));
                }
                let shading = face_shading_with_specular(
                    v0.view,
                    v1.view,
                    v2.view,
                    face.ka,
                    face.ks,
                    face.ns,
                    ctx.light_dir_norm,
                    ctx.light_2_dir_norm,
                    ctx.half_dir_1,
                    ctx.half_dir_2,
                    ctx.light_2_intensity,
                    ctx.light_1_pos,
                    ctx.light_point_intensity,
                    ctx.light_2_pos,
                    ctx.light_point_2_intensity,
                    ctx.cel_levels,
                    ctx.tone_mix,
                    ctx.ambient,
                    ctx.view_dir,
                    ctx.light_point_falloff,
                    ctx.light_point_2_falloff,
                );
                let shaded_base = apply_shading(face.color, shading.0);
                let toned_color = apply_tone_palette(
                    shaded_base,
                    shading.1,
                    ctx.shadow_colour,
                    ctx.midtone_colour,
                    ctx.highlight_colour,
                    ctx.tone_mix,
                );
                let shaded_color = apply_point_light_tint(
                    toned_color,
                    ctx.light_point_colour,
                    shading.2,
                    ctx.light_point_2_colour,
                    shading.3,
                );
                Some((v0, v1, v2, shaded_color))
            }),
    );
}
