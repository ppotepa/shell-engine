//! Typed scene mutation request surface.
//!
//! This module is additive: it introduces typed request payloads that can be
//! produced by scripting/frontends and consumed by runtime adapters.

pub use crate::scene::camera::Camera3dMutationRequest;
pub use crate::scene::render::{
    Render3dMutationDomain, Render3dMutationRequest, Render3dProfileSlot,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Typed scene mutation request.
///
/// Camera requests primarily flow through object handles and render requests
/// primarily flow through split render domains.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SceneMutationRequest {
    /// Mutate generic 2D properties for a target object.
    #[serde(alias = "set_2d_props")]
    Set2dProps {
        target: String,
        visible: Option<bool>,
        dx: Option<i32>,
        dy: Option<i32>,
        text: Option<String>,
    },
    /// Mutate a single typed property path.
    #[serde(alias = "set_sprite_property")]
    SetSpriteProperty {
        target: String,
        path: String,
        value: JsonValue,
    },
    /// Mutate the shared 2D camera state.
    SetCamera2d { x: f32, y: f32, zoom: Option<f32> },
    /// Mutate camera state. Object-targeted camera variants are the primary
    /// multi-camera path; scene-wide variants remain available for scene-level
    /// camera state.
    SetCamera3d(Camera3dMutationRequest),
    /// Mutate typed 3D render state through split transform/material/lighting/
    /// atmosphere/generator/view domains.
    SetRender3d(Render3dMutationRequest),
    /// Carry an explicit request-construction failure through typed request
    /// building.
    RequestError { error: SceneMutationRequestError },
    /// Spawn an authored object/template instance.
    SpawnObject { template: String, target: String },
    /// Despawn a runtime object.
    DespawnObject { target: String },
}

impl SceneMutationRequest {
    /// Stable request kind label used by diagnostics and logging.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Set2dProps { .. } => "set_2d_props",
            Self::SetSpriteProperty { .. } => "set_sprite_property",
            Self::SetCamera2d { .. } => "set_camera2d",
            Self::SetCamera3d(_) => "set_camera3d",
            Self::SetRender3d(_) => "set_render3d",
            Self::RequestError { .. } => "request_error",
            Self::SpawnObject { .. } => "spawn_object",
            Self::DespawnObject { .. } => "despawn_object",
        }
    }
}

/// Explicit error returned while building a typed scene mutation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneMutationRequestError {
    UnsupportedSetPath { target: String, path: String },
    InvalidValue { target: String, path: String },
}

impl SceneMutationRequestError {
    pub fn unsupported_set_path(target: impl Into<String>, path: impl Into<String>) -> Self {
        Self::UnsupportedSetPath {
            target: target.into(),
            path: path.into(),
        }
    }

    pub fn invalid_value(target: impl Into<String>, path: impl Into<String>) -> Self {
        Self::InvalidValue {
            target: target.into(),
            path: path.into(),
        }
    }
}

/// Explicit outcome for runtime scene mutation application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMutationStatus {
    Applied,
    Rejected,
}

/// Explicit runtime scene mutation error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneMutationError {
    InvalidRequest { request: String, detail: String },
    UnsupportedRequest { request: String, detail: String },
    TargetNotFound { target: String },
}

impl SceneMutationError {
    pub fn from_request_error(error: &SceneMutationRequestError) -> Self {
        match error {
            SceneMutationRequestError::UnsupportedSetPath { target, path } => {
                Self::unsupported_request(
                    "set_path",
                    format!("target `{target}` does not support `{path}`"),
                )
            }
            SceneMutationRequestError::InvalidValue { target, path } => Self::invalid_request(
                "set_path",
                format!("target `{target}` received an invalid value for `{path}`"),
            ),
        }
    }

    pub fn invalid_request(request: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::InvalidRequest {
            request: request.into(),
            detail: detail.into(),
        }
    }

    pub fn unsupported_request(request: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::UnsupportedRequest {
            request: request.into(),
            detail: detail.into(),
        }
    }

    pub fn target_not_found(target: impl Into<String>) -> Self {
        Self::TargetNotFound {
            target: target.into(),
        }
    }
}

