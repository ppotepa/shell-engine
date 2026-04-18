use engine_asset::ObjMesh;
use rayon::prelude::*;

use crate::effects::terrain::{compute_terrain_noise_at, displace_sphere_vertex};
use crate::geom::math::rotate_xyz;
use crate::geom::types::ProjectedVertex;
use crate::ObjRenderParams;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TerrainNoisePolicy {
    SurfaceOrDisplacement,
    SurfaceUnlessSoftCloudsOrDisplacement,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectionStageConfig {
    pub terrain_noise_policy: TerrainNoisePolicy,
    pub apply_smooth_normals: bool,
    pub parallel_threshold: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectionStageInput {
    pub center: [f32; 3],
    pub model_scale: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub near_clip: f32,
    pub aspect: f32,
    pub inv_tan: f32,
    pub virtual_w: u16,
    pub virtual_h: u16,
}

#[inline]
fn should_compute_terrain_noise(params: &ObjRenderParams, policy: TerrainNoisePolicy) -> bool {
    match policy {
        TerrainNoisePolicy::SurfaceOrDisplacement => {
            params.terrain_color.is_some() || params.terrain_displacement > 0.0
        }
        TerrainNoisePolicy::SurfaceUnlessSoftCloudsOrDisplacement => {
            (params.terrain_color.is_some() && params.cloud_alpha_softness <= 0.0)
                || params.terrain_displacement > 0.0
        }
    }
}

pub(crate) fn project_vertices_into(
    mesh: &ObjMesh,
    params: &ObjRenderParams,
    input: ProjectionStageInput,
    config: ProjectionStageConfig,
    projected: &mut Vec<Option<ProjectedVertex>>,
) {
    projected.clear();
    projected.reserve(mesh.vertices.len());

    let should_compute_noise = should_compute_terrain_noise(params, config.terrain_noise_policy);
    let project_vertex = |v: &[f32; 3]| {
        let centered_raw = [
            (v[0] - input.center[0]) * input.model_scale,
            (v[1] - input.center[1]) * input.model_scale,
            (v[2] - input.center[2]) * input.model_scale,
        ];
        let terrain_noise_val = if should_compute_noise {
            compute_terrain_noise_at(centered_raw, params)
        } else {
            0.0
        };
        let centered = if params.terrain_displacement > 0.0 {
            displace_sphere_vertex(centered_raw, terrain_noise_val, params.terrain_displacement)
        } else {
            centered_raw
        };
        let rotated = rotate_xyz(centered, input.pitch, input.yaw, input.roll);
        let translated = [
            rotated[0] + params.object_translate_x,
            rotated[1] + params.object_translate_y,
            rotated[2] + params.object_translate_z,
        ];
        let rel = [
            translated[0] - params.camera_world_x,
            translated[1] - params.camera_world_y,
            translated[2] - params.camera_world_z,
        ];
        let cam_x = rel[0] * params.view_right_x
            + rel[1] * params.view_right_y
            + rel[2] * params.view_right_z
            - params.camera_pan_x;
        let cam_y =
            rel[0] * params.view_up_x + rel[1] * params.view_up_y + rel[2] * params.view_up_z
                - params.camera_pan_y;
        let view_z = rel[0] * params.view_forward_x
            + rel[1] * params.view_forward_y
            + rel[2] * params.view_forward_z;
        if view_z <= input.near_clip {
            return None;
        }
        let ndc_x = (cam_x / input.aspect) * input.inv_tan / view_z;
        let ndc_y = cam_y * input.inv_tan / view_z;
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return None;
        }

        Some(ProjectedVertex {
            x: (ndc_x + 1.0) * 0.5 * (input.virtual_w as f32 - 1.0),
            y: (1.0 - (ndc_y + 1.0) * 0.5) * (input.virtual_h as f32 - 1.0),
            depth: view_z,
            view: translated,
            normal: [0.0, 0.0, 1.0],
            local: centered,
            terrain_noise: terrain_noise_val,
        })
    };

    if mesh.vertices.len() > config.parallel_threshold {
        mesh.vertices
            .par_iter()
            .map(project_vertex)
            .collect_into_vec(projected);
    } else {
        projected.extend(mesh.vertices.iter().map(project_vertex));
    }

    if config.apply_smooth_normals && !mesh.smooth_normals.is_empty() {
        for (i, pv_opt) in projected.iter_mut().enumerate() {
            if let Some(pv) = pv_opt.as_mut() {
                if let Some(&n) = mesh.smooth_normals.get(i) {
                    pv.normal = rotate_xyz(n, input.pitch, input.yaw, input.roll);
                }
            }
        }
    }
}
