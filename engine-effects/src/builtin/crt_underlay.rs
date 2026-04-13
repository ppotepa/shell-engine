//! Metadata-only postfx definition for CRT underlay glow.

use crate::metadata::{slider, EffectMetadata, P_EASING};
use engine_core::buffer::Buffer;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-underlay",
    display_name: "CRT Underlay",
    summary: "PostFX: soft phosphor underlay glow beneath scene content.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider("intensity", "Intensity", "Overall glow strength.", 0.0, 2.0, 0.05, ""),
        slider("alpha", "Alpha", "Underlay blend opacity.", 0.0, 1.0, 0.05, ""),
        slider("sphericality", "Offset", "Glow offset bias.", 0.0, 1.0, 0.05, ""),
        slider("transparency", "Spread", "Glow spread / blur amount.", 0.0, 1.0, 0.05, ""),
        slider("brightness", "Brightness", "Glow luminance multiplier.", 0.0, 2.0, 0.05, ""),
        slider("speed", "Speed", "Temporal shimmer speed.", 0.0, 2.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-underlay\n  params:\n    intensity: 1.0\n    alpha: 0.30\n    sphericality: 0.0\n    transparency: 0.35\n    brightness: 1.1\n    speed: 0.4",
};

pub struct CtrUnderlayEffect;

impl Effect for CtrUnderlayEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, _region: Region, _buffer: &mut Buffer) {
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
