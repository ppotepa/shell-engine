use crate::buffer::Buffer;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::EffectParams;
use crossterm::style::Color;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "posterize",
    display_name: "Posterize",
    summary: "Reduces colour band count by quantizing RGB channels.",
    category: "colour",
    compatible_targets: EffectTargetMask::ANY,
    params: &[
        slider(
            "levels",
            "Levels",
            "Number of quantization bands per colour channel.",
            1.0,
            100.0,
            1.0,
            "",
        ),
        P_EASING,
    ],
    sample: "- name: posterize\n  duration: 1200\n  params:\n    levels: 7",
};

pub struct PosterizeEffect;

impl Effect for PosterizeEffect {
    /// `progress` is ignored — posterize stays at full strength while active.
    fn apply(&self, _progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let levels = params.levels.unwrap_or(7).clamp(1, 100);
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                let Some(cell) = buffer.get(x, y).cloned() else {
                    continue;
                };
                if cell.symbol == ' ' && matches!(cell.bg, Color::Reset) {
                    continue;
                }

                let fg = posterize_color(cell.fg, levels);
                let bg = if matches!(cell.bg, Color::Reset) {
                    Color::Reset
                } else {
                    posterize_color(cell.bg, levels)
                };
                buffer.set(x, y, cell.symbol, fg, bg);
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

fn posterize_color(color: Color, levels: u8) -> Color {
    let (r, g, b) = colour_to_rgb(color);
    Color::Rgb {
        r: posterize_component(r, levels),
        g: posterize_component(g, levels),
        b: posterize_component(b, levels),
    }
}

fn posterize_component(value: u8, levels: u8) -> u8 {
    if levels <= 1 {
        return 0;
    }
    let step = 255.0 / (levels as f32 - 1.0);
    ((value as f32 / step).round() * step)
        .clamp(0.0, 255.0)
        .round() as u8
}

#[cfg(test)]
mod tests {
    use super::{PosterizeEffect, METADATA};
    use crate::buffer::Buffer;
    use crate::effects::effect::{Effect, EffectTargetMask, Region};
    use crate::scene::EffectParams;
    use crossterm::style::Color;

    #[test]
    fn metadata_is_any_target() {
        assert_eq!(METADATA.compatible_targets, EffectTargetMask::ANY);
    }

    #[test]
    fn posterize_quantizes_rgb_channels() {
        let mut buf = Buffer::new(1, 1);
        buf.set(
            0,
            0,
            '█',
            Color::Rgb {
                r: 200,
                g: 101,
                b: 12,
            },
            Color::Black,
        );

        PosterizeEffect.apply(
            1.0,
            &EffectParams {
                levels: Some(7),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            &mut buf,
        );

        let cell = buf.get(0, 0).expect("posterized cell");
        assert_eq!(
            cell.fg,
            Color::Rgb {
                r: 213,
                g: 85,
                b: 0
            }
        );
    }
}
