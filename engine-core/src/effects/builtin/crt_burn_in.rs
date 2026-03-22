//! Metadata-only postfx definition for CRT phosphor burn-in / persistence.

use crate::buffer::Buffer;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING};
use crate::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-burn-in",
    display_name: "CRT Burn-In",
    summary: "PostFX: phosphor persistence — fading ghost of previous frames lingers under current content. Survives scene transitions for realistic CRT feel.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE,
    params: &[
        slider("intensity", "Intensity", "Ghost visibility strength.", 0.0, 1.0, 0.05, ""),
        slider("alpha", "Alpha", "Maximum blend opacity of the ghost layer.", 0.0, 1.0, 0.05, ""),
        slider("speed", "Decay", "How quickly the ghost fades (higher = faster fade).", 0.05, 0.95, 0.05, ""),
        slider("brightness", "Brightness", "Ghost luminance multiplier.", 0.2, 2.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-burn-in\n  params:\n    intensity: 0.45\n    alpha: 0.30\n    speed: 0.35\n    brightness: 1.0",
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
