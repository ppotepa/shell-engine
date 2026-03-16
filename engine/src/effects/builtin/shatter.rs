use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::color::colour_to_rgb;
use crate::effects::utils::math::smoothstep;
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use crossterm::style::Color;

#[inline]
fn noise(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

/// Sprite-scoped shatter glitch:
/// - partial row dropouts
/// - flickering/corrupted glyphs
/// - displaced large fragment copies ("memory shard blocks")
/// Never wipes the whole sprite in a frame.
pub struct ShatterGlitchEffect;

impl Effect for ShatterGlitchEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let t = (progress * 1000.0) as u32;
        let p = smoothstep(progress.clamp(0.0, 1.0));
        let stable_seed = (region.x as u32).wrapping_mul(73_856_093)
            ^ (region.y as u32).wrapping_mul(19_349_663)
            ^ (region.width as u32).wrapping_mul(83_492_791)
            ^ (region.height as u32).wrapping_mul(2_654_435_761);

        // Snapshot the full region before corruption so shards are recognizable replicas.
        let mut snapshot: Vec<Cell> =
            Vec::with_capacity((region.width as usize) * (region.height as usize));
        let mut signal_cells: Vec<(u16, u16, Cell)> = Vec::new();
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                let cell = buffer.get(x, y).cloned().unwrap_or_default();
                if has_signal(&cell) {
                    signal_cells.push((x, y, cell.clone()));
                }
                snapshot.push(cell);
            }
        }
        if signal_cells.is_empty() {
            return;
        }

        let row_drop_prob = 0.03 + 0.24 * p;
        let clear_prob = 0.05 + 0.22 * p;
        let flicker_prob = 0.08 + 0.26 * p;
        let block_shift_prob = 0.06 + 0.20 * p;
        let square_prob = 0.05 + 0.22 * p;

        let mut kept_visible = 0usize;
        for (x, y, src) in &signal_cells {
            let n = noise(*x, *y, t / 10);
            let row_n = noise(region.x, *y, t / 14);
            let row_drop = row_n < row_drop_prob && ((*y as u32 + t / 7) % 4 != 0);

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
                let ch = if n < square_prob {
                    square_char(n)
                } else if n > 0.94 {
                    glitch_char(n)
                } else {
                    src.symbol
                };
                buffer.set(*x, *y, ch, fg, TRUE_BLACK);
                kept_visible += 1;
            }

            // Local block shift keeps "broken VRAM" flavor while still recognizable.
            if n > 0.40 && n < 0.40 + block_shift_prob {
                let ox = ((noise(x.wrapping_add(11), *y, t / 6) * 7.0).floor() as i32) - 3;
                let oy = ((noise(*x, y.wrapping_add(17), t / 6) * 3.0).floor() as i32) - 1;
                let sx = (*x as i32 + ox).max(0) as u16;
                let sy = (*y as i32 + oy).max(0) as u16;
                let shard_ch = if src.symbol == ' ' {
                    glitch_char(n)
                } else {
                    src.symbol
                };
                let shard_fg = dim(channel_split_tint(src.fg, 1.0 - n), 0.75);
                buffer.set(sx, sy, shard_ch, shard_fg, TRUE_BLACK);
            }
        }

        // Large persistent shard blocks: fixed destinations => "stuck VRAM fragments".
        let persistent_chunk_count = (1.0 + p * 5.0).round() as usize;
        let max_src_x = region.width.saturating_sub(1);
        let max_src_y = region.height.saturating_sub(1);
        for i in 0..persistent_chunk_count {
            let seed = stable_seed.wrapping_add(i as u32 * 97);
            let src_w = choose_span(
                region.width,
                noise(seed as u16, seed.rotate_left(7) as u16, seed),
            );
            let src_h = choose_span(
                region.height,
                noise(
                    seed.rotate_left(3) as u16,
                    seed.rotate_left(11) as u16,
                    seed,
                ),
            );

            let sx0 = choose_offset(
                max_src_x.saturating_add(1),
                src_w,
                noise(
                    seed.rotate_left(5) as u16,
                    seed.rotate_left(13) as u16,
                    seed,
                ),
            );
            let sy0 = choose_offset(
                max_src_y.saturating_add(1),
                src_h,
                noise(
                    seed.rotate_left(9) as u16,
                    seed.rotate_left(17) as u16,
                    seed,
                ),
            );

            let dx0 = (noise(
                seed.rotate_left(19) as u16,
                seed.rotate_left(23) as u16,
                seed,
            ) * buffer.width as f32)
                .floor() as i32
                - (src_w as i32 / 2);
            let dy0 = (noise(
                seed.rotate_left(2) as u16,
                seed.rotate_left(29) as u16,
                seed,
            ) * buffer.height as f32)
                .floor() as i32
                - (src_h as i32 / 2);

            blit_chunk(
                &snapshot,
                region,
                sx0,
                sy0,
                src_w,
                src_h,
                dx0,
                dy0,
                p,
                seed,
                buffer,
                None,
                0.12 + 0.30 * p,
            );
        }

        // Dynamic shard blocks: blinking replicas layered over persistent ones.
        let dynamic_chunk_count = (1.0 + p * 3.0).round() as usize;
        let max_src_x = region.width.saturating_sub(1);
        let max_src_y = region.height.saturating_sub(1);
        for i in 0..dynamic_chunk_count {
            let seed = t / 4 + (i as u32 * 37);
            let src_w = choose_span(
                region.width,
                noise(region.x.wrapping_add(3), region.y.wrapping_add(5), seed),
            );
            let src_h = choose_span(
                region.height,
                noise(region.x.wrapping_add(7), region.y.wrapping_add(11), seed),
            );

            let sx0 = choose_offset(
                max_src_x.saturating_add(1),
                src_w,
                noise(region.x.wrapping_add(13), region.y.wrapping_add(17), seed),
            );
            let sy0 = choose_offset(
                max_src_y.saturating_add(1),
                src_h,
                noise(region.x.wrapping_add(19), region.y.wrapping_add(23), seed),
            );

            let dx0 = (noise(region.x.wrapping_add(29), region.y.wrapping_add(31), seed)
                * buffer.width as f32)
                .floor() as i32
                - (src_w as i32 / 2);
            let dy0 = (noise(region.x.wrapping_add(37), region.y.wrapping_add(41), seed)
                * buffer.height as f32)
                .floor() as i32
                - (src_h as i32 / 2);

            blit_chunk(
                &snapshot,
                region,
                sx0,
                sy0,
                src_w,
                src_h,
                dx0,
                dy0,
                p,
                seed,
                buffer,
                Some(t / 38 + i as u32),
                0.20 + 0.30 * p,
            );
        }

        // Full-width line dropouts across the entire screen (cinema-style digital breakdown).
        let line_count = (1.0 + p * 4.0).round() as usize;
        let line_strength = 0.15 + 0.55 * p;
        for i in 0..line_count {
            let seed = stable_seed.wrapping_add(10_000 + i as u32 * 131);
            let row = (noise(seed as u16, seed.rotate_left(7) as u16, seed) * buffer.height as f32)
                .floor() as u16;
            if row >= buffer.height {
                continue;
            }
            let gate = noise(
                seed.rotate_left(3) as u16,
                seed.rotate_left(9) as u16,
                t / 16 + seed,
            );
            let blink_on = ((t / 42 + i as u32) & 1) == 0;
            if gate < line_strength && (blink_on || p > 0.78) {
                for x in 0..buffer.width {
                    buffer.set(x, row, ' ', TRUE_BLACK, TRUE_BLACK);
                }
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

fn blit_chunk(
    snapshot: &[Cell],
    region: Region,
    sx0: u16,
    sy0: u16,
    w: u16,
    h: u16,
    dx0: i32,
    dy0: i32,
    progress: f32,
    seed: u32,
    buffer: &mut Buffer,
    blink_phase: Option<u32>,
    square_bias: f32,
) {
    for yy in 0..h {
        for xx in 0..w {
            let src_x = sx0 + xx;
            let src_y = sy0 + yy;
            let idx = src_y as usize * region.width as usize + src_x as usize;
            let cell = match snapshot.get(idx) {
                Some(c) => c,
                None => continue,
            };
            if !has_signal(cell) {
                continue;
            }

            let out_x = dx0 + xx as i32;
            let out_y = dy0 + yy as i32;
            if out_x < 0 || out_y < 0 {
                continue;
            }
            let out_x = out_x as u16;
            let out_y = out_y as u16;
            if out_x >= buffer.width || out_y >= buffer.height {
                continue;
            }

            if let Some(phase) = blink_phase {
                if ((phase + xx as u32 + yy as u32 * 2) & 1) != 0 {
                    continue;
                }
            }

            let n = noise(out_x, out_y, seed / 2 + xx as u32 + yy as u32 * 3);
            let symbol = if n < square_bias {
                square_char(n)
            } else if n > 0.97 {
                glitch_char(n)
            } else {
                cell.symbol
            };
            let fg = dim(channel_split_tint(cell.fg, n), 0.85 - 0.30 * progress);
            buffer.set(out_x, out_y, symbol, fg, TRUE_BLACK);
        }
    }
}

fn choose_span(total: u16, n: f32) -> u16 {
    if total <= 1 {
        return total;
    }
    let min = total.min((total / 4).max(2));
    let max = total.min((total / 2).max(min));
    let span = max.saturating_sub(min).saturating_add(1);
    min + ((n * span as f32).floor() as u16).min(span.saturating_sub(1))
}

fn choose_offset(total: u16, span: u16, n: f32) -> u16 {
    if total <= span {
        return 0;
    }
    let max = total - span;
    ((n * (max as f32 + 1.0)).floor() as u16).min(max)
}

fn has_signal(cell: &Cell) -> bool {
    if cell.symbol != ' ' {
        return true;
    }
    let (fr, fg, fb) = colour_to_rgb(cell.fg);
    let (br, bg, bb) = colour_to_rgb(cell.bg);
    fr > 0 || fg > 0 || fb > 0 || br > 0 || bg > 0 || bb > 0
}

const GLITCH_CHARS: &[char] = &[
    '#', '%', '@', '&', '/', '\\', '|', '{', '}', '?', '!', 'A', 'R', '0', '8',
];

fn glitch_char(n: f32) -> char {
    GLITCH_CHARS[(n * GLITCH_CHARS.len() as f32) as usize % GLITCH_CHARS.len()]
}

const SQUARE_CHARS: &[char] = &['■', '□', '▣', '▤', '▥', '▦', '▧', '▨', '▩', '▓'];

fn square_char(n: f32) -> char {
    SQUARE_CHARS[(n * SQUARE_CHARS.len() as f32) as usize % SQUARE_CHARS.len()]
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
