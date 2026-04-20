use engine_asset::ObjMesh;

use crate::geom::types::ProjectedVertex;
use crate::pipeline::stages::classify::{classify_and_sort_faces_into, FaceClassificationConfig};
use crate::pipeline::stages::frame_context::FrameShadingContext;
use crate::pipeline::stages::shade::{prepare_gouraud_faces_into, GouraudFace};

pub(crate) fn prepare_visible_gouraud_faces_into(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    backface_cull: bool,
    depth_sort_faces: bool,
    min_projected_face_double_area: f32,
    max_faces: usize,
    frame_ctx: FrameShadingContext,
    sorted_faces: &mut Vec<(f32, usize)>,
    shaded_gouraud: &mut Vec<GouraudFace>,
) -> usize {
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
    prepare_gouraud_faces_into(
        mesh,
        sorted_faces,
        face_limit,
        projected,
        frame_ctx.unlit,
        frame_ctx.fg_rgb,
        |normal| frame_ctx.shade_at_vertex(normal),
        shaded_gouraud,
    );
    face_limit
}
