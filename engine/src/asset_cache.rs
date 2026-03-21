use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Shared lazy cache for optional assets loaded by key.
pub struct AssetCache<T> {
    inner: OnceLock<Mutex<HashMap<String, Option<Arc<T>>>>>,
}

impl<T> AssetCache<T> {
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }
}

impl<T> AssetCache<T> {
    pub fn get_or_load<F>(&self, key: String, loader: F) -> Option<Arc<T>>
    where
        F: FnOnce() -> Option<T>,
    {
        let cache = self.inner.get_or_init(|| Mutex::new(HashMap::new()));
        if let Ok(guard) = cache.lock() {
            if let Some(cached) = guard.get(&key) {
                return cached.clone();
            }
        }

        let loaded = loader().map(Arc::new);
        if let Ok(mut guard) = cache.lock() {
            guard.insert(key, loaded.clone());
        }
        loaded
    }
}
