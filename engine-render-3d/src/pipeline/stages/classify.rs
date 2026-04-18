use engine_asset::{ObjFace, ObjMesh};

use crate::geom::raster::edge;
use crate::geom::types::ProjectedVertex;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FaceClassificationConfig {
    pub backface_cull: bool,
    pub depth_sort_faces: bool,
    pub min_projected_face_double_area: f32,
    pub max_faces: usize,
}

#[inline(always)]
fn face_avg_depth(projected: &[Option<ProjectedVertex>], face: &ObjFace) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for &i in &face.indices {
        if let Some(Some(v)) = projected.get(i) {
            sum += v.depth;
            count += 1;
        }
    }
    if count == 0 {
        f32::INFINITY
    } else {
        sum / count as f32
    }
}

pub(crate) fn classify_and_sort_faces_into(
    mesh: &ObjMesh,
    projected: &[Option<ProjectedVertex>],
    config: FaceClassificationConfig,
    sorted_faces: &mut Vec<(f32, usize)>,
) -> usize {
    sorted_faces.clear();
    sorted_faces.reserve(mesh.faces.len());

    for (face_idx, face) in mesh.faces.iter().enumerate() {
        let v0 = projected.get(face.indices[0]).and_then(|p| *p);
        let v1 = projected.get(face.indices[1]).and_then(|p| *p);
        let v2 = projected.get(face.indices[2]).and_then(|p| *p);
        let (Some(v0), Some(v1), Some(v2)) = (v0, v1, v2) else {
            continue;
        };
        let projected_area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
        if config.backface_cull && projected_area < 0.0 {
            continue;
        }
        if projected_area.abs() < config.min_projected_face_double_area {
            continue;
        }
        let key = if config.depth_sort_faces {
            face_avg_depth(projected, face)
        } else {
            0.0
        };
        sorted_faces.push((key, face_idx));
    }

    if config.depth_sort_faces {
        sorted_faces
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    sorted_faces.len().min(config.max_faces)
}
