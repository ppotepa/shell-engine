use engine_core::scene_runtime_types::SceneCamera3D;
use serde::{Deserialize, Serialize};

/// Typed 3D camera mutation requests.
///
/// Object-targeted camera mutations are the primary multi-camera path.
/// Scene-wide variants remain available for scene-level camera state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Camera3dMutationRequest {
    /// Update the scene-wide camera eye/look-at pair.
    LookAt { eye: [f32; 3], look_at: [f32; 3] },
    /// Update the scene-wide camera up vector.
    Up { up: [f32; 3] },
    /// Drive a scene object as a camera by supplying eye/look-at vectors.
    ObjectLookAt {
        target: String,
        eye: [f32; 3],
        look_at: [f32; 3],
        #[serde(default)]
        up: Option<[f32; 3]>,
    },
    /// Drive a scene object as a camera by supplying a full view basis.
    ObjectBasis {
        target: String,
        eye: [f32; 3],
        right: [f32; 3],
        up: [f32; 3],
        forward: [f32; 3],
    },
}

/// Normalized object-camera view state used when lowering object-handle camera
/// mutations into render-domain grouped params.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3dObjectViewState {
    pub eye: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub forward: [f32; 3],
}

impl Camera3dObjectViewState {
    /// Build an object-camera view state from an explicit basis.
    pub fn from_basis(eye: [f32; 3], right: [f32; 3], up: [f32; 3], forward: [f32; 3]) -> Self {
        Self {
            eye,
            right: normalize_or(right, [1.0, 0.0, 0.0]),
            up: normalize_or(up, [0.0, 1.0, 0.0]),
            forward: normalize_or(forward, [0.0, 0.0, 1.0]),
        }
    }

    /// Build an object-camera view state from an eye/look-at pair.
    pub fn from_look_at(eye: [f32; 3], look_at: [f32; 3], up_hint: [f32; 3]) -> Self {
        let forward = normalize_or(sub(look_at, eye), [0.0, 0.0, 1.0]);
        let up_hint = normalize_or(up_hint, [0.0, 1.0, 0.0]);
        let mut right = cross(forward, up_hint);
        if vector_len(right) <= 1e-6 {
            let fallback_up = if forward[1].abs() < 0.99 {
                [0.0, 1.0, 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            right = cross(forward, fallback_up);
        }
        let right = normalize_or(right, [1.0, 0.0, 0.0]);
        let up = normalize_or(cross(right, forward), up_hint);
        Self {
            eye,
            right,
            up,
            forward,
        }
    }
}

/// Normalized camera mutation payload shared by scene-wide and object-targeted
/// requests.
#[derive(Debug, Clone, PartialEq)]
pub enum Camera3dNormalizedMutation {
    Scene(SceneCamera3D),
    Object {
        target: String,
        view: Camera3dObjectViewState,
    },
}

impl Camera3dMutationRequest {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::LookAt { .. } => "look_at",
            Self::Up { .. } => "up",
            Self::ObjectLookAt { .. } => "object_look_at",
            Self::ObjectBasis { .. } => "object_basis",
        }
    }

    pub fn normalized(&self, current_camera_3d: SceneCamera3D) -> Camera3dNormalizedMutation {
        match self {
            Self::LookAt { eye, look_at } => Camera3dNormalizedMutation::Scene(SceneCamera3D {
                eye: *eye,
                look_at: *look_at,
                ..current_camera_3d
            }),
            Self::Up { up } => Camera3dNormalizedMutation::Scene(SceneCamera3D {
                up: *up,
                ..current_camera_3d
            }),
            Self::ObjectLookAt {
                target,
                eye,
                look_at,
                up,
            } => Camera3dNormalizedMutation::Object {
                target: target.clone(),
                view: Camera3dObjectViewState::from_look_at(
                    *eye,
                    *look_at,
                    up.as_ref().copied().unwrap_or([0.0, 1.0, 0.0]),
                ),
            },
            Self::ObjectBasis {
                target,
                eye,
                right,
                up,
                forward,
            } => Camera3dNormalizedMutation::Object {
                target: target.clone(),
                view: Camera3dObjectViewState::from_basis(*eye, *right, *up, *forward),
            },
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vector_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize_or(v: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = vector_len(v);
    if len <= 1e-6 {
        return fallback;
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::{Camera3dMutationRequest, Camera3dNormalizedMutation, Camera3dObjectViewState};
    use engine_core::scene_runtime_types::SceneCamera3D;
    use serde_json::json;

    #[test]
    fn camera_object_view_state_builds_basis_from_look_at() {
        let view = Camera3dObjectViewState::from_look_at(
            [0.0, 0.0, -5.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );

        assert_eq!(view.eye, [0.0, 0.0, -5.0]);
        assert_eq!(view.right, [-1.0, 0.0, 0.0]);
        assert_eq!(view.up, [0.0, 1.0, -0.0]);
        assert_eq!(view.forward, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn deserialize_object_camera_request_from_json_shape() {
        let raw = json!({
            "kind": "object_look_at",
            "target": "cockpit-camera",
            "eye": [1.0, 2.0, 3.0],
            "look_at": [0.0, 0.0, 0.0]
        });
        let decoded: Camera3dMutationRequest =
            serde_json::from_value(raw).expect("deserialize request");

        assert_eq!(
            decoded,
            Camera3dMutationRequest::ObjectLookAt {
                target: "cockpit-camera".to_string(),
                eye: [1.0, 2.0, 3.0],
                look_at: [0.0, 0.0, 0.0],
                up: None,
            }
        );
    }

    #[test]
    fn object_camera_requests_keep_distinct_handles_and_kinds() {
        let cockpit = Camera3dMutationRequest::ObjectLookAt {
            target: "cockpit-camera".to_string(),
            eye: [1.0, 2.0, 3.0],
            look_at: [0.0, 0.0, 0.0],
            up: None,
        };
        let chase = Camera3dMutationRequest::ObjectBasis {
            target: "chase-camera".to_string(),
            eye: [7.0, 8.0, 9.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            forward: [0.0, 0.0, 1.0],
        };

        assert_eq!(cockpit.kind_name(), "object_look_at");
        assert_eq!(chase.kind_name(), "object_basis");
        assert_ne!(cockpit, chase);
    }

    #[test]
    fn scene_and_object_camera_mutations_normalize_through_one_backbone() {
        let scene_request = Camera3dMutationRequest::LookAt {
            eye: [9.0, 8.0, 7.0],
            look_at: [6.0, 5.0, 4.0],
        };
        let object_request = Camera3dMutationRequest::ObjectLookAt {
            target: "cockpit-camera".to_string(),
            eye: [1.0, 2.0, 3.0],
            look_at: [0.0, 0.0, 0.0],
            up: Some([0.0, 1.0, 0.0]),
        };

        match scene_request.normalized(SceneCamera3D::default()) {
            Camera3dNormalizedMutation::Scene(camera) => {
                assert_eq!(camera.eye, [9.0, 8.0, 7.0]);
                assert_eq!(camera.look_at, [6.0, 5.0, 4.0]);
            }
            _ => panic!("scene mutation should normalize to scene camera state"),
        }

        match object_request.normalized(SceneCamera3D::default()) {
            Camera3dNormalizedMutation::Object { target, view } => {
                assert_eq!(target, "cockpit-camera");
                assert_eq!(view.eye, [1.0, 2.0, 3.0]);
            }
            _ => panic!("object mutation should normalize to object camera state"),
        }
    }
}
