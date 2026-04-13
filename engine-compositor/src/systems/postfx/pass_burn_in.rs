//! Phosphor persistence / afterglow (poświata luminoforu).
//!
//! Classic CRT phosphor model — on scene change the previous image
//! lingers as an additive glow on top of the new scene and decays
//! exponentially:
//!
//! ```text
//! result[p] = scene2[p] + f(t) × blur(scene1[p])
//! f(t) = alpha × exp(-k × t)
//! ```
//!
//! Enhancements over bare formula:
//! - **Phosphor bloom** — new scene overshoots ~30% brighter then settles
//! - **P31 phosphor tint** — per-channel decay (green lingers, blue dies)
//! - **3×3 blur** — phosphor spread simulation
//!
//! ## YAML parameters
//!
//! | param        | default | meaning                                      |
//! |--------------|---------|----------------------------------------------|
//! | `alpha`      | 0.70    | initial ghost brightness (0–1)               |
//! | `speed`      | 0.40    | decay time constant in seconds               |
//! | `brightness` | 1.0     | ghost luminance multiplier                   |
//! | `intensity`  | 1.0     | overall strength (0 = off)                   |
//! | `pump`       | 1.3     | bloom overshoot (1.0=none, 1.3=30% flash)    |
//! | `decay_tint` | 0.8     | P31 colour shift (0=uniform, 1=full green)   |

use super::{normalize_bg, PostFxContext};
use engine_core::buffer::{Buffer, Cell};
use engine_core::color::Color;
use engine_core::scene::Effect;
use engine_effects::utils::color::colour_to_rgb;
use std::cell::RefCell;

// ── State ─────────────────────────────────────────────────────────────────

#[derive(Default)]
struct BurnInState {
    /// Last frame captured (before transition).
    live_capture: Vec<Cell>,
    live_w: u16,
    live_h: u16,
    /// Ghost = promoted snapshot of live_capture at transition moment.
    ghost: Option<Vec<Cell>>,
    ghost_w: u16,
    ghost_h: u16,
    prev_scene_elapsed_ms: u64,
    has_capture: bool,
}

impl BurnInState {
    fn capture_live(&mut self, buf: &Buffer) {
        let n = buf.width as usize * buf.height as usize;
        if self.live_capture.len() != n || self.live_w != buf.width || self.live_h != buf.height {
            self.live_capture.resize(n, Cell::default());
            self.live_w = buf.width;
            self.live_h = buf.height;
        }
        for y in 0..buf.height {
            for x in 0..buf.width {
                let idx = y as usize * buf.width as usize + x as usize;
                self.live_capture[idx] = buf.get(x, y).cloned().unwrap_or_default();
            }
        }
        self.has_capture = true;
    }

    fn promote_to_ghost(&mut self) {
        if !self.has_capture {
            return;
        }
        // Reuse ghost buffer if exists and same size, else clone.
        if let Some(ref mut ghost_buf) = self.ghost {
            if ghost_buf.len() == self.live_capture.len() {
                // Swap instead of clone to avoid allocation.
                std::mem::swap(ghost_buf, &mut self.live_capture);
                self.live_capture.clear(); // Clear the swapped-in buffer so it's ready for next capture
            } else {
                self.ghost = Some(self.live_capture.clone());
            }
        } else {
            self.ghost = Some(self.live_capture.clone());
        }
        self.ghost_w = self.live_w;
        self.ghost_h = self.live_h;
    }

    fn clear_ghost(&mut self) {
        self.ghost = None;
    }
}

thread_local! {
    static BURN_IN: RefCell<BurnInState> = RefCell::new(BurnInState::default());
}

