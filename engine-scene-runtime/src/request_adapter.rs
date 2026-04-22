use engine_api::scene::{
    Camera3dMutationRequest, Camera3dNormalizedMutation, Camera3dObjectViewState,
    Render3dMutationDomain, Render3dMutationRequest, Render3dProfileSlot as RequestProfileSlot,
    SceneMutationError, SceneMutationRequest,
};
use engine_core::render_types::{Camera3DState, MaterialValue, Transform3D};
use engine_core::scene_runtime_types::SceneCamera3D;
use serde_json::Value as JsonValue;

use crate::render3d_state::{material_value_from_json, scene_mutation_from_render_path};
use crate::{
    LightingProfileParam, Render3DGroupedParam, Render3DMutation, Render3DProfileParam,
    Render3DProfileSlot, SceneMutation, Set2DPropsMutation, SetCamera2DMutation,
    SetSpritePropertyMutation, SpaceEnvironmentParam,
};

pub fn scene_mutation_from_request(
    request: &SceneMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Result<SceneMutation, SceneMutationError> {
    scene_mutation_from_request_result(request, current_camera_3d)
}

/// Lower a typed scene mutation request into runtime mutations.
///
/// Object-targeted camera requests lower into render-domain grouped params so
/// multiple camera handles can coexist without a singleton assumption. Split
/// render requests route through their dedicated domain adapters.
pub fn scene_mutation_from_request_result(
    request: &SceneMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Result<SceneMutation, SceneMutationError> {
    match request {
        SceneMutationRequest::Set2dProps {
            target,
            visible,
            dx,
            dy,
            text,
        } => {
            if visible.is_none() && dx.is_none() && dy.is_none() && text.is_none() {
                return Err(invalid_scene_request(
                    request,
                    "set_2d_props requires at least one field",
                ));
            }
            Ok(SceneMutation::Set2DProps(Set2DPropsMutation {
                target: target.clone(),
                visible: *visible,
                dx: *dx,
                dy: *dy,
                text: text.clone(),
            }))
        }
        SceneMutationRequest::SetSpriteProperty {
            target,
            path,
            value,
        } => {
            let mutation = match path.as_str() {
                "transform.heading" => SetSpritePropertyMutation::Heading {
                    heading: value.as_f64().ok_or_else(|| {
                        invalid_scene_request(
                            request,
                            format!("path `{path}` expects a numeric value"),
                        )
                    })? as f32,
                },
                "text.font" => SetSpritePropertyMutation::TextFont {
                    font: value
                        .as_str()
                        .ok_or_else(|| {
                            invalid_scene_request(
                                request,
                                format!("path `{path}` expects a string value"),
                            )
                        })?
                        .to_string(),
                },
                "style.fg" | "text.fg" => SetSpritePropertyMutation::TextColour {
                    fg: true,
                    value: value.clone(),
                },
                "style.bg" | "text.bg" => SetSpritePropertyMutation::TextColour {
                    fg: false,
                    value: value.clone(),
                },
                "vector.points" | "vector.closed" | "vector.draw_char" | "vector.fg"
                | "vector.bg" | "style.border" | "style.shadow" => {
                    SetSpritePropertyMutation::VectorProperty {
                        path: path.to_string(),
                        value: value.clone(),
                    }
                }
                "image.frame_index" => SetSpritePropertyMutation::ImageFrame {
                    frame_index: value
                        .as_u64()
                        .and_then(|v| u16::try_from(v).ok())
                        .ok_or_else(|| {
                            invalid_scene_request(
                                request,
                                format!("path `{path}` expects a u16 frame index"),
                            )
                        })?,
                },
                _ => {
                    return Err(unsupported_scene_request(
                        request,
                        format!("unsupported sprite property path `{path}`"),
                    ));
                }
            };
            Ok(SceneMutation::SetSpriteProperty {
                target: target.clone(),
                mutation,
            })
        }
        SceneMutationRequest::SetCamera2d { x, y, zoom } => {
            Ok(SceneMutation::SetCamera2D(SetCamera2DMutation {
                x: x.round() as i32,
                y: y.round() as i32,
                zoom: *zoom,
            }))
        }
        SceneMutationRequest::SetCamera3d(camera_request) => {
            camera3d_mutation_from_request_result(camera_request, current_camera_3d)
        }
        SceneMutationRequest::SetRender3d(render_request) => Ok(SceneMutation::SetRender3D(
            render3d_mutation_from_request_result(render_request)?,
        )),
        SceneMutationRequest::RequestError { error } => {
            Err(SceneMutationError::from_request_error(error))
        }
        SceneMutationRequest::SpawnObject { template, target } => Ok(SceneMutation::SpawnObject {
            template: template.clone(),
            target: target.clone(),
        }),
        SceneMutationRequest::DespawnObject { target } => Ok(SceneMutation::DespawnObject {
            target: target.clone(),
        }),
    }
}

fn camera3d_mutation_from_request_result(
    request: &Camera3dMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Result<SceneMutation, SceneMutationError> {
    match request.normalized(current_camera_3d) {
        Camera3dNormalizedMutation::Scene(camera) => {
            Ok(SceneMutation::SetCamera3D(Camera3DState {
                eye: camera.eye,
                look_at: camera.look_at,
                up: camera.up,
                fov_deg: camera.fov_degrees,
            }))
        }
        Camera3dNormalizedMutation::Object { target, view } => Ok(SceneMutation::SetRender3D(
            Render3DMutation::SetGroupedParams {
                target: Some(target),
                params: object_camera_grouped_params(view),
            },
        )),
    }
}

pub fn camera3d_state_from_request(
    request: &Camera3dMutationRequest,
    current_camera_3d: SceneCamera3D,
) -> Camera3DState {
    match request.normalized(current_camera_3d) {
        Camera3dNormalizedMutation::Scene(camera) => Camera3DState {
            eye: camera.eye,
            look_at: camera.look_at,
            up: camera.up,
            fov_deg: camera.fov_degrees,
        },
        Camera3dNormalizedMutation::Object { .. } => Camera3DState {
            eye: current_camera_3d.eye,
            look_at: current_camera_3d.look_at,
            up: current_camera_3d.up,
            fov_deg: current_camera_3d.fov_degrees,
        },
    }
}

fn object_camera_grouped_params(
    view: Camera3dObjectViewState,
) -> Vec<(Render3DGroupedParam, MaterialValue)> {
    use crate::mutations::ObjMaterialParam;

    vec![
        (
            Render3DGroupedParam::Material(ObjMaterialParam::CamWorldX),
            MaterialValue::Scalar(view.eye[0]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::CamWorldY),
            MaterialValue::Scalar(view.eye[1]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::CamWorldZ),
            MaterialValue::Scalar(view.eye[2]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewRightX),
            MaterialValue::Scalar(view.right[0]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewRightY),
            MaterialValue::Scalar(view.right[1]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewRightZ),
            MaterialValue::Scalar(view.right[2]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewUpX),
            MaterialValue::Scalar(view.up[0]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewUpY),
            MaterialValue::Scalar(view.up[1]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewUpZ),
            MaterialValue::Scalar(view.up[2]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewFwdX),
            MaterialValue::Scalar(view.forward[0]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewFwdY),
            MaterialValue::Scalar(view.forward[1]),
        ),
        (
            Render3DGroupedParam::Material(ObjMaterialParam::ViewFwdZ),
            MaterialValue::Scalar(view.forward[2]),
        ),
    ]
}

pub fn render3d_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    render3d_mutation_from_request_result(request)
}

/// Lower a typed split-domain render request into the corresponding runtime
/// render mutation.
pub fn render3d_mutation_from_request_result(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request.domain() {
        Render3dMutationDomain::Transform => render3d_transform_mutation_from_request(request),
        Render3dMutationDomain::Material => render3d_material_mutation_from_request(request),
        Render3dMutationDomain::Lighting => render3d_lighting_mutation_from_request(request),
        Render3dMutationDomain::Atmosphere => render3d_atmosphere_mutation_from_request(request),
        Render3dMutationDomain::Generator => render3d_generator_mutation_from_request(request),
        Render3dMutationDomain::View => render3d_view_mutation_from_request(request),
    }
}

fn render3d_transform_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetNodeTransform {
            target,
            translation,
            rotation_deg,
            scale,
        } => Ok(Render3DMutation::SetNodeTransform {
            target: target.clone(),
            transform: Transform3D {
                translation: translation.unwrap_or([0.0, 0.0, 0.0]),
                rotation_deg: rotation_deg.unwrap_or([0.0, 0.0, 0.0]),
                scale: scale.unwrap_or([1.0, 1.0, 1.0]),
            },
        }),
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the transform domain",
        )),
    }
}

