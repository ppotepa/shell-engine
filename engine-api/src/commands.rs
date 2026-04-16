//! Side-effect commands produced by scripts and consumed by engine systems.

use serde_json::Value as JsonValue;

/// A side-effect produced by a behavior and consumed by the engine systems.
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorCommand {
    PlayAudioCue {
        cue: String,
        volume: Option<f32>,
    },
    PlayAudioEvent {
        event: String,
        gain: Option<f32>,
    },
    PlaySong {
        song_id: String,
    },
    StopSong,
    SetVisibility {
        target: String,
        visible: bool,
    },
    SetOffset {
        target: String,
        dx: i32,
        dy: i32,
    },
    SetText {
        target: String,
        text: String,
    },
    SetProps {
        target: String,
        visible: Option<bool>,
        dx: Option<i32>,
        dy: Option<i32>,
        text: Option<String>,
    },
    SetProperty {
        target: String,
        path: String,
        value: JsonValue,
    },
    /// Apply a typed scene mutation request.
    ApplySceneMutation {
        request: crate::scene::SceneMutationRequest,
    },
    SceneSpawn {
        template: String,
        target: String,
    },
    SceneDespawn {
        target: String,
    },
    SceneTransition {
        to_scene_id: String,
    },
    DebugLog {
        scene_id: String,
        source: Option<String>,
        severity: DebugLogSeverity,
        message: String,
    },
    /// Rhai script error — consumed by the behavior system to push to DebugLogBuffer.
    ScriptError {
        scene_id: String,
        source: Option<String>,
        message: String,
    },
    /// Register or overwrite an input action binding (name → list of key codes).
    BindInputAction {
        action: String,
        keys: Vec<String>,
    },
    /// Trigger a named visual effect at runtime (not tied to authored scene steps).
    ///
    /// `params` is a JSON map carrying optional EffectParams fields
    /// (amplitude_x, amplitude_y, frequency, intensity, alpha, …).
    TriggerEffect {
        name: String,
        duration_ms: u64,
        looping: bool,
        params: serde_json::Value,
    },
    /// Programmatically set a GUI widget's value (e.g. reset slider from script).
    SetGuiValue {
        widget_id: String,
        value: f64,
    },
    /// Change the scene background color at runtime.
    SetSceneBg {
        color: String,
    },
    /// Move the world-space camera (viewport origin in world pixels).
    ///
    /// Non-UI layers are shifted by `(-x, -y)` before rendering so world-pos `(x, y)`
    /// maps to screen center. UI layers are not affected.
    SetCamera {
        x: f32,
        y: f32,
    },
    /// Set the 2D camera zoom factor (default 1.0).
    ///
    /// Values > 1.0 zoom in (fewer world pixels visible), < 1.0 zoom out.
    /// Non-UI layers are scaled by this factor around the camera centre.
    SetCameraZoom {
        zoom: f32,
    },
    /// Set the shared scene-level 3D camera eye/target pair.
    SetCamera3DLookAt {
        eye: [f32; 3],
        look_at: [f32; 3],
    },
    /// Set the shared scene-level 3D camera up vector.
    SetCamera3DUp {
        up: [f32; 3],
    },
}

pub fn scene_mutation_request_from_set_property_compat(
    target: &str,
    path: &str,
    value: &JsonValue,
) -> Option<crate::scene::SceneMutationRequest> {
    match path {
        "visible" => Some(crate::scene::SceneMutationRequest::Set2dProps {
            target: target.to_string(),
            visible: Some(value.as_bool()?),
            dx: None,
            dy: None,
            text: None,
        }),
        "text.content" => Some(crate::scene::SceneMutationRequest::Set2dProps {
            target: target.to_string(),
            visible: None,
            dx: None,
            dy: None,
            text: Some(value.as_str()?.to_string()),
        }),
        _ if is_render3d_compat_set_path(path) => {
            Some(crate::scene::SceneMutationRequest::SetRender3d(
                crate::scene::Render3dMutationRequest::SetWorldParam {
                    target: target.to_string(),
                    name: path.to_string(),
                    value: value.clone(),
                },
            ))
        }
        _ => None,
    }
}

fn is_render3d_compat_set_path(path: &str) -> bool {
    path == "scene3d.frame"
        || path.starts_with("planet.")
        || path.starts_with("obj.")
        || path.starts_with("terrain.")
        || path.starts_with("world.")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLogSeverity {
    Info,
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Render3dMutationRequest, SceneMutationRequest};

    #[test]
    fn maps_render_set_property_to_typed_render3d_request() {
        let request = scene_mutation_request_from_set_property_compat(
            "planet",
            "obj.world.x",
            &serde_json::json!(1.5),
        )
        .expect("typed request");

        assert_eq!(
            request,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetWorldParam {
                target: "planet".to_string(),
                name: "obj.world.x".to_string(),
                value: serde_json::json!(1.5),
            })
        );
    }

    #[test]
    fn maps_text_content_set_property_to_typed_2d_request() {
        let request =
            scene_mutation_request_from_set_property_compat("hud", "text.content", &"HELLO".into())
                .expect("typed request");

        assert_eq!(
            request,
            SceneMutationRequest::Set2dProps {
                target: "hud".to_string(),
                visible: None,
                dx: None,
                dy: None,
                text: Some("HELLO".to_string()),
            }
        );
    }
}
