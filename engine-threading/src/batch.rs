//! Batch read/write abstractions for component stores.
//!
//! Instead of locking per-element, we lock once and extract/write batches.

use std::collections::BTreeMap;

/// Trait for batch reading from a locked store.
pub trait BatchRead<K, V> {
    /// Extract all values for given keys in a single lock acquisition.
    fn batch_read(&self, keys: &[K]) -> Vec<(K, V)>;
    
    /// Extract all key-value pairs in a single lock acquisition.
    fn read_all(&self) -> Vec<(K, V)>;
}

/// Trait for batch writing to a locked store.
pub trait BatchWrite<K, V> {
    /// Write all values in a single lock acquisition.
    fn batch_write(&self, items: Vec<(K, V)>);
}

/// Generic batch accessor that wraps any mutex-protected BTreeMap.
/// Provides efficient single-lock batch operations.
pub struct BatchAccessor<K, V> {
    data: parking_lot::RwLock<BTreeMap<K, V>>,
}

impl<K: Ord + Copy, V: Clone> BatchAccessor<K, V> {
    pub fn new() -> Self {
        Self {
            data: parking_lot::RwLock::new(BTreeMap::new()),
        }
    }

    pub fn from_map(map: BTreeMap<K, V>) -> Self {
        Self {
            data: parking_lot::RwLock::new(map),
        }
    }

    /// Insert a single value.
    pub fn insert(&self, key: K, value: V) {
        self.data.write().insert(key, value);
    }

    /// Get a single value.
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.read().get(key).cloned()
    }

    /// Remove a single value.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.write().remove(key)
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<K> {
        self.data.read().keys().copied().collect()
    }

    /// Batch read: single lock, extract all requested values.
    pub fn batch_get(&self, keys: &[K]) -> Vec<(K, V)> {
        let guard = self.data.read();
        keys.iter()
            .filter_map(|k| guard.get(k).map(|v| (*k, v.clone())))
            .collect()
    }

    /// Read all: single lock, extract everything.
    pub fn get_all(&self) -> Vec<(K, V)> {
        let guard = self.data.read();
        guard.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// Batch write: single lock, write all values.
    pub fn batch_set(&self, items: Vec<(K, V)>) {
        let mut guard = self.data.write();
        for (k, v) in items {
            guard.insert(k, v);
        }
    }

    /// Batch remove: single lock, remove all keys.
    pub fn batch_remove(&self, keys: &[K]) -> Vec<(K, V)> {
        let mut guard = self.data.write();
        keys.iter()
            .filter_map(|k| guard.remove(k).map(|v| (*k, v)))
            .collect()
    }

    /// Count items.
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }
}

impl<K: Ord + Copy, V: Clone> Default for BatchAccessor<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Copy, V: Clone> BatchRead<K, V> for BatchAccessor<K, V> {
    fn batch_read(&self, keys: &[K]) -> Vec<(K, V)> {
        self.batch_get(keys)
    }

    fn read_all(&self) -> Vec<(K, V)> {
        self.get_all()
    }
}

impl<K: Ord + Copy, V: Clone> BatchWrite<K, V> for BatchAccessor<K, V> {
    fn batch_write(&self, items: Vec<(K, V)>) {
        self.batch_set(items)
    }
}

/// Multi-component batch read for physics (transform + physics body + optional particle physics).
/// Reads all three in a single conceptual "batch" operation.
#[derive(Clone, Debug)]
pub struct PhysicsBatchItem<T, P, PP> {
    pub id: u64,
    pub transform: T,
    pub physics: P,
    pub particle_physics: Option<PP>,
}

/// Result of physics computation ready to write back.
#[derive(Clone, Debug)]
pub struct PhysicsResult<T, P> {
    pub id: u64,
    pub transform: T,
    pub physics: P,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_accessor() {
        let accessor: BatchAccessor<u64, i32> = BatchAccessor::new();
        
        // Insert some values
        accessor.insert(1, 100);
        accessor.insert(2, 200);
        accessor.insert(3, 300);
        
        // Batch read
        let batch = accessor.batch_get(&[1, 3]);
        assert_eq!(batch.len(), 2);
        
        // Batch write
        accessor.batch_set(vec![(1, 111), (2, 222)]);
        assert_eq!(accessor.get(&1), Some(111));
        assert_eq!(accessor.get(&2), Some(222));
    }
}