fn render3d_material_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetMaterialParam {
            target,
            name,
            value,
        } => {
            use crate::mutations::ObjMaterialParam;
            let full_path = format!("obj.{}", name);
            let param = ObjMaterialParam::from_full_path(&full_path).ok_or_else(|| {
                unsupported_render_request(
                    request,
                    format!("unsupported material parameter `{name}`"),
                )
            })?;
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Material(param),
                    material_value_from_json(value).ok_or_else(|| {
                        invalid_render_request(
                            request,
                            format!("material parameter `{name}` has an unsupported value shape"),
                        )
                    })?,
                )],
            })
        }
        Render3dMutationRequest::SetMaterialParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(
                    request,
                    params,
                    material_grouped_param_from_name,
                )?,
            })
        }
        Render3dMutationRequest::SetSurfaceMode { target, mode } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Material(crate::mutations::ObjMaterialParam::SurfaceMode),
                    MaterialValue::Text(mode.clone()),
                )],
            })
        }
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the material domain",
        )),
    }
}

fn render3d_lighting_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetProfile {
            profile_slot,
            profile,
        } => Ok(Render3DMutation::SetProfile {
            slot: profile_slot_from_request(*profile_slot),
            profile: profile.clone(),
        }),
        Render3dMutationRequest::SetProfileParam {
            profile_slot,
            name,
            value,
        } => Ok(Render3DMutation::SetProfileParam {
            param: profile_param_from_request(*profile_slot, name).ok_or_else(|| {
                unsupported_render_request(
                    request,
                    format!(
                        "unsupported profile parameter `{name}` for slot `{}`",
                        profile_slot_name(*profile_slot)
                    ),
                )
            })?,
            value: material_value_from_json(value).ok_or_else(|| {
                invalid_render_request(
                    request,
                    format!("profile parameter `{name}` has an unsupported value shape"),
                )
            })?,
        }),
        Render3dMutationRequest::SetLightingProfile { profile } => {
            Ok(Render3DMutation::SetProfile {
                slot: Render3DProfileSlot::Lighting,
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentProfile { profile } => {
            Ok(Render3DMutation::SetProfile {
                slot: Render3DProfileSlot::SpaceEnvironment,
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetLightingParam { name, value } => {
            Ok(Render3DMutation::SetProfileParam {
                param: Render3DProfileParam::Lighting(
                    LightingProfileParam::from_name(name).ok_or_else(|| {
                        unsupported_render_request(
                            request,
                            format!("unsupported lighting parameter `{name}`"),
                        )
                    })?,
                ),
                value: material_value_from_json(value).ok_or_else(|| {
                    invalid_render_request(
                        request,
                        format!("lighting parameter `{name}` has an unsupported value shape"),
                    )
                })?,
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentParam { name, value } => {
            Ok(Render3DMutation::SetProfileParam {
                param: Render3DProfileParam::SpaceEnvironment(
                    SpaceEnvironmentParam::from_name(name).ok_or_else(|| {
                        unsupported_render_request(
                            request,
                            format!("unsupported space-environment parameter `{name}`"),
                        )
                    })?,
                ),
                value: material_value_from_json(value).ok_or_else(|| {
                    invalid_render_request(
                        request,
                        format!(
                            "space-environment parameter `{name}` has an unsupported value shape"
                        ),
                    )
                })?,
            })
        }
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the lighting domain",
        )),
    }
}

fn render3d_atmosphere_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetAtmosphereParam {
            target,
            name,
            value,
        } => {
            use crate::mutations::AtmosphereParam;
            let param = AtmosphereParam::from_full_path(name).ok_or_else(|| {
                unsupported_render_request(
                    request,
                    format!("unsupported atmosphere parameter `{name}`"),
                )
            })?;
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Atmosphere(param),
                    material_value_from_json(value).ok_or_else(|| {
                        invalid_render_request(
                            request,
                            format!("atmosphere parameter `{name}` has an unsupported value shape"),
                        )
                    })?,
                )],
            })
        }
        Render3dMutationRequest::SetAtmosphereParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(request, params, |name| {
                    crate::mutations::AtmosphereParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Atmosphere)
                })?,
            })
        }
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the atmosphere domain",
        )),
    }
}

