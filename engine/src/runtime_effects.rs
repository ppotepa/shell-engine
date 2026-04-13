//! Runtime effects resource: tracks visual effects triggered by Rhai scripts.
//!
//! These effects run independently of the authored scene-step effect pipeline.
//! They are stored as a scoped world resource and cleared on scene transitions.

use engine_core::scene::{Effect, EffectParams};
use serde_json::Value as JsonValue;

/// A single runtime-triggered effect entry.
#[derive(Debug, Clone)]
pub struct RuntimeEffect {
    pub name: String,
    pub started_ms: u64,
    pub duration_ms: u64,
    pub looping: bool,
    pub params: EffectParams,
}

impl RuntimeEffect {
    /// Progress 0.0–1.0 of this effect at the given scene elapsed time.
    pub fn progress(&self, scene_elapsed_ms: u64) -> f32 {
        if self.duration_ms == 0 {
            return 1.0;
        }
        let elapsed = scene_elapsed_ms.saturating_sub(self.started_ms);
        if self.looping {
            let cycle = elapsed % self.duration_ms;
            cycle as f32 / self.duration_ms as f32
        } else {
            (elapsed as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
        }
    }

    /// Whether this effect has expired at the given scene elapsed time.
    pub fn is_expired(&self, scene_elapsed_ms: u64) -> bool {
        if self.looping {
            false
        } else {
            scene_elapsed_ms >= self.started_ms + self.duration_ms
        }
    }

    /// Converts this runtime effect into a scene `Effect` reference suitable for `apply_effect`.
    pub fn as_scene_effect(&self) -> Effect {
        Effect {
            name: self.name.clone(),
            duration: self.duration_ms,
            looping: self.looping,
            target_kind: engine_core::scene::EffectTargetKind::Any,
            params: self.params.clone(),
        }
    }
}

/// Scoped world resource tracking all script-triggered runtime effects.
///
/// Registered via `world.register_scoped()` on scene load and cleared
/// automatically when the scene transitions.
#[derive(Debug, Default, Clone)]
pub struct RuntimeEffectsResource {
    effects: Vec<RuntimeEffect>,
}

impl RuntimeEffectsResource {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new runtime effect starting at `scene_elapsed_ms`.
    pub fn push(
        &mut self,
        name: String,
        duration_ms: u64,
        looping: bool,
        params: EffectParams,
        scene_elapsed_ms: u64,
    ) {
        self.effects.push(RuntimeEffect {
            name,
            started_ms: scene_elapsed_ms,
            duration_ms,
            looping,
            params,
        });
    }

    /// Remove all expired (non-looping) effects based on current scene time.
    pub fn retain_live(&mut self, scene_elapsed_ms: u64) {
        self.effects.retain(|e| !e.is_expired(scene_elapsed_ms));
    }

    /// Iterate all active effects.
    pub fn effects(&self) -> &[RuntimeEffect] {
        &self.effects
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

/// Deserialize a `serde_json::Value` params map into `EffectParams`.
/// Only extracts known fields; unknown keys are silently ignored.
pub fn params_from_json(value: &JsonValue) -> EffectParams {
    let mut p = EffectParams::default();
    let Some(map) = value.as_object() else {
        return p;
    };
    if let Some(v) = map.get("amplitude_x").and_then(JsonValue::as_f64) {
        p.amplitude_x = Some(v as f32);
    }
    if let Some(v) = map.get("amplitude_y").and_then(JsonValue::as_f64) {
        p.amplitude_y = Some(v as f32);
    }
    if let Some(v) = map.get("frequency").and_then(JsonValue::as_f64) {
        p.frequency = Some(v as f32);
    }
    if let Some(v) = map.get("intensity").and_then(JsonValue::as_f64) {
        p.intensity = Some(v as f32);
    }
    if let Some(v) = map.get("alpha").and_then(JsonValue::as_f64) {
        p.alpha = Some(v as f32);
    }
    if let Some(v) = map.get("distortion").and_then(JsonValue::as_f64) {
        p.distortion = Some(v as f32);
    }
    if let Some(v) = map.get("angle").and_then(JsonValue::as_f64) {
        p.angle = Some(v as f32);
    }
    if let Some(v) = map.get("width").and_then(JsonValue::as_f64) {
        p.width = Some(v as f32);
    }
    if let Some(v) = map.get("falloff").and_then(JsonValue::as_f64) {
        p.falloff = Some(v as f32);
    }
    if let Some(v) = map.get("sphericality").and_then(JsonValue::as_f64) {
        p.sphericality = Some(v as f32);
    }
    if let Some(v) = map.get("transparency").and_then(JsonValue::as_f64) {
        p.transparency = Some(v as f32);
    }
    if let Some(v) = map.get("brightness").and_then(JsonValue::as_f64) {
        p.brightness = Some(v as f32);
    }
    if let Some(v) = map.get("strikes").and_then(JsonValue::as_u64) {
        p.strikes = Some(v as u16);
    }
    if let Some(v) = map.get("thickness").and_then(JsonValue::as_f64) {
        p.thickness = Some(v as f32);
    }
    if let Some(v) = map.get("coverage").and_then(JsonValue::as_str) {
        p.coverage = Some(v.to_string());
    }
    if let Some(v) = map.get("orientation").and_then(JsonValue::as_str) {
        p.orientation = Some(v.to_string());
    }
    if let Some(v) = map.get("target").and_then(JsonValue::as_str) {
        p.target = Some(v.to_string());
    }
    p
}