// ── Main pass ─────────────────────────────────────────────────────────────

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    if src.width == 0 || src.height == 0 {
        dst.copy_back_from(src);
        return;
    }

    let alpha = pass.params.alpha.unwrap_or(0.70).clamp(0.0, 1.0);
    let speed = pass.params.speed.unwrap_or(0.40_f32).max(0.001);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.1, 3.0);
    let intensity = pass.params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);
    let pump = pass.params.pump.unwrap_or(1.3).clamp(1.0, 3.0);
    let decay_tint = pass.params.decay_tint.unwrap_or(0.8).clamp(0.0, 1.0);

    if intensity < 0.001 {
        dst.copy_back_from(src);
        return;
    }

    BURN_IN.with(|cell| {
        let mut s = cell.borrow_mut();

        let elapsed_ms = ctx.scene_elapsed_ms;
        let elapsed_s = elapsed_ms as f32 / 1000.0;

        // ── 1. Detect scene transition (elapsed jumps backwards) ──────
        if s.has_capture && elapsed_ms < s.prev_scene_elapsed_ms.saturating_sub(50) {
            s.promote_to_ghost();
        }
        s.prev_scene_elapsed_ms = elapsed_ms;

        // ── 2. No ghost → pass-through, capture for next time ─────────
        if s.ghost.is_none() {
            dst.copy_back_from(src);
            s.capture_live(src);
            return;
        }

        // ── 3. f(t) = alpha × exp(−k × t) ────────────────────────────
        //   k = 6.0 / speed  →  faster falloff, ghost fades by ~speed seconds.
        let k = 6.0 / speed;
        let decay = (-k * elapsed_s).exp();

        let f = (alpha * intensity * brightness * decay).clamp(0.0, 1.0);

        // Desaturation: ghost loses colour as it fades.
        // sat = 1 at full brightness, → 0 (greyscale) as ghost disappears.
        let desat = f.sqrt().clamp(0.0, 1.0);

        // ── 3b. Phosphor bloom: new scene overshoots then settles ─────
        //   g(t) = 1.0 + (pump - 1) × exp(−t / 0.05)
        //   At t=0: 30% brighter, at ~200ms: back to normal.
        let bloom_tau = 0.05_f32;
        let bloom_mul = 1.0 + (pump - 1.0) * (-elapsed_s / bloom_tau).exp();

        // Imperceptible → clear ghost.
        if f < 0.003 {
            s.clear_ghost();
            dst.copy_back_from(src);
            s.capture_live(src);
            return;
        }

        // P31 per-channel decay: green lingers, blue dies fastest.
        let fr = f * decay.powf(0.3 * decay_tint);
        let fg_ch = f * decay.powf(-0.3 * decay_tint);
        let fb = f * decay.powf(1.0 * decay_tint);

        let ghost = s.ghost.as_deref().unwrap();
        let gw = s.ghost_w as usize;

        // ── 4. result[p] = src[p] + f(t) × blur(ghost[p]) ────────────
        for y in 0..src.height {
            for x in 0..src.width {
                let src_cell = src.get(x, y).cloned().unwrap_or_default();

                let gx = (x as usize).min(gw.saturating_sub(1));
                let gy = (y as usize).min(s.ghost_h.saturating_sub(1) as usize);

                // 3×3 blur on ghost pixel.
                let (blur_r, blur_g, blur_b) = blur_sample(ghost, gw, s.ghost_h as usize, gx, gy);

                // Glow contribution (0–255 range).
                // Desaturate: as ghost fades, shift RGB toward greyscale luma.
                let raw_r = blur_r * fr * 255.0;
                let raw_g = blur_g * fg_ch * 255.0;
                let raw_b = blur_b * fb * 255.0;
                let luma = 0.299 * raw_r + 0.587 * raw_g + 0.114 * raw_b;
                let glow_r = luma + desat * (raw_r - luma);
                let glow_g = luma + desat * (raw_g - luma);
                let glow_b = luma + desat * (raw_b - luma);

                // Add glow to fg and bg SEPARATELY — preserves text contrast.
                // Bloom multiplies src brightness, glow adds ghost on top.
                let (sfr, sfg, sfb) = colour_to_rgb(src_cell.fg);
                let (sbr, sbg, sbb) = colour_to_rgb(normalize_bg(src_cell.bg));

                let out_fg = Color::Rgb {
                    r: (sfr as f32 * bloom_mul + glow_r).min(255.0) as u8,
                    g: (sfg as f32 * bloom_mul + glow_g).min(255.0) as u8,
                    b: (sfb as f32 * bloom_mul + glow_b).min(255.0) as u8,
                };
                let out_bg = Color::Rgb {
                    r: (sbr as f32 * bloom_mul + glow_r).min(255.0) as u8,
                    g: (sbg as f32 * bloom_mul + glow_g).min(255.0) as u8,
                    b: (sbb as f32 * bloom_mul + glow_b).min(255.0) as u8,
                };

                dst.set(x, y, src_cell.symbol, out_fg, out_bg);
            }
        }

        s.capture_live(src);
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// 3×3 blur: center 40%, cardinal 12%, corners 6%.
/// Unrolled kernel without closure overhead.
#[inline(always)]
fn blur_sample(ghost: &[Cell], gw: usize, gh: usize, gx: usize, gy: usize) -> (f32, f32, f32) {
    let lx = gx.saturating_sub(1);
    let rx = (gx + 1).min(gw - 1);
    let ty = gy.saturating_sub(1);
    let by = (gy + 1).min(gh - 1);

    // Pre-compute indices (with bounds guard).
    let len = ghost.len();
    let idx_c = gy * gw + gx;
    let idx_l = gy * gw + lx;
    let idx_r = gy * gw + rx;
    let idx_t = ty * gw + gx;
    let idx_b = by * gw + gx;
    let idx_tl = ty * gw + lx;
    let idx_tr = ty * gw + rx;
    let idx_bl = by * gw + lx;
    let idx_br = by * gw + rx;

    // Guard all indices once at start.
    if idx_c >= len
        || idx_l >= len
        || idx_r >= len
        || idx_t >= len
        || idx_b >= len
        || idx_tl >= len
        || idx_tr >= len
        || idx_bl >= len
        || idx_br >= len
    {
        return (0.0, 0.0, 0.0);
    }

    // Unrolled samples (no closure overhead).
    let (cr, cg, cb) = pixel_rgb(&ghost[idx_c]);
    let (lr, lg, lb) = pixel_rgb(&ghost[idx_l]);
    let (rr, rg, rb) = pixel_rgb(&ghost[idx_r]);
    let (tr, tg, tb) = pixel_rgb(&ghost[idx_t]);
    let (btr, btg, btb) = pixel_rgb(&ghost[idx_b]);
    let (tlr, tlg, tlb) = pixel_rgb(&ghost[idx_tl]);
    let (trr, trg, trb) = pixel_rgb(&ghost[idx_tr]);
    let (blr, blg, blb) = pixel_rgb(&ghost[idx_bl]);
    let (brr, brg, brb) = pixel_rgb(&ghost[idx_br]);

    (
        cr * 0.40 + (lr + rr + tr + btr) * 0.12 + (tlr + trr + blr + brr) * 0.06,
        cg * 0.40 + (lg + rg + tg + btg) * 0.12 + (tlg + trg + blg + brg) * 0.06,
        cb * 0.40 + (lb + rb + tb + btb) * 0.12 + (tlb + trb + blb + brb) * 0.06,
    )
}

/// Extract representative RGB (0.0–1.0) from a cell.
#[inline(always)]
fn pixel_rgb(cell: &Cell) -> (f32, f32, f32) {
    let c = if cell.symbol != ' ' {
        cell.fg
    } else {
        normalize_bg(cell.bg)
    };
    let (r, g, b) = colour_to_rgb(c);
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
