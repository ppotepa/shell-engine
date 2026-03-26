//! Fade-in and fade-out effects that linearly interpolate foreground brightness.

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{EffectMetadata, P_EASING};
use crate::effects::utils::color::lerp_colour;
use crate::scene::EffectParams;
use crate::color::Color;

/// Static effect metadata for the fade-in variant.
pub static METADATA_FADE_IN: EffectMetadata = EffectMetadata {
    name: "fade-in",
    display_name: "Fade In",
    summary: "Reveal from dark to full brightness.",
    category: "fade",
    compatible_targets: EffectTargetMask::ANY,
    params: &[P_EASING],
    sample: "- name: fade-in\n  duration: 500\n  params:\n    easing: linear",
};

/// Static effect metadata for the fade-out variant.
pub static METADATA_FADE_OUT: EffectMetadata = EffectMetadata {
    name: "fade-out",
    display_name: "Fade Out",
    summary: "Fade from full brightness to dark.",
    category: "fade",
    compatible_targets: EffectTargetMask::ANY,
    params: &[P_EASING],
    sample: "- name: fade-out\n  duration: 500\n  params:\n    easing: linear",
};

/// Effect that reveals the frame from black by linearly brightening each cell's foreground.
pub struct FadeInEffect;

impl Effect for FadeInEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let p = progress.clamp(0.0, 1.0);
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    if symbol == ' ' {
                        continue;
                    }
                    let target_fg = cell.fg;
                    let original_bg = cell.bg;
                    let fg = lerp_colour(TRUE_BLACK, target_fg, p);
                    buffer.set(x, y, symbol, fg, original_bg);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_FADE_IN
    }
}

/// Effect that hides the frame by linearly dimming each cell's foreground to black.
pub struct FadeOutEffect;

impl Effect for FadeOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let p = progress.clamp(0.0, 1.0);
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    if symbol == ' ' {
                        continue;
                    }
                    if p >= 0.999 {
                        // Clear to transparent so blit_from skips this cell,
                        // revealing the background layer instead of leaving an opaque rectangle.
                        buffer.set(x, y, ' ', Color::Reset, Color::Reset);
                        continue;
                    }
                    let fg = lerp_colour(cell.fg, TRUE_BLACK, p);
                    // Preserve cell.bg so we don't overwrite background-layer colours
                    // with an opaque TRUE_BLACK when this scratch layer is blit'd onto the scene.
                    buffer.set(x, y, symbol, fg, cell.bg);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA_FADE_OUT
    }
}
