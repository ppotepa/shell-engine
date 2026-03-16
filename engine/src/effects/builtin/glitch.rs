use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::color::lerp_colour;
use crate::effects::utils::math::{smoothstep, TICK_MS};
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use crossterm::style::Color;

// Phase thresholds (progress 0→1)
const PHASE_DECAY_END: f32 = 0.55; // 0.00–0.55: signal decay (line/pixel failures)
const PHASE_SNOW_END: f32 = 0.90; // 0.55–0.90: digital snow + blink corruption
                                  // 0.90–1.00: dying flicker to black

/// Normalise a crt_hash u32 → 0.0..1.0
#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Hacker/GPU-failure style glitch-out transition effect.
///
/// Three phases:
/// 1. **Signal decay** (0–55%): dead pixels, horizontal tear offsets, line dropouts.
/// 2. **Digital snow** (55–90%): most signal collapses to noisy blink/snow artefacts.
/// 3. **Dying flicker** (90–100%): short phosphor flickers, then full black.
pub struct GlitchOutEffect;

impl Effect for GlitchOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let t = (progress * 1000.0) as u32;

        if progress < PHASE_DECAY_END {
            // Phase 1: decaying signal, line failures, and mild horizontal tearing.
            let phase_t = (progress / PHASE_DECAY_END).clamp(0.0, 1.0);
            let decay = smoothstep(phase_t);
            let dead_pixel_prob = 0.02 + 0.25 * decay;
            let corrupt_prob = 0.03 + 0.22 * decay;
            let line_drop_prob = 0.02 + 0.40 * decay;
            let tear_prob = 0.04 + 0.28 * decay;
            let row_blink_prob = 0.03 + 0.20 * decay;

            let max_dx = region.width.saturating_sub(1) as i32;
            for dy in 0..region.height {
                let y = region.y + dy;
                let row_n = noise(region.x, y, t / 40);
                if row_n < line_drop_prob {
                    for dx in 0..region.width {
                        let x = region.x + dx;
                        let n = noise(x, y, t / 12);
                        if n < 0.12 * (1.0 - phase_t) {
                            buffer.set(
                                x,
                                y,
                                glitch_char(n),
                                phosphor_green(0.4 + n * 0.6),
                                TRUE_BLACK,
                            );
                        } else {
                            buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        }
                    }
                } else {
                    let row_cells = (0..region.width)
                        .map(|dx| buffer.get(region.x + dx, y).cloned().unwrap_or_default())
                        .collect::<Vec<_>>();

                    let tear = noise(region.x.wrapping_add(17), y, t / 30) < tear_prob;
                    let tear_offset = if tear {
                        ((noise(region.x.wrapping_add(31), y, t / 24) * 7.0).floor() as i32) - 3
                    } else {
                        0
                    };
                    let row_blink = noise(region.x.wrapping_add(9), y, t / 10) < row_blink_prob;

                    for dx in 0..region.width {
                        let x = region.x + dx;
                        let src_dx = (dx as i32 - tear_offset).clamp(0, max_dx) as usize;
                        let src = &row_cells[src_dx];
                        let n = noise(x, y, t / TICK_MS as u32);

                        if n < dead_pixel_prob {
                            buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                            continue;
                        }

                        if row_blink && n < 0.40 {
                            buffer.set(
                                x,
                                y,
                                glitch_char(n),
                                phosphor_green(0.65 + n * 0.35),
                                TRUE_BLACK,
                            );
                            continue;
                        }

                        if n < dead_pixel_prob + corrupt_prob {
                            let ch = if src.symbol == ' ' {
                                glitch_char(n)
                            } else {
                                src.symbol
                            };
                            let fg = hacker_tint(src.fg, n, decay);
                            buffer.set(x, y, ch, fg, TRUE_BLACK);
                        } else {
                            let ch = if n > 0.985 {
                                glitch_char(n)
                            } else {
                                src.symbol
                            };
                            let fg = dim(src.fg, 1.0 - 0.35 * decay);
                            buffer.set(x, y, ch, fg, TRUE_BLACK);
                        }
                    }
                }
            }
        } else if progress < PHASE_SNOW_END {
            // Phase 2: heavy corruption with digital snow and unstable scanline blinking.
            let phase_t =
                ((progress - PHASE_DECAY_END) / (PHASE_SNOW_END - PHASE_DECAY_END)).clamp(0.0, 1.0);
            let melt = smoothstep(phase_t);
            let keep_prob = 0.25 * (1.0 - melt) + 0.02;
            let snow_prob = 0.08 + 0.22 * melt;
            let row_flash_prob = (0.08 + 0.40 * melt) * 0.18;
            for dy in 0..region.height {
                let y = region.y + dy;
                let row_flash = noise(region.x.wrapping_add(5), y, t / 6) < row_flash_prob;
                for dx in 0..region.width {
                    let x = region.x + dx;
                    let n = noise(x, y, t / 8);
                    let src = buffer.get(x, y).cloned().unwrap_or_default();

                    if row_flash && n < 0.65 {
                        let fg = if n < 0.5 {
                            phosphor_green(1.0)
                        } else {
                            Color::Rgb {
                                r: 220,
                                g: 255,
                                b: 230,
                            }
                        };
                        buffer.set(x, y, glitch_char(n), fg, TRUE_BLACK);
                        continue;
                    }

                    if n > keep_prob {
                        if n < keep_prob + snow_prob {
                            buffer.set(x, y, snow_char(n), snow_colour(n), TRUE_BLACK);
                        } else {
                            buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        }
                    } else {
                        let ch = if n < 0.35 { glitch_char(n) } else { src.symbol };
                        let base = dim(src.fg, 0.35);
                        let fg = lerp_colour(base, phosphor_green(0.8), 0.35 + 0.40 * melt);
                        buffer.set(x, y, ch, fg, TRUE_BLACK);
                    }
                }
            }
        } else {
            // Phase 3: short dying flicker and phosphor sparks before complete black.
            let phase_t = ((progress - PHASE_SNOW_END) / (1.0 - PHASE_SNOW_END)).clamp(0.0, 1.0);
            let blackout = smoothstep(phase_t);
            let flicker_prob = (1.0 - blackout) * 0.35;
            let spark_prob = (1.0 - blackout) * 0.25;

            for dy in 0..region.height {
                let y = region.y + dy;
                for dx in 0..region.width {
                    let x = region.x + dx;
                    let n = noise(x, y, t / 4);
                    if n < flicker_prob {
                        let fg = if n < flicker_prob * 0.35 {
                            Color::Rgb {
                                r: 230,
                                g: 255,
                                b: 235,
                            }
                        } else {
                            phosphor_green(1.0)
                        };
                        buffer.set(x, y, glitch_char(n), fg, TRUE_BLACK);
                    } else if n < flicker_prob + spark_prob {
                        buffer.set(x, y, snow_char(n), phosphor_green(0.45), TRUE_BLACK);
                    } else {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                    }
                }
            }
        }
    }
}

