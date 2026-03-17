use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::metadata::{EffectMetadata, P_EASING, P_INTENSITY};
use crate::effects::utils::color::colour_to_rgb;
use crate::effects::utils::math::smoothstep;
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use crossterm::style::Color;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "artifact-out",
    display_name: "Artifact Out",
    summary: "Digital compression artefacts fragmenting the image.",
    category: "distortion",
    compatible_targets: crate::effects::effect::EffectTargetMask::ANY,
    params: &[P_INTENSITY, P_EASING],
    sample: "- name: artifact-out\n  duration: 600\n  params:\n    intensity: 1.0",
};

const PHASE_ARTIFACT_END: f32 = 0.65;

#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Digital/GPU artifact transition:
/// - glyph corruption and block tearing (not static noise snow)
/// - color channel split tinting (R/C fringes)
/// - progressive character dropout to black
pub struct ArtifactOutEffect;

impl Effect for ArtifactOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let t = (progress * 1000.0) as u32;

        if progress < PHASE_ARTIFACT_END {
            let phase_t = (progress / PHASE_ARTIFACT_END).clamp(0.0, 1.0);
            let glitch = smoothstep(phase_t);

            let mut row_cache: Vec<Vec<Cell>> = Vec::with_capacity(region.height as usize);
            for dy in 0..region.height {
                let y = region.y + dy;
                let row = (0..region.width)
                    .map(|dx| buffer.get(region.x + dx, y).cloned().unwrap_or_default())
                    .collect::<Vec<_>>();
                row_cache.push(row);
            }

            let drop_prob = 0.02 + 0.58 * glitch;
            let corrupt_prob = 0.07 + 0.43 * glitch;
            let tear_prob = 0.05 + 0.35 * glitch;
            let block_prob = 0.03 + 0.30 * glitch;

            for dy in 0..region.height {
                let y = region.y + dy;
                let row = &row_cache[dy as usize];

                // Mild horizontal tear by row.
                let row_tear = noise(region.x.wrapping_add(13), y, t / 24) < tear_prob;
                let tear_offset = if row_tear {
                    ((noise(region.x.wrapping_add(29), y, t / 18) * 9.0).floor() as i32) - 4
                } else {
                    0
                };
                let max_dx = region.width.saturating_sub(1) as i32;

                for dx in 0..region.width {
                    let x = region.x + dx;
                    let src_idx = (dx as i32 - tear_offset).clamp(0, max_dx) as usize;
                    let mut src = row[src_idx].clone();
                    let n = noise(x, y, t / 12);

                    if !has_signal(&src) {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        continue;
                    }

                    if n < drop_prob {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        continue;
                    }

                    // Block copy corruption: take a nearby glyph/color cell in short windows.
                    if n < drop_prob + block_prob {
                        let jump = ((noise(x.wrapping_add(7), y, t / 8) * 7.0).floor() as i32) - 3;
                        let j_idx = (src_idx as i32 + jump).clamp(0, max_dx) as usize;
                        src = row[j_idx].clone();
                    }

                    if n < drop_prob + block_prob + corrupt_prob {
                        let symbol = artifact_char(n);
                        let fg = channel_split_tint(src.fg, n);
                        buffer.set(x, y, symbol, fg, TRUE_BLACK);
                    } else {
                        let fg = dim(src.fg, 1.0 - 0.45 * glitch);
                        buffer.set(x, y, src.symbol, fg, TRUE_BLACK);
                    }
                }
            }
            return;
        }

        // Final collapse: digital chunks vanish to black in bands.
        let phase_t =
            ((progress - PHASE_ARTIFACT_END) / (1.0 - PHASE_ARTIFACT_END)).clamp(0.0, 1.0);
        let collapse = smoothstep(phase_t);
        let band_prob = 0.10 + 0.80 * collapse;

        for dy in 0..region.height {
            let y = region.y + dy;
            let band_n = noise(region.x.wrapping_add(3), y, t / 6);
            for dx in 0..region.width {
                let x = region.x + dx;
                let n = noise(x, y, t / 5);
                let wipe = n < band_prob || band_n < 0.22 + 0.60 * collapse;
                if wipe {
                    buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                } else if let Some(cell) = buffer.get(x, y).cloned() {
                    let symbol = if n > 0.88 {
                        artifact_char(n)
                    } else {
                        cell.symbol
                    };
                    let fg = dim(channel_split_tint(cell.fg, n), 0.55 * (1.0 - collapse));
                    buffer.set(x, y, symbol, fg, TRUE_BLACK);
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

fn has_signal(cell: &Cell) -> bool {
    if cell.symbol != ' ' {
        return true;
    }
    let (fr, fg, fb) = colour_to_rgb(cell.fg);
    let (br, bg, bb) = colour_to_rgb(cell.bg);
    fr > 0 || fg > 0 || fb > 0 || br > 0 || bg > 0 || bb > 0
}

const ARTIFACT_CHARS: &[char] = &[
    'A', 'E', 'R', '8', '0', '#', '%', '@', '&', '/', '\\', '|', '}', '{', '?', '!',
];

fn artifact_char(n: f32) -> char {
    ARTIFACT_CHARS[(n * ARTIFACT_CHARS.len() as f32) as usize % ARTIFACT_CHARS.len()]
}

fn channel_split_tint(base: Color, n: f32) -> Color {
    let (r, g, b) = colour_to_rgb(base);
    if n < 0.33 {
        // red fringe
        Color::Rgb {
            r: r.saturating_add(70),
            g: g.saturating_sub(30),
            b: b.saturating_sub(30),
        }
    } else if n < 0.66 {
        // cyan fringe
        Color::Rgb {
            r: r.saturating_sub(35),
            g: g.saturating_add(45),
            b: b.saturating_add(55),
        }
    } else {
        Color::Rgb { r, g, b }
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
