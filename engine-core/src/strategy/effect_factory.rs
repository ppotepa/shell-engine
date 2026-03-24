use crate::effects::effect::Effect;

/// Factory for mod-defined effects.
///
/// The `EffectDispatcher` checks this factory before its builtin registry,
/// allowing mods to override built-in effects or introduce entirely new ones
/// (e.g. Rhai-scripted effects, WASM plugins, or compiled extensions).
///
/// Implement this trait and register it in the engine to enable mod-defined effects.
pub trait ModEffectFactory: Send + Sync {
    /// Try to create an effect by name.
    /// Return `Some(effect)` if this factory handles the given name,
    /// or `None` to fall through to the builtin registry.
    fn create_effect(&self, name: &str) -> Option<Box<dyn Effect>>;
}
