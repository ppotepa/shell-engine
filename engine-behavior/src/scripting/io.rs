//! IO domain APIs: input.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use rhai::Engine as RhaiEngine;

use crate::rhai_util::normalize_input_code;
use crate::{catalog, BehaviorCommand};

#[derive(Clone)]
pub(crate) struct ScriptInputApi {
    keys_down: Arc<HashSet<String>>,
    keys_just_pressed: Arc<HashSet<String>>,
    scroll_y: f32,
    ctrl_scroll_y: f32,
    action_bindings: Arc<HashMap<String, Vec<String>>>,
    catalogs: Arc<catalog::ModCatalogs>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptInputApi {
    pub(crate) fn new(
        keys_down: Arc<HashSet<String>>,
        keys_just_pressed: Arc<HashSet<String>>,
        scroll_y: f32,
        ctrl_scroll_y: f32,
        action_bindings: Arc<HashMap<String, Vec<String>>>,
        catalogs: Arc<catalog::ModCatalogs>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            keys_down,
            keys_just_pressed,
            scroll_y,
            ctrl_scroll_y,
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

    fn scroll_y(&mut self) -> rhai::FLOAT {
        self.scroll_y as rhai::FLOAT
    }

    fn ctrl_scroll_y(&mut self) -> rhai::FLOAT {
        self.ctrl_scroll_y as rhai::FLOAT
    }

    fn action_down(&mut self, action: &str) -> bool {
        let Some(keys) = self.action_bindings.get(action) else {
            return false;
        };
        keys.iter().any(|k| {
            let n = normalize_input_code(k);
            !n.is_empty() && self.keys_down.contains(&n)
        })
    }

    fn action_just_pressed(&mut self, action: &str) -> bool {
        let Some(keys) = self.action_bindings.get(action) else {
            return false;
        };
        keys.iter().any(|k| {
            let n = normalize_input_code(k);
            !n.is_empty() && self.keys_just_pressed.contains(&n)
        })
    }

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
        false
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptInputApi>("InputApi");

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
    engine.register_get("scroll_y", |input: &mut ScriptInputApi| input.scroll_y());
    engine.register_get("ctrl_scroll_y", |input: &mut ScriptInputApi| {
        input.ctrl_scroll_y()
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

#[cfg(test)]
mod tests {
    use super::ScriptInputApi;
    use crate::catalog;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    #[test]
    fn exposes_frame_scroll_getters() {
        let mut input = ScriptInputApi::new(
            Arc::new(HashSet::new()),
            Arc::new(HashSet::new()),
            1.5,
            -2.0,
            Arc::new(HashMap::new()),
            Arc::new(catalog::ModCatalogs::default()),
            Arc::new(Mutex::new(Vec::new())),
        );

        assert!((input.scroll_y() - 1.5).abs() < f64::EPSILON);
        assert!((input.ctrl_scroll_y() + 2.0).abs() < f64::EPSILON);
    }
}
