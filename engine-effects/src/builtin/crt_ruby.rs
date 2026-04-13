//! Metadata-only postfx definition for ruby CRT tint pass.

use crate::metadata::{slider, EffectMetadata, P_EASING};
use engine_core::buffer::Buffer;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-ruby",
    display_name: "CRT Ruby",
    summary: "PostFX: ruby tint, edge reveal sweep and center darkening.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider("intensity", "Intensity", "Ruby tint and reveal strength.", 0.0, 2.0, 0.05, ""),
        slider("transparency", "Width", "Reveal band width.", 0.0, 1.0, 0.05, ""),
        slider("brightness", "Brightness", "Overall brightness multiplier.", 0.0, 2.0, 0.05, ""),
        slider("speed", "Speed", "Reveal sweep tempo.", 0.0, 2.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-ruby\n  params:\n    intensity: 0.2\n    transparency: 0.2\n    brightness: 0.98\n    speed: 0.55",
};

pub struct CtrRubyEffect;

impl Effect for CtrRubyEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, _region: Region, _buffer: &mut Buffer) {
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
