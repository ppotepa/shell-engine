//! Typed scene mutation request surface.
//!
//! This module is additive: it introduces typed request payloads that can be
//! produced by scripting/frontends and consumed by runtime adapters.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Typed scene mutation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SceneMutationRequest {
    /// Mutate generic 2D properties for a target object.
    Set2dProps {
        target: String,
        visible: Option<bool>,
        dx: Option<i32>,
        dy: Option<i32>,
        text: Option<String>,
    },
    /// Mutate the shared 2D camera state.
    SetCamera2d {
        x: f32,
        y: f32,
        zoom: Option<f32>,
    },
    /// Mutate the shared 3D camera state.
    SetCamera3d(Camera3dMutationRequest),
    /// Mutate typed 3D render/domain state.
    SetRender3d(Render3dMutationRequest),
    /// Spawn an authored object/template instance.
    SpawnObject { template: String, target: String },
    /// Despawn a runtime object.
    DespawnObject { target: String },
}

/// Typed 3D camera mutation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Camera3dMutationRequest {
    /// Set camera eye/look-at pair.
    LookAt {
        eye: [f32; 3],
        look_at: [f32; 3],
    },
    /// Set camera up vector.
    Up { up: [f32; 3] },
}

/// Typed 3D render mutation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Render3dMutationRequest {
    /// Set transform values for a render node.
    SetNodeTransform {
        target: String,
        translation: Option<[f32; 3]>,
        rotation_deg: Option<[f32; 3]>,
        scale: Option<[f32; 3]>,
    },
    /// Set a material parameter by name.
    SetMaterialParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Set an atmosphere parameter by name.
    SetAtmosphereParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Set a world/profile parameter by name.
    SetWorldParam {
        target: String,
        name: String,
        value: JsonValue,
    },
    /// Set surface mode using a typed string value.
    SetSurfaceMode { target: String, mode: String },
}

#[cfg(test)]
mod tests {
    use super::{Camera3dMutationRequest, Render3dMutationRequest, SceneMutationRequest};

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
}
