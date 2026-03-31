//! IO domain APIs: ScriptTerminalApi and ScriptInputApi.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use rhai::Engine as RhaiEngine;

use crate::rhai_util::normalize_input_code;
use crate::{catalog, BehaviorCommand};

// ── ScriptTerminalApi ────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptTerminalApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptTerminalApi {
    pub(crate) fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn push(&mut self, line: &str) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::TerminalPushOutput {
            line: line.to_string(),
        });
    }

    fn clear(&mut self) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::TerminalClearOutput);
    }
}

// ── ScriptInputApi ───────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptInputApi {
    keys_down: Arc<HashSet<String>>,
    keys_just_pressed: Arc<HashSet<String>>,
    action_bindings: Arc<HashMap<String, Vec<String>>>,
    catalogs: Arc<catalog::ModCatalogs>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptInputApi {
    pub(crate) fn new(
        keys_down: Arc<HashSet<String>>,
        keys_just_pressed: Arc<HashSet<String>>,
        action_bindings: Arc<HashMap<String, Vec<String>>>,
        catalogs: Arc<catalog::ModCatalogs>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            keys_down,
            keys_just_pressed,
            action_bindings,
            catalogs,
            queue,
        }
    }

    fn down(&mut self, code: &str) -> bool {
        let normalized = normalize_input_code(code);
        if normalized.is_empty() {
            return false;
        }
        self.keys_down.contains(&normalized)
    }

    /// Returns `true` only on the first frame a key is pressed (not while held).
    fn just_pressed(&mut self, code: &str) -> bool {
        let normalized = normalize_input_code(code);
        if normalized.is_empty() {
            return false;
        }
        self.keys_just_pressed.contains(&normalized)
    }

    fn any_down(&mut self) -> bool {
        !self.keys_down.is_empty()
    }

    fn down_count(&mut self) -> rhai::INT {
        self.keys_down.len() as rhai::INT
    }

    /// Returns `true` if any key bound to `action` is currently held.
    fn action_down(&mut self, action: &str) -> bool {
        let Some(keys) = self.action_bindings.get(action) else {
            return false;
        };
        keys.iter().any(|k| {
            let n = normalize_input_code(k);
            !n.is_empty() && self.keys_down.contains(&n)
        })
    }

    /// Returns `true` only on the first frame any key bound to `action` is pressed.
    fn action_just_pressed(&mut self, action: &str) -> bool {
        let Some(keys) = self.action_bindings.get(action) else {
            return false;
        };
        keys.iter().any(|k| {
            let n = normalize_input_code(k);
            !n.is_empty() && self.keys_just_pressed.contains(&n)
        })
    }

    /// Bind an action to a list of key codes. Emits a `BindInputAction` command.
    fn bind_action(&mut self, action: &str, keys: rhai::Array) -> bool {
        let key_strs: Vec<String> = keys
            .into_iter()
            .filter_map(|v| v.into_string().ok())
            .collect();
        if let Ok(mut q) = self.queue.lock() {
            q.push(BehaviorCommand::BindInputAction {
                action: action.to_string(),
                keys: key_strs,
            });
        }
        true
    }

    fn load_profile(&mut self, name: &str) -> bool {
        // Try to load from catalog first
        if let Some(profile) = self.catalogs.input_profiles.get(name) {
            let Ok(mut q) = self.queue.lock() else {
                return false;
            };
            for (action, keys) in &profile.bindings {
                q.push(BehaviorCommand::BindInputAction {
                    action: action.clone(),
                    keys: keys.clone(),
                });
            }
            return true;
        }

        // Fall back to hardcoded profiles for backward compatibility
        let bindings: &[(&str, &[&str])] = match name {
            "asteroids.default" => &[
                ("turn_left", &["Left", "a", "A"]),
                ("turn_right", &["Right", "d", "D"]),
                ("thrust", &["Up", "w", "W"]),
                ("fire", &[" ", "f", "F"]),
            ],
            _ => return false,
        };

        let Ok(mut q) = self.queue.lock() else {
            return false;
        };
        for (action, keys) in bindings {
            q.push(BehaviorCommand::BindInputAction {
                action: (*action).to_string(),
                keys: keys.iter().map(|key| (*key).to_string()).collect(),
            });
        }
        true
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptTerminalApi>("TerminalApi");
    engine.register_type_with_name::<ScriptInputApi>("InputApi");

    engine.register_fn("push", |terminal: &mut ScriptTerminalApi, line: &str| {
        terminal.push(line);
    });
    engine.register_fn("clear", |terminal: &mut ScriptTerminalApi| {
        terminal.clear();
    });

    engine.register_fn("down", |input: &mut ScriptInputApi, code: &str| {
        input.down(code)
    });
    engine.register_fn("just_pressed", |input: &mut ScriptInputApi, code: &str| {
        input.just_pressed(code)
    });
    engine.register_fn("any_down", |input: &mut ScriptInputApi| input.any_down());
    engine.register_fn("down_count", |input: &mut ScriptInputApi| {
        input.down_count()
    });
    engine.register_fn("action_down", |input: &mut ScriptInputApi, action: &str| {
        input.action_down(action)
    });
    engine.register_fn(
        "action_just_pressed",
        |input: &mut ScriptInputApi, action: &str| input.action_just_pressed(action),
    );
    engine.register_fn(
        "bind_action",
        |input: &mut ScriptInputApi, action: &str, keys: rhai::Array| {
            input.bind_action(action, keys)
        },
    );
    engine.register_fn("load_profile", |input: &mut ScriptInputApi, name: &str| {
        input.load_profile(name)
    });
}
