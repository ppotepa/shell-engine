//! CRT phosphor burn-in / persistence transition effect.
//!
//! On scene transition the ghost of the previous frame overlays the new scene
//! and fades out over `speed` seconds.  The new scene renders immediately
//! underneath — the ghost is purely additive on top.
//!
//! ## YAML parameters
//!
//! | param        | default | meaning                                        |
//! |--------------|---------|------------------------------------------------|
//! | `alpha`      | 0.15    | initial ghost brightness (fraction of original) |
//! | `speed`      | 0.20    | fade duration in seconds                        |
//! | `brightness` | 1.0     | ghost luminance multiplier                      |
//! | `intensity`  | 1.0     | overall effect strength (0 = off, 1 = full)     |

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

    let alpha = pass.params.alpha.unwrap_or(0.15).clamp(0.0, 1.0);
    let fade_secs = pass.params.speed.unwrap_or(0.20).clamp(0.01, 10.0);
    let fade_ms = (fade_secs * 1000.0) as u64;
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.1, 2.0);
    let intensity = pass.params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);

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

        // ── 2. Ghost opacity ──────────────────────────────────────────────
        let has_ghost = s.ghost.is_some() && elapsed < fade_ms;

        if !has_ghost {
            if s.ghost.is_some() {
                s.clear_ghost();
            }
            // No ghost — pure passthrough, just capture for next time.
            dst.clone_from(src);
            s.capture_live(src);
            return;
        }

        let ghost_t = (elapsed as f32 / fade_ms as f32).clamp(0.0, 1.0);
        let ghost_opacity = (alpha * intensity * brightness * (1.0 - ghost_t)).clamp(0.0, 1.0);

        let ghost = s.ghost.as_deref().unwrap();
        let gw = s.ghost_w as usize;

        // ── 3. Render: new scene + ghost overlay ──────────────────────────
        for y in 0..src.height {
            for x in 0..src.width {
                // Start from new scene pixel.
                let src_cell = src.get(x, y).cloned().unwrap_or_default();

                // Sample ghost pixel (3-pixel horizontal blur).
                let gx = (x as usize).min(gw.saturating_sub(1));
                let gy = (y as usize).min((s.ghost_h as usize).saturating_sub(1));

                let sample = |sx: usize, sy: usize| -> (f32, f32, f32) {
                    let idx = sy * gw + sx;
                    if idx >= ghost.len() {
                        return (0.0, 0.0, 0.0);
                    }
                    pixel_rgb(&ghost[idx])
                };

                let (cr, cg, cb) = sample(gx, gy);
                let (lr, lg, lb) = if gx > 0 { sample(gx - 1, gy) } else { (cr, cg, cb) };
                let (rr, rg, rb) = if gx + 1 < gw { sample(gx + 1, gy) } else { (cr, cg, cb) };

                let gr = lr * 0.2 + cr * 0.6 + rr * 0.2;
                let gg = lg * 0.2 + cg * 0.6 + rg * 0.2;
                let gb = lb * 0.2 + cb * 0.6 + rb * 0.2;

                // Skip near-black ghost pixels — no visible contribution.
                if gr + gg + gb < 0.01 {
                    dst.set(x, y, src_cell.symbol, src_cell.fg, src_cell.bg);
                    continue;
                }

                let ghost_col = Color::Rgb {
                    r: (gr * 255.0).clamp(0.0, 255.0) as u8,
                    g: (gg * 255.0).clamp(0.0, 255.0) as u8,
                    b: (gb * 255.0).clamp(0.0, 255.0) as u8,
                };

                // Blend ghost on top of new scene at ghost_opacity.
                let out_bg = lerp_colour(normalize_bg(src_cell.bg), ghost_col, ghost_opacity);
                let out_fg = if src_cell.symbol != ' ' {
                    lerp_colour(src_cell.fg, ghost_col, ghost_opacity * 0.5)
                } else {
                    out_bg
                };
                dst.set(x, y, src_cell.symbol, out_fg, out_bg);
            }
        }

        // Capture for next transition.
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