fn render3d_generator_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetWorldParam {
            target,
            name,
            value,
        } => {
            let mapped = scene_mutation_from_render_path(target, name, value).ok_or_else(|| {
                if supported_render_path(name) {
                    invalid_render_request(
                        request,
                        format!("render path `{name}` has an unsupported value shape"),
                    )
                } else {
                    unsupported_render_request(request, format!("unsupported render path `{name}`"))
                }
            })?;
            match mapped {
                crate::SceneMutation::SetRender3D(m) => Ok(m),
                _ => Err(unsupported_render_request(
                    request,
                    format!("render path `{name}` did not map to a render3d mutation"),
                )),
            }
        }
        Render3dMutationRequest::SetSurfaceParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(request, params, |name| {
                    crate::mutations::TerrainParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Surface)
                })?,
            })
        }
        Render3dMutationRequest::SetGeneratorParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(request, params, |name| {
                    crate::mutations::WorldgenParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Generator)
                })?,
            })
        }
        Render3dMutationRequest::SetBodyParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(request, params, |name| {
                    crate::mutations::PlanetParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Body)
                })?,
            })
        }
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the generator domain",
        )),
    }
}

fn render3d_view_mutation_from_request(
    request: &Render3dMutationRequest,
) -> Result<Render3DMutation, SceneMutationError> {
    match request {
        Render3dMutationRequest::SetProfile {
            profile_slot: RequestProfileSlot::View,
            profile,
        }
        | Render3dMutationRequest::SetViewProfile { profile } => Ok(Render3DMutation::SetProfile {
            slot: Render3DProfileSlot::View,
            profile: profile.clone(),
        }),
        Render3dMutationRequest::SetProfileParam {
            profile_slot: RequestProfileSlot::View,
            name,
            ..
        } => Err(unsupported_render_request(
            request,
            format!("unsupported profile parameter `{name}` for slot `view`"),
        )),
        Render3dMutationRequest::SetViewParams { target, params } => {
            Ok(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json_result(
                    request,
                    params,
                    grouped_view_param_from_name,
                )?,
            })
        }
        _ => Err(unsupported_render_request(
            request,
            "render request did not match the view domain",
        )),
    }
}

