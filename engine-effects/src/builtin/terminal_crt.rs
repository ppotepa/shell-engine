//! Scene/layer post-process that emulates a classic CRT terminal look.

use engine_core::buffer::{Buffer, Cell, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use crate::metadata::{slider, EffectMetadata, P_EASING};
use crate::utils::color::{colour_to_rgb, lerp_colour};
use crate::utils::math::smoothstep;
use crate::utils::noise::crt_hash;
use engine_core::scene::EffectParams;
use std::f32::consts::TAU;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "terminal-crt",
    display_name: "Terminal CRT",
    summary: "Subtle CRT post-process with safe curvature, scanlines, vignette and phosphor noise.",
    category: "crt",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider(
            "intensity",
            "Intensity",
            "Overall CRT strength (scanlines + vignette).",
            0.0,
            2.0,
            0.05,
            "",
        ),
        slider(
            "sphericality",
            "Sphericality",
            "Glass curvature warp strength.",
            0.0,
            1.0,
            0.05,
            "",
        ),
        slider(
            "transparency",
            "Noise",
            "Phosphor noise/grain amount.",
            0.0,
            1.0,
            0.05,
            "",
        ),
        slider(
            "brightness",
            "Brightness",
            "Final brightness multiplier.",
            0.0,
            2.0,
            0.05,
            "",
        ),
        slider(
            "speed",
            "Flicker Speed",
            "Temporal speed of subtle CRT flicker/noise.",
            0.0,
            2.0,
            0.1,
            "",
        ),
        P_EASING,
    ],
    sample: "- name: terminal-crt\n  duration: 9000\n  params:\n    intensity: 0.6\n    sphericality: 0.12\n    transparency: 0.10\n    brightness: 0.95\n    speed: 0.45",
};

pub struct TerminalCrtEffect;

