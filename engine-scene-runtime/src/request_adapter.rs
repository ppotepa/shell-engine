use engine_api::scene::{Camera3dMutationRequest, Render3dMutationRequest, SceneMutationRequest};
use engine_core::render_types::{Camera3DState, MaterialValue, Transform3D};
use engine_core::scene_runtime_types::SceneCamera3D;

use crate::render3d_state::{material_value_from_json, render3d_compat_property_from_param};
use crate::{
    Render3DMutation, SceneMutation, Set2DPropsMutation, SetCamera2DMutation,
    SetSpritePropertyMutation,
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
        SceneMutationRequest::SetSpriteProperty { target, path, value } => {
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
        } => Some(Render3DMutation::SetMaterialParam {
            target: target.clone(),
            param: name.clone(),
            value: material_value_from_json(value)?,
        }),
        Render3dMutationRequest::SetAtmosphereParam {
            target,
            name,
            value,
        } => {
            let param = name.strip_prefix("obj.atmo.").unwrap_or(name.as_str());
            Some(Render3DMutation::SetAtmosphereParam {
                target: target.clone(),
                param: param.to_string(),
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetWorldParam {
            target,
            name,
            value,
        } => {
            let value = material_value_from_json(value)?;
            if let Some(property) = render3d_compat_property_from_param(name, value.clone()) {
                Some(Render3DMutation::SetCompatProperty {
                    target: target.clone(),
                    property,
                })
            } else if name.starts_with("obj.atmo.") {
                let param = name.strip_prefix("obj.atmo.").unwrap_or(name.as_str());
                Some(Render3DMutation::SetAtmosphereParam {
                    target: target.clone(),
                    param: param.to_string(),
                    value,
                })
            } else if name.starts_with("obj.") {
                let obj_param = name.strip_prefix("obj.").unwrap_or(name.as_str());
                if obj_param.starts_with("world.") || obj_param.starts_with("terrain.") {
                    Some(Render3DMutation::SetWorldgenParam {
                        target: target.clone(),
                        param: name.clone(),
                        value,
                    })
                } else {
                    Some(Render3DMutation::SetMaterialParam {
                        target: target.clone(),
                        param: obj_param.to_string(),
                        value,
                    })
                }
            } else if name.starts_with("planet.") {
                Some(Render3DMutation::SetMaterialParam {
                    target: target.clone(),
                    param: name.clone(),
                    value,
                })
            } else {
                Some(Render3DMutation::SetWorldgenParam {
                    target: target.clone(),
                    param: name.clone(),
                    value,
                })
            }
        }
        Render3dMutationRequest::SetSurfaceMode { target, mode } => {
            Some(Render3DMutation::SetMaterialParam {
                target: target.clone(),
                param: "surface_mode".to_string(),
                value: MaterialValue::Text(mode.clone()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_mutation_from_set_property_3d;
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
            name: "seed".to_string(),
            value: serde_json::json!(42),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");
        match mutation {
            Render3DMutation::SetWorldgenParam {
                target,
                param,
                value,
            } => {
                assert_eq!(target, "planet-main");
                assert_eq!(param, "seed");
                assert_eq!(value, MaterialValue::Scalar(42.0));
            }
            _ => panic!("expected SetWorldgenParam"),
        }
    }

    #[test]
    fn maps_compat_world_param_request_to_typed_material_mutation() {
        let request = Render3dMutationRequest::SetWorldParam {
            target: "planet-main".to_string(),
            name: "obj.scale".to_string(),
            value: serde_json::json!(1.25),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");
        match mutation {
            Render3DMutation::SetMaterialParam { target, param, value } => {
                assert_eq!(target, "planet-main");
                assert_eq!(param, "scale");
                assert_eq!(value, MaterialValue::Scalar(1.25));
            }
            _ => panic!("expected SetMaterialParam"),
        }
    }

    #[test]
    fn maps_scene3d_frame_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_set_property_3d(
            "scene-view",
            "scene3d.frame",
            &serde_json::json!("main-7"),
        )
        .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetCompatProperty {
                target,
                property,
            }) => {
                assert_eq!(target, "scene-view");
                assert_eq!(
                    property,
                    crate::Render3DCompatProperty::Scene3dFrame {
                        frame: "main-7".to_string()
                    }
                );
            }
            _ => panic!("expected SetCompatProperty"),
        }
    }

    #[test]
    fn maps_planet_spin_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_set_property_3d(
            "planet-view",
            "planet.spin_deg",
            &serde_json::json!(15.0),
        )
        .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetMaterialParam {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "planet-view");
                assert_eq!(param, "planet.spin_deg");
                assert_eq!(value, MaterialValue::Scalar(15.0));
            }
            _ => panic!("expected SetMaterialParam"),
        }
    }

    #[test]
    fn leaves_unmapped_set_property_for_compatibility_fallback() {
        let mutation = scene_mutation_from_set_property_3d(
            "planet-view",
            "text.content",
            &serde_json::json!(0.42),
        );

        assert!(mutation.is_none());
    }
}