fn canonical_group_name(name: &str) -> String {
    name.trim().to_string()
}

fn grouped_params_from_json_result(
    request: &Render3dMutationRequest,
    params: &JsonValue,
    mut map_name: impl FnMut(&str) -> Option<Render3DGroupedParam>,
) -> Result<Vec<(Render3DGroupedParam, MaterialValue)>, SceneMutationError> {
    let object = params
        .as_object()
        .ok_or_else(|| invalid_render_request(request, "grouped params must be a JSON object"))?;
    if object.is_empty() {
        return Err(invalid_render_request(
            request,
            "grouped params must not be empty",
        ));
    }
    let mut grouped = Vec::with_capacity(object.len());
    for (name, value) in object {
        grouped.push((
            map_name(name).ok_or_else(|| {
                unsupported_render_request(
                    request,
                    format!("unsupported grouped parameter `{name}`"),
                )
            })?,
            material_value_from_json(value).ok_or_else(|| {
                invalid_render_request(
                    request,
                    format!("grouped parameter `{name}` has an unsupported value shape"),
                )
            })?,
        ));
    }
    Ok(grouped)
}

fn grouped_view_param_from_name(name: &str) -> Option<Render3DGroupedParam> {
    match canonical_group_name(name).as_str() {
        "distance" | "camera-distance" | "camera_distance" => Some(Render3DGroupedParam::View(
            crate::mutations::ViewParam::Distance,
        )),
        "yaw" | "yaw-deg" | "yaw_deg" => {
            Some(Render3DGroupedParam::View(crate::mutations::ViewParam::Yaw))
        }
        "pitch" | "pitch-deg" | "pitch_deg" => Some(Render3DGroupedParam::View(
            crate::mutations::ViewParam::Pitch,
        )),
        "roll" | "roll-deg" | "roll_deg" => Some(Render3DGroupedParam::View(
            crate::mutations::ViewParam::Roll,
        )),
        _ => None,
    }
}

fn material_grouped_param_from_name(name: &str) -> Option<Render3DGroupedParam> {
    let canonical = canonical_group_name(name);
    let full_path = match canonical.as_str() {
        "rotation_speed" => "obj.rotation-speed".to_string(),
        "orbit-speed" => "obj.orbit_speed".to_string(),
        "orbit_speed" => "obj.orbit_speed".to_string(),
        "surface-mode" => "obj.surface_mode".to_string(),
        "surface_mode" => "obj.surface_mode".to_string(),
        "world-x" => "obj.world.x".to_string(),
        "world_x" => "obj.world.x".to_string(),
        "world-y" => "obj.world.y".to_string(),
        "world_y" => "obj.world.y".to_string(),
        "world-z" => "obj.world.z".to_string(),
        "world_z" => "obj.world.z".to_string(),
        "light-x" => "obj.light.x".to_string(),
        "light_x" => "obj.light.x".to_string(),
        "light-y" => "obj.light.y".to_string(),
        "light_y" => "obj.light.y".to_string(),
        "light-z" => "obj.light.z".to_string(),
        "light_z" => "obj.light.z".to_string(),
        "clip-y-min" => "obj.clip_y_min".to_string(),
        "clip_y_min" => "obj.clip_y_min".to_string(),
        "clip-y-max" => "obj.clip_y_max".to_string(),
        "clip_y_max" => "obj.clip_y_max".to_string(),
        "camera-distance" => "obj.camera-distance".to_string(),
        "camera_distance" => "obj.camera-distance".to_string(),
        "cam-wx" | "cam_wx" => "obj.cam.wx".to_string(),
        "cam-wy" | "cam_wy" => "obj.cam.wy".to_string(),
        "cam-wz" | "cam_wz" => "obj.cam.wz".to_string(),
        "cam-world-x" | "cam_world_x" => "obj.cam.world_x".to_string(),
        "cam-world-y" | "cam_world_y" => "obj.cam.world_y".to_string(),
        "cam-world-z" | "cam_world_z" => "obj.cam.world_z".to_string(),
        "view-rx" | "view_rx" => "obj.view.rx".to_string(),
        "view-ry" | "view_ry" => "obj.view.ry".to_string(),
        "view-rz" | "view_rz" => "obj.view.rz".to_string(),
        "view-ux" | "view_ux" => "obj.view.ux".to_string(),
        "view-uy" | "view_uy" => "obj.view.uy".to_string(),
        "view-uz" | "view_uz" => "obj.view.uz".to_string(),
        "view-fx" | "view_fx" => "obj.view.fx".to_string(),
        "view-fy" | "view_fy" => "obj.view.fy".to_string(),
        "view-fz" | "view_fz" => "obj.view.fz".to_string(),
        other => format!("obj.{other}"),
    };
    crate::mutations::ObjMaterialParam::from_full_path(&full_path)
        .map(Render3DGroupedParam::Material)
}

