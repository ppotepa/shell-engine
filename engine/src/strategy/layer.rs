use engine_core::buffer::Buffer;
use crossterm::style::Color;

/// Controls whether a layer renders through a scratch buffer (safe) or
/// directly into the scene buffer (optimised, skips scratch when no effects).
pub trait LayerCompositor: Send + Sync {
    fn use_scratch(&self, layer_has_active_effects: bool) -> bool;
    /// Returns `true` when this is the experimental direct-render variant.
    fn is_direct(&self) -> bool { false }
}

/// Always uses the scratch-buffer path. Safe in all circumstances.
pub struct ScratchLayerCompositor;

impl LayerCompositor for ScratchLayerCompositor {
    #[inline]
    fn use_scratch(&self, _layer_has_active_effects: bool) -> bool {
        true
    }
}

/// Skips the scratch buffer for layers that have no active effects this frame.
/// Reduces one fill + blit per effectless layer per frame. Gate behind `--opt-comp`.
pub struct DirectLayerCompositor;

impl LayerCompositor for DirectLayerCompositor {
    #[inline]
    fn use_scratch(&self, layer_has_active_effects: bool) -> bool {
        layer_has_active_effects
    }
    fn is_direct(&self) -> bool { true }
}
