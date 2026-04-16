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
        let cases = vec![
            (
                Render3DMutation::SetNodeTransform {
                    target: "planet".to_string(),
                    transform: engine_core::render_types::Transform3D::default(),
                },
                DirtyMask3D::TRANSFORM,
            ),
            (
                Render3DMutation::SetNodeVisibility {
                    target: "planet".to_string(),
                    visible: true,
                },
                DirtyMask3D::VISIBILITY,
            ),
            (
                Render3DMutation::SetMaterialParam {
                    target: "planet".to_string(),
                    param: "albedo".to_string(),
                    value: engine_core::render_types::MaterialValue::Scalar(0.8),
                },
                DirtyMask3D::MATERIAL,
            ),
            (
                Render3DMutation::SetAtmosphereParam {
                    target: "planet".to_string(),
                    param: "density".to_string(),
                    value: engine_core::render_types::MaterialValue::Scalar(1.2),
                },
                DirtyMask3D::ATMOSPHERE,
            ),
            (
                Render3DMutation::SetLight {
                    index: 0,
                    light: engine_core::render_types::Light3D::default(),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetSceneCamera {
                    camera: Camera3DState::default(),
                },
                DirtyMask3D::CAMERA,
            ),
            (
                Render3DMutation::SetWorldgenParam {
                    target: "planet".to_string(),
                    param: "seed".to_string(),
                    value: engine_core::render_types::MaterialValue::Scalar(42.0),
                },
                DirtyMask3D::WORLDGEN,
            ),
            (
                Render3DMutation::RebuildMesh {
                    target: "planet".to_string(),
                },
                DirtyMask3D::MESH,
            ),
            (
                Render3DMutation::RebuildWorldgen {
                    target: "planet".to_string(),
                },
                DirtyMask3D::WORLDGEN | DirtyMask3D::MESH,
            ),
        ];

        for (mutation, expected) in cases {
            assert_eq!(dirty_for_render3d_mutation(&mutation), expected);
        }
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
