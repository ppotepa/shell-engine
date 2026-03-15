use crossterm::style::Color;
use crate::buffer::{Buffer, TRUE_BLACK};
use crate::scene::EffectParams;
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::noise::crt_hash;
use crate::effects::utils::math::smoothstep;

// Phase thresholds (progress 0→1)
const PHASE_LINES_END:  f32 = 0.60; // 0.00–0.60: lines drop out
const PHASE_PIXELS_END: f32 = 0.85; // 0.60–0.85: pixel scatter
                                     // 0.85–1.00: white → black flash

/// Normalise a crt_hash u32 → 0.0..1.0
#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Terminal-style glitch-out transition effect.
///
/// Three phases:
/// 1. **Line dropout** (0–60%): rows are randomly zeroed out, density increases with progress.
/// 2. **Pixel scatter** (60–85%): remaining pixels replaced with dim noise chars.
/// 3. **Flash** (85–100%): white flash collapses to true black.
pub struct GlitchOutEffect;

impl Effect for GlitchOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 { return; }

        let t = (progress * 1000.0) as u32;

        if progress < PHASE_LINES_END {
            // Phase 1: rows drop out + cell corruption.
            let phase_t = progress / PHASE_LINES_END;
            let drop_prob = smoothstep(phase_t) * 0.8;
            let corrupt_prob = smoothstep(phase_t) * 0.15;

            for dy in 0..region.height {
                let y = region.y + dy;
                let row_n = noise(y as u16, 0, t / 60);
                if row_n < drop_prob {
                    for dx in 0..region.width {
                        buffer.set(region.x + dx, y, ' ', TRUE_BLACK, TRUE_BLACK);
                    }
                } else {
                    for dx in 0..region.width {
                        let x = region.x + dx;
                        let cn = noise(x as u16, y as u16, t / 30);
                        if cn < corrupt_prob {
                            if let Some(cell) = buffer.get(x, y) {
                                let ch = glitch_char(cn);
                                let fg = dim(cell.fg, 0.5 + cn * 0.5);
                                buffer.set(x, y, ch, fg, TRUE_BLACK);
                            }
                        }
                    }
                }
            }

        } else if progress < PHASE_PIXELS_END {
            // Phase 2: scatter — clear most cells, leave dim noise pixels.
            let phase_t = (progress - PHASE_LINES_END) / (PHASE_PIXELS_END - PHASE_LINES_END);
            let keep_density = (1.0 - smoothstep(phase_t)) * 0.3;

            for dy in 0..region.height {
                for dx in 0..region.width {
                    let x = region.x + dx;
                    let y = region.y + dy;
                    let n = noise(x as u16, y as u16, t / 20);
                    if n > keep_density {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                    } else {
                        let c = (n * 200.0) as u8;
                        buffer.set(x, y, glitch_char(n), Color::Rgb { r: c, g: c, b: c }, TRUE_BLACK);
                    }
                }
            }

        } else {
            // Phase 3: white flash → black.
            let phase_t = ((progress - PHASE_PIXELS_END) / (1.0 - PHASE_PIXELS_END)).clamp(0.0, 1.0);
            let white_amount = if phase_t < 0.4 { phase_t / 0.4 } else { 1.0 - (phase_t - 0.4) / 0.6 };
            let v = (white_amount * 255.0) as u8;
            let flash = Color::Rgb { r: v, g: v, b: v };
            for dy in 0..region.height {
                for dx in 0..region.width {
                    buffer.set(region.x + dx, region.y + dy, ' ', flash, flash);
                }
            }
        }
    }
}

const GLITCH_CHARS: &[char] = &['█', '▓', '▒', '░', '▄', '▀', '▌', '▐', '■', '□', '╬', '╪', '╫', '┼', '╳'];

fn glitch_char(n: f32) -> char {
    GLITCH_CHARS[(n * GLITCH_CHARS.len() as f32) as usize % GLITCH_CHARS.len()]
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