/// Explicit runtime scene mutation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneMutationResult {
    pub status: SceneMutationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SceneMutationError>,
}

impl SceneMutationResult {
    pub fn applied() -> Self {
        Self {
            status: SceneMutationStatus::Applied,
            error: None,
        }
    }

    pub fn rejected(error: SceneMutationError) -> Self {
        Self {
            status: SceneMutationStatus::Rejected,
            error: Some(error),
        }
    }

    pub fn is_applied(&self) -> bool {
        matches!(self.status, SceneMutationStatus::Applied)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SceneMutationError, SceneMutationRequest, SceneMutationRequestError, SceneMutationResult,
        SceneMutationStatus,
    };
    use crate::scene::{Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot};
    use serde_json::json;

    #[test]
    fn scene_mutation_request_roundtrip_json() {
        let input = SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetNodeTransform {
            target: "planet-main".to_string(),
            translation: Some([1.0, 2.0, 3.0]),
            rotation_deg: Some([0.0, 45.0, 0.0]),
            scale: Some([1.0, 1.0, 1.0]),
        });
        let encoded = serde_json::to_string(&input).expect("serialize request");
        let decoded: SceneMutationRequest =
            serde_json::from_str(&encoded).expect("deserialize request");
        assert_eq!(decoded, input);
    }

    #[test]
    fn camera_3d_request_roundtrip_json() {
        let input = SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::LookAt {
            eye: [0.0, 0.0, 5.0],
            look_at: [0.0, 0.0, 0.0],
        });
        let encoded = serde_json::to_string(&input).expect("serialize request");
        let decoded: SceneMutationRequest =
            serde_json::from_str(&encoded).expect("deserialize request");
        assert_eq!(decoded, input);
    }

    #[test]
    fn deserialize_camera3d_up_from_json_shape() {
        let raw = json!({
            "type": "set_camera3d",
            "kind": "up",
            "up": [0.0, 1.0, 0.0]
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::Up {
                up: [0.0, 1.0, 0.0]
            })
        );
    }

    #[test]
    fn camera_3d_object_requests_roundtrip_distinct_targets() {
        let requests = [
            SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectLookAt {
                target: "cockpit-camera".to_string(),
                eye: [1.0, 2.0, 3.0],
                look_at: [0.0, 0.0, 0.0],
                up: Some([0.0, 1.0, 0.0]),
            }),
            SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectBasis {
                target: "chase-camera".to_string(),
                eye: [10.0, 5.0, -2.0],
                right: [1.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                forward: [0.0, 0.0, 1.0],
            }),
        ];

        for request in requests {
            let encoded = serde_json::to_string(&request).expect("serialize request");
            let decoded: SceneMutationRequest =
                serde_json::from_str(&encoded).expect("deserialize request");
            assert_eq!(decoded, request);
        }
    }

    #[test]
    fn deserialize_camera3d_object_basis_from_json_shape() {
        let raw = json!({
            "type": "set_camera3d",
            "kind": "object_basis",
            "target": "chase-camera",
            "eye": [10.0, 5.0, -2.0],
            "right": [1.0, 0.0, 0.0],
            "up": [0.0, 1.0, 0.0],
            "forward": [0.0, 0.0, 1.0]
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");

        assert_eq!(
            decoded,
            SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::ObjectBasis {
                target: "chase-camera".to_string(),
                eye: [10.0, 5.0, -2.0],
                right: [1.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                forward: [0.0, 0.0, 1.0],
            })
        );
    }

    #[test]
    fn deserialize_render3d_surface_mode_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_surface_mode",
            "target": "planet-main",
            "mode": "wireframe"
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetSurfaceMode {
                target: "planet-main".to_string(),
                mode: "wireframe".to_string(),
            })
        );
    }

    #[test]
    fn deserialize_render3d_view_profile_switch_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_view_profile",
            "profile": "orbit-realistic"
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetViewProfile {
                profile: "orbit-realistic".to_string(),
            })
        );
    }

    #[test]
    fn deserialize_render3d_neutral_profile_switch_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_profile",
            "profile_slot": "lighting",
            "profile": "space-hard-vacuum"
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetProfile {
                profile_slot: Render3dProfileSlot::Lighting,
                profile: "space-hard-vacuum".to_string(),
            })
        );
    }

    #[test]
    fn deserialize_render3d_neutral_profile_param_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_profile_param",
            "profile_slot": "space_environment",
            "name": "background_color",
            "value": "#010203"
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetProfileParam {
                profile_slot: Render3dProfileSlot::SpaceEnvironment,
                name: "background_color".to_string(),
                value: json!("#010203"),
            })
        );
    }

    #[test]
    fn deserialize_render3d_grouped_material_params_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_material_params",
            "target": "planet-main",
            "params": {
                "diffuse_color": "#aabbcc",
                "roughness": 0.3
            }
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetMaterialParams {
                target: "planet-main".to_string(),
                params: json!({
                    "diffuse_color": "#aabbcc",
                    "roughness": 0.3
                }),
            })
        );
    }

    #[test]
    fn deserialize_render3d_grouped_view_params_from_json_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "set_view_params",
            "target": "planet-main",
            "params": {
                "distance": 12.0,
                "yaw_deg": 25.0
            }
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetViewParams {
                target: "planet-main".to_string(),
                params: json!({
                    "distance": 12.0,
                    "yaw_deg": 25.0
                }),
            })
        );
    }

    #[test]
    fn deserialize_set_2d_props_from_json_shape() {
        let raw = json!({
            "type": "set_2d_props",
            "target": "hud-label",
            "visible": true,
            "dx": 12,
            "dy": -3,
            "text": "Hello"
        });
        let decoded: SceneMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");
        assert_eq!(
            decoded,
            SceneMutationRequest::Set2dProps {
                target: "hud-label".to_string(),
                visible: Some(true),
                dx: Some(12),
                dy: Some(-3),
                text: Some("Hello".to_string()),
            }
        );
    }

    #[test]
    fn reject_invalid_mutation_shape() {
        let raw = json!({
            "type": "set_render3d",
            "kind": "unknown_kind",
            "target": "planet-main"
        });
        let decoded = serde_json::from_value::<SceneMutationRequest>(raw);
        assert!(decoded.is_err());
    }

    #[test]
    fn scene_mutation_result_roundtrips_json() {
        let input = SceneMutationResult::rejected(SceneMutationError::target_not_found("hud"));
        let encoded = serde_json::to_string(&input).expect("serialize result");
        let decoded: SceneMutationResult =
            serde_json::from_str(&encoded).expect("deserialize result");

        assert_eq!(
            decoded,
            SceneMutationResult {
                status: SceneMutationStatus::Rejected,
                error: Some(SceneMutationError::TargetNotFound {
                    target: "hud".to_string(),
                }),
            }
        );
    }

    #[test]
    fn scene_mutation_request_error_roundtrips_json() {
        let input = SceneMutationRequestError::invalid_value("hud", "visible");
        let encoded = serde_json::to_string(&input).expect("serialize request error");
        let decoded: SceneMutationRequestError =
            serde_json::from_str(&encoded).expect("deserialize request error");

        assert_eq!(
            decoded,
            SceneMutationRequestError::InvalidValue {
                target: "hud".to_string(),
                path: "visible".to_string(),
            }
        );
    }

    #[test]
    fn scene_mutation_error_maps_request_error() {
        let unsupported = SceneMutationError::from_request_error(
            &SceneMutationRequestError::unsupported_set_path("hud", "audio.pitch"),
        );
        let invalid = SceneMutationError::from_request_error(
            &SceneMutationRequestError::invalid_value("hud", "visible"),
        );

        assert_eq!(
            unsupported,
            SceneMutationError::UnsupportedRequest {
                request: "set_path".to_string(),
                detail: "target `hud` does not support `audio.pitch`".to_string(),
            }
        );
        assert_eq!(
            invalid,
            SceneMutationError::InvalidRequest {
                request: "set_path".to_string(),
                detail: "target `hud` received an invalid value for `visible`".to_string(),
            }
        );
    }
}
