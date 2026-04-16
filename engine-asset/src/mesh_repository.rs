use std::collections::HashMap;

use crate::build_keys::MeshBuildKey;

#[derive(Debug, Clone)]
pub struct MeshRepository<T> {
    entries: HashMap<MeshBuildKey, T>,
}

impl<T> Default for MeshRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MeshRepository<T> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &MeshBuildKey) -> Option<&T> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: MeshBuildKey, value: T) -> Option<T> {
        self.entries.insert(key, value)
    }
}
