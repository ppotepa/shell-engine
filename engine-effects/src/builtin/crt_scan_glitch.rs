//! Metadata-only postfx definition for CRT scanline glitch.

use crate::metadata::{slider, EffectMetadata, P_EASING};
use engine_core::buffer::Buffer;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-scan-glitch",
    display_name: "CRT Scan Glitch",
    summary: "PostFX: sporadic scanline sweep with right-shift chromatic glitch.",
    category: "postfx",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider("intensity", "Intensity", "Shift and chroma strength.", 0.0, 2.0, 0.05, ""),
        slider("transparency", "Thickness", "Scanline band thickness.", 0.0, 1.0, 0.05, ""),
        slider("brightness", "Brightness", "Active band brightness boost.", 0.0, 2.0, 0.05, ""),
        slider("speed", "Speed", "Band trigger frequency.", 0.0, 2.0, 0.1, ""),
        P_EASING,
    ],
    sample: "- name: crt-scan-glitch\n  params:\n    intensity: 0.25\n    transparency: 0.25\n    brightness: 1.0\n    speed: 0.8",
};

pub struct CtrScanGlitchEffect;

impl Effect for CtrScanGlitchEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, _region: Region, _buffer: &mut Buffer) {
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
