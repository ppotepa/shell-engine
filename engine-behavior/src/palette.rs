//! Mod-scoped palette store: loads color palettes from `palettes/*.yml` in the mod directory.
//!
//! Each palette file defines a named set of colors for UI, entities, and particle ramps.
//! The active palette is selected via the persistence store and falls back to `default_palette`
//! from `mod.yaml`, then the first loaded palette.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// One named color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteData {
    pub id: String,
    pub name: String,
    /// Flat color map: key → hex string. Preserves YAML declaration order for indexed access.
    #[serde(default)]
    pub colors: IndexMap<String, String>,
    /// Particle ramp arrays: ramp-name → ordered list of hex strings.
    #[serde(default)]
    pub particles: HashMap<String, Vec<String>>,
}

/// All palettes available in a mod, indexed by their `id`.
#[derive(Debug, Clone)]
pub struct PaletteStore {
    pub palettes: HashMap<String, PaletteData>,
    /// Ordered list of ids in the order they were loaded (for cycling).
    pub order: Vec<String>,
    /// Monotonically increasing counter, bumped every time the active palette changes.
    /// Scripts can cache this value and skip palette re-reads when it hasn't changed.
    pub version: Arc<AtomicU64>,
}

impl Default for PaletteStore {
    fn default() -> Self {
        Self {
            palettes: HashMap::new(),
            order: Vec::new(),
            version: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl PaletteStore {
    /// Load every `*.yml` file from `palettes_dir` as a `PaletteData`.
    /// Returns an error string if a file cannot be parsed; missing directory is OK (empty store).
    pub fn load_from_directory(palettes_dir: &Path) -> Result<Self, String> {
        let mut store = Self::default();

        if !palettes_dir.exists() {
            return Ok(store);
        }

        let mut entries: Vec<_> = std::fs::read_dir(palettes_dir)
            .map_err(|e| format!("Failed to read palettes directory: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "yml" || ext == "yaml")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let palette: PaletteData = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
            let id = palette.id.clone();
            store.order.push(id.clone());
            store.palettes.insert(id, palette);
        }

        Ok(store)
    }

    /// Number of loaded palettes.
    pub fn len(&self) -> usize {
        self.palettes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.palettes.is_empty()
    }

    /// Resolve the active palette given an optional persisted id and an optional default id.
    pub fn resolve<'a>(&'a self, persisted: Option<&str>, default: Option<&str>) -> Option<&'a PaletteData> {
        if let Some(id) = persisted {
            if let Some(p) = self.palettes.get(id) {
                return Some(p);
            }
        }
        if let Some(id) = default {
            if let Some(p) = self.palettes.get(id) {
                return Some(p);
            }
        }
        // Fall back to the first loaded palette.
        self.order.first().and_then(|id| self.palettes.get(id))
    }

    /// Id that comes after `current_id` in load order (wraps around).
    pub fn next_id(&self, current_id: &str) -> Option<String> {
        if self.order.is_empty() {
            return None;
        }
        let pos = self.order.iter().position(|id| id == current_id).unwrap_or(0);
        let next = (pos + 1) % self.order.len();
        Some(self.order[next].clone())
    }
}