fn profile_slot_from_request(slot: RequestProfileSlot) -> Render3DProfileSlot {
    match slot {
        RequestProfileSlot::View => Render3DProfileSlot::View,
        RequestProfileSlot::Lighting => Render3DProfileSlot::Lighting,
        RequestProfileSlot::SpaceEnvironment => Render3DProfileSlot::SpaceEnvironment,
    }
}

fn profile_slot_name(slot: RequestProfileSlot) -> &'static str {
    match slot {
        RequestProfileSlot::View => "view",
        RequestProfileSlot::Lighting => "lighting",
        RequestProfileSlot::SpaceEnvironment => "space_environment",
    }
}

fn profile_param_from_request(
    slot: RequestProfileSlot,
    name: &str,
) -> Option<Render3DProfileParam> {
    match slot {
        RequestProfileSlot::View => None,
        RequestProfileSlot::Lighting => Some(Render3DProfileParam::Lighting(
            LightingProfileParam::from_name(name)?,
        )),
        RequestProfileSlot::SpaceEnvironment => Some(Render3DProfileParam::SpaceEnvironment(
            SpaceEnvironmentParam::from_name(name)?,
        )),
    }
}

fn supported_render_path(name: &str) -> bool {
    if name == "scene3d.frame" {
        return true;
    }
    if name.starts_with("planet.") {
        return crate::mutations::PlanetParam::from_full_path(name).is_some();
    }
    if name.starts_with("obj.atmo.") {
        return crate::mutations::AtmosphereParam::from_full_path(name).is_some();
    }
    if name.starts_with("obj.") {
        return crate::mutations::ObjMaterialParam::from_full_path(name).is_some();
    }
    if name.starts_with("terrain.") {
        return crate::mutations::TerrainParam::from_full_path(name).is_some();
    }
    if name.starts_with("world.") {
        return crate::mutations::WorldgenParam::from_full_path(name).is_some();
    }
    false
}

fn invalid_scene_request(
    request: &SceneMutationRequest,
    detail: impl Into<String>,
) -> SceneMutationError {
    SceneMutationError::invalid_request(request.kind_name(), detail)
}

fn unsupported_scene_request(
    request: &SceneMutationRequest,
    detail: impl Into<String>,
) -> SceneMutationError {
    SceneMutationError::unsupported_request(request.kind_name(), detail)
}

fn invalid_render_request(
    request: &Render3dMutationRequest,
    detail: impl Into<String>,
) -> SceneMutationError {
    SceneMutationError::invalid_request(request.kind_name(), detail)
}

