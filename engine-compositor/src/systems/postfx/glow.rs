//! Shared half-resolution glow/bloom pipeline used by both the standalone
//! Underlay pass and the unified CRT composite pass.

use super::{colour_luma, normalize_bg, rand01};
use engine_core::buffer::{Buffer, Cell};
use engine_core::color::Color;
use engine_effects::utils::color::colour_to_rgb;
use rayon::prelude::*;
use std::cell::RefCell;

/// Minimum buffer size to use parallel processing.
/// Below this, serial is faster due to rayon thread spawn overhead.
const PARALLEL_PIXEL_THRESHOLD: usize = 4096; // ~64x64 buffer

// ── Glow pixel ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default)]
pub(super) struct GlowPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl GlowPixel {
    pub fn add_scaled(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.r += r * a;
        self.g += g * a;
        self.b += b * a;
        self.a += a;
    }

    pub fn normalized(self) -> Self {
        if self.a <= 0.0001 {
            return Self::default();
        }
        Self {
            r: (self.r / self.a).clamp(0.0, 1.0),
            g: (self.g / self.a).clamp(0.0, 1.0),
            b: (self.b / self.a).clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }
}

// ── Reusable scratch buffers ──────────────────────────────────────────────

pub(super) struct GlowScratch {
    pub a: Vec<GlowPixel>,
    pub b: Vec<GlowPixel>,
    pub out: Vec<GlowPixel>,
}

thread_local! {
    pub(super) static GLOW_SCRATCH: RefCell<GlowScratch> = const { RefCell::new(GlowScratch {
        a: Vec::new(),
        b: Vec::new(),
        out: Vec::new(),
    }) };
}

// ── Public builder ────────────────────────────────────────────────────────

/// Builds the glow map in-place using pre-allocated ping-pong scratch buffers.
///
/// **Full-resolution blur pipeline**: seed and blur run at native buffer
/// resolution for tight, pixel-accurate phosphor glow (1–2 cell spread).
/// Fewer blur passes keep the glow realistic — CRT phosphor bleeds into
/// immediate neighbours only, not across half the screen.
pub(super) fn build_glow_map_inplace(
    src: &Buffer,
    intensity: f32,
    spread: f32,
    frame: u32,
    a: &mut Vec<GlowPixel>,
    b: &mut Vec<GlowPixel>,
    out: &mut Vec<GlowPixel>,
) {
    let width = src.width as usize;
    let height = src.height as usize;
    let n = width * height;
    if n == 0 {
        out.clear();
        return;
    }

    if a.len() < n {
        a.resize(n, GlowPixel::default());
    }
    if b.len() < n {
        b.resize(n, GlowPixel::default());
    }
    if out.len() < n {
        out.resize(n, GlowPixel::default());
    }

    // ── 1. Build seed at full resolution ──────────────────────────────────
    for p in &mut a[..n] {
        *p = GlowPixel::default();
    }

    for y in 0..src.height {
        for x in 0..src.width {
            let Some(cell) = src.get(x, y) else {
                continue;
            };
            let Some(source_colour) = glow_source_colour(cell) else {
                continue;
            };
            let (sr, sg, sb) = colour_to_rgb(source_colour);
            let r = sr as f32 / 255.0;
            let g = sg as f32 / 255.0;
            let bch = sb as f32 / 255.0;
            let luma = (0.299 * r + 0.587 * g + 0.114 * bch).clamp(0.0, 1.0);
            let mut base =
                (0.22 + 0.78 * luma) * (0.26 + 0.72 * intensity) * (0.42 + 0.55 * spread);
            if rand01(x, y, frame.wrapping_add(911)) > 0.86 {
                let sparkle = rand01(x, y, frame.wrapping_add(1337));
                base *= 1.0 + 0.45 * sparkle;
            }
            if base <= 0.0 {
                continue;
            }
            let idx = y as usize * width + x as usize;
            a[idx].add_scaled(r, g, bch, base.clamp(0.0, 1.0));
        }
    }

    // ── 2. Blur at full resolution (tight kernel, few passes) ─────────────
    let blur_passes = 1 + (spread * 2.5).round() as usize; // 1–3 passes
    let mut src_is_a = true;
    for _ in 0..blur_passes {
        if src_is_a {
            blur_glow3x3_into(&a[..n], &mut b[..n], width, height);
        } else {
            blur_glow3x3_into(&b[..n], &mut a[..n], width, height);
        }
        src_is_a = !src_is_a;
    }

    // ── 3. One extra blur for halo, then combine into out ─────────────────
    if src_is_a {
        blur_glow3x3_into(&a[..n], &mut b[..n], width, height);
        combine_core_halo(&a[..n], &b[..n], n, frame, out);
    } else {
        blur_glow3x3_into(&b[..n], &mut a[..n], width, height);
        combine_core_halo(&b[..n], &a[..n], n, frame, out);
    }
}

// ── Private helpers ───────────────────────────────────────────────────────

#[inline(always)]
fn glow_source_colour(cell: &Cell) -> Option<Color> {
    if cell.symbol != ' ' {
        return Some(cell.fg);
    }
    let bg = normalize_bg(cell.bg);
    if colour_luma(bg) > 0.02 {
        Some(bg)
    } else {
        None
    }
}

/// Combines core + halo glow at full resolution into `out`.
/// Uses rayon for large buffers.
fn combine_core_halo(
    core: &[GlowPixel],
    halo: &[GlowPixel],
    n: usize,
    frame: u32,
    out: &mut [GlowPixel],
) {
    if n > PARALLEL_PIXEL_THRESHOLD {
        // Parallel path: process chunks of pixels
        const CHUNK_SIZE: usize = 256;
        out[..n]
            .par_chunks_mut(CHUNK_SIZE)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let base_i = chunk_idx * CHUNK_SIZE;
                for (offset, out_pix) in chunk.iter_mut().enumerate() {
                    let i = base_i + offset;
                    if i >= n {
                        break;
                    }
                    let c = core[i];
                    let h = halo[i];
                    let mut mix = GlowPixel {
                        r: c.r * 0.70 + h.r * 0.30,
                        g: c.g * 0.70 + h.g * 0.30,
                        b: c.b * 0.70 + h.b * 0.30,
                        a: c.a * 0.72 + h.a * 0.28,
                    }
                    .normalized();
                    // Only apply shimmer if alpha is significant.
                    if mix.a > 0.01 {
                        let shimmer = 0.92
                            + 0.16 * rand01(i as u16, (i >> 8) as u16, frame.wrapping_add(1703));
                        mix.a = (mix.a * shimmer).clamp(0.0, 1.0);
                    }
                    *out_pix = mix;
                }
            });
    } else {
        // Serial path for small buffers
        for i in 0..n {
            let c = core[i];
            let h = halo[i];
            let mut mix = GlowPixel {
                r: c.r * 0.70 + h.r * 0.30,
                g: c.g * 0.70 + h.g * 0.30,
                b: c.b * 0.70 + h.b * 0.30,
                a: c.a * 0.72 + h.a * 0.28,
            }
            .normalized();
            // Only apply shimmer if alpha is significant (avoid wasted rand for transparent pixels).
            if mix.a > 0.01 {
                let shimmer =
                    0.92 + 0.16 * rand01(i as u16, (i >> 8) as u16, frame.wrapping_add(1703));
                mix.a = (mix.a * shimmer).clamp(0.0, 1.0);
            }
            out[i] = mix;
        }
    }
}

