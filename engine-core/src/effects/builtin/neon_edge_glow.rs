//! Cyberpunk neon edge glow with volumetric spillover.
//!
//! Detects content silhouette edges and paints a coloured glow halo
//! that bleeds outward into empty cells.  A breathing pulse driven by
//! `progress × speed` keeps the glow alive.

use crate::buffer::Buffer;
use crate::color::Color;
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{
    slider, EffectMetadata, ParamControl, ParamMetadata, P_EASING, P_INTENSITY, P_SPEED,
};
use crate::effects::utils::color::lerp_colour;
use crate::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "neon-edge-glow",
    display_name: "Neon Edge Glow",
    summary: "Cyberpunk neon glow around content edges with volumetric spillover.",
    category: "colour",
    compatible_targets: EffectTargetMask::ANY,
    params: &[
        ParamMetadata {
            name: "colour",
            label: "Colour",
            description: "Neon glow colour (name or #rrggbb).",
            control: ParamControl::Colour { default: "#00ccff" },
        },
        P_INTENSITY,
        slider("alpha", "Alpha", "Spillover glow opacity.", 0.0, 1.0, 0.05, ""),
        P_SPEED,
        P_EASING,
    ],
    sample: "- name: neon-edge-glow\n  duration: 2000\n  params:\n    colour: \"#00ccff\"\n    intensity: 0.6\n    alpha: 0.4\n    speed: 1.0",
};

const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// Maximum glow rings radiating outward from content edges.
const MAX_RINGS: u8 = 3;

/// Per-ring falloff multiplier (ring 1 = full, ring 2 = ~half, ring 3 = faint).
const RING_FALLOFF: [f32; 3] = [1.0, 0.45, 0.15];

pub struct NeonEdgeGlowEffect;

impl Effect for NeonEdgeGlowEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let w = region.width as usize;
        let h = region.height as usize;
        if w == 0 || h == 0 {
            return;
        }

        let intensity = params.intensity.unwrap_or(0.6).clamp(0.0, 2.0);
        let alpha = params.alpha.unwrap_or(0.4).clamp(0.0, 1.0);
        let speed = params.speed.unwrap_or(1.0);
        let neon = params
            .colour
            .as_ref()
            .map(Color::from)
            .unwrap_or(Color::Rgb {
                r: 0,
                g: 204,
                b: 255,
            });

        // Breathing pulse — completes a full sine cycle per loop so looping is seamless.
        let phase = progress * std::f32::consts::TAU * speed;
        let pulse = 0.6 + 0.4 * phase.sin();
        let glow_mul = intensity * pulse;

        let total = w * h;

        // ── Pass 1: content bitmap ────────────────────────────────────────
        let mut content = vec![false; total];
        for dy in 0..h {
            for dx in 0..w {
                if let Some(cell) = buffer.get(region.x + dx as u16, region.y + dy as u16) {
                    content[dy * w + dx] = cell.symbol != ' ';
                }
            }
        }

        // ── Pass 2: distance rings (BFS from content) ────────────────────
        let mut dist = vec![u8::MAX; total];
        for i in 0..total {
            if content[i] {
                dist[i] = 0;
            }
        }
        for ring in 1..=MAX_RINGS {
            let prev = ring - 1;
            for dy in 0..h {
                for dx in 0..w {
                    let idx = dy * w + dx;
                    if dist[idx] != u8::MAX {
                        continue;
                    }
                    for &(ndx, ndy) in &NEIGHBOURS {
                        let nx = dx as i32 + ndx;
                        let ny = dy as i32 + ndy;
                        if nx >= 0
                            && nx < w as i32
                            && ny >= 0
                            && ny < h as i32
                            && dist[ny as usize * w + nx as usize] == prev
                        {
                            dist[idx] = ring;
                            break;
                        }
                    }
                }
            }
        }

        // ── Pass 3: colouring ─────────────────────────────────────────────
        for dy in 0..h {
            for dx in 0..w {
                let x = region.x + dx as u16;
                let y = region.y + dy as u16;
                let idx = dy * w + dx;
                let d = dist[idx];

                if d == 0 {
                    // Content cell — tint fg on silhouette edges only.
                    let is_edge = NEIGHBOURS.iter().any(|&(ndx, ndy)| {
                        let nx = dx as i32 + ndx;
                        let ny = dy as i32 + ndy;
                        nx < 0
                            || nx >= w as i32
                            || ny < 0
                            || ny >= h as i32
                            || !content[ny as usize * w + nx as usize]
                    });
                    if is_edge {
                        if let Some(cell) = buffer.get(x, y) {
                            let sym = cell.symbol;
                            let fg = lerp_colour(cell.fg, neon, glow_mul.min(1.0));
                            let bg = cell.bg;
                            buffer.set(x, y, sym, fg, bg);
                        }
                    }
                } else if d >= 1 && d <= MAX_RINGS {
                    let falloff = RING_FALLOFF[(d - 1) as usize];
                    let glow_str = (alpha * glow_mul * falloff).min(1.0);
                    if glow_str > 0.01 {
                        if let Some(cell) = buffer.get(x, y) {
                            let sym = cell.symbol;
                            let fg = cell.fg;
                            let bg = lerp_colour(cell.bg, neon, glow_str);
                            buffer.set(x, y, sym, fg, bg);
                        }
                    }
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}