impl Effect for TerminalCrtEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let intensity = params.intensity.unwrap_or(0.60).clamp(0.0, 1.5);
        if intensity <= 0.0 {
            return;
        }
        let sphericality = params.sphericality.unwrap_or(0.12).clamp(0.0, 0.4);
        let noise_amount = params.transparency.unwrap_or(0.10).clamp(0.0, 0.6);
        let brightness = params.brightness.unwrap_or(0.95).clamp(0.6, 1.2);
        let speed = params.speed.unwrap_or(0.45).clamp(0.0, 1.2);

        let frame = (progress.clamp(0.0, 1.0) * 10_000.0) as u32;
        let snapshot = snapshot_region(region, buffer);
        if snapshot.is_empty() {
            return;
        }

        let ramp = smoothstep(progress.clamp(0.0, 1.0));
        let flicker_phase = progress * TAU * (2.0 + speed * 3.0);
        let flicker = 1.0 + flicker_phase.sin() * (0.004 + 0.010 * intensity) * ramp;

        for dy in 0..region.height {
            for dx in 0..region.width {
                let (nx, ny) = normalized_coords(dx, dy, region.width, region.height);
                let radius2 = (nx * nx + ny * ny).min(1.0);

                let warp = 1.0 - sphericality * (0.08 + 0.26 * radius2);
                let src_nx = (nx * warp).clamp(-0.98, 0.98);
                let src_ny =
                    (ny * (1.0 - sphericality * (0.06 + 0.22 * radius2))).clamp(-0.98, 0.98);
                let sx = remap_axis_with_safe_margin(src_nx, region.width, 1);
                let sy = remap_axis_with_safe_margin(src_ny, region.height, 1);
                let sample = &snapshot[sy as usize * region.width as usize + sx as usize];

                let Some(existing) = buffer.get(region.x + dx, region.y + dy).cloned() else {
                    continue;
                };

                let scanline = if ((dy.wrapping_add((frame & 1) as u16)) & 1) == 0 {
                    1.0 - (0.04 + 0.11 * intensity)
                } else {
                    1.0
                };
                let vignette = 1.0
                    - smoothstep(((radius2 - 0.45) / 0.55).clamp(0.0, 1.0))
                        * (0.08 + 0.20 * intensity);
                let noise = (rand01(
                    region.x.wrapping_add(dx),
                    region.y.wrapping_add(dy),
                    frame.wrapping_add((speed * 97.0) as u32),
                ) - 0.5)
                    * (0.03 * noise_amount);
                let glow = (1.0 - ny.abs()).powf(2.2) * 0.02 * intensity;
                let mul = (brightness * flicker * scanline * vignette + noise).clamp(0.6, 1.2);

                let sampled_fg = scale_colour(sample.fg, mul + glow);
                let fg = lerp_colour(
                    existing.fg,
                    sampled_fg,
                    (0.24 + 0.38 * intensity).clamp(0.0, 0.65),
                );

                let existing_bg = if matches!(existing.bg, Color::Reset) {
                    TRUE_BLACK
                } else {
                    existing.bg
                };
                let sampled_bg_base = if matches!(sample.bg, Color::Reset) {
                    TRUE_BLACK
                } else {
                    sample.bg
                };
                let sampled_bg = scale_colour(sampled_bg_base, (mul * 0.72).clamp(0.5, 1.0));
                let bg = lerp_colour(
                    existing_bg,
                    sampled_bg,
                    (0.08 + 0.14 * intensity).clamp(0.0, 0.30),
                );

                // Keep original glyphs to avoid duplicate-font artifacts at low terminal resolutions.
                buffer.set(region.x + dx, region.y + dy, existing.symbol, fg, bg);
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

fn remap_axis_with_safe_margin(value: f32, extent: u16, margin: u16) -> u16 {
    if extent <= 1 {
        return 0;
    }
    let max_index = extent - 1;
    let lo = margin.min(max_index);
    let hi = max_index.saturating_sub(margin);
    let idx = remap_axis(value, extent);
    idx.clamp(lo, hi.max(lo))
}

fn rand01(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

fn scale_colour(base: Color, mul: f32) -> Color {
    let (r, g, b) = colour_to_rgb(base);
    let mul = mul.clamp(0.0, 2.0);
    Color::Rgb {
        r: ((r as f32 * mul).round()).clamp(0.0, 255.0) as u8,
        g: ((g as f32 * mul).round()).clamp(0.0, 255.0) as u8,
        b: ((b as f32 * mul).round()).clamp(0.0, 255.0) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::{TerminalCrtEffect, METADATA};
    use engine_core::buffer::Buffer;
    use engine_core::color::Color;
    use engine_core::effects::{Effect, EffectTargetMask, Region};
    use engine_core::scene::EffectParams;

    #[test]
    fn metadata_supports_scene_and_layer_only() {
        assert!(METADATA
            .compatible_targets
            .supports(engine_core::scene::EffectTargetKind::Scene));
        assert!(METADATA
            .compatible_targets
            .supports(engine_core::scene::EffectTargetKind::Layer));
        assert_eq!(
            METADATA.compatible_targets,
            EffectTargetMask::SCENE.union(EffectTargetMask::LAYER)
        );
    }

    #[test]
    fn intensity_zero_is_noop() {
        let mut buffer = Buffer::new(5, 2);
        buffer.fill(Color::Black);
        buffer.set(1, 0, 'X', Color::White, Color::Black);
        let before = buffer.clone();

        TerminalCrtEffect.apply(
            0.5,
            &EffectParams {
                intensity: Some(0.0),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 5,
                height: 2,
            },
            &mut buffer,
        );

        for y in 0..2 {
            for x in 0..5 {
                assert_eq!(buffer.get(x, y), before.get(x, y));
            }
        }
    }

    #[test]
    fn modifies_cells_inside_region() {
        let mut buffer = Buffer::new(6, 3);
        buffer.fill(Color::Black);
        for y in 0..3 {
            for x in 0..6 {
                buffer.set(
                    x,
                    y,
                    'X',
                    Color::Rgb {
                        r: 180,
                        g: 180,
                        b: 180,
                    },
                    Color::Black,
                );
            }
        }
        let before_outside = buffer
            .get(0, 0)
            .cloned()
            .expect("outside cell should exist");

        TerminalCrtEffect.apply(
            0.75,
            &EffectParams::default(),
            Region {
                x: 1,
                y: 1,
                width: 4,
                height: 2,
            },
            &mut buffer,
        );

        assert_eq!(
            buffer.get(0, 0).expect("outside cell should exist"),
            &before_outside
        );
        assert_ne!(
            buffer.get(2, 1).expect("inside cell should exist").fg,
            Color::Rgb {
                r: 180,
                g: 180,
                b: 180
            }
        );
    }
}
