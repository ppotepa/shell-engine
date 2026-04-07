use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::{Effect, Region};
use crate::metadata::{EffectMetadata, P_EASING, P_INTENSITY};
use crate::utils::color::lerp_colour;
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "whiteout",
    display_name: "Whiteout",
    summary: "Flash all pixels toward white.",
    category: "fade",
    compatible_targets: engine_core::effects::EffectTargetMask::ANY,
    params: &[P_INTENSITY, P_EASING],
    sample: "- name: whiteout\n  duration: 200\n  params:\n    intensity: 1.0",
};

/// Overexposes the image toward white with a soft pulse.
pub struct WhiteoutEffect;

impl Effect for WhiteoutEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let p = progress.clamp(0.0, 1.0);
        let pulse = (1.0 - ((p * 2.0) - 1.0).abs()).powf(0.55);
        let intensity = params.intensity.unwrap_or(1.0).max(0.0);
        let mix = (pulse * intensity).clamp(0.0, 1.0);
        let white = Color::White;

        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y).cloned() {
                    if cell.symbol == ' ' && cell.bg == Color::Reset {
                        continue;
                    }
                    let fg = lerp_colour(cell.fg, white, mix);
                    let bg = lerp_colour(cell.bg, white, (mix * 0.9).clamp(0.0, 1.0));
                    buffer.set(x, y, cell.symbol, fg, bg);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
