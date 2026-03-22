//! CRT phosphor burn-in / persistence effect.
//!
//! Maintains a ring buffer of recent frames and composites fading ghosts of
//! older content under the current frame.  The history is **not** cleared on
//! scene transitions, so the old scene fades out naturally under the new one —
//! replicating the phosphor-persistence look of real CRT monitors.

use super::{lerp_colour_local, normalize_bg, PostFxContext};
use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::Effect;
use crossterm::style::Color;
use std::cell::RefCell;

/// How many historical snapshots we keep in the ring buffer.
const MAX_HISTORY: usize = 16;

/// Interval between captured snapshots (in frames).  Capturing every frame
/// wastes memory on nearly-identical content; every 3rd frame gives smoother
/// decay at lower cost.
const CAPTURE_INTERVAL: u64 = 3;

/// Per-pixel RGB accumulator used during the ghost compositing pass.
#[derive(Clone, Copy, Default)]
struct Rgb {
    r: f32,
    g: f32,
    b: f32,
}

/// Ring buffer of recent frame snapshots used for the persistence effect.
struct BurnInHistory {
    /// Circular buffer of captured frames (oldest → newest when walking back
    /// from `head`).
    slots: Vec<Option<Vec<Cell>>>,
    /// Points to the *next* slot to write into.
    head: usize,
    /// Dimensions of the last stored snapshot (invalidate on resize).
    width: u16,
    height: u16,
    /// Frame counter (monotonic, never reset).
    frame_seq: u64,
}

impl Default for BurnInHistory {
    fn default() -> Self {
        Self {
            slots: (0..MAX_HISTORY).map(|_| None).collect(),
            head: 0,
            width: 0,
            height: 0,
            frame_seq: 0,
        }
    }
}

impl BurnInHistory {
    /// Record a snapshot of the current frame.
    fn capture(&mut self, buf: &Buffer) {
        if buf.width != self.width || buf.height != self.height {
            // Resolution changed — flush stale data.
            for slot in &mut self.slots {
                *slot = None;
            }
            self.width = buf.width;
            self.height = buf.height;
        }
        let n = buf.width as usize * buf.height as usize;
        let mut cells = Vec::with_capacity(n);
        for y in 0..buf.height {
            for x in 0..buf.width {
                cells.push(buf.get(x, y).cloned().unwrap_or_default());
            }
        }
        self.slots[self.head] = Some(cells);
        self.head = (self.head + 1) % MAX_HISTORY;
        self.frame_seq += 1;
    }

    /// Iterate over stored frames from **newest → oldest**, yielding
    /// `(age_index, &[Cell])` where age_index 0 = most recent snapshot.
    fn iter_newest_first(&self) -> impl Iterator<Item = (usize, &[Cell])> {
        let head = self.head;
        (0..MAX_HISTORY).filter_map(move |i| {
            let slot_idx = (head + MAX_HISTORY - 1 - i) % MAX_HISTORY;
            self.slots[slot_idx].as_deref().map(|cells| (i, cells))
        })
    }
}

thread_local! {
    static BURN_IN: RefCell<BurnInHistory> = RefCell::new(BurnInHistory::default());
}

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    if src.width == 0 || src.height == 0 {
        dst.clone_from(src);
        return;
    }

    let intensity = pass.params.intensity.unwrap_or(0.45).clamp(0.0, 1.0);
    let decay = pass.params.speed.unwrap_or(0.35).clamp(0.05, 0.95);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.2, 2.0);
    let alpha = pass.params.alpha.unwrap_or(0.30).clamp(0.0, 1.0);

    BURN_IN.with(|history| {
        let mut hist = history.borrow_mut();

        let w = src.width as usize;
        let h = src.height as usize;
        let n = w * h;

        // ── 1. Composite ghost from history ───────────────────────────────────
        // Accumulate weighted ghost colour per-pixel from the ring buffer.
        // Each older snapshot is blended with exponentially decreasing weight.
        let mut ghost: Vec<Rgb> = vec![Rgb::default(); n];
        let mut weight_sum: Vec<f32> = vec![0.0; n];

        for (age, cells) in hist.iter_newest_first() {
            if cells.len() != n {
                continue; // stale dimension — skip
            }
            // Exponential decay: w = intensity * decay^age
            let w_factor = intensity * decay.powi(age as i32 + 1) * brightness;
            if w_factor < 0.005 {
                break; // remaining frames would be invisible
            }
            for idx in 0..n {
                let cell = &cells[idx];
                let (cr, cg, cb) = pixel_colour(cell);
                if cr + cg + cb < 0.01 {
                    continue; // black pixel — no contribution
                }
                ghost[idx].r += cr * w_factor;
                ghost[idx].g += cg * w_factor;
                ghost[idx].b += cb * w_factor;
                weight_sum[idx] += w_factor;
            }
        }

        // ── 2. Blend ghost under current frame ───────────────────────────────
        for y in 0..src.height {
            for x in 0..src.width {
                let Some(current) = src.get(x, y).cloned() else {
                    continue;
                };
                let idx = y as usize * w + x as usize;
                let ws = weight_sum[idx];
                if ws < 0.005 {
                    // No ghost contribution — pass through current pixel.
                    dst.set(x, y, current.symbol, current.fg, current.bg);
                    continue;
                }
                // Normalize accumulated ghost colour.
                let gr = (ghost[idx].r / ws).clamp(0.0, 1.0);
                let gg = (ghost[idx].g / ws).clamp(0.0, 1.0);
                let gb = (ghost[idx].b / ws).clamp(0.0, 1.0);
                let ghost_col = Color::Rgb {
                    r: (gr * 255.0).round() as u8,
                    g: (gg * 255.0).round() as u8,
                    b: (gb * 255.0).round() as u8,
                };

                // Blend intensity: ghost shows strongly through dark/empty areas,
                // weakly (if at all) through bright current content.
                let current_luma = pixel_luma(&current);
                let ghost_alpha = (alpha * ws.min(1.0) * (1.0 - current_luma * 0.7)).clamp(0.0, 0.6);

                let bg = lerp_colour_local(normalize_bg(current.bg), ghost_col, ghost_alpha);
                let fg = if current.symbol != ' ' {
                    lerp_colour_local(current.fg, ghost_col, ghost_alpha * 0.3)
                } else {
                    current.fg
                };
                dst.set(x, y, current.symbol, fg, bg);
            }
        }

        // ── 3. Capture current source into ring buffer ───────────────────────
        if ctx.frame_count % CAPTURE_INTERVAL == 0 {
            hist.capture(src);
        }
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
