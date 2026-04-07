//! Effect that overlays classic CRT horizontal scanlines onto the frame.

use engine_core::buffer::{Buffer, TRUE_BLACK};
use engine_core::effects::{Effect, EffectTargetMask, Region};
use crate::metadata::{EffectMetadata, P_INTENSITY};
use engine_core::scene::EffectParams;

/// Static effect metadata exposed to the editor and effect registry.
pub static METADATA: EffectMetadata = EffectMetadata {
    name: "scanlines",
    display_name: "Scanlines",
    summary: "Classic CRT horizontal scanline overlay.",
    category: "crt",
    compatible_targets: EffectTargetMask::SCENE,
    params: &[P_INTENSITY],
    sample: "- name: scanlines\n  duration: 0\n  params:\n    intensity: 0.5",
};

/// Effect that darkens every other row to produce a CRT scanline appearance.
pub struct ScanlinesEffect;

impl Effect for ScanlinesEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        for dy in (0..region.height).step_by(2) {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    let fg = cell.fg;
                    buffer.set(x, y, symbol, fg, TRUE_BLACK);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
