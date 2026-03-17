//! Effect that sweeps a Gaussian highlight beam across the frame.

use crate::buffer::Buffer;
use crate::effects::effect::{Effect, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING, P_INTENSITY, P_SPEED};
use crate::effects::utils::color::lerp_colour;
use crate::scene::EffectParams;
use crossterm::style::Color;

/// Static effect metadata exposed to the editor and effect registry.
pub static METADATA: EffectMetadata = EffectMetadata {
    name: "shine",
    display_name: "Shine",
    summary: "Moving highlight beam crossing the whole frame.",
    category: "colour",
    compatible_targets: crate::effects::effect::EffectTargetMask::ANY,
    params: &[
        slider(
            "angle",
            "Angle",
            "Beam angle in degrees (0=vertical sweep).",
            0.0,
            90.0,
            2.0,
            "deg",
        ),
        slider(
            "width",
            "Width",
            "Gaussian beam half-width in cells.",
            1.0,
            12.0,
            0.5,
            "cols",
        ),
        slider(
            "falloff",
            "Falloff",
            "Edge sharpness exponent (>1 = sharper).",
            0.0,
            5.0,
            0.2,
            "",
        ),
        P_INTENSITY,
        P_SPEED,
        P_EASING,
    ],
    sample:
        "- name: shine\n  duration: 800\n  params:\n    angle: 18\n    width: 6\n    intensity: 1.0",
};

/// Effect that sweeps a directional Gaussian highlight beam across the target region.
pub struct ShineEffect;

impl Effect for ShineEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let angle_deg = params.angle.unwrap_or(0.0);
        let width = params.width.unwrap_or(4.0);
        let falloff = params.falloff.unwrap_or(1.0);
        let peak = params.intensity.unwrap_or(1.0);
        let highlight = params
            .colour
            .as_ref()
            .map(Color::from)
            .unwrap_or(Color::White);

        let angle_rad = angle_deg.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let mut min_proj = f32::MAX;
        let mut max_proj = f32::MIN;
        for &cy in &[region.y, region.y + region.height.saturating_sub(1)] {
            for &cx in &[region.x, region.x + region.width.saturating_sub(1)] {
                let p = cx as f32 * cos_a + cy as f32 * sin_a;
                if p < min_proj {
                    min_proj = p;
                }
                if p > max_proj {
                    max_proj = p;
                }
            }
        }
        let margin = width * 2.0;
        let beam_pos = (min_proj - margin) + progress * (max_proj - min_proj + margin * 2.0);

        let sigma = width.max(1.0);
        let two_sigma_sq = 2.0 * sigma * sigma;
        let falloff = falloff.max(0.1);
        let peak = peak.clamp(0.0, 1.0);

        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    if symbol == ' ' {
                        continue;
                    }

                    let proj = x as f32 * cos_a + y as f32 * sin_a;
                    let dist = proj - beam_pos;
                    let raw = (-dist * dist / two_sigma_sq).exp();
                    let intensity = raw.powf(falloff) * peak;

                    if intensity < 0.01 {
                        continue;
                    }

                    let fg = lerp_colour(cell.fg, highlight, intensity);
                    buffer.set(x, y, symbol, fg, cell.bg);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
