//! Metadata-only postfx definition for CRT phosphor burn-in / persistence.

use crate::buffer::Buffer;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING};
use crate::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-burn-in",
    display_name: "CRT Burn-In",
    summary: "PostFX: phosphor persistence — ghost of old scene overlays new one with exponential decay, brightness pump, P31 colour shift, and 2D blur.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE,
    params: &[
        slider("alpha", "Initial brightness", "Ghost starts at this fraction of original brightness.", 0.0, 1.0, 0.05, ""),
        slider("speed", "Fade duration", "How many seconds the ghost takes to disappear.", 0.01, 10.0, 0.05, ""),
        slider("brightness", "Luminance", "Ghost luminance multiplier.", 0.1, 2.0, 0.1, ""),
        slider("intensity", "Intensity", "Overall effect strength (0 = off, 1 = full).", 0.0, 1.0, 0.05, ""),
        slider("pump", "Brightness pump", "First-frame flash multiplier (1.0 = no pump).", 1.0, 3.0, 0.1, ""),
        slider("decay_tint", "Phosphor tint", "Colour decay shift: 0 = uniform, 1 = full P31 green.", 0.0, 1.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-burn-in\n  params:\n    alpha: 0.70\n    speed: 0.15\n    brightness: 1.0\n    intensity: 1.0\n    pump: 1.3\n    decay_tint: 0.8",
};

pub struct CrtBurnInEffect;

impl Effect for CrtBurnInEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, _region: Region, _buffer: &mut Buffer) {
        // Metadata-only: actual implementation lives in postfx/pass_burn_in.rs
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
