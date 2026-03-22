//! CRT phosphor burn-in / persistence transition effect.
//!
//! On scene transition the ghost of the previous frame overlays the new scene
//! and fades out.  The new scene renders immediately underneath.
//!
//! ## Realism features
//!
//! - **Exponential decay** — fast initial drop, long subtle tail (`exp(-4t)`)
//! - **Brightness pump** — ghost flashes brighter on first frame then drops
//! - **Phosphor colour decay** — blue fades fastest, green lingers (P31 tint)
//! - **2D blur kernel** — 3×3 weighted average for phosphor spread
//!
//! ## YAML parameters
//!
//! | param        | default | meaning                                        |
//! |--------------|---------|------------------------------------------------|
//! | `alpha`      | 0.60    | initial ghost brightness (fraction of original) |
//! | `speed`      | 0.35    | fade duration in seconds                        |
//! | `brightness` | 1.0     | ghost luminance multiplier                      |
//! | `intensity`  | 1.0     | overall effect strength (0 = off, 1 = full)     |
//! | `pump`       | 1.3     | first-frame brightness multiplier (≥1.0)        |
//! | `decay_tint` | 0.8     | phosphor colour shift (0=uniform, 1=full P31)   |

use super::{normalize_bg, PostFxContext};
use crate::buffer::{Buffer, Cell};
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::scene::Effect;
use crossterm::style::Color;
use std::cell::RefCell;

struct BurnInState {
    live_capture: Vec<Cell>,
    live_w: u16,
    live_h: u16,
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

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    if src.width == 0 || src.height == 0 {
        dst.clone_from(src);
        return;
    }

