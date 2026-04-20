use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Slot selector for split render-domain profile mutations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Render3dProfileSlot {
    View,
    Lighting,
    SpaceEnvironment,
}

/// Primary routing domains for typed 3D render mutations.
///
/// Public callers should treat this split as the primary render API surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Render3dMutationDomain {
    Transform,
    Material,
    Lighting,
    Atmosphere,
    Generator,
    View,
}

/// Typed split-domain 3D render mutation request.
///
/// `domain()` is the primary routing surface used by runtime adapters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Render3dMutationRequest {
    /// Switch an active render profile by split-domain slot.
    SetProfile {
        profile_slot: Render3dProfileSlot,
        profile: String,
    },
    /// Override a single render profile field by split-domain slot.
    SetProfileParam {
        profile_slot: Render3dProfileSlot,
        name: String,
        value: JsonValue,
    },
    /// Switch the active top-level view profile.
    SetViewProfile { profile: String },
    /// Switch the active lighting profile feeding the resolved view.
    SetLightingProfile { profile: String },
    /// Switch the active space-environment profile feeding the resolved view.
    SetSpaceEnvironmentProfile { profile: String },
    /// Override a single lighting-profile field on the active resolved view.
    SetLightingParam { name: String, value: JsonValue },
    /// Override a single space-environment-profile field on the active resolved view.
    SetSpaceEnvironmentParam { name: String, value: JsonValue },
    /// Set transform-domain values for a render node.
    SetNodeTransform {
        target: String,
        translation: Option<[f32; 3]>,
        rotation_deg: Option<[f32; 3]>,
        scale: Option<[f32; 3]>,
    },
    /// Set a material-domain parameter by name.
    SetMaterialParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Additively set multiple material-domain parameters by name.
    SetMaterialParams { target: String, params: JsonValue },
    /// Set an atmosphere-domain parameter by name.
    SetAtmosphereParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Additively set multiple atmosphere-domain parameters by name.
    SetAtmosphereParams { target: String, params: JsonValue },
    /// Set a generator-domain render path by name.
    SetWorldParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Additively set multiple generator-domain surface parameters by name.
    SetSurfaceParams { target: String, params: JsonValue },
    /// Additively set multiple generator-domain parameters by name.
    SetGeneratorParams { target: String, params: JsonValue },
    /// Additively set multiple generator-domain body parameters by name.
    SetBodyParams { target: String, params: JsonValue },
    /// Additively set multiple view-domain parameters by name.
    SetViewParams { target: String, params: JsonValue },
    /// Set a material-domain surface mode using a typed string value.
    SetSurfaceMode { target: String, mode: String },
}

