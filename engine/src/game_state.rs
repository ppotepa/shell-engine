//! Persistent game state singleton for cross-scene data.
//!
//! The `GameState` is a domain-agnostic JSON store registered as a singleton in `World`.
//! Scripts can access it through generic path-based helpers without baking shell, filesystem,
//! email, or quest semantics into the engine.

use serde_json::{json, Value as JsonValue};
use std::sync::{Arc, Mutex};

/// Thread-safe persistent game state container.
///
/// This state persists across scene transitions and provides a generic key-value
/// store for game logic, session data, quest flags, and virtual resources.
#[derive(Clone, Debug)]
pub struct GameState {
    data: Arc<Mutex<JsonValue>>,
}

impl GameState {
    /// Creates a new empty game state.
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(json!({}))),
        }
    }

    /// Creates a game state initialized with the provided JSON value.
    pub fn with_initial_data(initial: JsonValue) -> Self {
        Self {
            data: Arc::new(Mutex::new(initial)),
        }
    }

    /// Gets a value at the specified path using JSON pointer notation.
    ///
    /// Path format: `/session/user`, `/flags/intro/login_ok`, etc.
    ///
    /// Returns `None` if the path does not exist or if the state is poisoned.
    pub fn get(&self, path: &str) -> Option<JsonValue> {
        let data = self.data.lock().ok()?;
        data.pointer(path).cloned()
    }

    /// Sets a value at the specified path using JSON pointer notation.
    ///
    /// Creates intermediate objects if they don't exist.
    /// Returns `false` if the state is poisoned or the path is invalid.
    pub fn set(&self, path: &str, value: JsonValue) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        if path.is_empty() || path == "/" {
            *data = value;
            return true;
        }

        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let (parent_parts, key) = match parts.split_last() {
            Some((k, p)) => (p, *k),
            None => return false,
        };

        let mut current = &mut *data;
        for &part in parent_parts {
            if !current.is_object() {
                *current = json!({});
            }
            if let Some(obj) = current.as_object_mut() {
                if !obj.contains_key(part) {
                    obj.insert(part.to_string(), json!({}));
                }
                current = obj.get_mut(part).expect("just inserted");
            } else {
                return false;
            }
        }

        if !current.is_object() {
            *current = json!({});
        }

        if let Some(obj) = current.as_object_mut() {
            obj.insert(key.to_string(), value);
            true
        } else {
            false
        }
    }

    /// Checks if a value exists at the specified path.
    pub fn has(&self, path: &str) -> bool {
        let Ok(data) = self.data.lock() else {
            return false;
        };
        data.pointer(path).is_some()
    }

    /// Removes a value at the specified path.
    ///
    /// Returns `true` if the value existed and was removed.
    pub fn remove(&self, path: &str) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        if path.is_empty() || path == "/" {
            return false;
        }

        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let (parent_parts, key) = match parts.split_last() {
            Some((k, p)) => (p, *k),
            None => return false,
        };

        let mut current = &mut *data;
        for &part in parent_parts {
            current = match current.as_object_mut() {
                Some(obj) => match obj.get_mut(part) {
                    Some(val) => val,
                    None => return false,
                },
                None => return false,
            };
        }

        if let Some(obj) = current.as_object_mut() {
            obj.remove(key).is_some()
        } else {
            false
        }
    }

    /// Pushes a value to an array at the specified path.
    ///
    /// Creates the array if it doesn't exist. If the path exists but is not
    /// an array, converts it to an array containing the old value and the new value.
    pub fn push(&self, path: &str, value: JsonValue) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        if path.is_empty() || path == "/" {
            return false;
        }

        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let (parent_parts, key) = match parts.split_last() {
            Some((k, p)) => (p, *k),
            None => return false,
        };

        let mut current = &mut *data;
        for &part in parent_parts {
            if !current.is_object() {
                *current = json!({});
            }
            if let Some(obj) = current.as_object_mut() {
                if !obj.contains_key(part) {
                    obj.insert(part.to_string(), json!({}));
                }
                let val = obj.get_mut(part);
                match val {
                    Some(v) => current = v,
                    None => return false,
                }
            } else {
                return false;
            }
        }

        if !current.is_object() {
            *current = json!({});
        }

        if let Some(obj) = current.as_object_mut() {
            let entry = obj.entry(key.to_string()).or_insert_with(|| json!([]));
            if let Some(arr) = entry.as_array_mut() {
                arr.push(value);
            } else {
                let old_value = entry.clone();
                *entry = json!([old_value, value]);
            }
            true
        } else {
            false
        }
    }

    /// Returns a clone of the entire state as a JSON value.
    pub fn snapshot(&self) -> Option<JsonValue> {
        self.data.lock().ok().map(|data| data.clone())
    }

    /// Clears all data in the state.
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            *data = json!({});
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_none_for_missing_path() {
        let state = GameState::new();
        assert!(state.get("/missing").is_none());
    }

    #[test]
    fn set_and_get_simple_value() {
        let state = GameState::new();
        assert!(state.set("/foo", json!("bar")));
        assert_eq!(state.get("/foo"), Some(json!("bar")));
    }

    #[test]
    fn set_nested_path_creates_intermediates() {
        let state = GameState::new();
        assert!(state.set("/session/user", json!("linus")));
        assert_eq!(state.get("/session/user"), Some(json!("linus")));
    }

    #[test]
    fn has_returns_true_for_existing_path() {
        let state = GameState::new();
        state.set("/foo", json!(42));
        assert!(state.has("/foo"));
    }

    #[test]
    fn has_returns_false_for_missing_path() {
        let state = GameState::new();
        assert!(!state.has("/missing"));
    }

    #[test]
    fn remove_deletes_value() {
        let state = GameState::new();
        state.set("/foo", json!(42));
        assert!(state.remove("/foo"));
        assert!(!state.has("/foo"));
    }

    #[test]
    fn remove_returns_false_for_missing_path() {
        let state = GameState::new();
        assert!(!state.remove("/missing"));
    }

    #[test]
    fn push_creates_array_if_missing() {
        let state = GameState::new();
        assert!(state.push("/events", json!("event1")));
        assert_eq!(state.get("/events"), Some(json!(["event1"])));
    }

    #[test]
    fn push_appends_to_existing_array() {
        let state = GameState::new();
        state.push("/events", json!("event1"));
        state.push("/events", json!("event2"));
        assert_eq!(state.get("/events"), Some(json!(["event1", "event2"])));
    }

    #[test]
    fn push_converts_non_array_to_array() {
        let state = GameState::new();
        state.set("/value", json!("old"));
        state.push("/value", json!("new"));
        assert_eq!(state.get("/value"), Some(json!(["old", "new"])));
    }

    #[test]
    fn snapshot_returns_full_state() {
        let state = GameState::new();
        state.set("/session/user", json!("linus"));
        state.set("/flags/intro/login_ok", json!(true));

        let snapshot = state.snapshot().expect("snapshot should succeed");
        assert_eq!(snapshot["session"]["user"], json!("linus"));
        assert_eq!(snapshot["flags"]["intro"]["login_ok"], json!(true));
    }

    #[test]
    fn clear_removes_all_data() {
        let state = GameState::new();
        state.set("/foo", json!(42));
        state.set("/bar", json!("test"));

        state.clear();

        assert!(!state.has("/foo"));
        assert!(!state.has("/bar"));
        assert_eq!(state.snapshot(), Some(json!({})));
    }
}