/// In-place 3×3 blur with tight, center-heavy kernel.
/// Realistic CRT phosphor bleeds ~1 cell, not across the screen.
/// Unrolled to avoid per-neighbor match branch; split interior vs border for bounds-check elimination.
/// Uses rayon for large buffers.
fn blur_glow3x3_into(src: &[GlowPixel], dst: &mut [GlowPixel], width: usize, height: usize) {
    if width == 0 || height == 0 {
        return;
    }

    let n = width * height;

    // Interior pixels (not on edge) — no bounds checks needed.
    // Process in parallel for large buffers using row-based chunks.
    if n > PARALLEL_PIXEL_THRESHOLD && height > 2 {
        // Process interior rows in parallel (rows 1..height-1)
        let interior_height = height.saturating_sub(2);
        if interior_height > 0 {
            dst[width..n - width]
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(row_offset, dst_row)| {
                    let y = row_offset + 1; // actual row index
                    for x in 1..width.saturating_sub(1) {
                        let idx = y * width + x;
                        let c = src[idx]; // center 0.34
                        let u = src[(y - 1) * width + x]; // up 0.11
                        let d = src[(y + 1) * width + x]; // down 0.11
                        let l = src[y * width + (x - 1)]; // left 0.11
                        let r = src[y * width + (x + 1)]; // right 0.11
                        let ul = src[(y - 1) * width + (x - 1)]; // up-left 0.06
                        let ur = src[(y - 1) * width + (x + 1)]; // up-right 0.06
                        let dl = src[(y + 1) * width + (x - 1)]; // down-left 0.06
                        let dr = src[(y + 1) * width + (x + 1)]; // down-right 0.06

                        dst_row[x] = GlowPixel {
                            r: c.r * 0.34
                                + u.r * 0.11
                                + d.r * 0.11
                                + l.r * 0.11
                                + r.r * 0.11
                                + ul.r * 0.06
                                + ur.r * 0.06
                                + dl.r * 0.06
                                + dr.r * 0.06,
                            g: c.g * 0.34
                                + u.g * 0.11
                                + d.g * 0.11
                                + l.g * 0.11
                                + r.g * 0.11
                                + ul.g * 0.06
                                + ur.g * 0.06
                                + dl.g * 0.06
                                + dr.g * 0.06,
                            b: c.b * 0.34
                                + u.b * 0.11
                                + d.b * 0.11
                                + l.b * 0.11
                                + r.b * 0.11
                                + ul.b * 0.06
                                + ur.b * 0.06
                                + dl.b * 0.06
                                + dr.b * 0.06,
                            a: c.a * 0.34
                                + u.a * 0.11
                                + d.a * 0.11
                                + l.a * 0.11
                                + r.a * 0.11
                                + ul.a * 0.06
                                + ur.a * 0.06
                                + dl.a * 0.06
                                + dr.a * 0.06,
                        };
                    }
                });
        }
    } else {
        // Serial path for small buffers
        for y in 1..height.saturating_sub(1) {
            for x in 1..width.saturating_sub(1) {
                let idx = y * width + x;
                let c = src[idx]; // center 0.34
                let u = src[(y - 1) * width + x]; // up 0.11
                let d = src[(y + 1) * width + x]; // down 0.11
                let l = src[y * width + (x - 1)]; // left 0.11
                let r = src[y * width + (x + 1)]; // right 0.11
                let ul = src[(y - 1) * width + (x - 1)]; // up-left 0.06
                let ur = src[(y - 1) * width + (x + 1)]; // up-right 0.06
                let dl = src[(y + 1) * width + (x - 1)]; // down-left 0.06
                let dr = src[(y + 1) * width + (x + 1)]; // down-right 0.06

                dst[idx] = GlowPixel {
                    r: c.r * 0.34
                        + u.r * 0.11
                        + d.r * 0.11
                        + l.r * 0.11
                        + r.r * 0.11
                        + ul.r * 0.06
                        + ur.r * 0.06
                        + dl.r * 0.06
                        + dr.r * 0.06,
                    g: c.g * 0.34
                        + u.g * 0.11
                        + d.g * 0.11
                        + l.g * 0.11
                        + r.g * 0.11
                        + ul.g * 0.06
                        + ur.g * 0.06
                        + dl.g * 0.06
                        + dr.g * 0.06,
                    b: c.b * 0.34
                        + u.b * 0.11
                        + d.b * 0.11
                        + l.b * 0.11
                        + r.b * 0.11
                        + ul.b * 0.06
                        + ur.b * 0.06
                        + dl.b * 0.06
                        + dr.b * 0.06,
                    a: c.a * 0.34
                        + u.a * 0.11
                        + d.a * 0.11
                        + l.a * 0.11
                        + r.a * 0.11
                        + ul.a * 0.06
                        + ur.a * 0.06
                        + dl.a * 0.06
                        + dr.a * 0.06,
                };
            }
        }
    }

    // Border pixels — use fallback with bounds checks (rare, don't optimize).
    if height >= 2 {
        // Top and bottom rows
        for x in 0..width {
            for y in [0, height - 1] {
                let mut acc = GlowPixel::default();
                let mut wsum = 0.0_f32;
                for oy in -1_i32..=1 {
                    for ox in -1_i32..=1 {
                        let nx = x as i32 + ox;
                        let ny = y as i32 + oy;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let weight = match (ox.abs(), oy.abs()) {
                            (0, 0) => 0.34,
                            (0, 1) | (1, 0) => 0.11,
                            _ => 0.06,
                        };
                        let p = src[ny as usize * width + nx as usize];
                        acc.r += p.r * weight;
                        acc.g += p.g * weight;
                        acc.b += p.b * weight;
                        acc.a += p.a * weight;
                        wsum += weight;
                    }
                }
                dst[y * width + x] = if wsum > 0.0 {
                    GlowPixel {
                        r: acc.r / wsum,
                        g: acc.g / wsum,
                        b: acc.b / wsum,
                        a: acc.a / wsum,
                    }
                } else {
                    GlowPixel::default()
                };
            }
        }
    }

    if width >= 2 {
        // Left and right columns (interior rows only, to avoid double-processing corners)
        for y in 1..height.saturating_sub(1) {
            for x in [0, width - 1] {
                let mut acc = GlowPixel::default();
                let mut wsum = 0.0_f32;
                for oy in -1_i32..=1 {
                    for ox in -1_i32..=1 {
                        let nx = x as i32 + ox;
                        let ny = y as i32 + oy;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let weight = match (ox.abs(), oy.abs()) {
                            (0, 0) => 0.34,
                            (0, 1) | (1, 0) => 0.11,
                            _ => 0.06,
                        };
                        let p = src[ny as usize * width + nx as usize];
                        acc.r += p.r * weight;
                        acc.g += p.g * weight;
                        acc.b += p.b * weight;
                        acc.a += p.a * weight;
                        wsum += weight;
                    }
                }
                dst[y * width + x] = if wsum > 0.0 {
                    GlowPixel {
                        r: acc.r / wsum,
                        g: acc.g / wsum,
                        b: acc.b / wsum,
                        a: acc.a / wsum,
                    }
                } else {
                    GlowPixel::default()
                };
            }
        }
    }
}
