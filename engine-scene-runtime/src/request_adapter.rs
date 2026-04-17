use engine_api::scene::{Camera3dMutationRequest, Render3dMutationRequest, SceneMutationRequest};
use engine_core::render_types::{Camera3DState, MaterialValue, Transform3D};
use engine_core::scene_runtime_types::SceneCamera3D;

use crate::render3d_state::{material_value_from_json, scene_mutation_from_render_path};
use crate::{
    LightingProfileParam, Render3DMutation, SceneMutation, Set2DPropsMutation,
    SetCamera2DMutation, SetSpritePropertyMutation, SpaceEnvironmentParam,
};

pub fn scene_mutation_from_request(
    request: &SceneMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Option<SceneMutation> {
    match request {
        SceneMutationRequest::Set2dProps {
            target,
            visible,
            dx,
            dy,
            text,
        } => Some(SceneMutation::Set2DProps(Set2DPropsMutation {
            target: target.clone(),
            visible: *visible,
            dx: *dx,
            dy: *dy,
            text: text.clone(),
        })),
        SceneMutationRequest::SetSpriteProperty {
            target,
            path,
            value,
        } => {
            let mutation = match path.as_str() {
                "transform.heading" => Some(SetSpritePropertyMutation::Heading {
                    heading: value.as_f64()? as f32,
                }),
                "text.font" => Some(SetSpritePropertyMutation::TextFont {
                    font: value.as_str()?.to_string(),
                }),
                "style.fg" | "text.fg" => Some(SetSpritePropertyMutation::TextColour {
                    fg: true,
                    value: value.clone(),
                }),
                "style.bg" | "text.bg" => Some(SetSpritePropertyMutation::TextColour {
                    fg: false,
                    value: value.clone(),
                }),
                "vector.points" | "vector.closed" | "vector.draw_char" | "vector.fg"
                | "vector.bg" | "style.border" | "style.shadow" => {
                    Some(SetSpritePropertyMutation::VectorProperty {
                        path: path.to_string(),
                        value: value.clone(),
                    })
                }
                "image.frame_index" => Some(SetSpritePropertyMutation::ImageFrame {
                    frame_index: value.as_u64().and_then(|v| u16::try_from(v).ok())?,
                }),
                _ => None,
            };
            let mutation = mutation?;
            Some(SceneMutation::SetSpriteProperty {
                target: target.clone(),
                mutation,
            })
        }
        SceneMutationRequest::SetCamera2d { x, y, zoom } => {
            Some(SceneMutation::SetCamera2D(SetCamera2DMutation {
                x: x.round() as i32,
                y: y.round() as i32,
                zoom: *zoom,
            }))
        }
        SceneMutationRequest::SetCamera3d(camera_request) => Some(SceneMutation::SetCamera3D(
            camera3d_state_from_request(camera_request, current_camera_3d),
        )),
        SceneMutationRequest::SetRender3d(render_request) => {
            render3d_mutation_from_request(render_request).map(SceneMutation::SetRender3D)
        }
        SceneMutationRequest::SpawnObject { template, target } => {
            Some(SceneMutation::SpawnObject {
                template: template.clone(),
                target: target.clone(),
            })
        }
        SceneMutationRequest::DespawnObject { target } => Some(SceneMutation::DespawnObject {
            target: target.clone(),
        }),
    }
}

pub fn camera3d_state_from_request(
    request: &Camera3dMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Camera3DState {
    let mut camera = Camera3DState {
        eye: current_camera_3d.eye,
        look_at: current_camera_3d.look_at,
        up: current_camera_3d.up,
        fov_deg: current_camera_3d.fov_degrees,
    };
    match request {
        Camera3dMutationRequest::LookAt { eye, look_at } => {
            camera.eye = *eye;
            camera.look_at = *look_at;
        }
        Camera3dMutationRequest::Up { up } => {
            camera.up = *up;
        }
    }
    camera
}

pub fn render3d_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Option<Render3DMutation> {
    match request {
        Render3dMutationRequest::SetViewProfile { profile } => {
            Some(Render3DMutation::SetViewProfile {
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetLightingProfile { profile } => {
            Some(Render3DMutation::SetLightingProfile {
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentProfile { profile } => {
            Some(Render3DMutation::SetSpaceEnvironmentProfile {
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetLightingParam { name, value } => {
            Some(Render3DMutation::SetLightingParam {
                param: LightingProfileParam::from_name(name)?,
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentParam { name, value } => {
            Some(Render3DMutation::SetSpaceEnvironmentParam {
                param: SpaceEnvironmentParam::from_name(name)?,
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetNodeTransform {
            target,
            translation,
            rotation_deg,
            scale,
        } => Some(Render3DMutation::SetNodeTransform {
            target: target.clone(),
            transform: Transform3D {
                translation: translation.unwrap_or([0.0, 0.0, 0.0]),
                rotation_deg: rotation_deg.unwrap_or([0.0, 0.0, 0.0]),
                scale: scale.unwrap_or([1.0, 1.0, 1.0]),
            },
        }),
        Render3dMutationRequest::SetMaterialParam {
            target,
            name,
            value,
        } => {
            use crate::mutations::ObjMaterialParam;
            let full_path = format!("obj.{}", name);
            let param = ObjMaterialParam::from_full_path(&full_path)?;
            Some(Render3DMutation::SetObjMaterialParam {
                target: target.clone(),
                param,
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetAtmosphereParam {
            target,
            name,
            value,
        } => {
            use crate::mutations::AtmosphereParam;
            let param = AtmosphereParam::from_full_path(name)?;
            Some(Render3DMutation::SetAtmosphereParamTyped {
                target: target.clone(),
                param,
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetWorldParam {
            target,
            name,
            value,
        } => match scene_mutation_from_render_path(target, name, value)? {
            crate::SceneMutation::SetRender3D(m) => Some(m),
            _ => None,
        },
        Render3dMutationRequest::SetSurfaceMode { target, mode } => {
            Some(Render3DMutation::SetObjMaterialParam {
                target: target.clone(),
                param: crate::mutations::ObjMaterialParam::SurfaceMode,
                value: MaterialValue::Text(mode.clone()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_mutation_from_render_path;
    use engine_api::scene::SceneMutationRequest;

    #[test]
    fn maps_request_to_scene_mutation() {
        let request = SceneMutationRequest::Set2dProps {
            target: "hud".to_string(),
            visible: Some(false),
            dx: Some(2),
            dy: Some(3),
            text: Some("ok".to_string()),
        };
        let mutation = scene_mutation_from_request(&request, SceneCamera3D::default())
            .expect("scene mutation");

        match mutation {
            SceneMutation::Set2DProps(value) => {
                assert_eq!(value.target, "hud");
                assert_eq!(value.visible, Some(false));
                assert_eq!(value.dx, Some(2));
                assert_eq!(value.dy, Some(3));
                assert_eq!(value.text.as_deref(), Some("ok"));
            }
            _ => panic!("expected Set2DProps"),
        }
    }

    #[test]
    fn maps_render_request_to_worldgen_mutation() {
        let request = Render3dMutationRequest::SetWorldParam {
            target: "planet-main".to_string(),
            name: "world.seed".to_string(),
            value: serde_json::json!(42),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");
        match mutation {
            Render3DMutation::SetWorldgenParamTyped {
                target,
                param,
                value,
            } => {
                assert_eq!(target, "planet-main");
                assert_eq!(param, crate::mutations::WorldgenParam::Seed);
                assert_eq!(value, MaterialValue::Scalar(42.0));
            }
            _ => panic!("expected SetWorldgenParamTyped"),
        }
    }

    #[test]
    fn maps_world_param_request_to_typed_material_mutation() {
        let request = Render3dMutationRequest::SetWorldParam {
            target: "planet-main".to_string(),
            name: "obj.scale".to_string(),
            value: serde_json::json!(1.25),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");
        match mutation {
            Render3DMutation::SetObjMaterialParam {
                target,
                param,
                value,
            } => {
                assert_eq!(target, "planet-main");
                assert_eq!(param, crate::mutations::ObjMaterialParam::Scale);
                assert_eq!(value, MaterialValue::Scalar(1.25));
            }
            _ => panic!("expected SetObjMaterialParam"),
        }
    }

    #[test]
    fn maps_scene3d_frame_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_render_path(
            "scene-view",
            "scene3d.frame",
            &serde_json::json!("main-7"),
        )
        .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetScene3DFrame { target, frame }) => {
                assert_eq!(target, "scene-view");
                assert_eq!(frame, "main-7");
            }
            _ => panic!("expected SetScene3DFrame"),
        }
    }

    #[test]
    fn maps_planet_spin_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_render_path(
            "planet-view",
            "planet.spin_deg",
            &serde_json::json!(15.0),
        )
        .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetPlanetParamTyped {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "planet-view");
                assert_eq!(param, crate::mutations::PlanetParam::SpinDeg);
                assert_eq!(value, MaterialValue::Scalar(15.0));
            }
            _ => panic!("expected SetPlanetParamTyped"),
        }
    }

    #[test]
    fn leaves_unmapped_render_path_unmapped() {
        let mutation = scene_mutation_from_render_path(
            "planet-view",
            "text.content",
            &serde_json::json!(0.42),
        );

        assert!(mutation.is_none());
    }

    #[test]
    fn maps_lighting_param_request_to_typed_mutation() {
        let request = Render3dMutationRequest::SetLightingParam {
            name: "exposure".to_string(),
            value: serde_json::json!(0.82),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");
        match mutation {
            Render3DMutation::SetLightingParam { param, value } => {
                assert_eq!(param, crate::LightingProfileParam::Exposure);
                assert_eq!(value, MaterialValue::Scalar(0.82));
            }
            _ => panic!("expected SetLightingParam"),
        }
    }
}
