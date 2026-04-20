//! Side-effect commands produced by scripts and consumed by engine systems.

use engine_core::scene_runtime_types::ObjectRuntimeState;
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
    /// Apply a typed scene mutation request.
    ApplySceneMutation {
        request: crate::scene::SceneMutationRequest,
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

fn rounded_i32(value: &JsonValue) -> Option<i32> {
    if let Some(number) = value.as_i64() {
        return i32::try_from(number).ok();
    }
    value
        .as_f64()
        .and_then(|number| i32::try_from(number.round() as i64).ok())
}

pub fn is_supported_scene_set_path(path: &str) -> bool {
    matches!(
        path,
        "visible"
            | "text.content"
            | "transform.heading"
            | "text.font"
            | "style.fg"
            | "text.fg"
            | "style.bg"
            | "text.bg"
            | "vector.points"
            | "vector.closed"
            | "vector.draw_char"
            | "vector.fg"
            | "vector.bg"
            | "style.border"
            | "style.shadow"
            | "image.frame_index"
            | "offset.x"
            | "position.x"
            | "offset.y"
            | "position.y"
    ) || is_render3d_set_path(path)
}

pub fn scene_mutation_request_from_set_path(
    target: &str,
    path: &str,
    value: &JsonValue,
    current_state: Option<&ObjectRuntimeState>,
) -> Option<crate::scene::SceneMutationRequest> {
    if !is_supported_scene_set_path(path) {
        return None;
    }
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
        "transform.heading" => Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
            target: target.to_string(),
            path: "transform.heading".to_string(),
            value: value.clone(),
        }),
        "text.font" => Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
            target: target.to_string(),
            path: "text.font".to_string(),
            value: value.clone(),
        }),
        "style.fg" | "text.fg" => Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
            target: target.to_string(),
            path: "style.fg".to_string(),
            value: value.clone(),
        }),
        "style.bg" | "text.bg" => Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
            target: target.to_string(),
            path: "style.bg".to_string(),
            value: value.clone(),
        }),
        "vector.points" | "vector.closed" | "vector.draw_char" | "vector.fg" | "vector.bg"
        | "style.border" | "style.shadow" => {
            Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
                target: target.to_string(),
                path: path.to_string(),
                value: value.clone(),
            })
        }
        "image.frame_index" => Some(crate::scene::SceneMutationRequest::SetSpriteProperty {
            target: target.to_string(),
            path: "image.frame_index".to_string(),
            value: value.clone(),
        }),
        "offset.x" | "position.x" => {
            let state = current_state.cloned().unwrap_or_default();
            let next_x = rounded_i32(value)?;
            Some(crate::scene::SceneMutationRequest::Set2dProps {
                target: target.to_string(),
                visible: None,
                dx: Some(next_x.saturating_sub(state.offset_x)),
                dy: None,
                text: None,
            })
        }
        "offset.y" | "position.y" => {
            let state = current_state.cloned().unwrap_or_default();
            let next_y = rounded_i32(value)?;
            Some(crate::scene::SceneMutationRequest::Set2dProps {
                target: target.to_string(),
                visible: None,
                dx: None,
                dy: Some(next_y.saturating_sub(state.offset_y)),
                text: None,
            })
        }
        _ if is_render3d_set_path(path) => render3d_request_from_set_path(target, path, value)
            .map(crate::scene::SceneMutationRequest::SetRender3d),
        _ => None,
    }
}

fn render3d_request_from_set_path(
    target: &str,
    path: &str,
    value: &JsonValue,
) -> Option<crate::scene::Render3dMutationRequest> {
    let mut params = serde_json::Map::new();
    if let Some(name) = path.strip_prefix("obj.atmo.") {
        params.insert(name.replace('-', "_"), value.clone());
        return Some(crate::scene::Render3dMutationRequest::SetAtmosphereParams {
            target: target.to_string(),
            params: JsonValue::Object(params),
        });
    }
    if let Some(name) = path.strip_prefix("obj.") {
        let group_name = match name {
            "camera-distance" => "distance".to_string(),
            other => other.replace('-', "_"),
        };
        params.insert(group_name, value.clone());
        return Some(crate::scene::Render3dMutationRequest::SetMaterialParams {
            target: target.to_string(),
            params: JsonValue::Object(params),
        });
    }
    if let Some(name) = path.strip_prefix("terrain.") {
        params.insert(name.replace('-', "_"), value.clone());
        return Some(crate::scene::Render3dMutationRequest::SetSurfaceParams {
            target: target.to_string(),
            params: JsonValue::Object(params),
        });
    }
    if let Some(name) = path.strip_prefix("world.") {
        params.insert(name.replace('-', "_"), value.clone());
        return Some(crate::scene::Render3dMutationRequest::SetGeneratorParams {
            target: target.to_string(),
            params: JsonValue::Object(params),
        });
    }
    if let Some(name) = path.strip_prefix("planet.") {
        params.insert(name.replace('.', "_").replace('-', "_"), value.clone());
        return Some(crate::scene::Render3dMutationRequest::SetBodyParams {
            target: target.to_string(),
            params: JsonValue::Object(params),
        });
    }
    if path == "scene3d.frame" {
        return Some(crate::scene::Render3dMutationRequest::SetWorldParam {
            target: target.to_string(),
            name: path.to_string(),
            value: value.clone(),
        });
    }
    None
}

fn is_render3d_set_path(path: &str) -> bool {
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
        let request = scene_mutation_request_from_set_path(
            "planet",
            "obj.world.x",
            &serde_json::json!(1.5),
            None,
        )
        .expect("typed request");

        assert_eq!(
            request,
            SceneMutationRequest::SetRender3d(Render3dMutationRequest::SetMaterialParams {
                target: "planet".to_string(),
                params: serde_json::json!({
                    "world.x": 1.5,
                }),
            })
        );
    }

    #[test]
    fn maps_text_content_set_property_to_typed_2d_request() {
        let request =
            scene_mutation_request_from_set_path("hud", "text.content", &"HELLO".into(), None)
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

    #[test]
    fn maps_position_x_to_delta_when_state_is_available() {
        let request = scene_mutation_request_from_set_path(
            "hud",
            "position.x",
            &serde_json::json!(9.8),
            Some(&ObjectRuntimeState {
                offset_x: 4,
                ..ObjectRuntimeState::default()
            }),
        )
        .expect("typed request");

        assert_eq!(
            request,
            SceneMutationRequest::Set2dProps {
                target: "hud".to_string(),
                visible: None,
                dx: Some(6),
                dy: None,
                text: None,
            }
        );
    }

    #[test]
    fn reports_supported_text_paths() {
        assert!(is_supported_scene_set_path("text.content"));
        assert!(is_supported_scene_set_path("text.font"));
        assert!(is_supported_scene_set_path("style.fg"));
        assert!(is_supported_scene_set_path("style.bg"));
        assert!(!is_supported_scene_set_path("text.color"));
    }
}
