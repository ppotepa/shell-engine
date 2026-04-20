use engine_api::scene::{
    Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot as RequestProfileSlot,
    SceneMutationRequest,
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
        Render3dMutationRequest::SetProfile {
            profile_slot,
            profile,
        } => Some(Render3DMutation::SetProfile {
            slot: profile_slot_from_request(*profile_slot),
            profile: profile.clone(),
        }),
        Render3dMutationRequest::SetProfileParam {
            profile_slot,
            name,
            value,
        } => Some(Render3DMutation::SetProfileParam {
            param: profile_param_from_request(*profile_slot, name)?,
            value: material_value_from_json(value)?,
        }),
        Render3dMutationRequest::SetViewProfile { profile } => Some(Render3DMutation::SetProfile {
            slot: Render3DProfileSlot::View,
            profile: profile.clone(),
        }),
        Render3dMutationRequest::SetLightingProfile { profile } => {
            Some(Render3DMutation::SetProfile {
                slot: Render3DProfileSlot::Lighting,
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentProfile { profile } => {
            Some(Render3DMutation::SetProfile {
                slot: Render3DProfileSlot::SpaceEnvironment,
                profile: profile.clone(),
            })
        }
        Render3dMutationRequest::SetLightingParam { name, value } => {
            Some(Render3DMutation::SetProfileParam {
                param: Render3DProfileParam::Lighting(LightingProfileParam::from_name(name)?),
                value: material_value_from_json(value)?,
            })
        }
        Render3dMutationRequest::SetSpaceEnvironmentParam { name, value } => {
            Some(Render3DMutation::SetProfileParam {
                param: Render3DProfileParam::SpaceEnvironment(SpaceEnvironmentParam::from_name(
                    name,
                )?),
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
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Material(param),
                    material_value_from_json(value)?,
                )],
            })
        }
        Render3dMutationRequest::SetMaterialParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, material_grouped_param_from_name)?,
            })
        }
        Render3dMutationRequest::SetAtmosphereParam {
            target,
            name,
            value,
        } => {
            use crate::mutations::AtmosphereParam;
            let param = AtmosphereParam::from_full_path(name)?;
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Atmosphere(param),
                    material_value_from_json(value)?,
                )],
            })
        }
        Render3dMutationRequest::SetAtmosphereParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, |name| {
                    crate::mutations::AtmosphereParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Atmosphere)
                })?,
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
        Render3dMutationRequest::SetSurfaceParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, |name| {
                    crate::mutations::TerrainParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Surface)
                })?,
            })
        }
        Render3dMutationRequest::SetGeneratorParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, |name| {
                    crate::mutations::WorldgenParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Generator)
                })?,
            })
        }
        Render3dMutationRequest::SetBodyParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, |name| {
                    crate::mutations::PlanetParam::from_full_path(&canonical_group_name(name))
                        .map(Render3DGroupedParam::Body)
                })?,
            })
        }
        Render3dMutationRequest::SetViewParams { target, params } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: grouped_params_from_json(params, grouped_view_param_from_name)?,
            })
        }
        Render3dMutationRequest::SetSurfaceMode { target, mode } => {
            Some(Render3DMutation::SetGroupedParams {
                target: Some(target.clone()),
                params: vec![(
                    Render3DGroupedParam::Material(crate::mutations::ObjMaterialParam::SurfaceMode),
                    MaterialValue::Text(mode.clone()),
                )],
            })
        }
    }
}

fn canonical_group_name(name: &str) -> String {
    name.trim().to_string()
}

fn grouped_params_from_json(
    params: &JsonValue,
    mut map_name: impl FnMut(&str) -> Option<Render3DGroupedParam>,
) -> Option<Vec<(Render3DGroupedParam, MaterialValue)>> {
    let object = params.as_object()?;
    let mut grouped = Vec::with_capacity(object.len());
    for (name, value) in object {
        grouped.push((map_name(name)?, material_value_from_json(value)?));
    }
    Some(grouped)
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
}
