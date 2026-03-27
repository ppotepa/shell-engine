use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::color::Color;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING};
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::math::smoothstep;
use crate::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-reflection",
    display_name: "CRT Reflection",
    summary: "Curved CRT glass reflection warp for bitmap sprites.",
    category: "distortion",
    compatible_targets: EffectTargetMask::SPRITE_BITMAP,
    params: &[
        slider(
            "sphericality",
            "Sphericality",
            "Curved-glass spherical distortion strength.",
            0.0,
            1.0,
            0.05,
            "",
        ),
        slider(
            "transparency",
            "Transparency",
            "How strongly the reflected image remains visible.",
            0.0,
            1.0,
            0.05,
            "",
        ),
        slider(
            "brightness",
            "Brightness",
            "Brightness multiplier applied to the reflected image.",
            0.0,
            2.0,
            0.05,
            "",
        ),
        P_EASING,
    ],
    sample: "- name: crt-reflection\n  duration: 1200\n  params:\n    sphericality: 0.28\n    transparency: 0.42\n    brightness: 0.82",
};

pub struct CrtReflectionEffect;

impl Effect for CrtReflectionEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let sphericality = params.sphericality.unwrap_or(0.22).clamp(0.0, 1.0);
        let transparency = params.transparency.unwrap_or(0.40).clamp(0.0, 1.0);
        let brightness = params.brightness.unwrap_or(0.82).clamp(0.0, 2.0);
        let strength = smoothstep(progress.clamp(0.0, 1.0));
        let alpha = (transparency * strength).clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return;
        }

        let snapshot = snapshot_region(region, buffer);
        if !snapshot.iter().any(has_signal) {
            return;
        }

        for dy in 0..region.height {
            for dx in 0..region.width {
                let (nx, ny) = normalized_coords(dx, dy, region.width, region.height);
                let radius2 = (nx * nx + ny * ny).min(1.0);
                let curvature = 1.0 - sphericality * (0.34 + 0.66 * radius2);
                let src_nx =
                    (nx * curvature + nx * ny.abs() * sphericality * 0.08).clamp(-1.0, 1.0);
                let src_ny = (ny * (1.0 - sphericality * (0.20 + 0.45 * radius2))).clamp(-1.0, 1.0);
                let sx = remap_axis(src_nx, region.width);
                let sy = remap_axis(src_ny, region.height);
                let sample = &snapshot[sy as usize * region.width as usize + sx as usize];
                if !has_signal(sample) {
                    continue;
                }

                let edge_fade = (1.0
                    - smoothstep(((radius2 - 0.20) / 0.80).clamp(0.0, 1.0)) * 0.48)
                    .clamp(0.28, 1.0);
                let glint = (1.0 - ny.abs()).powf(2.0) * sphericality * 0.18;
                let amount = (alpha * edge_fade).clamp(0.0, 1.0);
                let x = region.x + dx;
                let y = region.y + dy;
                let existing = buffer.get(x, y).cloned().unwrap_or_default();
                let reflected_fg = scale_toward_white(sample.fg, brightness, glint);
                let reflected_bg = scale_background(sample.bg, brightness * 0.55);
                let symbol = pick_symbol(&existing, sample, amount);
                let fg = lerp_colour(existing.fg, reflected_fg, amount);
                let bg = lerp_colour(
                    normalize_bg(existing.bg),
                    reflected_bg,
                    (amount * 0.55).clamp(0.0, 1.0),
                );
                buffer.set(x, y, symbol, fg, bg);
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

fn snapshot_region(region: Region, buffer: &Buffer) -> Vec<Cell> {
    let mut out = Vec::with_capacity(region.width as usize * region.height as usize);
    for dy in 0..region.height {
        for dx in 0..region.width {
            out.push(
                buffer
                    .get(region.x + dx, region.y + dy)
                    .cloned()
                    .unwrap_or_default(),
            );
        }
    }
    out
}

fn normalized_coords(dx: u16, dy: u16, width: u16, height: u16) -> (f32, f32) {
    let nx = if width <= 1 {
        0.0
    } else {
        (dx as f32 / (width - 1) as f32) * 2.0 - 1.0
    };
    let ny = if height <= 1 {
        0.0
    } else {
        (dy as f32 / (height - 1) as f32) * 2.0 - 1.0
    };
    (nx, ny)
}

fn remap_axis(value: f32, extent: u16) -> u16 {
    if extent <= 1 {
        return 0;
    }
    let scaled = ((value + 1.0) * 0.5 * (extent - 1) as f32).round();
    scaled.clamp(0.0, (extent - 1) as f32) as u16
}

fn has_signal(cell: &Cell) -> bool {
    cell.symbol != ' ' || cell.bg != Color::Reset
}

fn normalize_bg(color: Color) -> Color {
    if matches!(color, Color::Reset) {
        TRUE_BLACK
    } else {
        color
    }
}

fn scale_toward_white(base: Color, brightness: f32, glint: f32) -> Color {
    let scale = brightness.clamp(0.0, 2.0);
    let (r, g, b) = colour_to_rgb(base);
    let scaled = Color::Rgb {
        r: ((r as f32 * scale).round()).clamp(0.0, 255.0) as u8,
        g: ((g as f32 * scale).round()).clamp(0.0, 255.0) as u8,
        b: ((b as f32 * scale).round()).clamp(0.0, 255.0) as u8,
    };
    if scale <= 1.0 && glint <= 0.0 {
        scaled
    } else {
        lerp_colour(scaled, Color::White, glint.clamp(0.0, 0.35))
    }
}

fn scale_background(base: Color, brightness: f32) -> Color {
    let scale = brightness.clamp(0.0, 2.0);
    let (r, g, b) = colour_to_rgb(normalize_bg(base));
    Color::Rgb {
        r: ((r as f32 * scale).round()).clamp(0.0, 255.0) as u8,
        g: ((g as f32 * scale).round()).clamp(0.0, 255.0) as u8,
        b: ((b as f32 * scale).round()).clamp(0.0, 255.0) as u8,
    }
}

fn pick_symbol(existing: &Cell, sample: &Cell, amount: f32) -> char {
    if sample.symbol != ' ' && (existing.symbol == ' ' || amount > 0.42) {
        sample.symbol
    } else {
        existing.symbol
    }
}

#[cfg(test)]
mod tests {
    use super::{CrtReflectionEffect, METADATA};
    use crate::buffer::{Buffer, TRUE_BLACK};
    use crate::color::Color;
    use crate::effects::effect::{Effect, EffectTargetMask, Region};
    use crate::scene::EffectParams;

    #[test]
    fn metadata_is_bitmap_only() {
        assert!(METADATA
            .compatible_targets
            .supports(crate::scene::EffectTargetKind::SpriteBitmap));
        assert!(!METADATA
            .compatible_targets
            .supports(crate::scene::EffectTargetKind::SpriteText));
        assert_eq!(METADATA.compatible_targets, EffectTargetMask::SPRITE_BITMAP);
    }

    #[test]
    fn transparency_zero_keeps_region_unchanged() {
        let mut buffer = Buffer::new(4, 2);
        buffer.fill(TRUE_BLACK);
        buffer.set(
            1,
            0,
            '█',
            Color::Rgb {
                r: 20,
                g: 30,
                b: 40,
            },
            TRUE_BLACK,
        );
        let before = buffer.clone();

        CrtReflectionEffect.apply(
            1.0,
            &EffectParams {
                transparency: Some(0.0),
                sphericality: Some(0.8),
                brightness: Some(1.4),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 4,
                height: 2,
            },
            &mut buffer,
        );

        assert_eq!(buffer.diff().len(), before.diff().len());
        for y in 0..2 {
            for x in 0..4 {
                assert_eq!(buffer.get(x, y), before.get(x, y));
            }
        }
    }

    #[test]
    fn modifies_only_cells_inside_region() {
        let mut buffer = Buffer::new(6, 3);
        buffer.fill(TRUE_BLACK);
        for y in 0..3 {
            for x in 0..6 {
                buffer.set(
                    x,
                    y,
                    '█',
                    Color::Rgb {
                        r: (x * 20) as u8,
                        g: (y * 50) as u8,
                        b: 120,
                    },
                    TRUE_BLACK,
                );
            }
        }
        let outside_before = buffer.get(0, 0).cloned().expect("outside cell");

        CrtReflectionEffect.apply(
            1.0,
            &EffectParams {
                sphericality: Some(1.0),
                transparency: Some(0.9),
                brightness: Some(1.3),
                ..EffectParams::default()
            },
            Region {
                x: 1,
                y: 0,
                width: 4,
                height: 3,
            },
            &mut buffer,
        );

        assert_eq!(
            buffer.get(0, 0).cloned().expect("outside after"),
            outside_before
        );
        assert_ne!(
            buffer.get(2, 1).cloned().expect("inside after").fg,
            Color::Rgb {
                r: 40,
                g: 50,
                b: 120
            }
        );
    }
}
