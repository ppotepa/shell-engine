pub mod effect;
pub mod utils;
pub mod builtin;

pub use effect::{Effect, Region};

use std::collections::HashMap;
use crate::scene::{Effect as SceneEffect, EffectParams};
use crate::buffer::Buffer;

/// Dispatches effects by name to their implementations.
pub struct EffectDispatcher {
    registry: HashMap<&'static str, Box<dyn Effect>>,
}

impl EffectDispatcher {
    pub fn new() -> Self {
        let mut d = Self { registry: HashMap::new() };
        d.register_builtins();
        d
    }

    fn register_builtins(&mut self) {
        use builtin::*;
        self.registry.insert("crt-on",           Box::new(CrtOnEffect));
        self.registry.insert("power-off",         Box::new(PowerOffEffect));
        self.registry.insert("fade-in",           Box::new(FadeInEffect));
        self.registry.insert("fade-out",          Box::new(FadeOutEffect));
        self.registry.insert("fade-to-black",     Box::new(FadeToBlackEffect));
        self.registry.insert("scanlines",         Box::new(ScanlinesEffect));
        self.registry.insert("shine",             Box::new(ShineEffect));
        self.registry.insert("clear-to-colour",   Box::new(ClearToColourEffect));
        self.registry.insert("brighten",          Box::new(BrightenEffect));
        self.registry.insert("glitch-out",        Box::new(GlitchOutEffect));
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
}

impl Default for EffectDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience wrapper for call sites that already have a `&SceneEffect`.
/// Applies easing, then dispatches to the effect registry.
pub fn apply_effect(effect: &SceneEffect, progress: f32, region: Region, buffer: &mut Buffer) {
    let p = effect.params.easing.apply(progress);
    let dispatcher = EffectDispatcher::new();
    dispatcher.apply(&effect.name, p, &effect.params, region, buffer);
}
