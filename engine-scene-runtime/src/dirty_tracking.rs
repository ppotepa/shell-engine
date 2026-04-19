use engine_core::render_types::DirtyMask3D;

use crate::mutations::{Render3DGroupedParam, Render3DMutation, Render3DProfileParam, SceneMutation};

pub fn dirty_for_render3d_mutation(mutation: &Render3DMutation) -> DirtyMask3D {
    match mutation {
        Render3DMutation::SetGroupedParams { params, .. } => params.iter().fold(
            DirtyMask3D::empty(),
            |mask, (param, _)| mask | dirty_for_grouped_param(param),
        ),
        Render3DMutation::SetProfile { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetViewProfile { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetLightingProfile { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetSpaceEnvironmentProfile { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetProfileParam { param, .. } => match param {
            Render3DProfileParam::Lighting(_) => DirtyMask3D::LIGHTING,
            Render3DProfileParam::SpaceEnvironment(_) => DirtyMask3D::LIGHTING,
        },
        Render3DMutation::SetLightingParam { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetSpaceEnvironmentParam { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::SetNodeTransform { .. } => DirtyMask3D::TRANSFORM,
        Render3DMutation::SetNodeVisibility { .. } => DirtyMask3D::VISIBILITY,
        Render3DMutation::SetObjMaterialParam { .. } => DirtyMask3D::MATERIAL,
        Render3DMutation::SetAtmosphereParamTyped { .. } => DirtyMask3D::ATMOSPHERE,
        Render3DMutation::SetTerrainParamTyped { .. } => DirtyMask3D::WORLDGEN,
        Render3DMutation::SetWorldgenParamTyped { .. } => DirtyMask3D::WORLDGEN,
        Render3DMutation::SetPlanetParamTyped { .. } => DirtyMask3D::MATERIAL,
        Render3DMutation::SetScene3DFrame { .. } => DirtyMask3D::WORLDGEN,
        Render3DMutation::SetSceneCamera { .. } => DirtyMask3D::CAMERA,
        Render3DMutation::SetLight { .. } => DirtyMask3D::LIGHTING,
        Render3DMutation::RebuildMesh { .. } => DirtyMask3D::MESH,
        Render3DMutation::RebuildWorldgen { .. } => DirtyMask3D::WORLDGEN | DirtyMask3D::MESH,
    }
}

fn dirty_for_grouped_param(param: &Render3DGroupedParam) -> DirtyMask3D {
    match param {
        Render3DGroupedParam::Material(_) => DirtyMask3D::MATERIAL,
        Render3DGroupedParam::Atmosphere(_) => DirtyMask3D::ATMOSPHERE,
        Render3DGroupedParam::Surface(_) => DirtyMask3D::WORLDGEN,
        Render3DGroupedParam::Generator(_) => DirtyMask3D::WORLDGEN,
        Render3DGroupedParam::Body(_) => DirtyMask3D::MATERIAL,
        Render3DGroupedParam::View(_) => DirtyMask3D::MATERIAL,
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
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::Material(crate::mutations::ObjMaterialParam::Scale),
                        engine_core::render_types::MaterialValue::Scalar(0.8),
                    )],
                },
                DirtyMask3D::MATERIAL,
            ),
            (
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::Atmosphere(crate::mutations::AtmosphereParam::Density),
                        engine_core::render_types::MaterialValue::Scalar(1.2),
                    )],
                },
                DirtyMask3D::ATMOSPHERE,
            ),
            (
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::Surface(crate::mutations::TerrainParam::Amplitude),
                        engine_core::render_types::MaterialValue::Scalar(0.5),
                    )],
                },
                DirtyMask3D::WORLDGEN,
            ),
            (
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::Generator(crate::mutations::WorldgenParam::Seed),
                        engine_core::render_types::MaterialValue::Scalar(42.0),
                    )],
                },
                DirtyMask3D::WORLDGEN,
            ),
            (
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::Body(crate::mutations::PlanetParam::SpinDeg),
                        engine_core::render_types::MaterialValue::Scalar(12.0),
                    )],
                },
                DirtyMask3D::MATERIAL,
            ),
            (
                Render3DMutation::SetGroupedParams {
                    target: Some("planet".to_string()),
                    params: vec![(
                        crate::Render3DGroupedParam::View(crate::mutations::ViewParam::Distance),
                        engine_core::render_types::MaterialValue::Scalar(0.85),
                    )],
                },
                DirtyMask3D::MATERIAL,
            ),
            (
                Render3DMutation::SetProfile {
                    slot: crate::Render3DProfileSlot::View,
                    profile: "orbit-realistic".to_string(),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetViewProfile {
                    profile: "orbit-realistic".to_string(),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetLightingProfile {
                    profile: "space-hard-vacuum".to_string(),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetSpaceEnvironmentProfile {
                    profile: "deep-space-sparse".to_string(),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetProfileParam {
                    param: crate::Render3DProfileParam::Lighting(
                        crate::mutations::LightingProfileParam::Exposure,
                    ),
                    value: engine_core::render_types::MaterialValue::Scalar(0.85),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetLightingParam {
                    param: crate::mutations::LightingProfileParam::Exposure,
                    value: engine_core::render_types::MaterialValue::Scalar(0.85),
                },
                DirtyMask3D::LIGHTING,
            ),
            (
                Render3DMutation::SetSpaceEnvironmentParam {
                    param: crate::mutations::SpaceEnvironmentParam::StarfieldBrightness,
                    value: engine_core::render_types::MaterialValue::Scalar(0.75),
                },
                DirtyMask3D::LIGHTING,
            ),
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
                Render3DMutation::SetObjMaterialParam {
                    target: "planet".to_string(),
                    param: crate::mutations::ObjMaterialParam::Scale,
                    value: engine_core::render_types::MaterialValue::Scalar(0.8),
                },
                DirtyMask3D::MATERIAL,
            ),
            (
                Render3DMutation::SetAtmosphereParamTyped {
                    target: "planet".to_string(),
                    param: crate::mutations::AtmosphereParam::Density,
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
                Render3DMutation::SetWorldgenParamTyped {
                    target: "planet".to_string(),
                    param: crate::mutations::WorldgenParam::Seed,
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
