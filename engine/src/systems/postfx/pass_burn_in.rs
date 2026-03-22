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
//! - **P31 phosphor tint** — per-channel decay (green lingers, blue dies)
//! - **3×3 blur** — phosphor spread simulation
//! - **Brightness pump** — brief flash on first frame (electron discharge)
//!
//! ## YAML parameters
//!
//! | param        | default | meaning                                      |
//! |--------------|---------|----------------------------------------------|
//! | `alpha`      | 0.70    | initial ghost brightness (0–1)               |
//! | `speed`      | 0.15    | decay time constant in seconds               |
//! | `brightness` | 1.0     | ghost luminance multiplier                   |
//! | `intensity`  | 1.0     | overall strength (0 = off)                   |
//! | `pump`       | 1.3     | first-frame flash multiplier (≥1.0)          |
//! | `decay_tint` | 0.8     | P31 colour shift (0=uniform, 1=full green)   |

use super::{normalize_bg, PostFxContext};
use crate::buffer::{Buffer, Cell};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::Effect;
use crossterm::style::Color;
use std::cell::RefCell;

// ── State ─────────────────────────────────────────────────────────────────

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

impl Default for BurnInState {
    fn default() -> Self {
        Self {
            live_capture: Vec::new(),
            live_w: 0,
            live_h: 0,
            ghost: None,
            ghost_w: 0,
            ghost_h: 0,
            prev_scene_elapsed_ms: 0,
            has_capture: false,
        }
    }
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
        self.ghost = Some(self.live_capture.clone());
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
        dst.clone_from(src);
        return;
    }

    let alpha = pass.params.alpha.unwrap_or(0.70).clamp(0.0, 1.0);
    let speed = pass.params.speed.unwrap_or(0.15_f32).max(0.001);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.1, 3.0);
    let intensity = pass.params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);
    let pump = pass.params.pump.unwrap_or(1.3).clamp(1.0, 3.0);
    let decay_tint = pass.params.decay_tint.unwrap_or(0.8).clamp(0.0, 1.0);

    if intensity < 0.001 {
        dst.clone_from(src);
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
            dst.clone_from(src);
            s.capture_live(src);
            return;
        }

        // ── 3. f(t) = alpha × exp(−k × t) ────────────────────────────
        //   k = 4.6 / speed  →  at t = speed, f ≈ 1% of alpha.
        let k = 4.6 / speed;
        let decay = (-k * elapsed_s).exp();

        // Pump: brief flash for first ~30ms then settle to 1.0.
        let pump_t = (elapsed_s / 0.030).clamp(0.0, 1.0);
        let pump_mul = pump + (1.0 - pump) * pump_t;

        let f = (alpha * intensity * brightness * decay * pump_mul).clamp(0.0, 1.0);

        // Imperceptible → clear ghost.
        if f < 0.003 {
            s.clear_ghost();
            dst.clone_from(src);
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
                let (sr, sg, sb) = cell_rgb(&src_cell);

                let gx = (x as usize).min(gw.saturating_sub(1));
                let gy = (y as usize).min(s.ghost_h.saturating_sub(1) as usize);

                // 3×3 blur on ghost pixel.
                let (br, bg, bb) = blur_sample(ghost, gw, s.ghost_h as usize, gx, gy);

                // Additive glow: new scene + phosphor afterglow.
                let out_r = (sr + br * fr * 255.0).min(255.0) as u8;
                let out_g = (sg + bg * fg_ch * 255.0).min(255.0) as u8;
                let out_b = (sb + bb * fb * 255.0).min(255.0) as u8;

                let out = Color::Rgb { r: out_r, g: out_g, b: out_b };

                // Keep original symbol + use glow as both fg and bg.
                dst.set(x, y, src_cell.symbol, out, out);
            }
        }

        s.capture_live(src);
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// 3×3 blur: center 40%, cardinal 12%, corners 6%.
fn blur_sample(
    ghost: &[Cell],
    gw: usize,
    gh: usize,
    gx: usize,
    gy: usize,
) -> (f32, f32, f32) {
    let sample = |sx: usize, sy: usize| -> (f32, f32, f32) {
        let idx = sy * gw + sx;
        if idx >= ghost.len() {
            return (0.0, 0.0, 0.0);
        }
        pixel_rgb(&ghost[idx])
    };

    let (cr, cg, cb) = sample(gx, gy);

    let lx = gx.saturating_sub(1);
    let rx = if gx + 1 < gw { gx + 1 } else { gx };
    let ty = gy.saturating_sub(1);
    let by = if gy + 1 < gh { gy + 1 } else { gy };

    let (lr, lg, lb) = sample(lx, gy);
    let (rr, rg, rb) = sample(rx, gy);
    let (tr, tg, tb) = sample(gx, ty);
    let (btr, btg, btb) = sample(gx, by);

    let (tlr, tlg, tlb) = sample(lx, ty);
    let (trr, trg, trb) = sample(rx, ty);
    let (blr, blg, blb) = sample(lx, by);
    let (brr, brg, brb) = sample(rx, by);

    (
        cr * 0.40 + (lr + rr + tr + btr) * 0.12 + (tlr + trr + blr + brr) * 0.06,
        cg * 0.40 + (lg + rg + tg + btg) * 0.12 + (tlg + trg + blg + brg) * 0.06,
        cb * 0.40 + (lb + rb + tb + btb) * 0.12 + (tlb + trb + blb + brb) * 0.06,
    )
}

/// Extract representative RGB (0.0–1.0) from a cell.
fn pixel_rgb(cell: &Cell) -> (f32, f32, f32) {
    let c = if cell.symbol != ' ' { cell.fg } else { normalize_bg(cell.bg) };
    let (r, g, b) = colour_to_rgb(c);
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// Extract RGB (0.0–255.0) from a cell for additive blend.
fn cell_rgb(cell: &Cell) -> (f32, f32, f32) {
    let c = if cell.symbol != ' ' { cell.fg } else { normalize_bg(cell.bg) };
    let (r, g, b) = colour_to_rgb(c);
    (r as f32, g as f32, b as f32)
}
