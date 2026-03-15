use crossterm::style::Color;
use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::color::colour_to_rgb;
use crate::effects::utils::math::smoothstep;
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;

#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Sprite-scoped shatter glitch:
/// - partial row dropouts
/// - flickering/corrupted glyphs
/// - displaced fragment copies ("memory shards")
/// Never wipes the whole sprite in a frame.
pub struct ShatterGlitchEffect;

impl Effect for ShatterGlitchEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let t = (progress * 1000.0) as u32;
        let p = smoothstep(progress.clamp(0.0, 1.0));

        // Snapshot all visible sprite cells.
        let mut signal_cells: Vec<(u16, u16, Cell)> = Vec::new();
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y).cloned() {
                    if has_signal(&cell) {
                        signal_cells.push((x, y, cell));
                    }
                }
            }
        }
        if signal_cells.is_empty() {
            return;
        }

        let row_drop_prob = 0.05 + 0.30 * p;
        let clear_prob = 0.08 + 0.42 * p;
        let flicker_prob = 0.10 + 0.32 * p;
        let shard_prob = 0.06 + 0.28 * p;

        let mut kept_visible = 0usize;
        for (x, y, src) in &signal_cells {
            let n = noise(*x, *y, t / 10);
            let row_n = noise(region.x, *y, t / 14);
            let row_drop = row_n < row_drop_prob && ((*y as u32 + t / 8) % 3 != 0);

            if row_drop || n < clear_prob {
                buffer.set(*x, *y, ' ', TRUE_BLACK, TRUE_BLACK);
            } else if n < clear_prob + flicker_prob {
                let blink_on = ((t / 40 + *x as u32 + *y as u32) & 1) == 0;
                if blink_on {
                    let ch = glitch_char(n);
                    let fg = channel_split_tint(src.fg, n);
                    buffer.set(*x, *y, ch, fg, TRUE_BLACK);
                    kept_visible += 1;
                } else {
                    buffer.set(*x, *y, ' ', TRUE_BLACK, TRUE_BLACK);
                }
            } else {
                let fg = dim(src.fg, 1.0 - 0.30 * p);
                let ch = if n > 0.94 { glitch_char(n) } else { src.symbol };
                buffer.set(*x, *y, ch, fg, TRUE_BLACK);
                kept_visible += 1;
            }

            // "Memory shard": displaced copy of sprite fragments.
            if n > 0.35 && n < 0.35 + shard_prob {
                let ox = ((noise(x.wrapping_add(11), *y, t / 6) * 9.0).floor() as i32) - 4;
                let oy = ((noise(*x, y.wrapping_add(17), t / 6) * 5.0).floor() as i32) - 2;
                let sx = (*x as i32 + ox).max(0) as u16;
                let sy = (*y as i32 + oy).max(0) as u16;
                let shard_ch = if src.symbol == ' ' { glitch_char(n) } else { src.symbol };
                let shard_fg = dim(channel_split_tint(src.fg, 1.0 - n), 0.75);
                buffer.set(sx, sy, shard_ch, shard_fg, TRUE_BLACK);
            }
        }

        // Safety: keep at least one visible source pixel/glyph.
        if kept_visible == 0 {
            let idx = (t as usize) % signal_cells.len();
            let (x, y, src) = &signal_cells[idx];
            buffer.set(*x, *y, src.symbol, src.fg, TRUE_BLACK);
        }
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

const GLITCH_CHARS: &[char] = &['#', '%', '@', '&', '/', '\\', '|', '{', '}', '?', '!', 'A', 'R', '0', '8'];

fn glitch_char(n: f32) -> char {
    GLITCH_CHARS[(n * GLITCH_CHARS.len() as f32) as usize % GLITCH_CHARS.len()]
}

fn channel_split_tint(base: Color, n: f32) -> Color {
    let (r, g, b) = colour_to_rgb(base);
    if n < 0.33 {
        Color::Rgb {
            r: r.saturating_add(65),
            g: g.saturating_sub(25),
            b: b.saturating_sub(25),
        }
    } else if n < 0.66 {
        Color::Rgb {
            r: r.saturating_sub(30),
            g: g.saturating_add(38),
            b: b.saturating_add(48),
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
