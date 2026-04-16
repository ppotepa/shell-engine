use engine_api::scene::{Camera3dMutationRequest, Render3dMutationRequest, SceneMutationRequest};
use engine_core::render_types::{Camera3DState, MaterialValue, Transform3D};
use engine_core::scene_runtime_types::SceneCamera3D;

use crate::{Render3DMutation, SceneMutation, Set2DPropsMutation, SetCamera2DMutation};

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
        } => Some(Render3DMutation::SetAtmosphereParam {
            target: target.clone(),
            param: name.clone(),
            value: material_value_from_json(value)?,
        }),
        Render3dMutationRequest::SetWorldParam {
            target,
            name,
            value,
        } => Some(Render3DMutation::SetWorldgenParam {
            target: target.clone(),
            param: name.clone(),
            value: material_value_from_json(value)?,
        }),
        Render3dMutationRequest::SetSurfaceMode { target, mode } => {
            Some(Render3DMutation::SetMaterialParam {
                target: target.clone(),
                param: "surface_mode".to_string(),
                value: MaterialValue::Text(mode.clone()),
            })
        }
    }
}

pub fn scene_mutation_from_set_property_3d(
    target: &str,
    path: &str,
    value: &serde_json::Value,
) -> Option<SceneMutation> {
    let worldgen_scalar = |key: &str, n: f64| {
        Some(SceneMutation::SetRender3D(
            Render3DMutation::SetWorldgenParam {
                target: target.to_string(),
                param: key.to_string(),
                value: MaterialValue::Scalar(n as f32),
            },
        ))
    };

    match path {
        "scene3d.frame" => value.as_str().map(|frame| {
            SceneMutation::SetRender3D(Render3DMutation::SetWorldgenParam {
                target: target.to_string(),
                param: path.to_string(),
                value: MaterialValue::Text(frame.to_string()),
            })
        }),
        "planet.spin_deg"
        | "planet.cloud_spin_deg"
        | "planet.cloud2_spin_deg"
        | "planet.sun_dir.x"
        | "planet.sun_dir.y"
        | "planet.sun_dir.z"
        | "obj.world.x"
        | "obj.world.y"
        | "obj.world.z" => value.as_f64().and_then(|n| worldgen_scalar(path, n)),
        _ => None,
    }
}

fn material_value_from_json(value: &serde_json::Value) -> Option<MaterialValue> {
    if let Some(n) = value.as_f64() {
        return Some(MaterialValue::Scalar(n as f32));
    }
    if let Some(b) = value.as_bool() {
        return Some(MaterialValue::Bool(b));
    }
    if let Some(s) = value.as_str() {
        return Some(MaterialValue::Text(s.to_string()));
    }
    if let Some(arr) = value.as_array() {
        if arr.len() == 3 {
            let r = arr.first()?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            let g = arr.get(1)?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            let b = arr.get(2)?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            return Some(MaterialValue::ColorRgb([r, g, b]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn maps_scene3d_frame_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_set_property_3d(
            "scene-view",
            "scene3d.frame",
            &serde_json::json!("main-7"),
        )
        .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetWorldgenParam {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "scene-view");
                assert_eq!(param, "scene3d.frame");
                assert_eq!(value, MaterialValue::Text("main-7".to_string()));
            }
            _ => panic!("expected SetWorldgenParam"),
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
            SceneMutation::SetRender3D(Render3DMutation::SetWorldgenParam {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "planet-view");
                assert_eq!(param, "planet.spin_deg");
                assert_eq!(value, MaterialValue::Scalar(15.0));
            }
            _ => panic!("expected SetWorldgenParam"),
        }
    }

    #[test]
    fn leaves_unmapped_set_property_for_compatibility_fallback() {
        let mutation = scene_mutation_from_set_property_3d(
            "planet-view",
            "planet.albedo",
            &serde_json::json!(0.42),
        );

        assert!(mutation.is_none());
    }
}
