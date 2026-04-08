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
    SceneSpawn {
        template: String,
        target: String,
    },
    SceneDespawn {
        target: String,
    },
    TerminalPushOutput {
        line: String,
    },
    TerminalClearOutput,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLogSeverity {
    Info,
    Warn,
    Error,
}
