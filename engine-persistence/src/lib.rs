//! Persistent JSON storage domain for cross-run game data.
//!
//! This crate intentionally stays domain-agnostic. It provides a path-based
//! JSON store with immediate disk flush on mutation.

use serde_json::{json, Value as JsonValue};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const DEFAULT_SAVE_ROOT: &str = "saves";
const SAVE_FILE_NAME: &str = "state.json";

/// Thread-safe JSON persistence container.
#[derive(Clone, Debug)]
pub struct PersistenceStore {
    file_path: Arc<PathBuf>,
    data: Arc<Mutex<JsonValue>>,
}

impl PersistenceStore {
    /// Creates a persistence store under the default root for `namespace`.
    pub fn new(namespace: &str) -> Self {
        Self::from_root(default_save_root(), namespace)
    }

    /// Creates a persistence store under `save_root` for `namespace`.
    pub fn from_root(save_root: impl Into<PathBuf>, namespace: &str) -> Self {
        let namespace = sanitize_namespace(namespace);
        let file_path = save_root.into().join(namespace).join(SAVE_FILE_NAME);
        let initial = load_json_file(&file_path).unwrap_or_else(|| json!({}));
        Self {
            file_path: Arc::new(file_path),
            data: Arc::new(Mutex::new(initial)),
        }
    }

    /// Returns the backing file path.
    pub fn file_path(&self) -> &Path {
        self.file_path.as_path()
    }

    /// Gets a value using JSON pointer notation.
    pub fn get(&self, path: &str) -> Option<JsonValue> {
        let data = self.data.lock().ok()?;
        data.pointer(path).cloned()
    }

    /// Checks if a value exists at `path`.
    pub fn has(&self, path: &str) -> bool {
        let Ok(data) = self.data.lock() else {
            return false;
        };
        data.pointer(path).is_some()
    }

    /// Sets `value` at `path` and flushes to disk.
    pub fn set(&self, path: &str, value: JsonValue) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        if !set_path(&mut data, path, value) {
            return false;
        }
        flush_json_file(self.file_path(), &data)
    }

    /// Removes value at `path` and flushes to disk.
    pub fn remove(&self, path: &str) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        if !remove_path(&mut data, path) {
            return false;
        }
        flush_json_file(self.file_path(), &data)
    }

    /// Pushes `value` to array at `path` and flushes to disk.
    pub fn push(&self, path: &str, value: JsonValue) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        if !push_path(&mut data, path, value) {
            return false;
        }
        flush_json_file(self.file_path(), &data)
    }

    /// Returns in-memory JSON snapshot.
    pub fn snapshot(&self) -> Option<JsonValue> {
        self.data.lock().ok().map(|data| data.clone())
    }

    /// Clears state and flushes to disk.
    pub fn clear(&self) -> bool {
        let mut data = match self.data.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        *data = json!({});
        flush_json_file(self.file_path(), &data)
    }

    /// Reloads current file from disk into memory.
    pub fn reload(&self) -> bool {
        let Some(on_disk) = load_json_file(self.file_path()) else {
            return false;
        };
        let Ok(mut data) = self.data.lock() else {
            return false;
        };
        *data = on_disk;
        true
    }
}

impl Default for PersistenceStore {
    fn default() -> Self {
        Self::new("shell-quest")
    }
}

fn default_save_root() -> PathBuf {
    std::env::var("SHELL_QUEST_SAVE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SAVE_ROOT))
}

fn sanitize_namespace(namespace: &str) -> String {
    let mut out = String::with_capacity(namespace.len().max(1));
    for ch in namespace.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() {
            out.push('-');
        }
    }
    if out.is_empty() {
        String::from("default")
    } else {
        out
    }
}

fn load_json_file(path: &Path) -> Option<JsonValue> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<JsonValue>(&text).ok()
}

fn flush_json_file(path: &Path, data: &JsonValue) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return false;
    }
    let encoded = match serde_json::to_string_pretty(data) {
        Ok(value) => value,
        Err(_) => return false,
    };
    std::fs::write(path, encoded).is_ok()
}

fn split_path(path: &str) -> Option<(Vec<&str>, &str)> {
    if path.is_empty() || path == "/" {
        return None;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (leaf, parent) = parts.split_last()?;
    Some((parent.to_vec(), *leaf))
}

fn set_path(data: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    if path.is_empty() || path == "/" {
        *data = value;
        return true;
    }
    let Some((parent_parts, leaf)) = split_path(path) else {
        return false;
    };
    let mut current = data;
    for part in parent_parts {
        if !current.is_object() {
            *current = json!({});
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = json!({});
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    obj.insert(leaf.to_string(), value);
    true
}

fn remove_path(data: &mut JsonValue, path: &str) -> bool {
    let Some((parent_parts, leaf)) = split_path(path) else {
        return false;
    };
    let mut current = data;
    for part in parent_parts {
        current = match current.as_object_mut() {
            Some(obj) => match obj.get_mut(part) {
                Some(val) => val,
                None => return false,
            },
            None => return false,
        };
    }
    match current.as_object_mut() {
        Some(obj) => obj.remove(leaf).is_some(),
        None => false,
    }
}

fn push_path(data: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    let Some((parent_parts, leaf)) = split_path(path) else {
        return false;
    };
    let mut current = data;
    for part in parent_parts {
        if !current.is_object() {
            *current = json!({});
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = json!({});
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    let entry = obj.entry(leaf.to_string()).or_insert_with(|| json!([]));
    if let Some(arr) = entry.as_array_mut() {
        arr.push(value);
    } else {
        let old = entry.clone();
        *entry = json!([old, value]);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_and_reload_work() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = PersistenceStore::from_root(temp.path(), "Test Mod");
        assert!(store.set("/highscores/0/name", json!("AAA")));
        assert!(store.set("/highscores/0/score", json!(1234)));
        assert_eq!(store.get("/highscores/0/name"), Some(json!("AAA")));
        assert!(store.reload());
        assert_eq!(store.get("/highscores/0/score"), Some(json!(1234)));
    }

    #[test]
    fn push_and_remove_work() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = PersistenceStore::from_root(temp.path(), "mod");
        assert!(store.push("/scores", json!(100)));
        assert!(store.push("/scores", json!(200)));
        assert_eq!(store.get("/scores"), Some(json!([100, 200])));
        assert!(store.remove("/scores"));
        assert!(!store.has("/scores"));
    }
}
