//! Effects domain API: runtime-triggerable visual effects from Rhai scripts.

use std::sync::{Arc, Mutex};

use rhai::{Engine as RhaiEngine, Map as RhaiMap};
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::BehaviorCommand;

/// Script-facing API for triggering runtime visual effects.
#[derive(Clone)]
pub struct ScriptEffectsApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptEffectsApi {
    pub fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn push(&self, cmd: BehaviorCommand) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(cmd);
        }
    }

    /// Trigger a screen-shake effect for `duration_ms` milliseconds.
    ///
    /// * `amp_x` — horizontal amplitude in cells (e.g. 1.5)
    /// * `amp_y` — vertical amplitude in cells (e.g. 0.5)
    /// * `frequency` — oscillations over the effect duration (e.g. 8.0)
    pub fn shake(&mut self, duration_ms: rhai::INT, amp_x: rhai::FLOAT, amp_y: rhai::FLOAT, frequency: rhai::FLOAT) {
        let mut params = JsonMap::new();
        params.insert("amplitude_x".to_string(), JsonValue::from(amp_x as f64));
        params.insert("amplitude_y".to_string(), JsonValue::from(amp_y as f64));
        params.insert("frequency".to_string(), JsonValue::from(frequency as f64));
        self.push(BehaviorCommand::TriggerEffect {
            name: "screen-shake".to_string(),
            duration_ms: duration_ms.max(0) as u64,
            looping: false,
            params: JsonValue::Object(params),
        });
    }

    /// Trigger any named built-in effect with arbitrary params.
    ///
    /// `name` — effect name (e.g. `"screen-shake"`, `"flash"`)
    /// `duration_ms` — how long the effect runs in milliseconds
    /// `params` — Rhai map of effect parameters (e.g. `#{amplitude_x: 1.0}`)
    pub fn trigger(&mut self, name: &str, duration_ms: rhai::INT, params: RhaiMap) {
        let mut json_params = JsonMap::new();
        for (key, val) in &params {
            if let Some(f) = val.clone().try_cast::<rhai::FLOAT>() {
                json_params.insert(key.to_string(), JsonValue::from(f as f64));
            } else if let Some(i) = val.clone().try_cast::<rhai::INT>() {
                json_params.insert(key.to_string(), JsonValue::from(i as f64));
            } else if let Some(s) = val.clone().try_cast::<String>() {
                json_params.insert(key.to_string(), JsonValue::from(s));
            } else if let Some(b) = val.clone().try_cast::<bool>() {
                json_params.insert(key.to_string(), JsonValue::from(b));
            }
        }
        self.push(BehaviorCommand::TriggerEffect {
            name: name.to_string(),
            duration_ms: duration_ms.max(0) as u64,
            looping: false,
            params: JsonValue::Object(json_params),
        });
    }

    /// Trigger a looping effect until cleared or scene transition.
    pub fn trigger_loop(&mut self, name: &str, duration_ms: rhai::INT, params: RhaiMap) {
        let mut json_params = JsonMap::new();
        for (key, val) in &params {
            if let Some(f) = val.clone().try_cast::<rhai::FLOAT>() {
                json_params.insert(key.to_string(), JsonValue::from(f as f64));
            } else if let Some(i) = val.clone().try_cast::<rhai::INT>() {
                json_params.insert(key.to_string(), JsonValue::from(i as f64));
            } else if let Some(s) = val.clone().try_cast::<String>() {
                json_params.insert(key.to_string(), JsonValue::from(s));
            } else if let Some(b) = val.clone().try_cast::<bool>() {
                json_params.insert(key.to_string(), JsonValue::from(b));
            }
        }
        self.push(BehaviorCommand::TriggerEffect {
            name: name.to_string(),
            duration_ms: duration_ms.max(0) as u64,
            looping: true,
            params: JsonValue::Object(json_params),
        });
    }
}

pub fn register_effects_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptEffectsApi>("EffectsApi");

    engine.register_fn(
        "shake",
        |api: &mut ScriptEffectsApi, dur: rhai::INT, ax: rhai::FLOAT, ay: rhai::FLOAT, freq: rhai::FLOAT| {
            api.shake(dur, ax, ay, freq);
        },
    );
    engine.register_fn(
        "trigger",
        |api: &mut ScriptEffectsApi, name: &str, dur: rhai::INT, params: RhaiMap| {
            api.trigger(name, dur, params);
        },
    );
    engine.register_fn(
        "trigger_loop",
        |api: &mut ScriptEffectsApi, name: &str, dur: rhai::INT, params: RhaiMap| {
            api.trigger_loop(name, dur, params);
        },
    );

    // Dual-name: effects.* namespace aliases (called as methods on the `effects` scope var)
    engine.register_fn(
        "effects.shake",
        |api: &mut ScriptEffectsApi, dur: rhai::INT, ax: rhai::FLOAT, ay: rhai::FLOAT, freq: rhai::FLOAT| {
            api.shake(dur, ax, ay, freq);
        },
    );
    engine.register_fn(
        "effects.trigger",
        |api: &mut ScriptEffectsApi, name: &str, dur: rhai::INT, params: RhaiMap| {
            api.trigger(name, dur, params);
        },
    );
    engine.register_fn(
        "effects.trigger_loop",
        |api: &mut ScriptEffectsApi, name: &str, dur: rhai::INT, params: RhaiMap| {
            api.trigger_loop(name, dur, params);
        },
    );
}