fn unsupported_render_request(
    request: &Render3dMutationRequest,
    detail: impl Into<String>,
) -> SceneMutationError {
    SceneMutationError::unsupported_request(request.kind_name(), detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_mutation_from_render_path;
    use engine_api::scene::{Camera3dMutationRequest, SceneMutationError, SceneMutationRequest};

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
    fn rejects_empty_set_2d_props_request() {
        let request = SceneMutationRequest::Set2dProps {
            target: "hud".to_string(),
            visible: None,
            dx: None,
            dy: None,
            text: None,
        };

        let error = scene_mutation_from_request_result(&request, SceneCamera3D::default())
            .expect_err("empty set_2d_props should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_2d_props".to_string(),
                detail: "set_2d_props requires at least one field".to_string(),
            }
        );
    }

    #[test]
    fn maps_object_camera_request_to_grouped_material_mutation() {
        let request = SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectLookAt {
            target: "cockpit-camera".to_string(),
            eye: [0.0, 0.0, -5.0],
            look_at: [0.0, 0.0, 0.0],
            up: None,
        });
        let mutation = scene_mutation_from_request(&request, SceneCamera3D::default())
            .expect("scene mutation");

        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams { target, params }) => {
                assert_eq!(target.as_deref(), Some("cockpit-camera"));
                assert_eq!(
                    params,
                    vec![
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::CamWorldX
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::CamWorldY
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::CamWorldZ
                            ),
                            MaterialValue::Scalar(-5.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewRightX
                            ),
                            MaterialValue::Scalar(-1.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewRightY
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewRightZ
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewUpX
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewUpY
                            ),
                            MaterialValue::Scalar(1.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewUpZ
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewFwdX
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewFwdY
                            ),
                            MaterialValue::Scalar(0.0),
                        ),
                        (
                            crate::Render3DGroupedParam::Material(
                                crate::mutations::ObjMaterialParam::ViewFwdZ
                            ),
                            MaterialValue::Scalar(1.0),
                        ),
                    ]
                );
            }
            _ => panic!("expected SetGroupedParams"),
        }
    }

    #[test]
    fn maps_multiple_object_camera_handles_without_singleton_camera_assumptions() {
        let scene_camera_a = SceneCamera3D {
            eye: [100.0, 100.0, 100.0],
            look_at: [90.0, 95.0, 80.0],
            up: [0.0, 0.0, 1.0],
            fov_degrees: 75.0,
            near_clip: 0.1,
        };
        let scene_camera_b = SceneCamera3D {
            eye: [-12.0, 7.0, 3.0],
            look_at: [0.0, 0.0, 0.0],
            up: [1.0, 0.0, 0.0],
            fov_degrees: 25.0,
            near_clip: 0.5,
        };
        let cockpit = SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectLookAt {
            target: "cockpit-camera".to_string(),
            eye: [1.0, 2.0, 3.0],
            look_at: [4.0, 2.0, 3.0],
            up: Some([0.0, 1.0, 0.0]),
        });
        let chase = SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectBasis {
            target: "chase-camera".to_string(),
            eye: [7.0, 8.0, 9.0],
            right: [0.0, 0.0, -1.0],
            up: [0.0, 1.0, 0.0],
            forward: [1.0, 0.0, 0.0],
        });

        let cockpit_a = scene_mutation_from_request(&cockpit, scene_camera_a).expect("mutation");
        let cockpit_b = scene_mutation_from_request(&cockpit, scene_camera_b).expect("mutation");
        let chase_mutation = scene_mutation_from_request(&chase, scene_camera_a).expect("mutation");

        assert_eq!(cockpit_a, cockpit_b);
        assert_ne!(cockpit_a, chase_mutation);

        match (cockpit_a, chase_mutation) {
            (
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: cockpit_target,
                    params: cockpit_params,
                }),
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: chase_target,
                    params: chase_params,
                }),
            ) => {
                assert_eq!(cockpit_target.as_deref(), Some("cockpit-camera"));
                assert_eq!(chase_target.as_deref(), Some("chase-camera"));
                assert_ne!(cockpit_params, chase_params);
            }
            _ => panic!("expected object-camera grouped mutations"),
        }
    }

    #[test]
    fn maps_multiple_object_camera_path_aliases_to_distinct_handles() {
        let cockpit = scene_mutation_from_render_path(
            "cockpit-camera",
            "obj.cam.wx",
            &serde_json::json!(1.0),
        )
        .expect("typed mutation");
        let chase =
            scene_mutation_from_render_path("chase-camera", "obj.view.fx", &serde_json::json!(0.5))
                .expect("typed mutation");

        match (cockpit, chase) {
            (
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: cockpit_target,
                    params: cockpit_params,
                }),
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: chase_target,
                    params: chase_params,
                }),
            ) => {
                assert_eq!(cockpit_target.as_deref(), Some("cockpit-camera"));
                assert_eq!(chase_target.as_deref(), Some("chase-camera"));
                assert_ne!(cockpit_params, chase_params);
            }
            _ => panic!("expected object-camera grouped mutations"),
        }
    }

    #[test]
    fn maps_long_form_object_camera_path_aliases_to_distinct_handles() {
        let cockpit = scene_mutation_from_render_path(
            "cockpit-camera",
            "obj.cam.world.x",
            &serde_json::json!(1.0),
        )
        .expect("typed mutation");
        let chase = scene_mutation_from_render_path(
            "chase-camera",
            "obj.view.right_y",
            &serde_json::json!(0.5),
        )
        .expect("typed mutation");

        match (cockpit, chase) {
            (
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: cockpit_target,
                    params: cockpit_params,
                }),
                SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                    target: chase_target,
                    params: chase_params,
                }),
            ) => {
                assert_eq!(cockpit_target.as_deref(), Some("cockpit-camera"));
                assert_eq!(chase_target.as_deref(), Some("chase-camera"));
                assert_ne!(cockpit_params, chase_params);
            }
            _ => panic!("expected object-camera grouped mutations"),
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
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target.as_deref(), Some("planet-main"));
                assert_eq!(
                    params,
                    vec![(
                        crate::Render3DGroupedParam::Generator(
                            crate::mutations::WorldgenParam::Seed
                        ),
                        MaterialValue::Scalar(42.0),
                    )]
                );
            }
            _ => panic!("expected SetGroupedParams"),
        }
    }

    #[test]
    fn rejects_empty_grouped_render_params() {
        let request = Render3dMutationRequest::SetMaterialParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({}),
        };

        let error = render3d_mutation_from_request_result(&request)
            .expect_err("empty grouped params should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_material_params".to_string(),
                detail: "grouped params must not be empty".to_string(),
            }
        );
    }

    #[test]
    fn rejects_invalid_scene3d_frame_payload() {
        let request = Render3dMutationRequest::SetWorldParam {
            target: "scene-view".to_string(),
            name: "scene3d.frame".to_string(),
            value: serde_json::json!(42),
        };

        let error = render3d_mutation_from_request_result(&request)
            .expect_err("invalid render path payload should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_world_param".to_string(),
                detail: "render path `scene3d.frame` has an unsupported value shape".to_string(),
            }
        );
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
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target.as_deref(), Some("planet-main"));
                assert_eq!(
                    params,
                    vec![(
                        crate::Render3DGroupedParam::Material(
                            crate::mutations::ObjMaterialParam::Scale
                        ),
                        MaterialValue::Scalar(1.25),
                    )]
                );
            }
            _ => panic!("expected SetGroupedParams"),
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
            SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams { target, params }) => {
                assert_eq!(target.as_deref(), Some("planet-view"));
                assert_eq!(
                    params,
                    vec![(
                        crate::Render3DGroupedParam::Body(crate::mutations::PlanetParam::SpinDeg),
                        MaterialValue::Scalar(15.0),
                    )]
                );
            }
            _ => panic!("expected SetGroupedParams"),
        }
    }

    #[test]
    fn maps_object_camera_set_property_to_typed_mutation() {
        let mutation = scene_mutation_from_render_path(
            "cockpit-camera",
            "obj.cam.wx",
            &serde_json::json!(1.5),
        )
        .expect("typed mutation");

        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams { target, params }) => {
                assert_eq!(target.as_deref(), Some("cockpit-camera"));
                assert_eq!(
                    params,
                    vec![(
                        crate::Render3DGroupedParam::Material(
                            crate::mutations::ObjMaterialParam::CamWorldX
                        ),
                        MaterialValue::Scalar(1.5),
                    )]
                );
            }
            _ => panic!("expected SetGroupedParams"),
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
            Render3DMutation::SetProfileParam { param, value } => {
                assert_eq!(
                    param,
                    crate::Render3DProfileParam::Lighting(crate::LightingProfileParam::Exposure)
                );
                assert_eq!(value, MaterialValue::Scalar(0.82));
            }
            _ => panic!("expected SetProfileParam"),
        }
    }

    #[test]
    fn maps_neutral_profile_request_to_neutral_runtime_mutation() {
        let request = Render3dMutationRequest::SetProfile {
            profile_slot: RequestProfileSlot::Lighting,
            profile: "lab-neutral".to_string(),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");

        match mutation {
            Render3DMutation::SetProfile { slot, profile } => {
                assert_eq!(slot, crate::Render3DProfileSlot::Lighting);
                assert_eq!(profile, "lab-neutral");
            }
            _ => panic!("expected SetProfile"),
        }
    }

    #[test]
    fn maps_grouped_view_request_to_view_grouped_runtime_mutation() {
        let request = Render3dMutationRequest::SetViewParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({
                "distance": 12.0,
                "yaw_deg": 25.0
            }),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");

        match mutation {
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target.as_deref(), Some("planet-main"));
                assert_eq!(
                    params,
                    vec![
                        (
                            crate::Render3DGroupedParam::View(
                                crate::mutations::ViewParam::Distance
                            ),
                            MaterialValue::Scalar(12.0),
                        ),
                        (
                            crate::Render3DGroupedParam::View(crate::mutations::ViewParam::Yaw),
                            MaterialValue::Scalar(25.0),
                        ),
                    ]
                );
            }
            _ => panic!("expected SetGroupedParams"),
        }
    }

    #[test]
    fn maps_neutral_profile_param_request_to_neutral_runtime_mutation() {
        let request = Render3dMutationRequest::SetProfileParam {
            profile_slot: RequestProfileSlot::SpaceEnvironment,
            name: "background_color".to_string(),
            value: serde_json::json!("#010203"),
        };
        let mutation = render3d_mutation_from_request(&request).expect("render mutation");

        match mutation {
            Render3DMutation::SetProfileParam { param, value } => {
                assert_eq!(
                    param,
                    crate::Render3DProfileParam::SpaceEnvironment(
                        crate::SpaceEnvironmentParam::BackgroundColor
                    )
                );
                assert_eq!(value, MaterialValue::Text("#010203".to_string()));
            }
            _ => panic!("expected SetProfileParam"),
        }
    }

    #[test]
    fn split_render_domain_dispatch_matches_dedicated_helpers() {
        let transform = Render3dMutationRequest::SetNodeTransform {
            target: "planet-main".to_string(),
            translation: Some([1.0, 2.0, 3.0]),
            rotation_deg: None,
            scale: None,
        };
        let material = Render3dMutationRequest::SetMaterialParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "scale": 1.25 }),
        };
        let lighting = Render3dMutationRequest::SetLightingParam {
            name: "exposure".to_string(),
            value: serde_json::json!(0.8),
        };
        let atmosphere = Render3dMutationRequest::SetAtmosphereParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "density": 0.4 }),
        };
        let generator = Render3dMutationRequest::SetGeneratorParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "seed": 42 }),
        };
        let view = Render3dMutationRequest::SetViewParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "distance": 12.0 }),
        };

        assert_eq!(
            render3d_mutation_from_request_result(&transform),
            render3d_transform_mutation_from_request(&transform)
        );
        assert_eq!(
            render3d_mutation_from_request_result(&material),
            render3d_material_mutation_from_request(&material)
        );
        assert_eq!(
            render3d_mutation_from_request_result(&lighting),
            render3d_lighting_mutation_from_request(&lighting)
        );
        assert_eq!(
            render3d_mutation_from_request_result(&atmosphere),
            render3d_atmosphere_mutation_from_request(&atmosphere)
        );
        assert_eq!(
            render3d_mutation_from_request_result(&generator),
            render3d_generator_mutation_from_request(&generator)
        );
        assert_eq!(
            render3d_mutation_from_request_result(&view),
            render3d_view_mutation_from_request(&view)
        );
    }

    #[test]
    fn split_render_domain_helpers_reject_cross_domain_requests() {
        let material = Render3dMutationRequest::SetMaterialParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "scale": 1.25 }),
        };
        let lighting = Render3dMutationRequest::SetLightingParam {
            name: "exposure".to_string(),
            value: serde_json::json!(0.8),
        };
        let view = Render3dMutationRequest::SetViewParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "distance": 12.0 }),
        };
        let atmosphere = Render3dMutationRequest::SetAtmosphereParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "density": 0.4 }),
        };
        let generator = Render3dMutationRequest::SetGeneratorParams {
            target: "planet-main".to_string(),
            params: serde_json::json!({ "seed": 42 }),
        };
        let transform = Render3dMutationRequest::SetNodeTransform {
            target: "planet-main".to_string(),
            translation: None,
            rotation_deg: None,
            scale: None,
        };

        assert_eq!(
            render3d_transform_mutation_from_request(&material),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_material_params".to_string(),
                detail: "render request did not match the transform domain".to_string(),
            })
        );
        assert_eq!(
            render3d_material_mutation_from_request(&lighting),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_lighting_param".to_string(),
                detail: "render request did not match the material domain".to_string(),
            })
        );
        assert_eq!(
            render3d_lighting_mutation_from_request(&view),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_view_params".to_string(),
                detail: "render request did not match the lighting domain".to_string(),
            })
        );
        assert_eq!(
            render3d_atmosphere_mutation_from_request(&generator),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_generator_params".to_string(),
                detail: "render request did not match the atmosphere domain".to_string(),
            })
        );
        assert_eq!(
            render3d_generator_mutation_from_request(&transform),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_node_transform".to_string(),
                detail: "render request did not match the generator domain".to_string(),
            })
        );
        assert_eq!(
            render3d_view_mutation_from_request(&atmosphere),
            Err(SceneMutationError::UnsupportedRequest {
                request: "set_atmosphere_params".to_string(),
                detail: "render request did not match the view domain".to_string(),
            })
        );
    }

    #[test]
    fn reports_invalid_sprite_property_value() {
        let request = SceneMutationRequest::SetSpriteProperty {
            target: "hud".to_string(),
            path: "image.frame_index".to_string(),
            value: serde_json::json!("next"),
        };

        let error = scene_mutation_from_request_result(&request, SceneCamera3D::default())
            .expect_err("invalid request");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_sprite_property".to_string(),
                detail: "path `image.frame_index` expects a u16 frame index".to_string(),
            }
        );
    }

    #[test]
    fn maps_request_error_variant_to_runtime_error() {
        let request = SceneMutationRequest::RequestError {
            error: engine_api::scene::SceneMutationRequestError::UnsupportedSetPath {
                target: "hud".to_string(),
                path: "audio.pitch".to_string(),
            },
        };

        let error = scene_mutation_from_request(&request, SceneCamera3D::default())
            .expect_err("request error should stay explicit");

        assert_eq!(
            error,
            SceneMutationError::UnsupportedRequest {
                request: "set_path".to_string(),
                detail: "target `hud` does not support `audio.pitch`".to_string(),
            }
        );
    }

    #[test]
    fn reports_unsupported_render_profile_param_slot() {
        let request = Render3dMutationRequest::SetProfileParam {
            profile_slot: RequestProfileSlot::View,
            name: "distance".to_string(),
            value: serde_json::json!(12.0),
        };

        let error = render3d_mutation_from_request_result(&request).expect_err("unsupported slot");

        assert_eq!(
            error,
            SceneMutationError::UnsupportedRequest {
                request: "set_profile_param".to_string(),
                detail: "unsupported profile parameter `distance` for slot `view`".to_string(),
            }
        );
    }
}
