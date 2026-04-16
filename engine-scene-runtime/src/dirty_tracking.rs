use engine_core::render_types::DirtyMask3D;

use crate::mutations::{Render3DMutation, SceneMutation};

pub fn dirty_for_render3d_mutation(mutation: &Render3DMutation) -> DirtyMask3D {
    match mutation {
        Render3DMutation::SetNodeTransform { .. } => DirtyMask3D::TRANSFORM,
        Render3DMutation::SetNodeVisibility { .. } => DirtyMask3D::VISIBILITY,
        Render3DMutation::SetMaterialParam { .. } => DirtyMask3D::MATERIAL,
        Render3DMutation::SetAtmosphereParam { .. } => DirtyMask3D::ATMOSPHERE,
        Render3DMutation::SetWorldgenParam { .. } => DirtyMask3D::WORLDGEN,
        Render3DMutation::SetSceneCamera { .. } => DirtyMask3D::CAMERA,
        Render3DMutation::SetLight { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::RebuildMesh { .. } => DirtyMask3D::MESH,
        Render3DMutation::RebuildWorldgen { .. } => DirtyMask3D::WORLDGEN | DirtyMask3D::MESH,
    }
}

pub fn dirty_for_scene_mutation(mutation: &SceneMutation) -> DirtyMask3D {
    match mutation {
        SceneMutation::SetRender3D(inner) => dirty_for_render3d_mutation(inner),
        SceneMutation::SetCamera3D(_) => DirtyMask3D::CAMERA,
        _ => DirtyMask3D::empty(),
    }
}

#[cfg(test)]
mod tests {
    use engine_core::render_types::{Camera3DState, DirtyMask3D};

    use super::*;

    #[test]
    fn maps_render3d_mutations_to_expected_dirty_masks() {
        let material = Render3DMutation::SetMaterialParam {
            target: "planet".to_string(),
            param: "albedo".to_string(),
            value: engine_core::render_types::MaterialValue::Scalar(0.8),
        };
        let rebuild_world = Render3DMutation::RebuildWorldgen {
            target: "planet".to_string(),
        };

        assert_eq!(dirty_for_render3d_mutation(&material), DirtyMask3D::MATERIAL);
        assert_eq!(
            dirty_for_render3d_mutation(&rebuild_world),
            DirtyMask3D::WORLDGEN | DirtyMask3D::MESH
        );
    }

    #[test]
    fn scene_mutation_returns_empty_mask_for_non_3d_cases() {
        let scene_only = SceneMutation::SpawnObject {
            template: "ship".to_string(),
            target: "ship-1".to_string(),
        };
        let camera = SceneMutation::SetCamera3D(Camera3DState::default());

        assert_eq!(dirty_for_scene_mutation(&scene_only), DirtyMask3D::empty());
        assert_eq!(dirty_for_scene_mutation(&camera), DirtyMask3D::CAMERA);
    }
}
