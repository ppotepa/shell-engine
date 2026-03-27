pub mod builtin;
pub mod effect;
pub mod metadata;
pub mod utils;

pub use effect::{Effect, EffectTargetMask, Region};
pub use metadata::{EffectMetadata, ParamControl, ParamMetadata};

use crate::buffer::Buffer;
use crate::scene::{Effect as SceneEffect, EffectParams};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Dispatches effects by name to their implementations.
pub struct EffectDispatcher {
    registry: HashMap<&'static str, Box<dyn Effect>>,
}

impl EffectDispatcher {
    pub fn new() -> Self {
        let mut d = Self {
            registry: HashMap::new(),
        };
        d.register_builtins();
        d
    }

    fn register_builtins(&mut self) {
        for def in builtin::BUILTIN_EFFECTS {
            self.registry.insert(def.name, (def.constructor)());
        }
    }

    pub fn apply(
        &self,
        name: &str,
        progress: f32,
        params: &EffectParams,
        region: Region,
        buffer: &mut Buffer,
    ) {
        if let Some(effect) = self.registry.get(name) {
            effect.apply(progress, params, region, buffer);
        }
    }

    pub fn supports(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }

    pub fn supports_target_kind(
        &self,
        name: &str,
        target_kind: crate::scene::EffectTargetKind,
    ) -> bool {
        self.registry
            .get(name)
            .map(|effect| effect.compatible_targets().supports(target_kind))
            .unwrap_or(false)
    }

    /// Return static metadata for a builtin effect by name.
    pub fn metadata(&self, name: &str) -> &'static EffectMetadata {
        self.registry
            .get(name)
            .map(|e| e.metadata())
            .unwrap_or(&crate::effects::metadata::META_UNKNOWN)
    }

    /// Return list of all registered builtin effect names.
    pub fn builtin_names() -> &'static [&'static str] {
        builtin::builtin_names()
    }
}

impl Default for EffectDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

static SHARED_DISPATCHER: OnceLock<EffectDispatcher> = OnceLock::new();

pub fn shared_dispatcher() -> &'static EffectDispatcher {
    SHARED_DISPATCHER.get_or_init(EffectDispatcher::new)
}

/// Convenience wrapper for call sites that already have a `&SceneEffect`.
/// Applies easing, then dispatches to the effect registry.
pub fn apply_effect(effect: &SceneEffect, progress: f32, region: Region, buffer: &mut Buffer) {
    // Fast-path: skip zero-sized regions
    if region.width == 0 || region.height == 0 {
        return;
    }
    let p = effect.params.easing.apply(progress);
    shared_dispatcher().apply(&effect.name, p, &effect.params, region, buffer);
}
