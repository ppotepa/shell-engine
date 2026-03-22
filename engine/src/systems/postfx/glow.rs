//! Shared half-resolution glow/bloom pipeline used by both the standalone
//! Underlay pass and the unified CRT composite pass.

use super::{colour_luma, normalize_bg, rand01};
use crate::buffer::{Buffer, Cell};
use crate::effects::utils::color::colour_to_rgb;
use crossterm::style::Color;
use std::cell::RefCell;

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
    pub(super) static GLOW_SCRATCH: RefCell<GlowScratch> = RefCell::new(GlowScratch {
        a: Vec::new(),
        b: Vec::new(),
        out: Vec::new(),
    });
}

// ── Public builder ────────────────────────────────────────────────────────

/// Builds the glow map in-place using pre-allocated ping-pong scratch buffers.
///
/// **Half-resolution blur pipeline**: the seed is built and all blur passes run at
/// half resolution (⌈W/2⌉ × ⌈H/2⌉), then nearest-neighbour upsampled to full
/// resolution.  This gives ~4× fewer pixels in the blur inner loop — the dominant
/// cost — with no visible quality loss (glow is inherently soft).
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

    let hw = (width + 1) / 2;
    let hh = (height + 1) / 2;
    let hn = hw * hh;

    let cap = n.max(hn);
    if a.len() < cap {
        a.resize(cap, GlowPixel::default());
    }
    if b.len() < cap {
        b.resize(cap, GlowPixel::default());
    }
    if out.len() < n {
        out.resize(n, GlowPixel::default());
    }

    // ── 1. Build seed at half resolution ──────────────────────────────────
    for p in &mut a[..hn] {
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
            let idx = (y as usize / 2) * hw + (x as usize / 2);
            a[idx].add_scaled(r, g, bch, base.clamp(0.0, 1.0));
        }
    }

    // ── 2. Blur at half resolution ────────────────────────────────────────
    let blur_passes = 2 + (spread * 4.0).round() as usize;
    let mut src_is_a = true;
    for _ in 0..blur_passes {
        if src_is_a {
            blur_glow3x3_into(&a[..hn], &mut b[..hn], hw, hh);
        } else {
            blur_glow3x3_into(&b[..hn], &mut a[..hn], hw, hh);
        }
        src_is_a = !src_is_a;
    }

    // ── 3. Broad blur + combine + upsample ────────────────────────────────
    if src_is_a {
        blur_glow3x3_into(&a[..hn], &mut b[..hn], hw, hh);
        upsample_combine(&a[..hn], &b[..hn], hw, hh, width, height, frame, out);
    } else {
        blur_glow3x3_into(&b[..hn], &mut a[..hn], hw, hh);
        upsample_combine(&b[..hn], &a[..hn], hw, hh, width, height, frame, out);
    }
}

// ── Private helpers ───────────────────────────────────────────────────────

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

/// Combines core + halo glow at half resolution and nearest-neighbour upsamples
/// to full resolution into `out`.
fn upsample_combine(
    core: &[GlowPixel],
    halo: &[GlowPixel],
    hw: usize,
    hh: usize,
    width: usize,
    height: usize,
    frame: u32,
    out: &mut [GlowPixel],
) {
    for y in 0..height {
        let hy = (y / 2).min(hh - 1);
        let row_off = hy * hw;
        for x in 0..width {
            let hx = (x / 2).min(hw - 1);
            let hi = row_off + hx;
            let c = core[hi];
            let h = halo[hi];
            let mut mix = GlowPixel {
                r: c.r * 0.60 + h.r * 0.40,
                g: c.g * 0.60 + h.g * 0.40,
                b: c.b * 0.60 + h.b * 0.40,
                a: c.a * 0.62 + h.a * 0.38,
            }
            .normalized();
            let shimmer = 0.92 + 0.16 * rand01(x as u16, y as u16, frame.wrapping_add(1703));
            mix.a = (mix.a * shimmer).clamp(0.0, 1.0);
            out[y * width + x] = mix;
        }
    }
}

/// In-place 3×3 box blur — writes result into `dst`, reads from `src`.
fn blur_glow3x3_into(src: &[GlowPixel], dst: &mut [GlowPixel], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
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
                        (0, 0) => 0.22,
                        (0, 1) | (1, 0) => 0.14,
                        _ => 0.09,
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