    let alpha = pass.params.alpha.unwrap_or(0.60).clamp(0.0, 1.0);
    let fade_secs = pass.params.speed.unwrap_or(0.35).clamp(0.01, 10.0);
    let fade_ms = (fade_secs * 1000.0) as u64;
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.1, 2.0);
    let intensity = pass.params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);
    let pump = pass.params.pump.unwrap_or(1.3).clamp(1.0, 3.0);
    let decay_tint = pass.params.decay_tint.unwrap_or(0.8).clamp(0.0, 1.0);

    if intensity < 0.001 {
        dst.clone_from(src);
        return;
    }

    BURN_IN.with(|cell| {
        let mut s = cell.borrow_mut();

        // ── 1. Detect scene transition (elapsed jumps backwards) ──────────
        let elapsed = ctx.scene_elapsed_ms;
        if s.has_capture && elapsed < s.prev_scene_elapsed_ms.saturating_sub(50) {
            s.promote_to_ghost();
        }
        s.prev_scene_elapsed_ms = elapsed;

        // ── 2. Ghost active? ──────────────────────────────────────────────
        //   No hard cutoff — ghost persists until opacity drops below
        //   perceptual threshold.  Safety cap at 5× fade duration.
        let max_ghost_ms = fade_ms * 5;
        let has_ghost = s.ghost.is_some() && elapsed < max_ghost_ms;

        if !has_ghost {
            if s.ghost.is_some() {
                s.clear_ghost();
            }
            dst.clone_from(src);
            s.capture_live(src);
            return;
        }

        // ── 3. Compute ghost envelope ─────────────────────────────────────
        //   t is NOT clamped to 1.0 — it runs past 1.0 so the exponential
        //   tail extends beyond fade_ms, giving a natural logarithmic fade.
        let t = elapsed as f32 / fade_ms as f32;

        // Exponential decay: fast drop, long tail.
        let decay = (-2.5 * t).exp();

        // Brightness pump: flash on first ~30ms then settle.
        let pump_t = (elapsed as f32 / 30.0).clamp(0.0, 1.0);
        let pump_mul = pump + (1.0 - pump) * pump_t; // pump → 1.0 over 30ms

        let ghost_opacity = (alpha * intensity * brightness * decay).clamp(0.0, 1.0);
        let pumped_opacity = (ghost_opacity * pump_mul).clamp(0.0, 1.0);

        // Below perceptual threshold → clear ghost, pass through.
        if pumped_opacity < 0.003 {
            s.clear_ghost();
            dst.clone_from(src);
            s.capture_live(src);
            return;
        }

        // Phosphor colour decay channels (P31 green phosphor model):
        // Blue dies fastest, red medium, green lingers.
        let r_decay = decay.powf(1.0 + 0.3 * decay_tint); // slightly faster
        let g_decay = decay.powf(1.0 - 0.3 * decay_tint); // slightly slower
        let b_decay = decay.powf(1.0 + 1.0 * decay_tint); // much faster

        let ghost = s.ghost.as_deref().unwrap();
        let gw = s.ghost_w as usize;
        let gh = s.ghost_h as usize;

        // ── 4. Render: new scene + ghost overlay ──────────────────────────
        for y in 0..src.height {
            for x in 0..src.width {
                let src_cell = src.get(x, y).cloned().unwrap_or_default();

                let gx = (x as usize).min(gw.saturating_sub(1));
                let gy = (y as usize).min(gh.saturating_sub(1));

                // 3×3 blur kernel: center 40%, cardinal 12%, corners 6%
                let sample = |sx: usize, sy: usize| -> (f32, f32, f32) {
                    let idx = sy * gw + sx;
                    if idx >= ghost.len() {
                        return (0.0, 0.0, 0.0);
                    }
                    pixel_rgb(&ghost[idx])
                };

                let (cr, cg, cb) = sample(gx, gy);

                let lx = if gx > 0 { gx - 1 } else { gx };
                let rx = if gx + 1 < gw { gx + 1 } else { gx };
                let ty = if gy > 0 { gy - 1 } else { gy };
                let by = if gy + 1 < gh { gy + 1 } else { gy };

                // Cardinal neighbours (12% each)
                let (nl, ng_l, nb_l) = sample(lx, gy);
                let (nr, ng_r, nb_r) = sample(rx, gy);
                let (nt, ng_t, nb_t) = sample(gx, ty);
                let (nb_, ng_b, nb_b) = sample(gx, by);

                // Corner neighbours (6% each)
                let (c_tl_r, c_tl_g, c_tl_b) = sample(lx, ty);
                let (c_tr_r, c_tr_g, c_tr_b) = sample(rx, ty);
                let (c_bl_r, c_bl_g, c_bl_b) = sample(lx, by);
                let (c_br_r, c_br_g, c_br_b) = sample(rx, by);

                let blur_r = cr * 0.40
                    + (nl + nr + nt + nb_) * 0.12
                    + (c_tl_r + c_tr_r + c_bl_r + c_br_r) * 0.06;
                let blur_g = cg * 0.40
                    + (ng_l + ng_r + ng_t + ng_b) * 0.12
                    + (c_tl_g + c_tr_g + c_bl_g + c_br_g) * 0.06;
                let blur_b = cb * 0.40
                    + (nb_l + nb_r + nb_t + nb_b) * 0.12
                    + (c_tl_b + c_tr_b + c_bl_b + c_br_b) * 0.06;

                // Apply per-channel phosphor decay + overall opacity.
                let gr = blur_r * r_decay * pumped_opacity;
                let gg = blur_g * g_decay * pumped_opacity;
                let gb = blur_b * b_decay * pumped_opacity;

                if gr + gg + gb < 0.005 {
                    dst.set(x, y, src_cell.symbol, src_cell.fg, src_cell.bg);
                    continue;
                }

                let ghost_col = Color::Rgb {
                    r: (gr * 255.0).clamp(0.0, 255.0) as u8,
                    g: (gg * 255.0).clamp(0.0, 255.0) as u8,
                    b: (gb * 255.0).clamp(0.0, 255.0) as u8,
                };

                let out_bg = lerp_colour(normalize_bg(src_cell.bg), ghost_col, pumped_opacity);
                let out_fg = if src_cell.symbol != ' ' {
                    lerp_colour(src_cell.fg, ghost_col, pumped_opacity * 0.4)
                } else {
                    out_bg
                };
                dst.set(x, y, src_cell.symbol, out_fg, out_bg);
            }
        }

        s.capture_live(src);
    });
}

fn pixel_rgb(cell: &Cell) -> (f32, f32, f32) {
    let c = if cell.symbol != ' ' {
        cell.fg
    } else {
        normalize_bg(cell.bg)
    };
    let (r, g, b) = colour_to_rgb(c);
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
