use crate::buffer::Buffer;
use crate::effects::effect::{Effect, Region};
use crate::effects::metadata::{EffectMetadata, P_EASING, P_INTENSITY};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::EffectParams;
use crossterm::style::Color;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "brighten",
    display_name: "Brighten",
    summary: "Brightens pixels toward white by intensity over duration.",
    category: "colour",
    compatible_targets: crate::effects::effect::EffectTargetMask::ANY,
    params: &[P_INTENSITY, P_EASING],
    sample: "- name: brighten\n  duration: 400\n  params:\n    intensity: 0.6",
};

/// Brightens sprite pixels toward white by `intensity` (0.0–1.0) over the effect duration.
/// At progress=1.0 and intensity=0.1, each colour channel is boosted by 10%.
/// Useful for a "highlight reveal" moment on logo text.
pub struct BrightenEffect;

impl Effect for BrightenEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let intensity = params.intensity.unwrap_or(0.1).clamp(0.0, 1.0);
        let boost = progress * intensity;

        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    if cell.symbol == ' ' {
                        continue;
                    }
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    let r2 = ((r as f32 + (255.0 - r as f32) * boost) as u8).min(255);
                    let g2 = ((g as f32 + (255.0 - g as f32) * boost) as u8).min(255);
                    let b2 = ((b as f32 + (255.0 - b as f32) * boost) as u8).min(255);
                    buffer.set(
                        x,
                        y,
                        cell.symbol,
                        Color::Rgb {
                            r: r2,
                            g: g2,
                            b: b2,
                        },
                        cell.bg,
                    );
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