const GLITCH_CHARS: &[char] = &[
    '█', '▓', '▒', '░', '▄', '▀', '▌', '▐', '╳', '#', '%', '@', '&', '/', '\\', '|',
];
const SNOW_CHARS: &[char] = &['·', '.', ':', ';', ',', '\'', '`', '░'];

fn glitch_char(n: f32) -> char {
    GLITCH_CHARS[(n * GLITCH_CHARS.len() as f32) as usize % GLITCH_CHARS.len()]
}

fn snow_char(n: f32) -> char {
    SNOW_CHARS[(n * SNOW_CHARS.len() as f32) as usize % SNOW_CHARS.len()]
}

fn snow_colour(n: f32) -> Color {
    if n < 0.33 {
        phosphor_green(0.55 + n)
    } else if n < 0.66 {
        Color::Rgb {
            r: 120,
            g: 175,
            b: 160,
        }
    } else {
        Color::Rgb {
            r: 190,
            g: 215,
            b: 205,
        }
    }
}

fn phosphor_green(intensity: f32) -> Color {
    let i = intensity.clamp(0.0, 1.0);
    let r = (12.0 + 30.0 * i).round() as u8;
    let g = (110.0 + 145.0 * i).round() as u8;
    let b = (12.0 + 36.0 * i).round() as u8;
    Color::Rgb { r, g, b }
}

fn hacker_tint(base: Color, n: f32, decay: f32) -> Color {
    let green_mix = (0.20 + 0.50 * decay + 0.20 * n).clamp(0.0, 1.0);
    let mut out = lerp_colour(base, phosphor_green(1.0), green_mix);
    if n > 0.92 {
        out = lerp_colour(
            out,
            Color::Rgb {
                r: 230,
                g: 255,
                b: 235,
            },
            (n - 0.92) / 0.08,
        );
    }
    out
}

fn dim(c: Color, factor: f32) -> Color {
    use crate::effects::utils::color::colour_to_rgb;
    let (r, g, b) = colour_to_rgb(c);
    Color::Rgb {
        r: (r as f32 * factor) as u8,
        g: (g as f32 * factor) as u8,
        b: (b as f32 * factor) as u8,
    }
}
