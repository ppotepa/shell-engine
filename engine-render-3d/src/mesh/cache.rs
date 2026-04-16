use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MeshCache<T> {
    entries: HashMap<String, T>,
}

impl<T> Default for MeshCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MeshCache<T> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: T) -> Option<T> {
        self.entries.insert(key.into(), value)
    }
}
