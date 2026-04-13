//! Diagnostics for gameplay entity lifecycle and object growth tracking.
//!
//! This module provides utilities to track entity spawning, despawning, and
//! categorization to identify memory leaks and unbalanced object growth.

use std::collections::{BTreeMap, HashMap};

/// Snapshot of entity counts by kind and lifecycle policy.
#[derive(Clone, Debug, Default)]
pub struct EntityCountSnapshot {
    pub total: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_policy: BTreeMap<String, usize>,
    pub timestamp_ms: u64,
}

impl EntityCountSnapshot {
    /// Returns a human-readable summary of the snapshot.
    pub fn summary(&self) -> String {
        let mut lines = vec![format!("Total entities: {}", self.total)];

        if !self.by_kind.is_empty() {
            lines.push("By kind:".to_string());
            for (kind, count) in &self.by_kind {
                lines.push(format!("  {}: {}", kind, count));
            }
        }

        if !self.by_policy.is_empty() {
            lines.push("By lifecycle policy:".to_string());
            for (policy, count) in &self.by_policy {
                lines.push(format!("  {}: {}", policy, count));
            }
        }

        lines.join("\n")
    }

    /// Computes the delta between this snapshot and a previous one.
    pub fn delta(&self, prev: &EntityCountSnapshot) -> EntityCountSnapshot {
        EntityCountSnapshot {
            total: self.total.saturating_sub(prev.total),
            by_kind: {
                let mut delta = BTreeMap::new();
                for (kind, count) in &self.by_kind {
                    let prev_count = prev.by_kind.get(kind).copied().unwrap_or(0);
                    if *count != prev_count {
                        delta.insert(kind.clone(), count.saturating_sub(prev_count));
                    }
                }
                delta
            },
            by_policy: {
                let mut delta = BTreeMap::new();
                for (policy, count) in &self.by_policy {
                    let prev_count = prev.by_policy.get(policy).copied().unwrap_or(0);
                    if *count != prev_count {
                        delta.insert(policy.clone(), count.saturating_sub(prev_count));
                    }
                }
                delta
            },
            timestamp_ms: self.timestamp_ms,
        }
    }
}

/// Track per-entity spawn/despawn events for forensics.
#[derive(Clone, Debug)]
pub struct EntityEventLog {
    pub entries: Vec<EntityEvent>,
    pub max_entries: usize,
}

#[derive(Clone, Debug)]
pub struct EntityEvent {
    pub event_type: EntityEventType,
    pub entity_id: u64,
    pub kind: String,
    pub frame: u64,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug)]
pub enum EntityEventType {
    Spawned,
    Despawned,
    Lifecycle(String),
}

impl Default for EntityEventLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000, // Keep last 1000 events
        }
    }
}

impl EntityEventLog {
    /// Record a spawn event.
    pub fn record_spawn(&mut self, entity_id: u64, kind: String, frame: u64, timestamp_ms: u64) {
        self.entries.push(EntityEvent {
            event_type: EntityEventType::Spawned,
            entity_id,
            kind,
            frame,
            timestamp_ms,
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Record a despawn event.
    pub fn record_despawn(&mut self, entity_id: u64, kind: String, frame: u64, timestamp_ms: u64) {
        self.entries.push(EntityEvent {
            event_type: EntityEventType::Despawned,
            entity_id,
            kind,
            frame,
            timestamp_ms,
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get recent spawned entities per kind.
    pub fn recent_spawned_by_kind(&self, count: usize) -> HashMap<String, u64> {
        let mut result = HashMap::new();
        for entry in self.entries.iter().rev().take(count) {
            if matches!(entry.event_type, EntityEventType::Spawned) {
                *result.entry(entry.kind.clone()).or_insert(0) += 1;
            }
        }
        result
    }

    /// Get recent despawned entities per kind.
    pub fn recent_despawned_by_kind(&self, count: usize) -> HashMap<String, u64> {
        let mut result = HashMap::new();
        for entry in self.entries.iter().rev().take(count) {
            if matches!(entry.event_type, EntityEventType::Despawned) {
                *result.entry(entry.kind.clone()).or_insert(0) += 1;
            }
        }
        result
    }
}
