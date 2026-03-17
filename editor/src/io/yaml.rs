//! Generic YAML file loading helper.

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

/// Reads `path` from disk and deserializes it as YAML into `T`, returning `None` on any error.
pub fn load_yaml<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let raw = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&raw).ok()
}
