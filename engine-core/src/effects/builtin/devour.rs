//! Effect that corrupts and consumes cell content, spreading a decay across the frame.

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::color::Color;
use crate::effects::effect::{Effect, Region};
use crate::effects::metadata::{EffectMetadata, P_EASING, P_INTENSITY};
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::math::smoothstep;
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;

/// Static effect metadata exposed to the editor and effect registry.
pub static METADATA: EffectMetadata = EffectMetadata {
    name: "devour-out",
    display_name: "Devour Out",
    summary: "Pixel dropout and corruption spreading across the frame.",
    category: "distortion",
    compatible_targets: crate::effects::effect::EffectTargetMask::ANY,
    params: &[P_INTENSITY, P_EASING],
    sample: "- name: devour-out\n  duration: 700\n  params:\n    intensity: 1.0",
};

const PHASE_CONSUME_END: f32 = 0.70;

#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Effect that corrupts pixels with dropout and spreading infection, fading the frame to black.
pub struct DevourOutEffect;

impl Effect for DevourOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let t = (progress * 1000.0) as u32;

        if progress < PHASE_CONSUME_END {
            // Phase 1: "eat" current scene children in mixed ways:
            // per-row dropouts, random dead pixels, and local corruption fragments.
            let phase_t = (progress / PHASE_CONSUME_END).clamp(0.0, 1.0);
            let eat = smoothstep(phase_t);
            let row_drop_prob = 0.03 + 0.47 * eat;
            let pixel_drop_prob = 0.05 + 0.55 * eat;
            let corrupt_prob = 0.04 + 0.30 * eat;

            for dy in 0..region.height {
                let y = region.y + dy;
                let row_noise = noise(region.x, y, t / 24);
                let row_dropped = row_noise < row_drop_prob;

                for dx in 0..region.width {
                    let x = region.x + dx;
                    let n = noise(x, y, t / 10);
                    let cell = match buffer.get(x, y) {
                        Some(c) => c.clone(),
                        None => continue,
                    };
                    if !has_signal(&cell) {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        continue;
                    }

                    if row_dropped || n < pixel_drop_prob {
                        if n < 0.08 * (1.0 - phase_t) {
                            buffer.set(x, y, residue_char(n), residue_colour(n), TRUE_BLACK);
                        } else {
                            buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        }
                        continue;
                    }

                    if n < pixel_drop_prob + corrupt_prob {
                        let ch = if cell.symbol == ' ' {
                            residue_char(n)
                        } else {
                            cell.symbol
                        };
                        let fg = lerp_colour(cell.fg, residue_colour(n), 0.25 + 0.60 * eat);
                        buffer.set(x, y, ch, fg, TRUE_BLACK);
                    } else {
                        let fg = dim(cell.fg, 1.0 - 0.45 * eat);
                        buffer.set(x, y, cell.symbol, fg, TRUE_BLACK);
                    }
                }
            }
            return;
        }

        // Phase 2: spread infection over whole screen background, then collapse to black.
        let phase_t = ((progress - PHASE_CONSUME_END) / (1.0 - PHASE_CONSUME_END)).clamp(0.0, 1.0);
        let spread = smoothstep((phase_t / 0.78).clamp(0.0, 1.0));
        let collapse = smoothstep(((phase_t - 0.78) / 0.22).clamp(0.0, 1.0));

        for dy in 0..region.height {
            let y = region.y + dy;
            for dx in 0..region.width {
                let x = region.x + dx;
                let n = noise(x, y, t / 6);

                let edge = min_edge_distance(x, y, region);
                let edge_norm = edge / ((region.width.min(region.height) as f32 / 2.0).max(1.0));
                let edge_wave = 1.0 - edge_norm.clamp(0.0, 1.0);
                let infect_threshold = (spread * 0.72 + edge_wave * 0.50).clamp(0.0, 1.0);
                let infected = n < infect_threshold;

                if infected {
                    let spread_col = lerp_colour(
                        Color::Rgb {
                            r: 25,
                            g: 65,
                            b: 38,
                        },
                        Color::Rgb {
                            r: 150,
                            g: 215,
                            b: 165,
                        },
                        n,
                    );
                    let fg = lerp_colour(spread_col, TRUE_BLACK, collapse);
                    let ch = if collapse > 0.85 || n > 0.84 {
                        ' '
                    } else {
                        spread_char(n)
                    };
                    buffer.set(x, y, ch, fg, TRUE_BLACK);
                } else if collapse > 0.92 {
                    buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

fn has_signal(cell: &crate::buffer::Cell) -> bool {
    if cell.symbol != ' ' {
        return true;
    }
    let (fr, fg, fb) = colour_to_rgb(cell.fg);
    let (br, bg, bb) = colour_to_rgb(cell.bg);
    fr > 0 || fg > 0 || fb > 0 || br > 0 || bg > 0 || bb > 0
}

fn min_edge_distance(x: u16, y: u16, region: Region) -> f32 {
    let left = x.saturating_sub(region.x) as f32;
    let top = y.saturating_sub(region.y) as f32;
    let right = (region.x + region.width - 1).saturating_sub(x) as f32;
    let bottom = (region.y + region.height - 1).saturating_sub(y) as f32;
    left.min(top).min(right).min(bottom)
}

const RESIDUE_CHARS: &[char] = &['#', '%', '@', '/', '\\', '|', ':', ';', '.'];
const SPREAD_CHARS: &[char] = &['.', ':', ';', '+', '*', '#', '%', '@'];

fn residue_char(n: f32) -> char {
    RESIDUE_CHARS[(n * RESIDUE_CHARS.len() as f32) as usize % RESIDUE_CHARS.len()]
}

fn spread_char(n: f32) -> char {
    SPREAD_CHARS[(n * SPREAD_CHARS.len() as f32) as usize % SPREAD_CHARS.len()]
}

fn residue_colour(n: f32) -> Color {
    let c = (90.0 + n * 130.0).round() as u8;
    Color::Rgb {
        r: c / 4,
        g: c,
        b: c / 3,
    }
}

fn dim(c: Color, factor: f32) -> Color {
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * factor) as u8,
        g: (g as f32 * factor) as u8,
        b: (b as f32 * factor) as u8,
    }
}