impl Render3dMutationRequest {
    /// Stable request kind label used by diagnostics and logging.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::SetProfile { .. } => "set_profile",
            Self::SetProfileParam { .. } => "set_profile_param",
            Self::SetViewProfile { .. } => "set_view_profile",
            Self::SetLightingProfile { .. } => "set_lighting_profile",
            Self::SetSpaceEnvironmentProfile { .. } => "set_space_environment_profile",
            Self::SetLightingParam { .. } => "set_lighting_param",
            Self::SetSpaceEnvironmentParam { .. } => "set_space_environment_param",
            Self::SetNodeTransform { .. } => "set_node_transform",
            Self::SetMaterialParam { .. } => "set_material_param",
            Self::SetMaterialParams { .. } => "set_material_params",
            Self::SetAtmosphereParam { .. } => "set_atmosphere_param",
            Self::SetAtmosphereParams { .. } => "set_atmosphere_params",
            Self::SetWorldParam { .. } => "set_world_param",
            Self::SetSurfaceParams { .. } => "set_surface_params",
            Self::SetGeneratorParams { .. } => "set_generator_params",
            Self::SetBodyParams { .. } => "set_body_params",
            Self::SetViewParams { .. } => "set_view_params",
            Self::SetSurfaceMode { .. } => "set_surface_mode",
        }
    }

    /// Resolve the split render domain that should handle this request.
    pub fn domain(&self) -> Render3dMutationDomain {
        match self {
            Self::SetNodeTransform { .. } => Render3dMutationDomain::Transform,
            Self::SetMaterialParam { .. }
            | Self::SetMaterialParams { .. }
            | Self::SetSurfaceMode { .. } => Render3dMutationDomain::Material,
            Self::SetAtmosphereParam { .. } | Self::SetAtmosphereParams { .. } => {
                Render3dMutationDomain::Atmosphere
            }
            Self::SetWorldParam { .. }
            | Self::SetSurfaceParams { .. }
            | Self::SetGeneratorParams { .. }
            | Self::SetBodyParams { .. } => Render3dMutationDomain::Generator,
            Self::SetProfile {
                profile_slot: Render3dProfileSlot::View,
                ..
            }
            | Self::SetProfileParam {
                profile_slot: Render3dProfileSlot::View,
                ..
            }
            | Self::SetViewProfile { .. }
            | Self::SetViewParams { .. } => Render3dMutationDomain::View,
            Self::SetProfile { .. }
            | Self::SetProfileParam { .. }
            | Self::SetLightingProfile { .. }
            | Self::SetSpaceEnvironmentProfile { .. }
            | Self::SetLightingParam { .. }
            | Self::SetSpaceEnvironmentParam { .. } => Render3dMutationDomain::Lighting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Render3dMutationDomain, Render3dMutationRequest, Render3dProfileSlot};
    use serde_json::json;

    #[test]
    fn render_request_reports_expected_domain_for_each_split_subdomain() {
        let cases = vec![
            (
                Render3dMutationRequest::SetNodeTransform {
                    target: "planet-main".to_string(),
                    translation: None,
                    rotation_deg: None,
                    scale: None,
                },
                Render3dMutationDomain::Transform,
            ),
            (
                Render3dMutationRequest::SetMaterialParam {
                    target: "planet-main".to_string(),
                    name: "scale".to_string(),
                    value: json!(1.25),
                },
                Render3dMutationDomain::Material,
            ),
            (
                Render3dMutationRequest::SetMaterialParams {
                    target: "planet-main".to_string(),
                    params: json!({"scale": 1.25}),
                },
                Render3dMutationDomain::Material,
            ),
            (
                Render3dMutationRequest::SetSurfaceMode {
                    target: "planet-main".to_string(),
                    mode: "wireframe".to_string(),
                },
                Render3dMutationDomain::Material,
            ),
            (
                Render3dMutationRequest::SetAtmosphereParam {
                    target: "planet-main".to_string(),
                    name: "density".to_string(),
                    value: json!(0.4),
                },
                Render3dMutationDomain::Atmosphere,
            ),
            (
                Render3dMutationRequest::SetAtmosphereParams {
                    target: "planet-main".to_string(),
                    params: json!({"density": 0.4}),
                },
                Render3dMutationDomain::Atmosphere,
            ),
            (
                Render3dMutationRequest::SetWorldParam {
                    target: "planet-main".to_string(),
                    name: "world.seed".to_string(),
                    value: json!(42),
                },
                Render3dMutationDomain::Generator,
            ),
            (
                Render3dMutationRequest::SetGeneratorParams {
                    target: "planet-main".to_string(),
                    params: json!({"seed": 42}),
                },
                Render3dMutationDomain::Generator,
            ),
            (
                Render3dMutationRequest::SetViewProfile {
                    profile: "orbit-realistic".to_string(),
                },
                Render3dMutationDomain::View,
            ),
            (
                Render3dMutationRequest::SetProfile {
                    profile_slot: Render3dProfileSlot::View,
                    profile: "orbit-realistic".to_string(),
                },
                Render3dMutationDomain::View,
            ),
            (
                Render3dMutationRequest::SetProfile {
                    profile_slot: Render3dProfileSlot::Lighting,
                    profile: "space-hard-vacuum".to_string(),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetProfile {
                    profile_slot: Render3dProfileSlot::SpaceEnvironment,
                    profile: "deep-space".to_string(),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetProfileParam {
                    profile_slot: Render3dProfileSlot::View,
                    name: "distance".to_string(),
                    value: json!(12.0),
                },
                Render3dMutationDomain::View,
            ),
            (
                Render3dMutationRequest::SetProfileParam {
                    profile_slot: Render3dProfileSlot::Lighting,
                    name: "exposure".to_string(),
                    value: json!(0.82),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetLightingProfile {
                    profile: "sunset-rim".to_string(),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetSpaceEnvironmentProfile {
                    profile: "deep-space".to_string(),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetLightingParam {
                    name: "exposure".to_string(),
                    value: json!(0.82),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetSpaceEnvironmentParam {
                    name: "background_color".to_string(),
                    value: json!("#010203"),
                },
                Render3dMutationDomain::Lighting,
            ),
            (
                Render3dMutationRequest::SetSurfaceParams {
                    target: "planet-main".to_string(),
                    params: json!({"amplitude": 0.4}),
                },
                Render3dMutationDomain::Generator,
            ),
            (
                Render3dMutationRequest::SetBodyParams {
                    target: "planet-main".to_string(),
                    params: json!({"spin_deg": 15.0}),
                },
                Render3dMutationDomain::Generator,
            ),
            (
                Render3dMutationRequest::SetViewParams {
                    target: "planet-main".to_string(),
                    params: json!({"distance": 12.0}),
                },
                Render3dMutationDomain::View,
            ),
        ];

        for (request, expected) in cases {
            assert_eq!(request.domain(), expected);
        }
    }

    #[test]
    fn deserialize_render_view_profile_switch_from_json_shape() {
        let raw = json!({
            "kind": "set_view_profile",
            "profile": "orbit-realistic"
        });
        let decoded: Render3dMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");

        assert_eq!(
            decoded,
            Render3dMutationRequest::SetViewProfile {
                profile: "orbit-realistic".to_string(),
            }
        );
    }

    #[test]
    fn deserialize_render_neutral_profile_switch_from_json_shape() {
        let raw = json!({
            "kind": "set_profile",
            "profile_slot": "lighting",
            "profile": "space-hard-vacuum"
        });
        let decoded: Render3dMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");

        assert_eq!(
            decoded,
            Render3dMutationRequest::SetProfile {
                profile_slot: Render3dProfileSlot::Lighting,
                profile: "space-hard-vacuum".to_string(),
            }
        );
    }
}
