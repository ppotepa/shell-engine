//! CRT phosphor burn-in / persistence transition effect.
//!
//! Captures the last rendered frame before a scene transition and displays it
//! as a fading ghost under the new scene — replicating the phosphor-persistence
//! look of real CRT monitors.
//!
//! The ghost is **time-based**: it starts at `alpha` brightness (default 30% of
//! the original) and fades to black over `speed` seconds (default 0.35 s).
//! All parameters are YAML-configurable via the standard `params:` block.
//!
//! ## YAML parameters
//!
//! | param        | default | meaning                                        |
//! |--------------|---------|------------------------------------------------|
//! | `alpha`      | 0.30    | initial ghost brightness (fraction of original) |
//! | `speed`      | 0.35    | fade duration in seconds                        |
//! | `brightness` | 1.0     | ghost luminance multiplier                      |
//! | `intensity`  | 1.0     | overall effect strength (0 = off, 1 = full)     |

use super::{lerp_colour_local, normalize_bg, PostFxContext};
use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::Effect;
use crossterm::style::Color;
use std::cell::RefCell;

/// Persistent state for the burn-in effect.  Stored in a thread-local so it
/// survives scene transitions (the whole point of this effect).
struct BurnInState {
    /// Snapshot of the last fully-rendered frame (continuously updated).
    /// When a scene transition is detected this becomes the ghost source.
    live_capture: Vec<Cell>,
    live_w: u16,
    live_h: u16,

    /// The ghost frame shown under the new scene — promoted from
    /// `live_capture` the moment a transition is detected.
    ghost: Option<Vec<Cell>>,
    ghost_w: u16,
    ghost_h: u16,

    /// `scene_elapsed_ms` from the previous frame — used to detect scene
    /// transitions (elapsed resets to a smaller value on new scene).
    prev_scene_elapsed_ms: u64,
    /// Whether we've ever captured at least one frame.
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
    /// Overwrite the live capture buffer with the current source frame.
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

    /// Promote the current live capture to the ghost layer.
    fn promote_to_ghost(&mut self) {
        if !self.has_capture {
            return;
        }
        self.ghost = Some(self.live_capture.clone());
        self.ghost_w = self.live_w;
        self.ghost_h = self.live_h;
    }

    /// Discard the ghost (fade complete).
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

    // ── YAML-configurable parameters ──────────────────────────────────────
    let alpha = pass.params.alpha.unwrap_or(0.30).clamp(0.0, 1.0);
    let fade_secs = pass.params.speed.unwrap_or(0.35).clamp(0.01, 10.0);
    let fade_ms = (fade_secs * 1000.0) as u64;
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.1, 2.0);
    let intensity = pass.params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);

    if intensity < 0.001 {
        dst.clone_from(src);
        return;
    }

    BURN_IN.with(|cell| {
        let mut s = cell.borrow_mut();

        // ── 1. Detect scene transition ────────────────────────────────────
        // `scene_elapsed_ms` resets to a small value on a new scene.
        // A significant backwards jump means a transition happened.
        let elapsed = ctx.scene_elapsed_ms;
        if s.has_capture && elapsed < s.prev_scene_elapsed_ms.saturating_sub(50) {
            s.promote_to_ghost();
        }
        s.prev_scene_elapsed_ms = elapsed;

        // ── 2. Compute ghost opacity (time-based linear fade) ─────────────
        let ghost_opacity = if elapsed < fade_ms {
            let t = elapsed as f32 / fade_ms as f32; // 0.0 → 1.0
            alpha * intensity * brightness * (1.0 - t)
        } else {
            0.0
        };

        // If ghost expired, free memory.
        if ghost_opacity < 0.002 && s.ghost.is_some() && elapsed >= fade_ms {
            s.clear_ghost();
        }

        // ── 3. Composite: ghost under current frame ───────────────────────
        let has_ghost = ghost_opacity >= 0.002 && s.ghost.is_some();

        if has_ghost {
            let ghost = s.ghost.as_deref().unwrap();
            let gw = s.ghost_w as usize;
            let _gh = s.ghost_h as usize;

            for y in 0..src.height {
                for x in 0..src.width {
                    let Some(current) = src.get(x, y).cloned() else {
                        continue;
                    };

                    // Sample ghost pixel (handle resolution mismatch).
                    let gx = (x as usize).min(gw.saturating_sub(1));
                    let gy = (y as usize).min((s.ghost_h as usize).saturating_sub(1));
                    let gidx = gy * gw + gx;

                    if gidx >= ghost.len() {
                        dst.set(x, y, current.symbol, current.fg, current.bg);
                        continue;
                    }

                    let ghost_cell = &ghost[gidx];
                    let (gr, gg, gb) = pixel_colour(ghost_cell);

                    // Skip black ghost pixels — no contribution.
                    if gr + gg + gb < 0.01 {
                        dst.set(x, y, current.symbol, current.fg, current.bg);
                        continue;
                    }

                    // Ghost shows more through dark areas, less through bright ones.
                    let current_luma = pixel_luma(&current);
                    let px_alpha = (ghost_opacity * (1.0 - current_luma * 0.8)).clamp(0.0, 0.85);

                    let ghost_col = Color::Rgb {
                        r: (gr * 255.0).round().clamp(0.0, 255.0) as u8,
                        g: (gg * 255.0).round().clamp(0.0, 255.0) as u8,
                        b: (gb * 255.0).round().clamp(0.0, 255.0) as u8,
                    };

                    let bg = lerp_colour_local(normalize_bg(current.bg), ghost_col, px_alpha);
                    let fg = if current.symbol != ' ' {
                        lerp_colour_local(current.fg, ghost_col, px_alpha * 0.25)
                    } else {
                        current.fg
                    };
                    dst.set(x, y, current.symbol, fg, bg);
                }
            }
        } else {
            // No ghost — pass through unchanged.
            dst.clone_from(src);
        }

        // ── 4. Capture current frame for next transition ──────────────────
        s.capture_live(src);
    });
}

/// Extract the visible colour of a cell as normalized (r, g, b) floats.
fn pixel_colour(cell: &Cell) -> (f32, f32, f32) {
    let c = if cell.symbol != ' ' {
        cell.fg
    } else {
        normalize_bg(cell.bg)
    };
    let (r, g, b) = colour_to_rgb(c);
    (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// Perceptual luma (rec. 601) of a cell's visible colour.
fn pixel_luma(cell: &Cell) -> f32 {
    let (r, g, b) = pixel_colour(cell);
    0.299 * r + 0.587 * g + 0.114 * b
}
