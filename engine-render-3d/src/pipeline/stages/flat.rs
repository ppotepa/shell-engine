use engine_asset::ObjMesh;

use crate::geom::types::ProjectedVertex;
use crate::pipeline::stages::classify::{
    classify_and_sort_faces_into, FaceClassificationConfig,
};
use crate::pipeline::stages::raster_exec::execute_flat_rgb_faces;
use crate::pipeline::stages::shade::{prepare_flat_faces_into, FlatShadingStageContext};

pub(crate) fn render_flat_rgb_solid(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    virtual_w: u16,
    virtual_h: u16,
    clip_min_y: i32,
    clip_max_y: i32,
    backface_cull: bool,
    depth_sort_faces: bool,
    min_projected_face_double_area: f32,
    max_faces: usize,
    shade_ctx: FlatShadingStageContext,
    sorted_faces: &mut Vec<(f32, usize)>,
    shaded_faces: &mut Vec<(ProjectedVertex, ProjectedVertex, ProjectedVertex, [u8; 3])>,
) -> (u32, u32) {
    let face_limit = classify_and_sort_faces_into(
        mesh,
        projected,
        FaceClassificationConfig {
            backface_cull,
            depth_sort_faces,
            min_projected_face_double_area,
            max_faces,
        },
        sorted_faces,
    );
    prepare_flat_faces_into(
        mesh,
        sorted_faces,
        face_limit,
        projected,
        shade_ctx,
        shaded_faces,
    );
    execute_flat_rgb_faces(
        canvas,
        depth,
        shaded_faces,
        virtual_w,
        virtual_h,
        clip_min_y,
        clip_max_y,
    );
    (face_limit as u32, shaded_faces.len() as u32)
}
