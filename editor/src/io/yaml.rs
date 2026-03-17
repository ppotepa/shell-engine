use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

pub fn load_yaml<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let raw = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&raw).ok()
}
