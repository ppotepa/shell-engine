//! Metadata-only postfx definition for CRT tube distortion.

use engine_core::buffer::Buffer;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use crate::metadata::{slider, EffectMetadata, P_EASING};
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-distort",
    display_name: "CRT Distort",
    summary: "PostFX: tube-like cylindrical distortion with CRT margins.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider("intensity", "Intensity", "Global warp intensity.", 0.0, 2.0, 0.05, ""),
        slider("distortion", "Distortion", "Primary UV distortion strength.", 0.0, 1.0, 0.05, ""),
        slider("sphericality", "Curvature", "Cylindrical curve amount.", 0.0, 1.0, 0.05, ""),
        slider("transparency", "Margin", "CRT bezel margin size.", 0.0, 1.0, 0.05, ""),
        slider("brightness", "Brightness", "Post-warp brightness multiplier.", 0.0, 2.0, 0.05, ""),
        slider("speed", "Speed", "Geometry wobble speed.", 0.0, 2.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-distort\n  params:\n    intensity: 0.2\n    distortion: 0.10\n    sphericality: 0.2\n    transparency: 0.20\n    brightness: 1.0\n    speed: 0.3",
};

pub struct CtrDistortEffect;

impl Effect for CtrDistortEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, _region: Region, _buffer: &mut Buffer) {
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
