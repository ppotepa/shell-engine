use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct InnerEmitterState {
    active: HashMap<(String, Option<u64>), VecDeque<u64>>,
}

#[derive(Debug, Clone, Default)]
pub struct EmitterState {
    inner: Arc<Mutex<InnerEmitterState>>,
}

impl EmitterState {
    pub fn track_spawn(&self, emitter_name: &str, owner_id: Option<u64>, entity_id: u64) {
        if emitter_name.trim().is_empty() || entity_id == 0 {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .active
                .entry((emitter_name.to_string(), owner_id))
                .or_default()
                .push_back(entity_id);
        }
    }

    pub fn active_count(&self, emitter_name: &str, owner_id: Option<u64>) -> usize {
        let Ok(mut inner) = self.inner.lock() else {
            return 0;
        };
        let key = (emitter_name.to_string(), owner_id);
        let Some(queue) = inner.active.get_mut(&key) else {
            return 0;
        };
        queue.retain(|id| *id != 0);
        queue.len()
    }

    pub fn evict_oldest(&self, emitter_name: &str, owner_id: Option<u64>) -> Option<u64> {
        let Ok(mut inner) = self.inner.lock() else {
            return None;
        };
        inner
            .active
            .get_mut(&(emitter_name.to_string(), owner_id))
            .and_then(VecDeque::pop_front)
    }

    pub fn remove_entity(&self, entity_id: u64) {
        if entity_id == 0 {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            let mut empty = Vec::new();
            for (key, queue) in &mut inner.active {
                queue.retain(|id| *id != entity_id);
                if queue.is_empty() {
                    empty.push(key.clone());
                }
            }
            for key in empty {
                inner.active.remove(&key);
            }
        }
    }
}
