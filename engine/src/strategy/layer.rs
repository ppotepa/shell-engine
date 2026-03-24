use engine_core::buffer::Buffer;
use crossterm::style::Color;

/// Controls whether a layer renders through a scratch buffer (safe) or
/// directly into the scene buffer (optimised, skips scratch when no effects).
pub trait LayerCompositor: Send + Sync {
    /// Returns `true` if this layer should use the full scratch-buffer path
    /// (fill → render → effects → blit). When `false`, sprites render directly
    /// onto the destination buffer — only safe for layers with no active effects.
    fn use_scratch(&self, layer_has_active_effects: bool) -> bool;
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
}
