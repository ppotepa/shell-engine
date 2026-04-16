use std::collections::HashMap;

use crate::build_keys::MaterialBuildKey;

#[derive(Debug, Clone)]
pub struct MaterialRepository<T> {
    entries: HashMap<MaterialBuildKey, T>,
}

impl<T> Default for MaterialRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MaterialRepository<T> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &MaterialBuildKey) -> Option<&T> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: MaterialBuildKey, value: T) -> Option<T> {
        self.entries.insert(key, value)
    }
}
