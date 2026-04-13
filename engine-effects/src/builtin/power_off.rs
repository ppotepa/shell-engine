//! Effect simulating an old CRT display shutting down (collapse to centre line).

use super::crt_on::{crt_blank_row, crt_dim_row, crt_static_row};
use crate::metadata::{EffectMetadata, P_EASING};
use crate::utils::noise::crt_hash;
use engine_core::buffer::{Buffer, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

/// Static effect metadata exposed to the editor and effect registry.
pub static METADATA: EffectMetadata = EffectMetadata {
    name: "power-off",
    display_name: "Power Off",
    summary: "Old display shutdown collapse to center line.",
    category: "crt",
    compatible_targets: EffectTargetMask::SCENE,
    params: &[P_EASING],
    sample: "- name: power-off\n  duration: 700\n  params:\n    easing: ease-in",
};

/// Effect that collapses the display inward to a horizontal centre line, mimicking CRT power-off.
pub struct PowerOffEffect;

impl Effect for PowerOffEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.height == 0 {
            return;
        }

        let centre_y = region.y + region.height / 2;
        let half_h = region.height as f32 / 2.0;
        let remaining = (half_h * (1.0 - progress)) as u16;
        let frame_seed = (progress * 600.0) as u32;

        for dy in 0..region.height {
            let abs_y = region.y + dy;
            let dist = centre_y.abs_diff(abs_y);

            if progress > 0.96 {
                if dist == 0 {
                    for dx in 0..region.width {
                        let s = crt_hash(region.x + dx, abs_y, frame_seed);
                        let ch = ['─', '═', ' '][s as usize % 3];
                        buffer.set(region.x + dx, abs_y, ch, Color::DarkGrey, TRUE_BLACK);
                    }
                } else {
                    crt_blank_row(region, abs_y, buffer);
                }
            } else if dist > remaining + 2 {
                crt_blank_row(region, abs_y, buffer);
            } else if dist == remaining + 2 {
                crt_dim_row(region, abs_y, frame_seed.wrapping_add(3), buffer);
            } else if dist == remaining + 1 {
                crt_static_row(region, abs_y, Color::DarkGrey, frame_seed, buffer);
            } else if dist == remaining {
                for dx in 0..region.width {
                    let s = crt_hash(region.x + dx, abs_y, frame_seed);
                    let ch = ['═', '█', '▓', '═', '█'][s as usize % 5];
                    buffer.set(region.x + dx, abs_y, ch, Color::White, TRUE_BLACK);
                }
            }
            // dist < remaining: inner content — leave untouched
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
