pub mod builtin;
pub mod effect;
pub mod utils;

pub use effect::{Effect, Region};

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
        use builtin::*;
        self.registry.insert("crt-on", Box::new(CrtOnEffect));
        self.registry.insert("power-off", Box::new(PowerOffEffect));
        self.registry.insert("fade-in", Box::new(FadeInEffect));
        self.registry.insert("fade-out", Box::new(FadeOutEffect));
        self.registry
            .insert("fade-to-black", Box::new(FadeToBlackEffect));
        self.registry.insert("scanlines", Box::new(ScanlinesEffect));
        self.registry.insert("shine", Box::new(ShineEffect));
        self.registry
            .insert("clear-to-colour", Box::new(ClearToColourEffect));
        self.registry.insert("brighten", Box::new(BrightenEffect));
        self.registry
            .insert("lightning-flash", Box::new(LightningFlashEffect));
        self.registry
            .insert("lightning-branch", Box::new(LightningBranchEffect));
        self.registry
            .insert("lightning-optical-80s", Box::new(LightningOptical80sEffect));
        self.registry
            .insert("lightning-fbm", Box::new(LightningFbmEffect));
        self.registry
            .insert("lightning-growth", Box::new(LightningGrowthEffect));
        self.registry
            .insert("lightning-ambient", Box::new(LightningAmbientEffect));
        self.registry
            .insert("lightning-natural", Box::new(LightningNaturalEffect));
        self.registry.insert("tesla-orb", Box::new(TeslaOrbEffect));
        self.registry
            .insert("screen-shake", Box::new(ScreenShakeEffect));
        self.registry.insert("whiteout", Box::new(WhiteoutEffect));
        self.registry
            .insert("glitch-out", Box::new(GlitchOutEffect));
        self.registry
            .insert("devour-out", Box::new(DevourOutEffect));
        self.registry
            .insert("artifact-out", Box::new(ArtifactOutEffect));
        self.registry
            .insert("shatter-glitch", Box::new(ShatterGlitchEffect));
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
    let p = effect.params.easing.apply(progress);
    shared_dispatcher().apply(&effect.name, p, &effect.params, region, buffer);
}
