//! Rhai API for mod palettes: `palette.get(key)`, `palette.set_active(id)`, etc.

use std::sync::Arc;

use engine_persistence::PersistenceStore;
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine};

use crate::palette::PaletteStore;

const PERSIST_KEY: &str = "__palette__";

#[derive(Clone)]
pub(crate) struct ScriptPaletteApi {
    store: Arc<PaletteStore>,
    persistence: Option<PersistenceStore>,
    /// Default palette id from mod.yaml (pre-resolved before construction).
    default_id: Option<String>,
}

impl ScriptPaletteApi {
    pub(crate) fn new(
        store: Arc<PaletteStore>,
        persistence: Option<PersistenceStore>,
        default_id: Option<String>,
    ) -> Self {
        Self { store, persistence, default_id }
    }

    fn active_id(&self) -> Option<String> {
        let persisted = self
            .persistence
            .as_ref()
            .and_then(|p| p.get(PERSIST_KEY))
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        self.store
            .resolve(persisted.as_deref(), self.default_id.as_deref())
            .map(|p| p.id.clone())
    }

    fn active_palette(&self) -> Option<&crate::palette::PaletteData> {
        let persisted = self
            .persistence
            .as_ref()
            .and_then(|p| p.get(PERSIST_KEY))
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        self.store
            .resolve(persisted.as_deref(), self.default_id.as_deref())
    }

    fn get_color(&self, key: &str) -> String {
        self.active_palette()
            .and_then(|p| p.colors.get(key))
            .cloned()
            .unwrap_or_default()
    }

    fn get_particles(&self, ramp: &str) -> RhaiArray {
        self.active_palette()
            .and_then(|p| p.particles.get(ramp))
            .map(|v| v.iter().map(|s| RhaiDynamic::from(s.clone())).collect())
            .unwrap_or_default()
    }

    fn name(&self) -> String {
        self.active_palette()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    }

    fn id(&self) -> String {
        self.active_id().unwrap_or_default()
    }

    fn set_active(&self, id: &str) -> bool {
        if !self.store.palettes.contains_key(id) {
            return false;
        }
        if let Some(p) = &self.persistence {
            p.set(PERSIST_KEY, serde_json::Value::String(id.to_string()));
            return true;
        }
        false
    }

    fn cycle(&self) -> String {
        let current = self.active_id().unwrap_or_default();
        let next = self.store.next_id(&current).unwrap_or_default();
        self.set_active(&next);
        next
    }

    fn list(&self) -> RhaiArray {
        self.store
            .order
            .iter()
            .map(|id| RhaiDynamic::from(id.clone()))
            .collect()
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptPaletteApi>("ScriptPaletteApi");

    engine.register_fn("get", |api: &mut ScriptPaletteApi, key: &str| {
        api.get_color(key)
    });
    engine.register_fn("particles", |api: &mut ScriptPaletteApi, ramp: &str| {
        api.get_particles(ramp)
    });
    engine.register_fn("name", |api: &mut ScriptPaletteApi| api.name());
    engine.register_fn("id", |api: &mut ScriptPaletteApi| api.id());
    engine.register_fn("set_active", |api: &mut ScriptPaletteApi, id: &str| {
        api.set_active(id)
    });
    engine.register_fn("cycle", |api: &mut ScriptPaletteApi| api.cycle());
    engine.register_fn("list", |api: &mut ScriptPaletteApi| api.list());
}
